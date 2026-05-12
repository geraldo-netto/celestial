//! Longitude crossing searches.
//!
//! Find the Julian day when a body's ecliptic longitude crosses a target value.
//! Uses a bracketed bisection / Illinois method guaranteed to converge.
#![allow(dead_code)]

use super::calc_ut;
use crate::PlanetPos;

/// Search for the first time after `jd_start` that body `body` crosses
/// ecliptic longitude `x2cross` (degrees).
///
/// Returns `Some(jd_ut)` or `None` if not found within the search window.
/// Step size in days based on body speed (Moon=1, Sun/Mercury=5, …).
fn crossing_step(body: i32) -> f64 {
    match body {
        1 => 1.0,
        0 | 2 => 5.0,
        3 => 10.0,
        4 => 20.0,
        _ => 50.0,
    }
}

/// Walk forward/backward in `step`-day increments until a sign change in `diff`
/// is detected. Returns `(jd_a, d_a, jd_b, d_b)` bracket, or `None`.
fn bracket_crossing<F, D>(
    jd_start: f64,
    dir: f64,
    step: f64,
    lon_at: &F,
    diff: &D,
) -> Option<(f64, f64, f64, f64)>
where
    F: Fn(f64) -> Option<f64>,
    D: Fn(f64) -> f64,
{
    let mut jd = jd_start;
    let mut d0 = diff(lon_at(jd)?);
    let max_jd = jd_start + dir * 400.0;
    let max_iter = (400.0 / step) as i32 + 10;
    for _ in 0..max_iter {
        jd += dir * step;
        if (jd - jd_start) * dir > (max_jd - jd_start) * dir {
            return None;
        }
        let d1 = diff(lon_at(jd)?);
        // Sign change & not an antipodal ±180° jump
        if d0 * d1 <= 0.0 && (d1 - d0).abs() < 180.0 {
            return Some((jd - dir * step, d0, jd, d1));
        }
        d0 = d1;
    }
    None
}

/// Brent–Dekker refinement on a confirmed bracket. Combines inverse-quadratic
/// interpolation, secant, and bisection — typically converges in 5–7 evals
/// vs. bisection's 25–30 to the same tolerance.
///
/// Convergence criteria match the previous bisection: `|d| < 1e-8°` (~0.04″)
/// OR bracket width `< 1e-8 / 86400` days (~0.01 µs).
/// Inverse-quadratic interpolation when fa,fb,fc are distinct; secant otherwise.
fn brent_interpolate(a: f64, fa: f64, b: f64, fb: f64, c: f64, fc: f64) -> f64 {
    if (fa - fc).abs() > f64::EPSILON && (fb - fc).abs() > f64::EPSILON {
        a * fb * fc / ((fa - fb) * (fa - fc))
            + b * fa * fc / ((fb - fa) * (fb - fc))
            + c * fa * fb / ((fc - fa) * (fc - fb))
    } else {
        b - fb * (b - a) / (fb - fa)
    }
}

/// Brent's safeguard: should we fall back to bisection?
fn brent_should_bisect(
    s: f64,
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    used_bisect: bool,
    tol_x: f64,
) -> bool {
    let lo = (3.0 * a + b) / 4.0;
    let cond1 = (s - lo) * (s - b) > 0.0;
    let prev_step = if used_bisect {
        (b - c).abs()
    } else {
        (c - d).abs()
    };
    let cond_step = (s - b).abs() >= prev_step / 2.0;
    let cond_tiny = prev_step < tol_x;
    cond1 || cond_step || cond_tiny
}

fn refine_crossing<F, D>(
    mut a: f64,
    mut fa: f64,
    mut b: f64,
    fb_in: f64,
    lon_at: &F,
    diff: &D,
) -> Option<f64>
where
    F: Fn(f64) -> Option<f64>,
    D: Fn(f64) -> f64,
{
    let mut fb = fb_in;
    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }
    let mut c = a;
    let mut fc = fa;
    let mut d = a;
    let mut used_bisect = true;

    const TOL_F: f64 = 1.0e-8;
    const TOL_X: f64 = 1.0e-8 / 86400.0;
    const MAX_ITER: usize = 60;

    for _ in 0..MAX_ITER {
        if fb.abs() < TOL_F || (b - a).abs() < TOL_X {
            return Some(b);
        }

        let s_try = brent_interpolate(a, fa, b, fb, c, fc);
        let s = if brent_should_bisect(s_try, a, b, c, d, used_bisect, TOL_X) {
            used_bisect = true;
            (a + b) / 2.0
        } else {
            used_bisect = false;
            s_try
        };

        let fs = diff(lon_at(s)?);
        d = c;
        c = b;
        fc = fb;

        if fa * fs < 0.0 {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }

        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
    }
    Some(b)
}

