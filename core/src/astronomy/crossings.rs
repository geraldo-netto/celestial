//! Longitude crossing searches.
//!
//! Find the Julian day when a body's ecliptic longitude crosses a target value.
//! Uses a bracketed bisection / Illinois method guaranteed to converge.
#![allow(dead_code)]

use super::calc_ut;

/// Search for the first time after `jd_start` that body `body` crosses
/// ecliptic longitude `x2cross` (degrees).
///
/// Returns `Some(jd_ut)` or `None` if not found within the search window.
pub fn find_crossing(
    body: i32,
    x2cross: f64,
    jd_start: f64,
    forward: bool, // true = forward in time, false = backward
    flags: i32,
) -> Option<f64> {
    // Step size: choose based on body speed
    let step = match body {
        1 => 1.0,  // Moon: fast, 1 day steps
        0 => 5.0,  // Sun
        2 => 5.0,  // Mercury
        3 => 10.0, // Venus
        4 => 20.0, // Mars
        _ => 50.0, // outer planets
    };
    let dir = if forward { 1.0 } else { -1.0 };

    // Get body longitude, wrapping to range centred on x2cross
    let lon_at = |jd: f64| -> Option<f64> {
        let pos = calc_ut(jd, body, flags).ok()?;
        Some(pos.lon)
    };

    // Signed angular distance from `lon` to `x2cross`, in (-180, 180].
    // Positive = lon is "behind" the target (hasn't crossed it going forward yet).
    // Has EXACTLY ONE zero per revolution (at x2cross); the antipodal "crossing"
    // produces a ±180° discontinuity, which is rejected by the |d1-d0|>180 filter.
    let diff = |lon: f64| -> f64 { (x2cross - lon + 540.0).rem_euclid(360.0) - 180.0 };

    let mut jd = jd_start;
    let max_jd = jd_start + dir * 400.0; // search at most ~1 year ahead

    let mut d0 = diff(lon_at(jd)?);

    let max_iter = (400.0 / step) as i32 + 10;
    let mut bracket: Option<(f64, f64, f64, f64)> = None; // (jd_a, d_a, jd_b, d_b)

    for _ in 0..max_iter {
        jd += dir * step;
        if (jd - jd_start) * dir > (max_jd - jd_start) * dir {
            break;
        }

        let lon_new = lon_at(jd)?;
        let d1 = diff(lon_new);

        if d0 * d1 <= 0.0 {
            // Sign change → but skip antipodal jumps (|diff| change > 180°)
            if (d1 - d0).abs() < 180.0 {
                bracket = Some((jd - dir * step, d0, jd, d1));
                break;
            }
        }
        d0 = d1;
        let _lon = lon_new;
    }

    let (mut ja, mut da, mut jb, _db) = bracket?;

    // Bisection to refine
    for _ in 0..60 {
        let jm = (ja + jb) / 2.0;
        let lon_m = lon_at(jm)?;
        let dm = diff(lon_m);

        if dm.abs() < 1e-8 || (jb - ja).abs() < 1e-8 / 86400.0 {
            return Some(jm);
        }
        if da * dm <= 0.0 {
            jb = jm;
        } else {
            ja = jm;
            da = dm;
        }
    }

    Some((ja + jb) / 2.0)
}

/// Sun crosses longitude `x2cross` after `jd_start`.
pub fn solcross(x2cross: f64, jd_et: f64, flags: i32) -> Option<f64> {
    find_crossing(0, x2cross, jd_et, true, flags)
}

/// Sun crosses longitude `x2cross` after `jd_start` (UT input).
pub fn solcross_ut(x2cross: f64, jd_ut: f64, flags: i32) -> Option<f64> {
    solcross(x2cross, jd_ut, flags)
}

/// Moon crosses longitude `x2cross` after `jd_start`.
pub fn mooncross(x2cross: f64, jd_et: f64, flags: i32) -> Option<f64> {
    find_crossing(1, x2cross, jd_et, true, flags)
}

/// Moon crosses longitude `x2cross` (UT).
pub fn mooncross_ut(x2cross: f64, jd_ut: f64, flags: i32) -> Option<f64> {
    mooncross(x2cross, jd_ut, flags)
}

#[derive(Debug, Clone, Copy)]
pub struct NodeCrossing {
    pub jd_cross: f64,
    pub xlon: f64,
}

