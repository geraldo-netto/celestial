//! House cusp calculation functions.

use crate::body::{CalcFlags, HouseSystem};
use crate::error::Result;
// Re-export the canonical types from the astronomy layer
pub use crate::astronomy::houses::{HouseResult, HouseResultEx2};

// ─── Return types ─────────────────────────────────────────────────────────────

// ─── House functions ──────────────────────────────────────────────────────────

/// Get the display name of a house system.
#[must_use]
pub fn house_name(hsys: HouseSystem) -> &'static str {
    crate::astronomy::house_name(hsys.as_raw())
}

// ─── Pure-Rust dispatch ───────────────────────────────────────────────────────

/// Compute house cusps (UT) — pure-Rust engine.
pub fn houses(jd_ut: f64, geolat: f64, geolon: f64, hsys: HouseSystem) -> Result<HouseResult> {
    Ok(crate::astronomy::houses(
        jd_ut,
        geolat,
        geolon,
        hsys.as_raw(),
    ))
}

/// Compute house cusps with flags (UT) — pure-Rust engine.
pub fn houses_ex(
    _jd_ut: f64,
    _flags: CalcFlags,
    geolat: f64,
    geolon: f64,
    hsys: HouseSystem,
) -> Result<HouseResult> {
    houses(_jd_ut, geolat, geolon, hsys)
}

/// Compute house cusps and their diurnal speeds (deg/day) via numerical differentiation.
pub fn houses_ex2(
    jd_ut: f64,
    _flags: CalcFlags,
    geolat: f64,
    geolon: f64,
    hsys: HouseSystem,
) -> Result<HouseResultEx2> {
    let h = 10.0 / 1440.0; // 10 minutes in days
    let r0 = crate::astronomy::houses(jd_ut - h, geolat, geolon, hsys.as_raw());
    let r1 = crate::astronomy::houses(jd_ut, geolat, geolon, hsys.as_raw());
    let r2 = crate::astronomy::houses(jd_ut + h, geolat, geolon, hsys.as_raw());

    let mut cusp_speeds = [0f64; 13];
    let mut ascmc_speeds = [0f64; 10];
    for (speed, (&c2, &c0)) in cusp_speeds
        .iter_mut()
        .zip(r2.cusps.iter().zip(r0.cusps.iter()))
    {
        *speed = crate::functions::utils::wrap_signed_180(c2 - c0) / (2.0 * h);
    }
    for (speed, (&a2, &a0)) in ascmc_speeds
        .iter_mut()
        .zip(r2.ascmc.iter().zip(r0.ascmc.iter()))
    {
        *speed = crate::functions::utils::wrap_signed_180(a2 - a0) / (2.0 * h);
    }
    Ok(HouseResultEx2 {
        cusps: r1.cusps,
        ascmc: r1.ascmc,
        cusp_speeds,
        ascmc_speeds,
    })
}

/// Compute house cusps from ARMC — pure-Rust engine.
pub fn houses_armc(armc: f64, geolat: f64, eps: f64, hsys: HouseSystem) -> Result<HouseResult> {
    Ok(crate::astronomy::houses_armc(
        armc,
        geolat,
        eps,
        hsys.as_raw(),
    ))
}

/// Compute house cusps and speeds from ARMC — pure-Rust (speeds return zeros).
pub fn houses_armc_ex2(
    armc: f64,
    geolat: f64,
    eps: f64,
    hsys: HouseSystem,
) -> Result<HouseResultEx2> {
    let r = houses_armc(armc, geolat, eps, hsys)?;
    Ok(HouseResultEx2 {
        cusps: r.cusps,
        ascmc: r.ascmc,
        cusp_speeds: [0f64; 13],
        ascmc_speeds: [0f64; 10],
    })
}

/// House position — pure-Rust.
/// Compute house position (1.0–12.0) for a body at ecliptic coordinates `xpin`.
///
/// Uses cusp interpolation: finds which house pair contains the body's longitude
/// and returns fractional house number.
pub fn house_pos(
    armc: f64,
    geolat: f64,
    eps: f64,
    hsys: HouseSystem,
    xpin: [f64; 2],
) -> Result<f64> {
    let r = crate::astronomy::houses_armc(armc, geolat, eps, hsys.as_raw());
    let lon = xpin[0];
    let cusps = &r.cusps;
    for h in 1usize..=12 {
        let c1 = cusps[h];
        let c2 = if h < 12 { cusps[h + 1] } else { cusps[1] };
        // Angular span of this house
        let span = (c2 - c1 + 360.0).rem_euclid(360.0);
        // Distance into this house
        let dist = (lon - c1 + 360.0).rem_euclid(360.0);
        if dist <= span || span < 0.01 {
            let frac = if span > 0.01 {
                (dist / span).clamp(0.0, 1.0)
            } else {
                0.0
            };
            return Ok(h as f64 + frac);
        }
    }
    // Fallback: return house 1
    Ok(1.0)
}

/// House cusps from ARMC (sidereal time in degrees), latitude, and ecliptic obliquity.
/// House cusps from ARMC (sidereal time in degrees), geographic latitude,
/// ecliptic obliquity, and house system byte.
#[must_use]
pub fn houses_from_armc(armc: f64, geolat: f64, eps: f64, hsys: HouseSystem) -> HouseResult {
    crate::astronomy::houses_from_armc(armc, geolat, eps, hsys.as_raw())
}
