//! Planetary computation engine — core dispatch hub.
//!
//! This module owns the `calc_ut` / `calc_tt` entry points and routes each
//! body-number to the appropriate algorithm (VSOP87, ELP2000, nodes, Chiron).
//! All other `astronomy::*` modules contain the pure maths; this module just
//! coordinates them.

use crate::astronomy::{
    ayanamsa::{ayanamsa as calc_ayanamsa, SidMode},
    body,
    constants::norm_deg,
    delta_t::ut_to_tt,
    flag,
    planetary::{apparent_moon, apparent_planet, apparent_sun},
    vsop87::{heliocentric, heliocentric_with_speed, Planet},
};
use crate::error::{Error, Result};
use crate::types::PlanetPos;
use crate::units::JulianDay;

// ─── Entry points ─────────────────────────────────────────────────────────────

/// Compute apparent geocentric position using UT (Universal Time).
///
/// Body numbers follow the Swiss Ephemeris convention (see [`body`]).
/// Flags are the `FLG_*` constants from [`crate::constants`].
#[inline]
pub fn calc_ut(jd_ut: JulianDay, body_num: i32, flags: i32) -> Result<PlanetPos> {
    let jd_ut: f64 = jd_ut.into();
    let jde = ut_to_tt(JulianDay::new(jd_ut));
    calc_tt(jde, body_num, flags)
}

/// Compute apparent geocentric position using TT (Terrestrial Time / JDE).
#[inline]
pub fn calc_tt(jde: f64, body_num: i32, flags: i32) -> Result<PlanetPos> {
    let use_equatorial = flags as u32 & flag::FLG_EQUATORIAL != 0;

    let geo = match body_num {
        body::SUN => apparent_sun(jde),
        body::MERCURY => apparent_planet(Planet::Mercury, jde),
        body::VENUS => apparent_planet(Planet::Venus, jde),
        body::MARS => apparent_planet(Planet::Mars, jde),
        body::JUPITER => apparent_planet(Planet::Jupiter, jde),
        body::SATURN => apparent_planet(Planet::Saturn, jde),
        body::URANUS => apparent_planet(Planet::Uranus, jde),
        body::NEPTUNE => apparent_planet(Planet::Neptune, jde),
        body::MOON => apparent_moon(jde),
        body::MEAN_NODE | body::TRUE_NODE => return Ok(calc_node(jde, body_num, flags)),
        body::CHIRON => return Ok(calc_chiron(jde, flags)),
        body::PLUTO => return Ok(calc_pluto(jde, flags)),
        _ => {
            return Err(Error::Calc(
                "body not implemented in pure-Rust engine".into(),
            ))
        }
    };

    if flags as u32 & flag::FLG_HELCTR != 0 {
        if let Some(pos) = calc_heliocentric(jde, body_num, flags) {
            return Ok(pos);
        }
    }

    let mut lon = if use_equatorial { geo.ra } else { geo.lon };
    let lat = if use_equatorial { geo.dec } else { geo.lat };

    if flags & crate::FLG_TOPOCTR != 0 {
        lon = topocentric_lon(jde, &geo, use_equatorial);
    }

    if flags as u32 & flag::FLG_SIDEREAL != 0 {
        lon = norm_deg(lon - sidereal_ayanamsa(jde));
    }

    let (speed_lon, speed_lat, speed_dist) = compute_speed(jde, body_num, flags, use_equatorial)?;

    Ok(PlanetPos {
        lon,
        lat,
        dist: geo.dist,
        speed_lon,
        speed_lat,
        speed_dist,
        ret_flags: (flags as u32 | flag::FLG_BUILTIN) as i32,
    })
}

fn calc_node(jde: f64, body_num: i32, flags: i32) -> PlanetPos {
    let lon = if body_num == body::MEAN_NODE {
        crate::astronomy::nodes::moon_mean_node(JulianDay::new(jde))
    } else {
        crate::astronomy::nodes::moon_true_node(JulianDay::new(jde))
    };
    let spd = if body_num == body::MEAN_NODE {
        crate::astronomy::nodes::moon_mean_node_speed(JulianDay::new(jde))
    } else {
        crate::astronomy::nodes::moon_true_node_speed(JulianDay::new(jde))
    };
    let speed_lon = if flags as u32 & flag::FLG_SPEED != 0 {
        spd
    } else {
        0.0
    };
    PlanetPos {
        lon,
        lat: 0.0,
        dist: 1.0,
        speed_lon,
        speed_lat: 0.0,
        speed_dist: 0.0,
        ret_flags: (flags as u32 | flag::FLG_BUILTIN) as i32,
    }
}

