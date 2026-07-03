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
