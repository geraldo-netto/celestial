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
    vsop87::{heliocentric, Planet},
};
use crate::error::{Error, Result};
use crate::types::PlanetPos;

// ─── Entry points ─────────────────────────────────────────────────────────────

/// Compute apparent geocentric position using UT (Universal Time).
///
/// Body numbers follow the Swiss Ephemeris convention (see [`body`]).
/// Flags are the `FLG_*` constants from [`crate::constants`].
#[inline]
pub fn calc_ut(jd_ut: f64, body_num: i32, flags: i32) -> Result<PlanetPos> {
    let jde = ut_to_tt(jd_ut);
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
        body::MEAN_NODE | body::TRUE_NODE => return calc_node(jde, body_num, flags),
        body::CHIRON => return calc_chiron(jde, flags),
        body::PLUTO => return calc_pluto(jde, flags),
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

    if flags as u32 & flag::FLG_SIDEREAL != 0 {
        lon = norm_deg(lon - sidereal_ayanamsa(jde));
    }

    if flags & crate::FLG_TOPOCTR != 0 {
        lon = topocentric_lon(jde, &geo, use_equatorial);
    }

    let (speed_lon, speed_lat, speed_dist) =
        compute_speed(jde, body_num, flags, use_equatorial)?;

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

fn calc_node(jde: f64, body_num: i32, flags: i32) -> Result<PlanetPos> {
    let lon = if body_num == body::MEAN_NODE {
        crate::astronomy::nodes::moon_mean_node(jde)
    } else {
        crate::astronomy::nodes::moon_true_node(jde)
    };
    let spd = if body_num == body::MEAN_NODE {
        crate::astronomy::nodes::moon_mean_node_speed(jde)
    } else {
        crate::astronomy::nodes::moon_true_node_speed(jde)
    };
    let speed_lon = if flags as u32 & flag::FLG_SPEED != 0 { spd } else { 0.0 };
    Ok(PlanetPos {
        lon,
        lat: 0.0,
        dist: 1.0,
        speed_lon,
        speed_lat: 0.0,
        speed_dist: 0.0,
        ret_flags: (flags as u32 | flag::FLG_BUILTIN) as i32,
    })
}

fn calc_chiron(jde: f64, flags: i32) -> Result<PlanetPos> {
    let (lon, lat, dist) = crate::astronomy::chiron::chiron_pos(jde);
    let (speed_lon, speed_lat, speed_dist) = if flags as u32 & flag::FLG_SPEED != 0 {
        crate::astronomy::chiron::chiron_speed(jde)
    } else {
        (0.0, 0.0, 0.0)
    };
    Ok(PlanetPos {
        lon,
        lat,
        dist,
        speed_lon,
        speed_lat,
        speed_dist,
        ret_flags: (flags as u32 | flag::FLG_BUILTIN) as i32,
    })
}

fn calc_pluto(jde: f64, flags: i32) -> Result<PlanetPos> {
    let (lon, lat, dist) = crate::astronomy::pluto::pluto_geocentric(jde);
    let (speed_lon, speed_lat, speed_dist) = if flags as u32 & flag::FLG_SPEED != 0 {
        let p = crate::astronomy::pluto::pluto_geocentric(jde + 0.5);
        let m = crate::astronomy::pluto::pluto_geocentric(jde - 0.5);
        (angle_speed(p.0, m.0), p.1 - m.1, p.2 - m.2)
    } else {
        (0.0, 0.0, 0.0)
    };
    Ok(PlanetPos {
        lon,
        lat,
        dist,
        speed_lon,
        speed_lat,
        speed_dist,
        ret_flags: (flags as u32 | flag::FLG_BUILTIN) as i32,
    })
}

fn calc_heliocentric(jde: f64, body_num: i32, flags: i32) -> Option<PlanetPos> {
    let vsop_planet = body_to_vsop(body_num)?;
    let h = heliocentric(vsop_planet, jde);
    let lon_deg = h.lon.to_degrees().rem_euclid(360.0);
    let lat_deg = h.lat.to_degrees();
    let (speed_lon, speed_lat, speed_dist) = if flags as u32 & flag::FLG_SPEED != 0 {
        let h2 = heliocentric(vsop_planet, jde + 0.5);
        let h0 = heliocentric(vsop_planet, jde - 0.5);
        let sl = (h2.lon - h0.lon).to_degrees();
        (
            angle_speed(sl, 0.0),
            (h2.lat - h0.lat).to_degrees(),
            h2.rad - h0.rad,
        )
    } else {
        (0.0, 0.0, 0.0)
    };
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

    let gst_deg = crate::functions::time::sidtime(
        jde - crate::astronomy::delta_t::delta_t(jde) / 86400.0,
    ) * 15.0;
    let ha_deg = (gst_deg + obs_lon - geo.ra).rem_euclid(360.0);
    let ha_r = ha_deg.to_radians();

    let u = (0.996_647_19_f64 * obs_lat_r.tan()).atan();
    let rho_sin = 0.996_647_19 * u.sin() + (obs_alt / 6_378_137.0) * obs_lat_r.sin();
    let rho_cos = u.cos() + (obs_alt / 6_378_137.0) * obs_lat_r.cos();

    let dec_r = geo.dec.to_radians();
    let hp_sin = horiz_parallax_r.sin();
    let denom = dec_r.cos() - rho_cos * hp_sin * ha_r.cos();
    let delta_ra = (-rho_cos * hp_sin * ha_r.sin()) / denom;
    let delta_dec = ((-rho_sin * hp_sin
        + rho_cos * hp_sin * ha_r.cos() * delta_ra.sin())
        * dec_r.sin()
        - rho_cos * hp_sin * ha_r.cos())
        / denom;

    let topo_ra = geo.ra + delta_ra.atan().to_degrees();
    let topo_dec = geo.dec + delta_dec.atan().to_degrees();

    if use_equatorial {
        return topo_ra.rem_euclid(360.0);
    }
    let eps_r = crate::astronomy::obliquity(jde).to_radians();
    let ra_r2 = topo_ra.to_radians();
    let dec_r2 = topo_dec.to_radians();
    (ra_r2.sin() * eps_r.cos() + dec_r2.tan() * eps_r.sin())
        .atan2(ra_r2.cos())
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
        body::MEAN_NODE => return Ok((crate::astronomy::nodes::moon_mean_node(jde), 0.0, 1.0)),
        body::TRUE_NODE => return Ok((crate::astronomy::nodes::moon_true_node(jde), 0.0, 1.0)),
        body::CHIRON => {
            let (l, b, r) = crate::astronomy::chiron::chiron_pos(jde);
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