fn calc_chiron(jde: f64, flags: i32) -> PlanetPos {
    // GEOcentric — `chiron_pos` is heliocentric. The previous version
    // used `chiron_pos` directly here and returned heliocentric values
    // labelled as geocentric, giving 10–40° wrong longitudes.
    let (lon, lat, dist) = crate::astronomy::chiron::chiron_geocentric(jde);
    let (speed_lon, speed_lat, speed_dist) = if flags as u32 & flag::FLG_SPEED != 0 {
        crate::astronomy::chiron::chiron_speed(JulianDay::new(jde))
    } else {
        (0.0, 0.0, 0.0)
    };
    PlanetPos {
        lon,
        lat,
        dist,
        speed_lon,
        speed_lat,
        speed_dist,
        ret_flags: (flags as u32 | flag::FLG_BUILTIN) as i32,
    }
}

fn calc_pluto(jde: f64, flags: i32) -> PlanetPos {
    let (lon, lat, dist) = crate::astronomy::pluto::pluto_geocentric(jde);
    let (speed_lon, speed_lat, speed_dist) = if flags as u32 & flag::FLG_SPEED != 0 {
        let p = crate::astronomy::pluto::pluto_geocentric(jde + 0.5);
        let m = crate::astronomy::pluto::pluto_geocentric(jde - 0.5);
        (angle_speed(p.0, m.0), p.1 - m.1, p.2 - m.2)
    } else {
        (0.0, 0.0, 0.0)
    };
    PlanetPos {
        lon,
        lat,
        dist,
        speed_lon,
        speed_lat,
        speed_dist,
        ret_flags: (flags as u32 | flag::FLG_BUILTIN) as i32,
    }
}

fn calc_heliocentric(jde: f64, body_num: i32, flags: i32) -> Option<PlanetPos> {
    let vsop_planet = body_to_vsop(body_num)?;
    let (h, (speed_lon, speed_lat, speed_dist)) = if flags as u32 & flag::FLG_SPEED != 0 {
        heliocentric_with_speed(vsop_planet, jde)
    } else {
        (heliocentric(vsop_planet, jde), (0.0, 0.0, 0.0))
    };
    let lon_deg = h.lon.to_degrees().rem_euclid(360.0);
    let lat_deg = h.lat.to_degrees();
    Some(PlanetPos {
        lon: lon_deg,
        lat: lat_deg,
        dist: h.rad,
        speed_lon,
        speed_lat,
        speed_dist,
        ret_flags: (flags as u32 | flag::FLG_BUILTIN) as i32,
    })
}

fn sidereal_ayanamsa(jde: f64) -> f64 {
    let sid_mode = crate::functions::config::current_sid_mode();
    let mode = SidMode::from_i32(sid_mode).unwrap_or(SidMode::Lahiri);
    calc_ayanamsa(jde, mode)
}

