//! Longitude crossing searches.
//!
//! Find the Julian day when a body's ecliptic longitude crosses a target value.
//! Uses a bracketed bisection / Illinois method guaranteed to converge.
#![allow(dead_code)]

use super::calc_ut;
use crate::units::JulianDay;
use crate::PlanetPos;

const CROSSING_TOL_F: f64 = 1.0e-8;
const CROSSING_TOL_X: f64 = 1.0e-8 / 86400.0;

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

fn crossing_scan_flags(flags: i32) -> i32 {
    flags & !(crate::constants::FLG_SPEED | crate::constants::FLG_SPEED3)
}

fn crossing_window_step(body: i32) -> f64 {
    if body == 6 {
        100.0
    } else {
        crossing_step(body)
    }
}

fn crossing_iterations(window: f64, step: f64) -> Option<usize> {
    match step.partial_cmp(&0.0) {
        Some(std::cmp::Ordering::Greater) => Some((window / step).floor() as usize),
        _ => None,
    }
}

fn should_swap_residuals(first: f64, second: f64) -> bool {
    first.abs() < second.abs()
}

fn bracket_width(first: f64, second: f64) -> f64 {
    (second - first).abs()
}

fn crossing_converged(residual: f64, width: f64) -> bool {
    residual.abs() < CROSSING_TOL_F || width < CROSSING_TOL_X
}

fn midpoint(first: f64, second: f64) -> f64 {
    (first + second) / 2.0
}

fn opposite_sign(first: f64, second: f64) -> bool {
    (first < 0.0 && second > 0.0) || (first > 0.0 && second < 0.0)
}

fn brackets_root(first: f64, second: f64) -> bool {
    first == 0.0 || second == 0.0 || opposite_sign(first, second)
}

fn signed_angular_distance(target: f64, longitude: f64) -> f64 {
    (target - longitude + 540.0).rem_euclid(360.0) - 180.0
}

fn node_crossing_converged(latitude: f64) -> bool {
    latitude.abs() < 1.0e-8
}

fn is_south(latitude: f64) -> bool {
    latitude < 0.0
}

fn node_scan_jd(jd_start: f64, step_index: i32, step: f64) -> f64 {
    jd_start + f64::from(step_index) * step
}