/// Next Moon–node crossing (ascending node ≈ 0° latitude crossing going north).
/// Returns the Julian day and the ecliptic longitude of the crossing.
pub fn mooncross_node(jd_et: f64, _flags: i32) -> Option<NodeCrossing> {
    // The Moon crosses its ascending node when its latitude goes from - to +.
    // Search for latitude = 0 with northward velocity.
    let lat_at = |jd: f64| -> f64 {
        crate::astronomy::calc_ut(jd, 1, 0)
            .map(|p| p.lat)
            .unwrap_or(0.0)
    };

    let mut jd = jd_et;
    let step = 0.5; // half-day steps (Moon latitude period ~14 days)
    let max_jd = jd + 30.0; // search 30 days

    let mut lat0 = lat_at(jd);

    while jd < max_jd {
        let lat1 = lat_at(jd + step);
        // ascending node: latitude crosses zero from - to +
        if lat0 < 0.0 && lat1 >= 0.0 {
            // bisect
            let (mut ja, mut jb) = (jd, jd + step);
            for _ in 0..50 {
                let jm = (ja + jb) / 2.0;
                let lm = lat_at(jm);
                if lm.abs() < 1e-8 {
                    break;
                }
                if lm < 0.0 {
                    ja = jm;
                } else {
                    jb = jm;
                }
            }
            let jcross = (ja + jb) / 2.0;
            let lon = crate::astronomy::calc_ut(jcross, 1, 0)
                .map(|p| p.lon)
                .unwrap_or(0.0);
            return Some(NodeCrossing {
                jd_cross: jcross,
                xlon: lon,
            });
        }
        lat0 = lat1;
        jd += step;
    }
    None
}

/// Next Moon–node crossing (UT).
pub fn mooncross_node_ut(jd_ut: f64, flags: i32) -> Option<NodeCrossing> {
    mooncross_node(jd_ut, flags)
}

/// Planet crosses longitude `x2cross` heliocentrically.
pub fn helio_cross(body: i32, x2cross: f64, jd_et: f64, flags: i32, forward: bool) -> Option<f64> {
    // Use the heliocentric flag
    let hflags = flags | crate::astronomy::flag::FLG_HELCTR as i32;
    find_crossing(body, x2cross, jd_et, forward, hflags)
}

/// Heliocentric crossing (UT).
pub fn helio_cross_ut(
    body: i32,
    x2cross: f64,
    jd_ut: f64,
    flags: i32,
    forward: bool,
) -> Option<f64> {
    helio_cross(body, x2cross, jd_ut, flags, forward)
}

/// Like [`find_crossing`] but with an explicit search window in days.
pub(crate) fn find_crossing_window(
    body: i32,
    x2cross: f64,
    jd_start: f64,
    forward: bool,
    flags: i32,
    window: f64,
) -> Option<f64> {
    let step = match body {
        1 => 1.0,   // Moon
        0 => 5.0,   // Sun
        2 => 5.0,   // Mercury
        3 => 10.0,  // Venus
        4 => 20.0,  // Mars
        5 => 50.0,  // Jupiter
        6 => 100.0, // Saturn
        _ => 50.0,
    };
    let dir = if forward { 1.0 } else { -1.0 };

    let lon_at = |jd: f64| -> Option<f64> {
        let pos = calc_ut(jd, body, flags).ok()?;
        Some(pos.lon)
    };
    let diff = |lon: f64| -> f64 { (x2cross - lon + 540.0).rem_euclid(360.0) - 180.0 };

    let mut jd = jd_start;
    let limit = jd_start + dir * window;
    let mut d0 = diff(lon_at(jd)?);

    let max_iter = (window / step) as i32 + 10;
    for _ in 0..max_iter {
        jd += dir * step;
        if (jd - limit) * dir > 0.0 {
            break;
        }
        let lon_new = lon_at(jd)?;
        let d1 = diff(lon_new);
        if d0 * d1 <= 0.0 && (d1 - d0).abs() < 180.0 {
            let (mut ja, mut da, mut jb) = (jd - dir * step, d0, jd);
            for _ in 0..60 {
                let jm = (ja + jb) / 2.0;
                let dm = diff(lon_at(jm)?);
                if dm.abs() < 1e-8 {
                    return Some(jm);
                }
                if da * dm <= 0.0 {
                    jb = jm;
                } else {
                    ja = jm;
                    da = dm;
                }
            }
            return Some((ja + jb) / 2.0);
        }
        d0 = d1;
    }
    None
}