pub fn find_crossing(
    body: i32,
    x2cross: f64,
    jd_start: f64,
    forward: bool, // true = forward in time, false = backward
    flags: i32,
) -> Option<f64> {
    let step = crossing_step(body);
    let dir = if forward { 1.0 } else { -1.0 };

    let lon_at = |jd: f64| -> Option<f64> {
        let pos = calc_ut(jd, body, flags).ok()?;
        Some(pos.lon)
    };

    // Signed angular distance from `lon` to `x2cross`, in (-180, 180].
    // Positive = lon is "behind" the target (hasn't crossed it going forward yet).
    // Has EXACTLY ONE zero per revolution (at x2cross); the antipodal "crossing"
    // produces a ±180° discontinuity, which is rejected by the |d1-d0|>180 filter.
    let diff = |lon: f64| -> f64 { (x2cross - lon + 540.0).rem_euclid(360.0) - 180.0 };

    let (ja, da, jb, db) = bracket_crossing(jd_start, dir, step, &lon_at, &diff)?;
    refine_crossing(ja, da, jb, db, &lon_at, &diff)
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
/// Bisect to find the latitude=0 crossing inside [ja, jb] given the upper-end probe.
fn bisect_node_crossing<F>(mut ja: f64, mut jb: f64, p_hi: PlanetPos, pos_at: &F) -> NodeCrossing
where
    F: Fn(f64) -> Option<PlanetPos>,
{
    let mut pm = p_hi;
    for _ in 0..50 {
        let jm = (ja + jb) / 2.0;
        pm = match pos_at(jm) {
            Some(p) => p,
            None => break,
        };
        if pm.lat.abs() < 1e-8 {
            break;
        }
        if pm.lat < 0.0 {
            ja = jm;
        } else {
            jb = jm;
        }
    }
    NodeCrossing {
        jd_cross: (ja + jb) / 2.0,
        xlon: pm.lon,
    }
}

pub fn mooncross_node(jd_et: f64, _flags: i32) -> Option<NodeCrossing> {
    let pos_at = |jd: f64| crate::astronomy::calc_ut(jd, 1, 0).ok();

    let mut jd = jd_et;
    let step = 0.5;
    let max_jd = jd + 30.0;

    let mut lat0 = pos_at(jd).map_or(0.0, |p| p.lat);

    while jd < max_jd {
        let Some(p1) = pos_at(jd + step) else {
            jd += step;
            continue;
        };
        let lat1 = p1.lat;
        if lat0 < 0.0 && lat1 >= 0.0 {
            return Some(bisect_node_crossing(jd, jd + step, p1, &pos_at));
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

/// Like [`find_crossing`] but with an explicit search window in days. Used
/// by callers (e.g. sign-ingress for outer planets) where the canonical
/// 400-day window is too short.
pub(crate) fn find_crossing_window(
    body: i32,
    x2cross: f64,
    jd_start: f64,
    forward: bool,
    flags: i32,
    window: f64,
) -> Option<f64> {
    // Step is tuned per body: fast bodies (Moon) use 1d; outer planets use up
    // to 100d so the bracket walk doesn't dominate.
    let step = match body {
        1 => 1.0,
        0 | 2 => 5.0,
        3 => 10.0,
        4 => 20.0,
        6 => 100.0,
        _ => 50.0, // 5 (Jupiter) and unknown bodies
    };
    let dir = if forward { 1.0 } else { -1.0 };

    let lon_at = |jd: f64| -> Option<f64> {
        let pos = calc_ut(jd, body, flags).ok()?;
        Some(pos.lon)
    };
    let diff = |lon: f64| -> f64 { (x2cross - lon + 540.0).rem_euclid(360.0) - 180.0 };

    // Walk the window in `step`-day chunks looking for a non-antipodal sign change.
    let mut jd = jd_start;
    let limit = jd_start + dir * window;
    let mut d0 = diff(lon_at(jd)?);
    let max_iter = (window / step) as i32 + 10;
    for _ in 0..max_iter {
        jd += dir * step;
        if (jd - limit) * dir > 0.0 {
            return None;
        }
        let d1 = diff(lon_at(jd)?);
        if d0 * d1 <= 0.0 && (d1 - d0).abs() < 180.0 {
            // Found the bracket — hand off to the shared Brent-based refiner.
            return refine_crossing(jd - dir * step, d0, jd, d1, &lon_at, &diff);
        }
        d0 = d1;
    }
    None
}