/// Walk forward/backward in `step`-day increments until a sign change in `diff`
/// is detected. Returns `(jd_a, d_a, jd_b, d_b)` bracket, or `None`.
fn bracket_crossing<F, D>(
    jd_start: f64,
    dir: f64,
    step: f64,
    window: f64,
    lon_at: &F,
    diff: &D,
) -> Option<(f64, f64, f64, f64)>
where
    F: Fn(f64) -> Option<f64>,
    D: Fn(f64) -> f64,
{
    let mut jd = jd_start;
    let mut d0 = diff(lon_at(jd)?);
    let max_iter = crossing_iterations(window, step)?;
    for _ in 0..max_iter {
        jd += dir * step;
        let d1 = diff(lon_at(jd)?);
        // Sign change & not an antipodal ±180° jump
        if brackets_root(d0, d1) && (d1 - d0).abs() < 180.0 {
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
    if should_swap_residuals(fa, fb) {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }
    let mut c = a;
    let mut fc = fa;
    let mut d = a;
    let mut used_bisect = true;

    const MAX_ITER: usize = 60;

    for _ in 0..MAX_ITER {
        if crossing_converged(fb, bracket_width(a, b)) {
            return Some(b);
        }

        let s_try = brent_interpolate(a, fa, b, fb, c, fc);
        let s = if brent_should_bisect(s_try, a, b, c, d, used_bisect, CROSSING_TOL_X) {
            used_bisect = true;
            midpoint(a, b)
        } else {
            used_bisect = false;
            s_try
        };

        let fs = diff(lon_at(s)?);
        d = c;
        c = b;
        fc = fb;

        if opposite_sign(fa, fs) {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }

        if should_swap_residuals(fa, fb) {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
    }
    Some(b)
}

pub fn find_crossing(
    body: i32,
    x2cross: f64,
    jd_start: JulianDay,
    forward: bool, // true = forward in time, false = backward
    flags: i32,
) -> Option<f64> {
    let jd_start: f64 = jd_start.into();
    let step = crossing_step(body);
    let dir = if forward { 1.0 } else { -1.0 };

    // The bracket/refine scan only reads `pos.lon`; computing speed (the
    // SPEED/SPEED3 bits, set by the DEFAULT flag set) would do ~2 extra
    // full position evaluations per probe over potentially 100k+ probes.
    let scan_flags = crossing_scan_flags(flags);

    let lon_at = |jd: f64| -> Option<f64> {
        let pos = calc_ut(JulianDay::new(jd), body, scan_flags).ok()?;
        Some(pos.lon)
    };

    // Signed angular distance from `lon` to `x2cross`, in [-180, 180).
    // Positive = lon is "behind" the target (hasn't crossed it going forward yet).
    // Has EXACTLY ONE zero per revolution (at x2cross); the antipodal "crossing"
    // produces a ±180° discontinuity, which is rejected by the |d1-d0|>180 filter.
    let diff = |lon: f64| -> f64 { signed_angular_distance(x2cross, lon) };

    let (ja, da, jb, db) = bracket_crossing(jd_start, dir, step, 400.0, &lon_at, &diff)?;
    refine_crossing(ja, da, jb, db, &lon_at, &diff)
}

/// Sun crosses longitude `x2cross` after `jd_start`.
pub fn solcross(x2cross: f64, jd_et: JulianDay, flags: i32) -> Option<f64> {
    let jd_et: f64 = jd_et.into();
    find_crossing(0, x2cross, JulianDay::new(jd_et), true, flags)
}

/// Sun crosses longitude `x2cross` after `jd_start` (UT input).
pub fn solcross_ut(x2cross: f64, jd_ut: JulianDay, flags: i32) -> Option<f64> {
    let jd_ut: f64 = jd_ut.into();
    solcross(x2cross, JulianDay::new(jd_ut), flags)
}

/// Moon crosses longitude `x2cross` after `jd_start`.
pub fn mooncross(x2cross: f64, jd_et: JulianDay, flags: i32) -> Option<f64> {
    let jd_et: f64 = jd_et.into();
    find_crossing(1, x2cross, JulianDay::new(jd_et), true, flags)
}

/// Moon crosses longitude `x2cross` (UT).
pub fn mooncross_ut(x2cross: f64, jd_ut: JulianDay, flags: i32) -> Option<f64> {
    let jd_ut: f64 = jd_ut.into();
    mooncross(x2cross, JulianDay::new(jd_ut), flags)
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
        let jm = midpoint(ja, jb);
        pm = match pos_at(jm) {
            Some(p) => p,
            None => break,
        };
        if node_crossing_converged(pm.lat) {
            break;
        }
        if is_south(pm.lat) {
            ja = jm;
        } else {
            jb = jm;
        }
    }
    NodeCrossing {
        jd_cross: midpoint(ja, jb),
        xlon: pm.lon,
    }
}

pub fn mooncross_node(jd_et: JulianDay, _flags: i32) -> Option<NodeCrossing> {
    let jd_et: f64 = jd_et.into();
    let pos_at = |jd: f64| crate::astronomy::calc_ut(JulianDay::new(jd), 1, 0).ok();

    let step = 0.5;
    let mut lat0 = pos_at(jd_et).map_or(0.0, |p| p.lat);

    for step_index in 0..60 {
        let jd = node_scan_jd(jd_et, step_index, step);
        let Some(p1) = pos_at(jd + step) else {
            continue;
        };
        let lat1 = p1.lat;
        if is_south(lat0) && !is_south(lat1) {
            return Some(bisect_node_crossing(jd, jd + step, p1, &pos_at));
        }
        lat0 = lat1;
    }
    None
}

/// Planet crosses longitude `x2cross` heliocentrically.
pub fn helio_cross(
    body: i32,
    x2cross: f64,
    jd_et: JulianDay,
    flags: i32,
    forward: bool,
) -> Option<f64> {
    let jd_et: f64 = jd_et.into();
    // Use the heliocentric flag
    let hflags = flags | crate::astronomy::flag::FLG_HELCTR as i32;
    find_crossing(body, x2cross, JulianDay::new(jd_et), forward, hflags)
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
    let step = crossing_window_step(body);
    let dir = if forward { 1.0 } else { -1.0 };

    // Scan reads only `pos.lon`; drop the SPEED bits (see `find_crossing`).
    let scan_flags = crossing_scan_flags(flags);

    let lon_at = |jd: f64| -> Option<f64> {
        let pos = calc_ut(JulianDay::new(jd), body, scan_flags).ok()?;
        Some(pos.lon)
    };
    let diff = |lon: f64| -> f64 { signed_angular_distance(x2cross, lon) };

    let (ja, da, jb, db) = bracket_crossing(jd_start, dir, step, window, &lon_at, &diff)?;
    refine_crossing(ja, da, jb, db, &lon_at, &diff)
}

#[cfg(test)]
mod cov_tests {
    use super::*;
    use crate::astronomy::flag::FLG_BUILTIN;
    use std::cell::Cell;

    #[test]
    fn crossing_steps_are_body_tuned() {
        let cases = [
            (1, 1.0),
            (0, 5.0),
            (2, 5.0),
            (3, 10.0),
            (4, 20.0),
            (5, 50.0),
        ];
        for (body, expected) in cases {
            assert_eq!(crossing_step(body), expected);
        }
        assert_eq!(crossing_window_step(6), 100.0);
        assert_eq!(crossing_window_step(5), 50.0);
        assert_eq!(crossing_iterations(400.0, 100.0), Some(4));
        assert_eq!(crossing_iterations(400.0, 0.0), None);
        assert_eq!(crossing_iterations(400.0, -1.0), None);
        assert_eq!(crossing_iterations(400.0, f64::NAN), None);
        let all = crate::constants::FLG_BUILTIN
            | crate::constants::FLG_SPEED
            | crate::constants::FLG_SPEED3;
        assert_eq!(crossing_scan_flags(all), crate::constants::FLG_BUILTIN);
        assert_eq!(
            crossing_scan_flags(crate::constants::FLG_BUILTIN),
            crate::constants::FLG_BUILTIN
        );
    }

    #[test]
    fn crossing_helpers_pin_boundaries() {
        assert!(should_swap_residuals(1.0, 2.0));
        assert!(!should_swap_residuals(2.0, 1.0));
        assert!(!should_swap_residuals(-1.0, 1.0));
        assert_eq!(bracket_width(3.0, 1.0), 2.0);
        assert!(crossing_converged(CROSSING_TOL_F / 2.0, 1.0));
        assert!(crossing_converged(1.0, CROSSING_TOL_X / 2.0));
        assert!(!crossing_converged(CROSSING_TOL_F, CROSSING_TOL_X));
        assert_eq!(midpoint(2.0, 6.0), 4.0);
        assert!(opposite_sign(-1.0, 1.0));
        assert!(opposite_sign(1.0, -1.0));
        assert!(!opposite_sign(-1.0, 0.0));
        assert!(!opposite_sign(-0.0, 1.0));
        assert!(!opposite_sign(1.0, 1.0));
        assert!(!opposite_sign(f64::NAN, -1.0));
        assert_eq!(signed_angular_distance(10.0, 350.0), 20.0);
        assert_eq!(signed_angular_distance(350.0, 10.0), -20.0);
        assert_eq!(signed_angular_distance(190.0, 10.0), -180.0);
        assert_eq!(signed_angular_distance(10.0, 10.0), 0.0);
        assert!(node_crossing_converged(0.5e-8));
        assert!(!node_crossing_converged(1.0e-8));
        assert!(is_south(-1.0));
        assert!(!is_south(0.0));
        assert!(!is_south(1.0));
        assert_eq!(node_scan_jd(100.0, 3, 0.5), 101.5);
    }

    #[test]
    fn signed_distance_preserves_wrap_neighbors() {
        let below_wrap = 360.0_f64.next_down();
        let above_wrap = 360.0_f64.next_up();
        let wrap_ulp = 360.0 - below_wrap;
        assert_eq!(signed_angular_distance(0.0, below_wrap), wrap_ulp);
        assert_eq!(signed_angular_distance(0.0, above_wrap), 360.0 - above_wrap);
        assert_eq!(signed_angular_distance(360.0, 0.0), 0.0);
        assert_eq!(signed_angular_distance(0.0, 360.0), 0.0);
    }

    #[test]
    fn bracket_crossing_obeys_step_and_window() {
        let linear = |jd| Some(jd);
        let diff = |lon| lon - 5.0;
        assert_eq!(
            bracket_crossing(0.0, 1.0, 2.0, 400.0, &linear, &diff),
            Some((4.0, -1.0, 6.0, 1.0))
        );
        assert_eq!(
            bracket_crossing(10.0, -1.0, 2.0, 400.0, &linear, &diff),
            Some((6.0, 1.0, 4.0, -1.0))
        );
        assert_eq!(
            bracket_crossing(0.0, 1.0, 1.0, 400.0, &linear, &diff),
            Some((4.0, -1.0, 5.0, 0.0))
        );
        assert_eq!(
            bracket_crossing(10.0, -1.0, 1.0, 400.0, &linear, &diff),
            Some((6.0, 1.0, 5.0, 0.0))
        );
        assert!(
            bracket_crossing(0.0, 1.0, 2.0, 400.0, &linear, &|lon| lon * 90.0 - 90.0).is_none()
        );

        let calls = Cell::new(0);
        let constant = |_| {
            calls.set(calls.get() + 1);
            Some(1.0)
        };
        assert!(bracket_crossing(0.0, 1.0, 100.0, 400.0, &constant, &|_| 1.0).is_none());
        assert_eq!(calls.get(), 5);
    }

    #[test]
    fn sign_predicates_ignore_product_underflow() {
        let (tiny_neg, tiny_pos) = (-1e-200, 1e-200);
        assert_eq!(tiny_neg * tiny_pos, -0.0);
        assert_eq!(tiny_pos * tiny_pos, 0.0);
        assert!(opposite_sign(tiny_neg, tiny_pos));
        assert!(brackets_root(tiny_neg, tiny_pos));
        assert!(!opposite_sign(tiny_pos, tiny_pos));
        assert!(!brackets_root(tiny_pos, tiny_pos));
        assert!(!brackets_root(tiny_neg, tiny_neg));
        assert!(opposite_sign(-1e200, 1e200));
        assert!(!opposite_sign(1e200, 1e200));
    }

    #[test]
    fn brackets_root_keeps_zero_and_nan_semantics() {
        assert!(brackets_root(0.0, 1.0));
        assert!(brackets_root(-1.0, 0.0));
        assert!(brackets_root(-0.0, -1.0));
        assert!(brackets_root(1.0, -0.0));
        assert!(!brackets_root(1.0, 1.0));
        assert!(!brackets_root(-1.0, -1.0));
        assert!(!brackets_root(f64::NAN, -1.0));
        assert!(!brackets_root(-1.0, f64::NAN));
    }

    #[test]
    fn opposite_sign_excludes_zero_endpoints() {
        assert!(!opposite_sign(0.0, -1.0));
        assert!(!opposite_sign(1.0, 0.0));
        assert!(!opposite_sign(0.0, 1.0));
        assert!(!opposite_sign(-1.0, -0.0));
        assert!(!opposite_sign(0.0, 0.0));
        assert!(!opposite_sign(-1.0, f64::NAN));
    }

    #[test]
    fn bracket_crossing_rejects_underflowed_same_sign_pairs() {
        let linear = |jd| Some(jd);
        let crossing = |lon: f64| (lon - 5.0) * 1e-200;
        assert_eq!(
            bracket_crossing(0.0, 1.0, 2.0, 400.0, &linear, &crossing),
            Some((4.0, -1e-200, 6.0, 1e-200))
        );
        let same_sign = |lon: f64| (lon + 1.0) * 1e-200;
        assert!(bracket_crossing(0.0, 1.0, 2.0, 400.0, &linear, &same_sign).is_none());
    }

    #[test]
    fn brent_interpolation_selects_iqi_and_secant() {
        assert_eq!(brent_interpolate(1.0, 2.0, 4.0, -1.0, 6.0, 3.0), 0.0);
        assert_eq!(brent_interpolate(1.0, 2.0, 4.0, -1.0, 6.0, 2.0), 3.0);

        let secant = |a, fa, b, fb| b - fb * (b - a) / (fb - fa);
        let epsilon = f64::EPSILON;
        let boundary_cases = [
            (1.0, 1.0 + epsilon, 4.0, -1.0, 6.0, 1.0),
            (1.0, 3.0, 4.0, 1.0 + epsilon, 6.0, 1.0),
            (1.0, 3.0, 4.0, 1.0, 6.0, 1.0),
        ];
        for (a, fa, b, fb, c, fc) in boundary_cases {
            assert_eq!(brent_interpolate(a, fa, b, fb, c, fc), secant(a, fa, b, fb));
        }

        let fingerprints = [
            ((1.25, -2.5, 4.75, 1.5, 7.25, 4.0), 0x400a_ec4e_c4ec_4ec4),
            ((2.0, -3.0, 5.0, 2.0, 9.0, 6.0), 0x400c_4444_4444_4444),
            ((-2.0, 4.0, 3.0, -1.0, 8.0, -5.0), 0x3ffe_38e3_8e38_e38e),
        ];
        for ((a, fa, b, fb, c, fc), expected) in fingerprints {
            assert_eq!(brent_interpolate(a, fa, b, fb, c, fc).to_bits(), expected);
        }
    }

    #[test]
    fn brent_safeguard_covers_each_decision() {
        assert!(!brent_should_bisect(3.5, 0.0, 4.0, 2.0, 1.0, true, 0.1));
        assert!(brent_should_bisect(0.0, 0.0, 4.0, -100.0, 0.0, true, 0.1));
        assert!(brent_should_bisect(3.0, 0.0, 4.0, 2.0, 0.0, true, 0.1));
        assert!(brent_should_bisect(3.99, 0.0, 4.0, 3.9, 0.0, true, 0.2));
        assert!(!brent_should_bisect(3.5, 0.0, 4.0, 3.0, 1.0, false, 0.1));
        assert!(!brent_should_bisect(4.0, 0.0, 4.0, -100.0, 0.0, true, 0.1));
        assert!(!brent_should_bisect(3.5, 0.0, 4.0, 2.0, 0.0, true, 2.0));
        assert!(brent_should_bisect(2.9, 2.0, 6.0, -4.0, 0.0, true, 0.1));
        assert!(!brent_should_bisect(3.5, 0.0, 4.0, 1.0, -1.0, false, 0.1));
    }

    #[test]
    fn refine_crossing_converges_from_either_bracket_order() {
        let value = |x| Some(x);
        let diff = |x| x * x - 2.0;
        let forward = refine_crossing(1.0, -1.0, 2.0, 2.0, &value, &diff).unwrap();
        let reversed = refine_crossing(2.0, 2.0, 1.0, -1.0, &value, &diff).unwrap();
        assert_eq!(forward.to_bits(), 0x3ff6_a09e_667f_3c89);
        assert_eq!(reversed.to_bits(), 0x3ff6_a09e_667f_3c89);
    }

    #[test]
    fn helio_cross_returns_some_for_mars_j2000() {
        let jd = JulianDay::new(2_451_545.0);
        let crossing = helio_cross(2, 90.0, jd, FLG_BUILTIN as i32, true).unwrap();
        assert_eq!(crossing, 2_451_592.263_018_415);
        let heliocentric = FLG_BUILTIN as i32 | crate::astronomy::flag::FLG_HELCTR as i32;
        assert_eq!(helio_cross(2, 90.0, jd, heliocentric, true), Some(crossing));
    }

    #[test]
    fn mooncross_node_within_draconic_month() {
        let jd = JulianDay::new(2_451_545.0);
        let crossing = mooncross_node(jd, FLG_BUILTIN as i32).unwrap();
        assert_eq!(crossing.jd_cross, 2_451_564.912_438_109_5);
        assert_eq!(crossing.xlon, 123.678_251_251_218_46);
    }

    #[test]
    fn public_crossings_match_regressions() {
        let jd = JulianDay::new(2_451_545.0);
        let flags = FLG_BUILTIN as i32;
        let forward = find_crossing(0, 90.0, jd, true, flags).unwrap();
        let backward = find_crossing(0, 90.0, jd, false, flags).unwrap();
        assert_eq!(forward, 2_451_716.569_027_677_6);
        assert_eq!(backward, 2_451_351.320_080_211);
        assert_eq!(solcross(90.0, jd, flags), Some(forward));
        assert_eq!(solcross_ut(90.0, jd, flags), Some(forward));

        let moon = Some(2_451_562.667_374_155);
        assert_eq!(mooncross(90.0, jd, flags), moon);
        assert_eq!(mooncross_ut(90.0, jd, flags), moon);
    }

    #[test]
    fn crossing_window_matches_regression_and_limit() {
        let flags = FLG_BUILTIN as i32;
        assert_eq!(
            find_crossing_window(6, 90.0, 2_451_545.0, true, flags, 4_000.0),
            Some(2_452_798.481_687_132)
        );
        assert_eq!(
            find_crossing_window(6, 90.0, 2_451_545.0, false, flags, 4_000.0),
            None
        );
    }
}