/// Apply diurnal parallax (FLG_TOPOCTR) and return the corrected lon.
/// Significant for Moon (~57'), smaller for Sun (~8.8"), negligible for outer planets.
fn topocentric_lon(
    jde: f64,
    geo: &crate::astronomy::planetary::GeocentricPos,
    use_equatorial: bool,
) -> f64 {
    let (obs_lon, obs_lat, obs_alt) = crate::functions::config::current_topo();
    let obs_lat_r = obs_lat.to_radians();

    let dist_km = geo.dist * 149_597_870.7;
    let sin_pi = 6_378.137 / dist_km;
    let horiz_parallax_r = sin_pi.asin();

    let gst_deg = crate::functions::time::sidtime(JulianDay::new(
        jde - crate::astronomy::delta_t::delta_t(JulianDay::new(jde)) / 86400.0,
    )) * 15.0;
    let ha_deg = (gst_deg + obs_lon - geo.ra).rem_euclid(360.0);
    let ha_r = ha_deg.to_radians();

    let u = (0.996_647_19_f64 * obs_lat_r.tan()).atan();
    let (sin_u, cos_u) = u.sin_cos();
    let (sin_obs_lat, cos_obs_lat) = obs_lat_r.sin_cos();
    let alt_ratio = obs_alt / 6_378_137.0;
    let rho_sin = alt_ratio.mul_add(sin_obs_lat, 0.996_647_19 * sin_u);
    let rho_cos = alt_ratio.mul_add(cos_obs_lat, cos_u);

    let dec_r = geo.dec.to_radians();
    let hp_sin = horiz_parallax_r.sin();
    let (sin_ha, cos_ha) = ha_r.sin_cos();
    let (sin_dec, cos_dec) = dec_r.sin_cos();
    let rho_cos_hp = rho_cos * hp_sin;
    let denom = (-rho_cos_hp).mul_add(cos_ha, cos_dec);
    let delta_ra = (-rho_cos_hp * sin_ha) / denom;
    let delta_dec = rho_cos_hp
        .mul_add(cos_ha * delta_ra.sin(), -rho_sin * hp_sin)
        .mul_add(sin_dec, -rho_cos_hp * cos_ha)
        / denom;

    let topo_ra = geo.ra + delta_ra.atan().to_degrees();
    let topo_dec = geo.dec + delta_dec.atan().to_degrees();

    if use_equatorial {
        return topo_ra.rem_euclid(360.0);
    }
    let eps_r = crate::astronomy::obliquity(jde).to_radians();
    let ra_r2 = topo_ra.to_radians();
    let dec_r2 = topo_dec.to_radians();
    let (sin_ra2, cos_ra2) = ra_r2.sin_cos();
    let (sin_eps2, cos_eps2) = eps_r.sin_cos();
    dec_r2
        .tan()
        .mul_add(sin_eps2, sin_ra2 * cos_eps2)
        .atan2(cos_ra2)
        .to_degrees()
        .rem_euclid(360.0)
}

/// Compute (speed_lon, speed_lat, speed_dist) via numerical differentiation.
/// In equatorial mode, recursively calls calc_tt to get RA/Dec rates.
fn compute_speed(
    jde: f64,
    body_num: i32,
    flags: i32,
    use_equatorial: bool,
) -> Result<(f64, f64, f64)> {
    if flags as u32 & flag::FLG_SPEED == 0 {
        return Ok((0.0, 0.0, 0.0));
    }
    if use_equatorial {
        let flags_pos = flags & !(crate::astronomy::flag::FLG_SPEED as i32);
        let p_plus = calc_tt(jde + 0.5, body_num, flags_pos)?;
        let p_minus = calc_tt(jde - 0.5, body_num, flags_pos)?;
        return Ok((
            angle_speed(p_plus.lon, p_minus.lon),
            p_plus.lat - p_minus.lat,
            p_plus.dist - p_minus.dist,
        ));
    }
    let p_plus = body_position(body_num, jde + 0.5)?;
    let p_minus = body_position(body_num, jde - 0.5)?;
    Ok((
        angle_speed(p_plus.0, p_minus.0),
        p_plus.1 - p_minus.1,
        p_plus.2 - p_minus.2,
    ))
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Returns `(lon_deg, lat_deg, dist_au)` for a body at a given JDE.
pub(crate) fn body_position(body_num: i32, jde: f64) -> Result<(f64, f64, f64)> {
    let geo = match body_num {
        body::SUN => apparent_sun(jde),
        body::MERCURY => apparent_planet(Planet::Mercury, jde),
        body::VENUS => apparent_planet(Planet::Venus, jde),
        body::MARS => apparent_planet(Planet::Mars, jde),
        body::JUPITER => apparent_planet(Planet::Jupiter, jde),
        body::SATURN => apparent_planet(Planet::Saturn, jde),
        body::URANUS => apparent_planet(Planet::Uranus, jde),
        body::NEPTUNE => apparent_planet(Planet::Neptune, jde),
        body::MOON => apparent_moon(jde),
        body::MEAN_NODE => {
            return Ok((
                crate::astronomy::nodes::moon_mean_node(JulianDay::new(jde)),
                0.0,
                1.0,
            ))
        }
        body::TRUE_NODE => {
            return Ok((
                crate::astronomy::nodes::moon_true_node(JulianDay::new(jde)),
                0.0,
                1.0,
            ))
        }
        body::CHIRON => {
            let (l, b, r) = crate::astronomy::chiron::chiron_pos(JulianDay::new(jde));
            return Ok((l, b, r));
        }
        _ => return Err(Error::BodyNotImplemented { body: body_num }),
    };
    Ok((geo.lon, geo.lat, geo.dist))
}

/// Compute angular speed (°/day) accounting for 0°/360° wrap.
fn angle_speed(plus: f64, minus: f64) -> f64 {
    let raw = plus - minus;
    if raw > 180.0 {
        raw - 360.0
    } else if raw < -180.0 {
        raw + 360.0
    } else {
        raw
    }
}

/// Map a body constant to its VSOP87 Planet enum (returns None for Sun, Moon, nodes, Chiron).
fn body_to_vsop(body_num: i32) -> Option<Planet> {
    match body_num {
        body::MERCURY => Some(Planet::Mercury),
        body::VENUS => Some(Planet::Venus),
        body::MARS => Some(Planet::Mars),
        body::JUPITER => Some(Planet::Jupiter),
        body::SATURN => Some(Planet::Saturn),
        body::URANUS => Some(Planet::Uranus),
        body::NEPTUNE => Some(Planet::Neptune),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::{Latitude, Longitude};

    fn assert_position(actual: &PlanetPos, expected: [f64; 6]) {
        let actual = [
            actual.lon,
            actual.lat,
            actual.dist,
            actual.speed_lon,
            actual.speed_lat,
            actual.speed_dist,
        ];
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
        }
    }

    #[test]
    fn engine_dispatch_and_flags_match_regression_vectors() {
        crate::functions::config::set_topo(Longitude::new(12.5), Latitude::new(48.25), 350.0);
        let cases = [
            (
                body::SUN,
                258,
                [
                    140.138473031206,
                    0.000039474061,
                    1.013328649330,
                    0.959880589635,
                    0.000036097404,
                    -0.000177572264,
                ],
            ),
            (
                body::MOON,
                258,
                [
                    206.631082939444,
                    -0.587382732703,
                    0.002658571734,
                    12.203342549458,
                    1.071790224626,
                    -0.000026972158,
                ],
            ),
            (
                body::MERCURY,
                258,
                [
                    121.263176863644,
                    -1.162228653225,
                    0.905978736229,
                    0.984696893712,
                    0.266368708804,
                    0.025897576806,
                ],
            ),
            (
                body::VENUS,
                258,
                [
                    159.496596387277,
                    1.439460488886,
                    1.592722741162,
                    1.226727835483,
                    -0.011277485294,
                    -0.003908494429,
                ],
            ),
            (
                body::MARS,
                258,
                [
                    130.199851770482,
                    1.123247558328,
                    2.628318210763,
                    0.640477999749,
                    0.002992050452,
                    -0.001048037500,
                ],
            ),
            (
                body::JUPITER,
                258,
                [
                    293.997355195867,
                    -0.509430735960,
                    4.199110629655,
                    -0.101342131853,
                    -0.001434503866,
                    0.006659063168,
                ],
            ),
            (
                body::SATURN,
                258,
                [
                    93.074925369509,
                    -0.981274335499,
                    9.699323563932,
                    0.102913254787,
                    0.000153131881,
                    -0.012078131088,
                ],
            ),
            (
                body::URANUS,
                258,
                [
                    90.555994513748,
                    0.246366608031,
                    19.700638581716,
                    0.043185136127,
                    0.000308037152,
                    -0.013046514284,
                ],
            ),
            (
                body::NEPTUNE,
                258,
                [
                    17.711376286745,
                    -1.620020673557,
                    29.283821939741,
                    -0.011828198553,
                    -0.000871166894,
                    -0.014152452888,
                ],
            ),
            (
                body::PLUTO,
                258,
                [
                    313.416337535218,
                    -7.132850836902,
                    36.117006526666,
                    -0.022566094923,
                    -0.000787154267,
                    0.002842433721,
                ],
            ),
            (
                body::MEAN_NODE,
                258,
                [214.270702418787, 0.0, 1.0, -0.052953727788, 0.0, 0.0],
            ),
            (
                body::TRUE_NODE,
                258,
                [215.420458421669, 0.0, 1.0, -0.217098848816, 0.0, 0.0],
            ),
            (
                body::CHIRON,
                258,
                [
                    55.082625987682,
                    -2.611554029902,
                    16.260702032527,
                    0.017508213010,
                    -0.004105368138,
                    -0.017994451649,
                ],
            ),
            (
                body::MEAN_NODE,
                2,
                [214.270702418787, 0.0, 1.0, 0.0, 0.0, 0.0],
            ),
            (
                body::CHIRON,
                2,
                [
                    55.082625987682,
                    -2.611554029902,
                    16.260702032527,
                    0.0,
                    0.0,
                    0.0,
                ],
            ),
            (
                body::PLUTO,
                2,
                [
                    313.416337535218,
                    -7.132850836902,
                    36.117006526666,
                    0.0,
                    0.0,
                    0.0,
                ],
            ),
            (
                body::MOON,
                2_306,
                [
                    204.490436701499,
                    -10.815818002283,
                    0.002658571734,
                    11.979561403413,
                    -3.407519865668,
                    -0.000026972158,
                ],
            ),
            (
                body::MOON,
                33_026,
                [
                    206.903288796251,
                    -0.587382732703,
                    0.002658571734,
                    12.203342549458,
                    1.071790224626,
                    -0.000026972158,
                ],
            ),
            (
                body::MOON,
                35_074,
                [
                    204.985643424025,
                    -10.815818002283,
                    0.002658571734,
                    12.040510581409,
                    -3.407519865668,
                    -0.000026972158,
                ],
            ),
            (
                body::VENUS,
                266,
                [
                    187.337797426854,
                    3.182452991811,
                    0.720512072314,
                    1.616830979410,
                    -0.033339988704,
                    0.000111853616,
                ],
            ),
            (
                body::VENUS,
                10,
                [
                    187.337797426854,
                    3.182452991811,
                    0.720512072314,
                    0.0,
                    0.0,
                    0.0,
                ],
            ),
            (
                body::JUPITER,
                266,
                [
                    298.991667016757,
                    -0.417148554349,
                    5.128247317606,
                    0.087047397696,
                    -0.001867464050,
                    -0.000301989883,
                ],
            ),
            (
                body::SATURN,
                266,
                [
                    88.365817966768,
                    -1.052722230063,
                    9.039624059569,
                    0.038070318413,
                    0.001468596719,
                    -0.000023723987,
                ],
            ),
            (
                body::URANUS,
                266,
                [
                    88.234533698343,
                    0.254636609081,
                    19.059215480674,
                    0.011960808214,
                    0.000150050116,
                    -0.000167965045,
                ],
            ),
            (
                body::NEPTUNE,
                266,
                [
                    16.069396537020,
                    -1.589832629282,
                    29.839153757516,
                    0.006113584807,
                    -0.000086327495,
                    -0.000013896070,
                ],
            ),
        ];

        for (body, flags, expected) in cases {
            let actual = calc_tt(2_463_456.789, body, flags).unwrap();
            assert_position(&actual, expected);
            assert_eq!(actual.ret_flags, flags | flag::FLG_BUILTIN as i32);
        }
    }

    #[test]
    fn angle_speed_preserves_wrap_boundaries() {
        let cases = [
            (10.0, 350.0, 20.0),
            (350.0, 10.0, -20.0),
            (180.0, 0.0, 180.0),
            (0.0, 180.0, -180.0),
        ];
        for (plus, minus, expected) in cases {
            assert_eq!(angle_speed(plus, minus), expected);
        }
    }

    #[test]
    fn body_position_dispatches_special_bodies() {
        let cases = [
            (body::MEAN_NODE, [214.270_702_418_787, 0.0, 1.0]),
            (body::TRUE_NODE, [215.420_458_421_669, 0.0, 1.0]),
            (
                body::CHIRON,
                [
                    51.507_083_513_313_8,
                    -2.620_565_789_831_572,
                    16.204_838_297_179_347,
                ],
            ),
        ];
        for (body, expected) in cases {
            let actual = body_position(body, 2_463_456.789).unwrap();
            for (actual, expected) in [actual.0, actual.1, actual.2].into_iter().zip(expected) {
                assert!(
                    (actual - expected).abs() < 1e-9,
                    "body {body}: {actual} != {expected}"
                );
            }
        }
    }
}
