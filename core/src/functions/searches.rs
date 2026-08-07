//! Iterative aspect and retrograde search functions.

use crate::body::{Body, CalcFlags, HouseSystem};
use crate::functions::houses::{house_cusp, houses};
use crate::units::{JulianDay, Latitude, Longitude};

// ─── Internal helpers ─────────────────────────────────────────────────────────

use crate::{diff_deg, diff_deg_signed, norm_deg};

#[inline]
fn norm360(d: f64) -> f64 {
    norm_deg(d)
}

#[inline]
fn below_tolerance(value: f64, tolerance: f64) -> bool {
    value.abs() < tolerance
}

#[inline]
fn brackets_zero(a: f64, b: f64) -> bool {
    a == 0.0 || b == 0.0 || a.is_sign_negative() != b.is_sign_negative()
}

#[inline]
fn continuous_crossing(a: f64, b: f64) -> bool {
    brackets_zero(a, b) && (b - a).abs() < 180.0
}

#[inline]
fn directional_limit(start: f64, span: f64, backward: bool) -> f64 {
    if backward {
        start - span
    } else {
        start + span
    }
}

#[inline]
fn opposite_degree(degree: f64) -> f64 {
    if degree < 180.0 {
        degree + 180.0
    } else {
        degree - 180.0
    }
}

#[inline]
fn without_speed(flags: CalcFlags) -> CalcFlags {
    flags & !CalcFlags::SPEED & !CalcFlags::SPEED3
}

fn nearest_result<T, F>(r1: Option<T>, r2: Option<T>, backward: bool, jd: F) -> Option<T>
where
    F: Fn(&T) -> f64,
{
    match (r1, r2) {
        (Some(a), Some(b)) if prefers_first(jd(&a), jd(&b), backward) => Some(a),
        (Some(_), Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn prefers_first(a: f64, b: f64, backward: bool) -> bool {
    match a.partial_cmp(&b) {
        Some(std::cmp::Ordering::Less) => !backward,
        Some(std::cmp::Ordering::Greater) => backward,
        _ => false,
    }
}

fn within_search_window(jd: f64, start: f64, stop_days: f64, backward: bool) -> bool {
    if stop_days <= 0.0 {
        return true;
    }
    let limit = directional_limit(start, stop_days, backward);
    if backward {
        jd >= limit
    } else {
        jd <= limit
    }
}

fn lower_scan_plan(upper_transit: f64, step: f64) -> (f64, u16) {
    let start = upper_transit + 1.0 / 240.0;
    let steps = ((upper_transit + 1.0 - start) / step).round() as u16;
    (start, steps)
}

fn angle_in_interval(angle: f64, start: f64, end: f64) -> bool {
    if start < end {
        (start..end).contains(&angle)
    } else {
        (start..360.0).contains(&angle) || (0.0..end).contains(&angle)
    }
}

/// Bisect a fallible scalar function `f(x) -> Option<f64>` to find a zero
/// crossing within the bracket `[lo, hi]`, given `d_lo = f(lo)` (with the
/// guarantee `f(lo) * f(hi) ≤ 0` so a root exists).
///
/// Returns `None` if `f` fails at any sampled point. Otherwise converges
/// in ≤ `max_iter` halvings or when `|f(mid)| < tol`.
fn bisect_zero_fallible<F>(
    mut lo: f64,
    mut hi: f64,
    mut d_lo: f64,
    tol: f64,
    max_iter: usize,
    mut f: F,
) -> Option<f64>
where
    F: FnMut(f64) -> Option<f64>,
{
    for _ in 0..max_iter {
        let mid = (lo + hi) / 2.0;
        let d_mid = f(mid)?;
        if below_tolerance(d_mid, tol) {
            return Some(mid);
        }
        if brackets_zero(d_lo, d_mid) {
            hi = mid;
        } else {
            lo = mid;
            d_lo = d_mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// Approximate retrograde station duration (days) for a planet — used as search step.
fn approx_retro_time(body: Body) -> f64 {
    match body.as_raw() {
        2 => 18.0,   // Mercury
        3 => 38.0,   // Venus
        4 => 57.0,   // Mars
        5 => 115.0,  // Jupiter
        6 => 130.0,  // Saturn
        7 => 147.0,  // Uranus
        8 => 154.0,  // Neptune
        9 => 153.0,  // Pluto
        15 => 124.0, // Chiron
        _ => 0.5,
    }
}

/// Result of a retrograde station search.
#[must_use = "the search result contains the computed data — did you mean to use it?"]
pub struct RetroResult {
    /// Julian Day (UT) of the station.
    pub jd: f64,
    /// Position at the station: `[lon, lat, dist, speed_lon, speed_lat, speed_dist]`.
    pub pos: [f64; 6],
}

/// Find the next retrograde or direct station for a planet.
///
/// Returns `None` if the planet cannot go retrograde (Sun, Moon, Earth),
/// or if the station isn't found within the search window.
/// Bisect the retrograde-station bracket `[jd-dir, jd]` (sign change in speed_lon).
fn bisect_retro_station<F>(
    ja_in: f64,
    jb_in: f64,
    sa_in: f64,
    p_hi: crate::PlanetPos,
    pos_at: &F,
) -> Option<RetroResult>
where
    F: Fn(f64) -> Option<crate::PlanetPos>,
{
    let (mut ja, mut jb) = (ja_in, jb_in);
    let mut sa = sa_in;
    let mut pm = p_hi;
    let to_arr = |p: &crate::PlanetPos| -> [f64; 6] {
        [p.lon, p.lat, p.dist, p.speed_lon, p.speed_lat, p.speed_dist]
    };
    for _ in 0..50 {
        let jm = (ja + jb) / 2.0;
        pm = pos_at(jm)?;
        let sm = pm.speed_lon;
        if below_tolerance(sm, 1e-9) {
            return Some(RetroResult {
                jd: jm,
                pos: to_arr(&pm),
            });
        }
        if sa.is_sign_negative() != sm.is_sign_negative() {
            jb = jm;
        } else {
            ja = jm;
            sa = sm;
        }
    }
    Some(RetroResult {
        jd: (ja + jb) / 2.0,
        pos: to_arr(&pm),
    })
}

/// Find the next retrograde or direct station for `body` from `jd_start`.
///
/// Steps until the longitudinal speed changes sign, then bisects to the exact
/// station. Returns `None` for bodies that never retrograde (Sun, Moon, Earth)
/// or if no station is found within the search window. Set `backward` to search
/// toward earlier dates.
pub fn next_retro(
    body: Body,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: CalcFlags,
) -> Option<RetroResult> {
    use crate::functions::calc::calc_ut;

    if matches!(body.as_raw(), 0 | 1 | 14) {
        return None;
    }

    let step = approx_retro_time(body);
    if step <= 0.0 {
        return None;
    }
    let flags_with_speed = flags | CalcFlags::SPEED;

    let pos_at = |jd: f64| calc_ut(JulianDay::new(jd), body, flags_with_speed).ok();

    let mut jd = jd_start;
    let span = if stop_days > 0.0 { stop_days } else { 50_000.0 };
    let max_jd = directional_limit(jd_start, span, backward);

    let mut s0 = pos_at(jd)?.speed_lon;

    loop {
        let previous_jd = jd;
        jd = directional_limit(jd, step, backward);
        if passed_limit(jd, max_jd, backward) {
            return None;
        }

        let p1 = pos_at(jd)?;
        let s1 = p1.speed_lon;
        if s0.is_sign_negative() != s1.is_sign_negative() {
            return bisect_retro_station(previous_jd, jd, s0, p1, &pos_at);
        }
        s0 = s1;
    }
}

/// Result of an aspect search.
#[derive(Debug, Clone, PartialEq)]
#[must_use = "the search result contains the computed data — did you mean to use it?"]
pub struct AspectResult {
    /// Julian Day (UT) of the exact aspect.
    pub jd: f64,
    /// Planet positions at exact aspect.
    pub pos1: [f64; 6],
    /// Other planet / fixed point positions (zero if searching against fixed point).
    pub pos2: [f64; 6],
}

fn pos_to_arr(p: &crate::PlanetPos) -> [f64; 6] {
    [p.lon, p.lat, p.dist, p.speed_lon, p.speed_lat, p.speed_dist]
}

/// Find the next time `planet` makes `aspect` to a fixed point.
/// `aspect` in `[0, 360)`.
pub fn next_aspect(
    body: Body,
    aspect: f64,
    fixed_pt: f64,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: CalcFlags,
) -> Option<AspectResult> {
    use crate::astronomy::crossings::find_crossing;
    use crate::functions::calc::calc_ut;

    // We want: lon + aspect ≡ fixed_pt (mod 360)
    // → lon ≡ fixed_pt - aspect (mod 360)
    let target = norm360(fixed_pt - aspect);
    let jd = find_crossing(
        body.as_raw(),
        target,
        JulianDay::new(jd_start),
        !backward,
        flags.as_raw(),
    )?;

    if !within_search_window(jd, jd_start, stop_days, backward) {
        return None;
    }
    let p = calc_ut(JulianDay::new(jd), body, flags).ok()?;
    Some(AspectResult {
        jd,
        pos1: pos_to_arr(&p),
        pos2: [0.0; 6],
    })
}

/// Find the next time `planet` makes `aspect` to a fixed point, `aspect` in `[0, 180]`.
pub fn next_aspect2(
    body: Body,
    aspect: f64,
    fixed_pt: f64,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: CalcFlags,
) -> Option<AspectResult> {
    let asp_n = diff_deg_signed(0.0, aspect).abs(); // normalise to [0,180]
    let r1 = next_aspect(body, asp_n, fixed_pt, jd_start, backward, stop_days, flags);
    if [0.0, 180.0].contains(&asp_n) {
        return r1;
    }
    let r2 = next_aspect(body, -asp_n, fixed_pt, jd_start, backward, stop_days, flags);
    nearest_result(r1, r2, backward, |result| result.jd)
}

/// Find next exact aspect between two moving planets.
/// Returns `true` when `jd` has stepped past `max_jd` in the given direction.
#[inline]
fn passed_limit(jd: f64, max_jd: f64, backward: bool) -> bool {
    if backward {
        jd < max_jd
    } else {
        jd > max_jd
    }
}

/// Find the next moment when `body` makes the given `aspect` to `other`.
///
/// Walks ±0.5-day steps until the angular separation between
/// `(body.lon + aspect) mod 360` and `other.lon` changes sign, then
/// bisects to refine the exact JD. Tolerates wrap-around at 0°/360° by
/// requiring the speed-of-change between adjacent samples to stay below
/// 180° (so a wrap doesn't masquerade as a real crossing).
///
/// # Arguments
/// - `body`: primary planet whose longitude is being tracked
/// - `aspect`: aspect angle in degrees (e.g., 0 for conjunction, 90 for
///   square, 180 for opposition); is normalized into `[0, 360)`
/// - `other`: target body that `body` is aspecting; pass `Body::SUN`/etc.
///   for stations against another planet, or use [`next_aspect`] for
///   crossings of a fixed ecliptic longitude
/// - `jd_start`: Julian Day to begin the search from
/// - `backward`: search backwards in time if `true`
/// - `stop_days`: give up after this many days; pass `0.0` (or any
///   non-positive number) for the ~273-year default
/// - `flags`: usual `CalcFlags::BUILTIN`/`SIDEREAL`/etc.
///
/// # Returns
/// `Some(AspectResult { jd, pos1, pos2 })` with the JD of exact aspect
/// and the planetary positions at that moment, or `None` if no aspect
/// occurs within the window or if a Swiss Ephemeris call fails.
///
/// Uses the internal `bisect_zero_fallible` helper for the sign-crossing
/// refinement (shared with [`next_aspect_cusp`]).
pub fn next_aspect_with(
    body: Body,
    aspect: f64,
    other: Body,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: CalcFlags,
) -> Option<AspectResult> {
    use crate::functions::calc::calc_ut;
    let aspect = norm360(aspect);
    let step = 0.5_f64;

    // Scan reads only `.lon`; drop SPEED bits so each probe skips ~2 extra
    // position evals (the scan can run 100k+ iterations).
    let scan_flags = without_speed(flags);

    let mut diff_at = |jd: f64| -> Option<f64> {
        let p1 = calc_ut(JulianDay::new(jd), body, scan_flags).ok()?.lon;
        let p2 = calc_ut(JulianDay::new(jd), other, scan_flags).ok()?.lon;
        Some(diff_deg_signed(p1 + aspect, p2))
    };

    let span = if stop_days > 0.0 {
        stop_days
    } else {
        100_000.0
    };
    let max_jd = directional_limit(jd_start, span, backward);

    let mut jd = jd_start;
    let mut d0 = diff_at(jd)?;

    loop {
        let previous_jd = jd;
        jd = directional_limit(jd, step, backward);
        if passed_limit(jd, max_jd, backward) {
            return None;
        }

        let d1 = diff_at(jd)?;
        if continuous_crossing(d0, d1) {
            // Sign change → bisect on `diff_at` to find the crossing.
            let jd_ret = bisect_zero_fallible(previous_jd, jd, d0, 1e-8, 60, &mut diff_at)?;
            let p1 = calc_ut(JulianDay::new(jd_ret), body, flags).ok()?;
            let p2 = calc_ut(JulianDay::new(jd_ret), other, flags).ok()?;
            return Some(AspectResult {
                jd: jd_ret,
                pos1: pos_to_arr(&p1),
                pos2: pos_to_arr(&p2),
            });
        }
        d0 = d1;
    }
}

/// Like `next_aspect_with` but `aspect` in `[0, 180]`.
pub fn next_aspect_with2(
    body: Body,
    aspect: f64,
    other: Body,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: CalcFlags,
) -> Option<AspectResult> {
    let asp_n = diff_deg_signed(0.0, aspect).abs();
    let r1 = next_aspect_with(body, asp_n, other, jd_start, backward, stop_days, flags);
    if [0.0, 180.0].contains(&asp_n) {
        return r1;
    }
    let r2 = next_aspect_with(body, -asp_n, other, jd_start, backward, stop_days, flags);
    nearest_result(r1, r2, backward, |result| result.jd)
}

/// Result of an aspect-to-cusp search.
#[derive(Debug, Clone, PartialEq)]
#[must_use = "the search result contains the computed data — did you mean to use it?"]
pub struct AspectCuspResult {
    /// Julian Day (UT) of the exact aspect to the cusp.
    pub jd: f64,
    /// Planet position: `[lon, lat, dist, speed_lon, speed_lat, speed_dist]`.
    pub pos: [f64; 6],
    /// House cusp longitudes at that time (indices 1–12 used).
    pub cusps: [f64; 13],
    /// Ascendant/MC and related points at that time.
    pub ascmc: [f64; 10],
}

/// Find when a planet makes `aspect` to house cusp `cusp` (1–12).
#[allow(clippy::too_many_arguments)]
pub fn next_aspect_cusp(
    body: Body,
    aspect: f64,
    cusp: usize,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: HouseSystem,
    backward: bool,
    flags: CalcFlags,
) -> Option<AspectCuspResult> {
    use crate::functions::calc::calc_ut;

    if !(1..=12).contains(&cusp) {
        return None;
    }
    let aspect = norm360(aspect);
    let step = 0.05_f64;

    // Scan reads only `.lon`; drop SPEED bits (see `next_aspect_with`).
    let scan_flags = without_speed(flags);

    let mut diff_at = |jd: f64| -> Option<f64> {
        let p = calc_ut(JulianDay::new(jd), body, scan_flags).ok()?.lon;
        let cusp_lon = house_cusp(
            JulianDay::new(jd),
            Latitude::new(lat),
            Longitude::new(lon),
            hsys,
            cusp,
        )
        .ok()?;
        Some(diff_deg_signed(p + aspect, cusp_lon))
    };

    let max_jd = directional_limit(jd_start, 400.0, backward);
    let mut jd = jd_start;
    let mut d0 = diff_at(jd)?;

    loop {
        let previous_jd = jd;
        jd = directional_limit(jd, step, backward);
        if passed_limit(jd, max_jd, backward) {
            return None;
        }

        let d1 = diff_at(jd)?;
        if continuous_crossing(d0, d1) {
            let jd_ret = bisect_zero_fallible(previous_jd, jd, d0, 1e-8, 60, &mut diff_at)?;
            let p = calc_ut(JulianDay::new(jd_ret), body, flags).ok()?;
            let hr = houses(
                JulianDay::new(jd_ret),
                Latitude::new(lat),
                Longitude::new(lon),
                hsys,
            )
            .ok()?;
            return Some(AspectCuspResult {
                jd: jd_ret,
                pos: pos_to_arr(&p),
                cusps: hr.cusps,
                ascmc: hr.ascmc,
            });
        }
        d0 = d1;
    }
}

/// Like `next_aspect_cusp` but `aspect` in `[0, 180]`.
#[allow(clippy::too_many_arguments)]
pub fn next_aspect_cusp2(
    body: Body,
    aspect: f64,
    cusp: usize,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: HouseSystem,
    backward: bool,
    flags: CalcFlags,
) -> Option<AspectCuspResult> {
    let asp_n = diff_deg_signed(0.0, aspect).abs();
    let r1 = next_aspect_cusp(body, asp_n, cusp, jd_start, lat, lon, hsys, backward, flags);
    if [0.0, 180.0].contains(&asp_n) {
        return r1;
    }
    let r2 = next_aspect_cusp(
        body, -asp_n, cusp, jd_start, lat, lon, hsys, backward, flags,
    );
    nearest_result(r1, r2, backward, |result| result.jd)
}

/// Number of astrological years between two Julian days.
/// An "astrological year" = one solar revolution.
pub fn years_diff(jd1: f64, jd2: f64, flags: CalcFlags) -> crate::Result<f64> {
    use crate::functions::calc::calc_ut;
    // Only `.lon` is used — no need to compute solar speed.
    let flags = without_speed(flags);
    let sun1 = calc_ut(JulianDay::new(jd1), Body::SUN, flags)?.lon;
    let sun2 = calc_ut(JulianDay::new(jd2), Body::SUN, flags)?.lon;
    let mut years = 0.0_f64;

    match jd1.partial_cmp(&jd2) {
        Some(std::cmp::Ordering::Less) => {
            let dec = diff_deg(sun2, sun1) / 360.0;
            let mut jd = jd1;
            loop {
                let r = crate::functions::motion::solcross(
                    Longitude::new(sun1),
                    JulianDay::new(jd + 1e-5),
                    flags,
                )
                .map_err(|e| crate::error::Error::Calc(e.to_string()))?;
                if r <= jd {
                    return Err(crate::error::Error::Calc(
                        "solar crossing did not advance".into(),
                    ));
                }
                if r <= jd2 {
                    years += 1.0;
                    jd = r;
                } else {
                    break;
                }
            }
            years += dec;
        }
        Some(std::cmp::Ordering::Greater) => {
            let dec = diff_deg(sun1, sun2) / 360.0;
            let mut jd = jd1;
            loop {
                let rb = crate::astronomy::crossings::find_crossing(
                    0,
                    sun1,
                    JulianDay::new(jd - 1e-5),
                    false,
                    flags.as_raw(),
                )
                .ok_or_else(|| crate::error::Error::Calc("no crossing".into()))?;
                if rb >= jd {
                    return Err(crate::error::Error::Calc(
                        "solar crossing did not retreat".into(),
                    ));
                }
                if rb >= jd2 {
                    years -= 1.0;
                    jd = rb;
                } else {
                    break;
                }
            }
            years -= dec;
        }
        _ => {}
    }
    Ok(years)
}

// ─── MC / IC transit searches ──────────────────────────────────────────────────

/// Next time a planet crosses a fixed ecliptic longitude (generic crossing search).
///
/// This is the building block for natal chart transits:
/// given a fixed degree (e.g. your natal MC, ASC, or any planet), find when a
/// transiting body next reaches that degree.
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// // Your natal MC is at 15° Capricorn = 285°.
/// // When does transiting Saturn next reach it?
/// let natal_mc = 285.0;
/// if let Ok(jd) = transit_to_degree(Body::SATURN, natal_mc, 2_451_545.0, CalcFlags::BUILTIN, false) {
///     println!("Saturn conjunct natal MC: JD {jd:.2}");
/// }
/// ```
pub fn transit_to_degree(
    body: Body,
    target_lon: f64,
    jd_start: f64,
    flags: CalcFlags,
    backward: bool,
) -> crate::Result<f64> {
    // Use a search window appropriate to the planet's orbital period:
    // Sun/Moon/Mercury/Venus: 400 days more than covers one orbit
    // Mars: ~687 days, Jupiter: ~4333 days, Saturn: ~10759 days, etc.
    // We search up to 2× the synodic period to guarantee finding the next crossing.
    let window_days = body.ingress_search_window();
    crate::astronomy::crossings::find_crossing_window(
        body.as_raw(), target_lon, jd_start, !backward, flags.as_raw(), window_days
    )
    .ok_or_else(|| crate::Error::Calc(
        format!("transit_to_degree: body {body} did not reach {target_lon:.2}° within {window_days:.0}-day window")
    ))
}

/// Next time a planet transits the Midheaven (MC) of a natal chart.
///
/// Computes the natal MC from the given `jd_natal`, `lat`, `lon` and house system,
/// then finds when `planet` next crosses that fixed ecliptic degree.
///
/// This is the standard astrological "transiting planet conjunct natal MC":
/// - Sun takes ~1 year to complete one circuit, so it hits the MC once per year
/// - Moon: ~monthly  
/// - Mars: every ~2 years
/// - Saturn: every ~29 years
///
/// **Not** the same as solar noon — for physical meridian crossing use
/// [`meridian_transit_ut`].
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let natal = 2_437_837.0; // a birth date
/// // When does Saturn next conjunct the natal MC?
/// if let Ok(jd) = mc_transit_ut(Body::SATURN, natal, 2_451_545.0, 51.5, -0.1, HouseSystem::PLACIDUS, CalcFlags::BUILTIN, false) {
///     println!("Saturn conjunct natal MC: JD {jd:.2}");
/// }
/// ```
#[allow(clippy::too_many_arguments)]
pub fn mc_transit_ut(
    body: Body,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: HouseSystem,
    flags: CalcFlags,
    backward: bool,
) -> crate::Result<f64> {
    let chart = crate::functions::houses::houses(
        JulianDay::new(jd_natal),
        Latitude::new(lat),
        Longitude::new(lon),
        hsys,
    )
    .map_err(|e| crate::Error::Calc(format!("mc_transit_ut: {e}")))?;
    let natal_mc = chart.ascmc[1]; // true MC
    transit_to_degree(body, natal_mc, jd_start, flags, backward)
}

/// Next time a planet transits the Imum Coeli (IC) of a natal chart.
///
/// IC = natal MC + 180°. A planet at the IC is at the nadir of the chart.
#[allow(clippy::too_many_arguments)]
pub fn ic_transit_ut(
    body: Body,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: HouseSystem,
    flags: CalcFlags,
    backward: bool,
) -> crate::Result<f64> {
    let chart = crate::functions::houses::houses(
        JulianDay::new(jd_natal),
        Latitude::new(lat),
        Longitude::new(lon),
        hsys,
    )
    .map_err(|e| crate::Error::Calc(format!("ic_transit_ut: {e}")))?;
    let natal_ic = opposite_degree(chart.ascmc[1]);
    transit_to_degree(body, natal_ic, jd_start, flags, backward)
}

/// Next time a planet transits the Ascendant (ASC) of a natal chart.
#[allow(clippy::too_many_arguments)]
pub fn asc_transit_ut(
    body: Body,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: HouseSystem,
    flags: CalcFlags,
    backward: bool,
) -> crate::Result<f64> {
    let chart = crate::functions::houses::houses(
        JulianDay::new(jd_natal),
        Latitude::new(lat),
        Longitude::new(lon),
        hsys,
    )
    .map_err(|e| crate::Error::Calc(format!("asc_transit_ut: {e}")))?;
    let natal_asc = chart.ascmc[0];
    transit_to_degree(body, natal_asc, jd_start, flags, backward)
}

/// Next time a planet transits the Descendant (DSC) of a natal chart.
///
/// DSC = natal ASC + 180°.
#[allow(clippy::too_many_arguments)]
pub fn dsc_transit_ut(
    body: Body,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: HouseSystem,
    flags: CalcFlags,
    backward: bool,
) -> crate::Result<f64> {
    let chart = crate::functions::houses::houses(
        JulianDay::new(jd_natal),
        Latitude::new(lat),
        Longitude::new(lon),
        hsys,
    )
    .map_err(|e| crate::Error::Calc(format!("dsc_transit_ut: {e}")))?;
    let natal_dsc = opposite_degree(chart.ascmc[0]);
    transit_to_degree(body, natal_dsc, jd_start, flags, backward)
}

/// Current house number (1–12) that a planet occupies.
///
/// Given a planet's position and a full chart, returns which house the planet
/// is in.  Handles all house systems including irregular ones (Placidus, Koch).
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let jd = 2_451_545.0;
/// let chart = houses(JulianDay::new(jd), Latitude::new(48.85), Longitude::new(2.35), HouseSystem::PLACIDUS).unwrap();
/// let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap();
/// let house = planet_house_number(sun.lon, &chart.cusps);
/// println!("Sun is in house {house}");
/// ```
#[must_use]
pub fn planet_house_number(planet_lon: f64, cusps: &[f64; 13]) -> u8 {
    // Cusps are in [0,360) in ascending order for most systems.
    // Walk from cusp 1; the planet is in house h when it is between cusps[h] and cusps[h+1].
    // For the 12th house the upper bound wraps to cusps[1].
    for h in 1usize..=12 {
        let lo = cusps[h];
        let hi = if h == 12 { cusps[1] } else { cusps[h + 1] };
        let lon = planet_lon.rem_euclid(360.0);
        if angle_in_interval(lon, lo, hi) {
            return h as u8;
        }
    }
    1 // fallback
}

/// Distance in degrees from a planet's position to the Midheaven (MC).
///
/// Returns a signed angle in (−180°, +180°]:
/// * Positive → planet is approaching the MC (will conjunct MC in the future
///   as the chart moves forward)
/// * Negative → planet has passed the MC
///
/// Useful for gauging how far a planet is from culmination.
#[must_use]
pub fn distance_to_mc(planet_lon: f64, mc_lon: f64) -> f64 {
    // Positive = MC is ahead of planet (planet approaching MC)
    // Negative = planet has already passed MC
    crate::functions::utils::diff_deg_signed(mc_lon, planet_lon)
}

/// Is a planet within `orb` degrees of the Midheaven?
///
/// A shorthand for `distance_to_mc(planet_lon, mc_lon).abs() <= orb`.
#[must_use]
pub fn planet_conjunct_mc(planet_lon: f64, mc_lon: f64, orb: f64) -> bool {
    distance_to_mc(planet_lon, mc_lon).abs() <= orb
}

// ─── Physical meridian transit ────────────────────────────────────────────────

/// Next time a body physically crosses the local meridian (upper culmination).
///
/// This is the *astronomical* transit — the moment of maximum altitude, used for:
/// - Solar noon
/// - Moon's highest point
/// - Planet's culmination
///
/// Uses the iterative rise/set engine (`CALC_MTRANSIT`).
/// Unlike [`mc_transit_ut`] (which is an ecliptic longitude conjunction),
/// this finds when the body's **hour angle = 0** — i.e. true meridian crossing.
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// // When is solar noon today at Paris?
/// let noon = meridian_transit_ut(Body::SUN, 2_451_545.0, [2.35, 48.85, 35.0], CalcFlags::BUILTIN);
/// println!("Solar noon: JD {:.4}", noon.unwrap().tret);
/// ```
pub fn meridian_transit_ut(
    body: Body,
    jd_start: f64,
    geopos: [f64; 3],
    flags: CalcFlags,
) -> crate::Result<crate::functions::motion::RiseTransResult> {
    crate::functions::motion::rise_trans(
        JulianDay::new(jd_start),
        body,
        None,
        flags,
        crate::CALC_MTRANSIT,
        geopos,
        1013.25,
        15.0,
    )
}

/// Next time a body crosses the lower meridian (anti-culmination / nadir).
///
/// The lower meridian transit is ~12 hours before or after the upper transit.
/// Computed by finding the upper transit then searching for the transit
/// approximately 12 hours later.
pub fn lower_meridian_transit_ut(
    body: Body,
    jd_start: f64,
    geopos: [f64; 3],
    flags: CalcFlags,
) -> crate::Result<crate::functions::motion::RiseTransResult> {
    use crate::functions::motion::RiseTransResult;

    // Strategy: the lower transit (anti-culmination, H = 180°) occurs when the
    // body's local hour angle equals 180° — i.e. it is due north (or south for
    // observers in the opposite hemisphere), exactly at the nadir of its arc.
    //
    // We find it by scanning for the hour angle = 180° crossing:
    //   ha(t) = GAST(t) + lon_east - RA(t)
    // Find the moment ha crosses 180° (mod 360).
    //
    // Step at 2-minute intervals (1/720 day) — fine enough for any body.

    let step = 1.0 / 720.0; // 2 minutes
    let lon = geopos[0];
    let flags_pos = without_speed(flags);

    let ha_at = |jd: f64| -> f64 {
        let pos = match crate::calc_ut(JulianDay::new(jd), body, flags_pos) {
            Ok(p) => p,
            Err(_) => return 0.0, // treat as HA=0 if calc fails
        };
        // Convert ecliptic lon/lat → RA
        let eps = crate::true_obliquity(JulianDay::new(jd)).to_radians();
        let lon_r = pos.lon.to_radians();
        let lat_r = pos.lat.to_radians();
        // Meeus eq.13.3: RA = atan2(sin(lon)*cos(eps) - tan(lat)*sin(eps), cos(lon))
        let (sin_lon, cos_lon) = lon_r.sin_cos();
        let (sin_eps, cos_eps) = eps.sin_cos();
        let ra = (-lat_r.tan())
            .mul_add(sin_eps, sin_lon * cos_eps)
            .atan2(cos_lon)
            .to_degrees()
            .rem_euclid(360.0);
        let gast = crate::sidtime(JulianDay::new(jd)) * 15.0; // hours → degrees
        (gast + lon - ra).rem_euclid(360.0)
    };

    // Signed distance from 180° in (-180, +180]
    let diff = |jd: f64| -> f64 { diff_deg_signed(ha_at(jd), 180.0) };

    // Start from upper transit to find the NEXT lower transit (~12h later)
    let upper = crate::functions::motion::rise_trans(
        JulianDay::new(jd_start),
        body,
        None,
        flags,
        crate::CALC_MTRANSIT,
        geopos,
        1013.25,
        15.0,
    )?;
    let (scan_start, scan_steps) = lower_scan_plan(upper.tret, step);
    let mut d0 = diff(scan_start);

    for step_index in 1..=scan_steps {
        let jd = scan_start + f64::from(step_index) * step;
        let d1 = diff(jd);
        if continuous_crossing(d0, d1) {
            let tret = bisect_diff_zero(jd - step, jd, d0, &diff);
            return Ok(RiseTransResult { ret_flags: 0, tret });
        }
        d0 = d1;
    }
    Err(crate::Error::RiseTrans(
        "lower meridian transit not found within 2 days".into(),
    ))
}

/// Bisect a sign change in `diff` over `[lo, hi]` (closed) where `diff(lo) = dlo`.
fn bisect_diff_zero<F: Fn(f64) -> f64>(lo_in: f64, hi_in: f64, dlo_in: f64, diff: &F) -> f64 {
    let (mut lo, mut hi) = (lo_in, hi_in);
    let mut dlo = dlo_in;
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        let dm = diff(mid);
        if below_tolerance(dm, 1e-8) {
            return mid;
        }
        if brackets_zero(dlo, dm) {
            hi = mid;
        } else {
            lo = mid;
            dlo = dm;
        }
    }
    (lo + hi) / 2.0
}

#[cfg(test)]
mod cov_tests {
    use super::*;

    fn planet_pos(lon: f64, speed_lon: f64) -> crate::PlanetPos {
        crate::PlanetPos {
            lon,
            lat: 2.0,
            dist: 3.0,
            speed_lon,
            speed_lat: 4.0,
            speed_dist: 5.0,
            ret_flags: 6,
        }
    }

    #[test]
    fn bisect_zero_fallible_finds_linear_root() {
        let root = bisect_zero_fallible(0.0, 4.0, -3.0, 1e-12, 40, |x| Some(x - 3.0));
        assert_eq!(root, Some(3.0));
    }

    #[test]
    fn bisect_zero_fallible_does_not_accept_tolerance_boundary() {
        let root = bisect_zero_fallible(-1.0, 3.0, -1.0, 1.0, 2, Some);
        assert_eq!(root, Some(0.0));
    }

    #[test]
    fn bisect_zero_fallible_propagates_none() {
        let r = bisect_zero_fallible(-1.0, 1.0, -1.0, 1e-9, 40, |_| None::<f64>);
        assert!(r.is_none());
    }

    #[test]
    fn bisect_diff_zero_converges_on_linear() {
        let r = bisect_diff_zero(-1.0, 1.0, -1.0, &|x| x);
        assert!(r.abs() < 1e-6, "r={r}");
    }

    #[test]
    fn bisect_diff_zero_returns_value_in_bracket_on_no_convergence() {
        let r = bisect_diff_zero(0.0, 1.0, 1.0, &|_| 1.0);
        assert_eq!(r, 1.0 - 2.0_f64.powi(-41));
    }

    #[test]
    fn approx_retro_times_match_body_windows() {
        let cases = [
            (0, 0.5),
            (1, 0.5),
            (14, 0.5),
            (2, 18.0),
            (3, 38.0),
            (4, 57.0),
            (5, 115.0),
            (6, 130.0),
            (7, 147.0),
            (8, 154.0),
            (9, 153.0),
            (15, 124.0),
            (16, 0.5),
        ];
        for (raw, expected) in cases {
            assert_eq!(approx_retro_time(Body::from_raw(raw)), expected);
        }
    }

    #[test]
    fn bisect_retro_station_finds_zero_speed() {
        let pos_at = |t: f64| Some(planet_pos(t, t - 5.0));
        let p_hi = pos_at(10.0).unwrap();
        let r = bisect_retro_station(0.0, 10.0, -5.0, p_hi, &pos_at);
        let r = r.unwrap();
        assert_eq!(r.jd, 5.0);
        assert_eq!(r.pos, [5.0, 2.0, 3.0, 0.0, 4.0, 5.0]);
    }

    #[test]
    fn bisect_retro_station_rejects_tolerance_boundary() {
        let pos_at = |t: f64| Some(planet_pos(t, (t - 4.0) * 1e-9));
        let r = bisect_retro_station(0.0, 10.0, -4e-9, pos_at(10.0).unwrap(), &pos_at).unwrap();
        assert_eq!(r.jd, 3.75);
    }

    #[test]
    fn bisect_retro_station_fallback_preserves_last_sample() {
        let pos_at = |t: f64| Some(planet_pos(t, 1.0));
        let r = bisect_retro_station(0.0, 10.0, 1.0, pos_at(10.0).unwrap(), &pos_at).unwrap();
        assert_eq!(r.jd, 10.0 - 10.0 / 2.0_f64.powi(51));
        assert_eq!(r.pos[0], 10.0 - 10.0 / 2.0_f64.powi(50));
    }

    #[test]
    fn passed_limit_includes_boundary() {
        let cases = [
            (2.0, 2.0, false, false),
            (2.1, 2.0, false, true),
            (2.0, 2.0, true, false),
            (1.9, 2.0, true, true),
        ];
        for (jd, limit, backward, expected) in cases {
            assert_eq!(passed_limit(jd, limit, backward), expected);
        }
    }

    #[test]
    fn next_retro_reference_and_limits() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_retro(Body::MERCURY, 2_451_545.0, false, 400.0, flags).unwrap();
        assert_eq!(r.jd, 2_451_596.021_076_591_7);
        assert_eq!(r.pos[0], 347.176_128_649_668_56);
        assert_eq!(r.pos[3], -3.389_004_632_481_374e-10);
        assert!(next_retro(Body::MERCURY, 2_451_545.0, false, 2.0, flags).is_none());
        assert!(next_retro(Body::SUN, 2_451_545.0, false, 400.0, flags).is_none());
        let without_speed =
            next_retro(Body::MERCURY, 2_451_545.0, false, 400.0, CalcFlags::BUILTIN).unwrap();
        assert_eq!(without_speed.jd, r.jd);
    }

    #[test]
    fn next_retro_searches_backward() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let retro = next_retro(Body::MERCURY, 2_451_600.0, true, 400.0, flags).unwrap();
        assert_eq!(retro.jd, 2_451_596.021_076_584_2);
        assert_eq!(retro.pos[0], 347.176_128_649_667_53);
        assert_eq!(retro.pos[3], 9.246_718_946_087_64e-10);
    }

    #[test]
    fn next_aspect_fixed_point_reference() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_aspect(Body::SUN, 0.0, 90.0, 2_451_545.0, false, 400.0, flags).unwrap();
        assert_eq!(r.jd, 2_451_716.569_027_677_6);
        assert_eq!(r.pos1[0], 89.999_999_999_744_32);
        assert_eq!(r.pos1[3], 0.954_054_695_302_431_8);
        assert_eq!(r.pos2, [0.0; 6]);
        assert!(next_aspect(Body::SUN, 0.0, 90.0, 2_451_545.0, false, 2.0, flags).is_none());
    }

    #[test]
    fn next_aspect2_selects_nearest_branch() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_aspect2(Body::SUN, 60.0, 90.0, 2_451_545.0, false, 400.0, flags).unwrap();
        assert_eq!(r.jd, 2_451_654.274_029_908_7);
        assert_eq!(r.pos1[0], 30.000_000_008_716_2);
        assert_eq!(
            next_aspect(Body::SUN, 60.0, 90.0, 2_451_545.0, false, 400.0, flags),
            Some(r.clone())
        );
        assert_eq!(
            next_aspect2(Body::SUN, 0.0, 90.0, 2_451_545.0, false, 400.0, flags),
            next_aspect(Body::SUN, 0.0, 90.0, 2_451_545.0, false, 400.0, flags)
        );
    }

    #[test]
    fn moving_aspect_references_both_directions() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let forward = next_aspect_with(
            Body::MARS,
            0.0,
            Body::JUPITER,
            2_451_545.0,
            false,
            4_000.0,
            flags,
        )
        .unwrap();
        let backward = next_aspect_with(
            Body::MARS,
            0.0,
            Body::JUPITER,
            2_451_700.0,
            true,
            4_000.0,
            flags,
        )
        .unwrap();
        assert_eq!(forward.jd, 2_451_640.228_804_409_5);
        assert_eq!(backward, forward);
        assert_eq!(forward.pos1[0], 39.952_290_779_943_42);
        assert_eq!(forward.pos2[0], 39.952_290_778_342_96);
    }

    #[test]
    fn moving_aspect2_reference_and_limit() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_aspect_with2(Body::MOON, 90.0, Body::SUN, 2_451_545.0, false, 40.0, flags)
            .unwrap();
        assert_eq!(r.jd, 2_451_558.066_024_355_6);
        assert_eq!(r.pos1[0], 23.696_110_798_206_107);
        assert_eq!(r.pos2[0], 293.696_110_799_823_43);
        assert!(
            next_aspect_with(Body::MOON, 90.0, Body::SUN, 2_451_545.0, false, 2.0, flags).is_none()
        );
    }

    #[test]
    fn cusp_aspect_reference_and_validation() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_aspect_cusp(
            Body::SUN,
            0.0,
            10,
            2_451_545.0,
            48.85,
            2.35,
            HouseSystem::PLACIDUS,
            false,
            flags,
        )
        .unwrap();
        assert_eq!(r.jd, 2_451_545.996_101_840_4);
        assert_eq!(r.pos[0], 281.391_379_906_908_24);
        assert_eq!(r.cusps[10], 281.391_379_848_427_1);
        assert_eq!(r.ascmc[1], 281.391_379_848_427_1);
        assert!(next_aspect_cusp(
            Body::SUN,
            0.0,
            0,
            2_451_545.0,
            0.0,
            0.0,
            HouseSystem::PLACIDUS,
            false,
            flags
        )
        .is_none());
    }

    #[test]
    fn cusp_aspect2_selects_nearest_branch() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_aspect_cusp2(
            Body::SUN,
            60.0,
            10,
            2_451_545.0,
            48.85,
            2.35,
            HouseSystem::PLACIDUS,
            false,
            flags,
        )
        .unwrap();
        assert_eq!(r.jd, 2_451_545.164_090_501);
        assert_eq!(r.pos[0], 280.543_076_978_861_56);
        assert_eq!(r.cusps[10], 340.543_077_139_527_8);
    }

    #[test]
    fn years_diff_exact_references() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        assert_eq!(years_diff(2_451_545.0, 2_451_545.0, flags), Ok(0.0));
        assert_eq!(
            years_diff(2_451_545.0, 2_452_275.484_4, flags),
            Ok(1.999_997_030_446_534_4)
        );
        assert_eq!(
            years_diff(2_452_275.484_4, 2_451_545.0, flags),
            Ok(-1.999_997_030_446_534_4)
        );
    }

    #[test]
    fn natal_angle_transits_match_references() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let args = (2_451_545.0, 2_451_545.0, 48.85, 2.35, HouseSystem::PLACIDUS);
        assert_eq!(
            transit_to_degree(Body::SUN, 90.0, args.1, flags, false),
            Ok(2_451_716.569_027_677_6)
        );
        assert_eq!(
            mc_transit_ut(
                Body::SUN,
                args.0,
                args.1,
                args.2,
                args.3,
                args.4,
                flags,
                false
            ),
            Ok(2_451_546.378_275_586_3)
        );
        assert_eq!(
            ic_transit_ut(
                Body::SUN,
                args.0,
                args.1,
                args.2,
                args.3,
                args.4,
                flags,
                false
            ),
            Ok(2_451_728.919_015_651_6)
        );
        assert_eq!(
            asc_transit_ut(
                Body::SUN,
                args.0,
                args.1,
                args.2,
                args.3,
                args.4,
                flags,
                false
            ),
            Ok(2_451_650.963_376_643_6)
        );
        assert_eq!(
            dsc_transit_ut(
                Body::SUN,
                args.0,
                args.1,
                args.2,
                args.3,
                args.4,
                flags,
                false
            ),
            Ok(2_451_837.362_641_580_4)
        );
    }

    #[test]
    fn house_number_handles_boundaries_and_wrap() {
        let cusps = [
            0.0, 10.0, 40.0, 70.0, 100.0, 130.0, 160.0, 190.0, 220.0, 250.0, 280.0, 310.0, 340.0,
        ];
        let cases = [
            (10.0, 1),
            (39.999, 1),
            (40.0, 2),
            (339.999, 11),
            (340.0, 12),
            (0.0, 12),
            (370.0, 1),
        ];
        for (lon, expected) in cases {
            assert_eq!(planet_house_number(lon, &cusps), expected);
        }
    }

    #[test]
    fn mc_distance_and_orb_boundaries() {
        assert_eq!(distance_to_mc(350.0, 10.0), 20.0);
        assert_eq!(distance_to_mc(20.0, 10.0), -10.0);
        assert!(planet_conjunct_mc(350.0, 10.0, 20.0));
        assert!(!planet_conjunct_mc(350.0, 10.0, 19.999));
    }

    #[test]
    fn lower_meridian_reference() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r =
            lower_meridian_transit_ut(Body::SUN, 2_451_545.0, [2.35, 48.85, 35.0], flags).unwrap();
        assert_eq!(r.ret_flags, 0);
        assert_eq!(r.tret, 2_451_546.496_264_240_7);
    }

    #[test]
    fn search_options_store_builder_values() {
        let options = SearchOptions::new(Body::MARS, 2_451_545.0)
            .aspect(90.0)
            .cusp(10, 48.85, 2.35, HouseSystem::KOCH)
            .natal_chart(2_440_000.0, 40.0, -3.0, HouseSystem::WHOLE_SIGN)
            .backward(true)
            .flags(CalcFlags::SPEED);
        assert_eq!(options.body, Body::MARS);
        assert_eq!(options.jd_start, 2_451_545.0);
        assert_eq!(options.aspect, 90.0);
        assert!(options.backward);
        assert_eq!(options.flags, CalcFlags::SPEED);
        assert_eq!(options.cusp, Some(10));
        assert_eq!((options.lat, options.lon), (40.0, -3.0));
        assert_eq!(options.hsys, HouseSystem::WHOLE_SIGN);
        assert_eq!(options.jd_natal, Some(2_440_000.0));
    }
}

// ── SearchOptions builder ─────────────────────────────────────────────────────

/// Builder for aspect and angle transit searches.
///
/// # Example — planet aspecting a house cusp
/// ```no_run
/// # use celestial_core::*;
/// # use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let jd = SearchOptions::new(Body::SATURN, 2_451_545.0)
///     .aspect(90.0)
///     .cusp(10, 48.85, 2.35, HouseSystem::PLACIDUS)
///     .flags(CalcFlags::BUILTIN)
///     .search_cusp();
/// if let Some(r) = jd { println!("Saturn square MC at JD {}", r.jd); }
/// ```
///
/// # Example — planet transiting a natal angle
/// ```no_run
/// # use celestial_core::*;
/// # use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let jd = SearchOptions::new(Body::SATURN, 2_460_000.0)
///     .natal_chart(2_451_545.0, 48.85, 2.35, HouseSystem::PLACIDUS)
///     .flags(CalcFlags::BUILTIN)
///     .search_mc_transit()
///     .unwrap();
/// println!("Saturn transits natal MC at JD {jd}");
/// ```
#[derive(Debug, Clone)]
pub struct SearchOptions {
    body: Body,
    jd_start: f64,
    aspect: f64,
    backward: bool,
    flags: CalcFlags,
    // cusp search
    cusp: Option<usize>,
    lat: f64,
    lon: f64,
    hsys: HouseSystem,
    // natal angle transit
    jd_natal: Option<f64>,
}

impl SearchOptions {
    /// Create a new builder with a body and start JD.
    pub fn new(body: Body, jd_start: f64) -> Self {
        Self {
            body,
            jd_start,
            aspect: 0.0,
            backward: false,
            flags: crate::body::CalcFlags::BUILTIN,
            cusp: None,
            lat: 0.0,
            lon: 0.0,
            hsys: HouseSystem::PLACIDUS,
            jd_natal: None,
        }
    }

    /// Set the aspect angle in degrees (default: 0° = conjunction).
    pub fn aspect(mut self, degrees: f64) -> Self {
        self.aspect = degrees;
        self
    }

    /// Search for an aspect to a house cusp.
    ///
    /// `cusp` is 1–12; lat/lon are the observer's geographic coordinates.
    pub fn cusp(mut self, cusp: usize, lat: f64, lon: f64, hsys: HouseSystem) -> Self {
        self.cusp = Some(cusp);
        self.lat = lat;
        self.lon = lon;
        self.hsys = hsys;
        self
    }

    /// Set the natal chart for angle transit searches.
    pub fn natal_chart(mut self, jd_natal: f64, lat: f64, lon: f64, hsys: HouseSystem) -> Self {
        self.jd_natal = Some(jd_natal);
        self.lat = lat;
        self.lon = lon;
        self.hsys = hsys;
        self
    }

    /// Search backwards in time (default: forward).
    pub fn backward(mut self, back: bool) -> Self {
        self.backward = back;
        self
    }

    /// Set calculation flags (default: `CalcFlags::BUILTIN`).
    pub fn flags(mut self, flags: CalcFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Execute an aspect-to-house-cusp search.
    ///
    /// Requires `.cusp()` to have been called; returns `None` if no event is found.
    pub fn search_cusp(self) -> Option<AspectCuspResult> {
        let cusp = self.cusp.unwrap_or(1);
        next_aspect_cusp(
            self.body,
            self.aspect,
            cusp,
            self.jd_start,
            self.lat,
            self.lon,
            self.hsys,
            self.backward,
            self.flags,
        )
    }

    /// Execute a Midheaven (MC) transit search.
    ///
    /// Requires `.natal_chart()` to have been called.
    pub fn search_mc_transit(self) -> crate::Result<f64> {
        mc_transit_ut(
            self.body,
            self.jd_natal.unwrap_or(self.jd_start),
            self.jd_start,
            self.lat,
            self.lon,
            self.hsys,
            self.flags,
            self.backward,
        )
    }

    /// Execute an IC transit search.
    pub fn search_ic_transit(self) -> crate::Result<f64> {
        ic_transit_ut(
            self.body,
            self.jd_natal.unwrap_or(self.jd_start),
            self.jd_start,
            self.lat,
            self.lon,
            self.hsys,
            self.flags,
            self.backward,
        )
    }

    /// Execute an Ascendant transit search.
    pub fn search_asc_transit(self) -> crate::Result<f64> {
        asc_transit_ut(
            self.body,
            self.jd_natal.unwrap_or(self.jd_start),
            self.jd_start,
            self.lat,
            self.lon,
            self.hsys,
            self.flags,
            self.backward,
        )
    }

    /// Execute a Descendant transit search.
    pub fn search_dsc_transit(self) -> crate::Result<f64> {
        dsc_transit_ut(
            self.body,
            self.jd_natal.unwrap_or(self.jd_start),
            self.jd_start,
            self.lat,
            self.lon,
            self.hsys,
            self.flags,
            self.backward,
        )
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::*;

    #[test]
    fn fixed_aspect_covers_backward_and_second_branch() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let backward = next_aspect(Body::SUN, 60.0, 90.0, 2_451_800.0, true, 400.0, flags);
        assert_eq!(backward.unwrap().jd, 2_451_654.274_029_908_7);
        assert!(next_aspect(Body::SUN, 60.0, 90.0, 2_451_800.0, true, 1.0, flags).is_none());
        let second = next_aspect2(Body::SUN, 60.0, 90.0, 2_451_700.0, false, 400.0, flags);
        assert_eq!(second.unwrap().jd, 2_451_779.317_375_738);
        let back_second = next_aspect2(Body::SUN, 60.0, 90.0, 2_451_800.0, true, 400.0, flags);
        assert_eq!(back_second.unwrap().jd, 2_451_779.317_375_738);
    }

    #[test]
    fn fixed_aspect2_covers_opposition_boundary() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_aspect2(Body::SUN, 180.0, 90.0, 2_451_545.0, false, 400.0, flags);
        assert_eq!(r.unwrap().jd, 2_451_900.062_017_127);
    }

    #[test]
    fn moving_aspect_covers_signed_branches() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let direct = next_aspect_with(Body::MOON, 90.0, Body::SUN, 2_451_545.0, false, 40.0, flags);
        assert_eq!(direct.unwrap().jd, 2_451_571.831_738_567);
        let backward =
            next_aspect_with2(Body::MOON, 90.0, Body::SUN, 2_451_570.0, true, 40.0, flags);
        assert_eq!(backward.unwrap().jd, 2_451_558.066_024_355_6);
    }

    #[test]
    fn moving_aspect2_covers_opposition_boundary() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_aspect_with2(
            Body::MOON,
            180.0,
            Body::SUN,
            2_451_545.0,
            false,
            40.0,
            flags,
        );
        assert_eq!(r.unwrap().jd, 2_451_564.695_414_848_6);
    }

    #[test]
    fn cusp_aspects_cover_backward_and_second_branch() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let backward = next_aspect_cusp(
            Body::SUN,
            60.0,
            10,
            2_451_546.0,
            48.85,
            2.35,
            HouseSystem::PLACIDUS,
            true,
            flags,
        );
        assert_eq!(backward.unwrap().jd, 2_451_545.164_090_501);
        let second = next_aspect_cusp2(
            Body::SUN,
            60.0,
            10,
            2_451_545.5,
            48.85,
            2.35,
            HouseSystem::PLACIDUS,
            false,
            flags,
        );
        assert_eq!(second.unwrap().jd, 2_451_545.819_909_987);
    }

    #[test]
    fn cusp_aspect2_covers_opposition_boundary() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_aspect_cusp2(
            Body::SUN,
            180.0,
            10,
            2_451_545.0,
            48.85,
            2.35,
            HouseSystem::PLACIDUS,
            false,
            flags,
        );
        assert_eq!(r.unwrap().jd, 2_451_545.495_938_034);
    }

    #[test]
    fn sidereal_searches_preserve_frame_flags() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SIDEREAL | CalcFlags::SPEED;
        let cusp = next_aspect_cusp(
            Body::SUN,
            0.0,
            10,
            2_451_545.0,
            48.85,
            2.35,
            HouseSystem::PLACIDUS,
            false,
            flags,
        );
        assert_eq!(cusp.unwrap().jd, 2_451_545.921_391_65);
        let lower = lower_meridian_transit_ut(Body::SUN, 2_451_545.0, [2.35, 48.85, 35.0], flags);
        assert_eq!(lower.unwrap().tret, 2_451_546.421_551_781);
        assert_eq!(
            years_diff(2_451_545.0, 2_452_275.484_4, flags),
            Ok(1.999_919_422_350_134_9)
        );
    }

    #[test]
    fn bisect_diff_rejects_tolerance_boundary() {
        let r = bisect_diff_zero(-1.0, 3.0, -1e-8, &|x| x * 1e-8);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn house_number_handles_equal_adjacent_cusps() {
        let cusps = [
            0.0, 0.0, 10.0, 10.0, 40.0, 70.0, 100.0, 130.0, 160.0, 190.0, 220.0, 250.0, 280.0,
        ];
        assert_eq!(planet_house_number(20.0, &cusps), 2);
        assert_eq!(planet_house_number(10.0, &cusps), 2);
    }

    #[test]
    fn search_options_execute_all_routes() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let base = SearchOptions::new(Body::SUN, 2_451_545.0)
            .natal_chart(2_451_545.0, 48.85, 2.35, HouseSystem::PLACIDUS)
            .flags(flags);
        assert_eq!(
            base.clone().search_mc_transit(),
            Ok(2_451_546.378_275_586_3)
        );
        assert_eq!(
            base.clone().search_ic_transit(),
            Ok(2_451_728.919_015_651_6)
        );
        assert_eq!(
            base.clone().search_asc_transit(),
            Ok(2_451_650.963_376_643_6)
        );
        assert_eq!(base.search_dsc_transit(), Ok(2_451_837.362_641_580_4));
        let cusp = SearchOptions::new(Body::SUN, 2_451_545.0)
            .cusp(10, 48.85, 2.35, HouseSystem::PLACIDUS)
            .flags(flags)
            .search_cusp();
        assert_eq!(cusp.unwrap().jd, 2_451_545.996_101_840_4);
    }

    #[test]
    fn zero_stop_days_use_default_windows() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let retro = next_retro(Body::MERCURY, 2_451_545.0, false, 0.0, flags);
        assert_eq!(retro.unwrap().jd, 2_451_596.021_076_591_7);
        let fixed = next_aspect(Body::SUN, 60.0, 90.0, 2_451_545.0, false, 0.0, flags);
        assert_eq!(fixed.unwrap().jd, 2_451_654.274_029_908_7);
        let moving = next_aspect_with(
            Body::MARS,
            0.0,
            Body::JUPITER,
            2_451_545.0,
            false,
            0.0,
            flags,
        );
        assert_eq!(moving.unwrap().jd, 2_451_640.228_804_409_5);
    }

    #[test]
    fn fixed_aspect_stop_boundary_is_inclusive() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let forward = next_aspect(Body::SUN, 60.0, 90.0, 2_451_545.0, false, 0.0, flags).unwrap();
        let forward_span = forward.jd - 2_451_545.0;
        assert!(next_aspect(
            Body::SUN,
            60.0,
            90.0,
            2_451_545.0,
            false,
            forward_span,
            flags
        )
        .is_some());
        let backward = next_aspect(Body::SUN, 60.0, 90.0, 2_451_800.0, true, 0.0, flags).unwrap();
        let backward_span = 2_451_800.0 - backward.jd;
        assert!(next_aspect(
            Body::SUN,
            60.0,
            90.0,
            2_451_800.0,
            true,
            backward_span,
            flags
        )
        .is_some());
    }

    #[test]
    fn fixed_aspect2_backward_can_select_first_branch() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_aspect2(Body::SUN, 60.0, 90.0, 2_451_700.0, true, 400.0, flags);
        assert_eq!(r.unwrap().jd, 2_451_654.274_029_908_7);
    }

    #[test]
    fn moving_aspect2_selects_first_branch_both_ways() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let forward =
            next_aspect_with2(Body::MOON, 90.0, Body::SUN, 2_451_560.0, false, 40.0, flags);
        assert_eq!(forward.unwrap().jd, 2_451_571.831_738_567);
        let backward =
            next_aspect_with2(Body::MOON, 90.0, Body::SUN, 2_451_580.0, true, 40.0, flags);
        assert_eq!(backward.unwrap().jd, 2_451_571.831_738_567);
    }

    #[test]
    fn cusp_aspect2_backward_can_select_first_branch() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
        let r = next_aspect_cusp2(
            Body::SUN,
            60.0,
            10,
            2_451_545.5,
            48.85,
            2.35,
            HouseSystem::PLACIDUS,
            true,
            flags,
        );
        assert_eq!(r.unwrap().jd, 2_451_545.164_090_501);
    }

    #[test]
    fn numeric_predicates_pin_boundaries() {
        assert!(below_tolerance(0.999, 1.0));
        assert!(!below_tolerance(1.0, 1.0));
        assert!(brackets_zero(-1.0, 1.0));
        assert!(brackets_zero(1.0, -1.0));
        assert!(brackets_zero(0.0, 1.0));
        assert!(!brackets_zero(1.0, 1.0));
        assert!(!brackets_zero(-1.0, -1.0));
    }

    #[test]
    fn continuous_crossing_rejects_wrap_boundary() {
        assert!(continuous_crossing(-1.0, 1.0));
        assert!(!continuous_crossing(-90.0, 90.0));
        assert!(!continuous_crossing(1.0, 2.0));
    }

    #[test]
    fn directional_helpers_pin_both_directions() {
        assert_eq!(directional_limit(10.0, 2.0, false), 12.0);
        assert_eq!(directional_limit(10.0, 2.0, true), 8.0);
        assert_eq!(opposite_degree(350.0), 170.0);
        assert_eq!(opposite_degree(10.0), 190.0);
        assert_eq!(opposite_degree(180.0), 0.0);
        assert_eq!(opposite_degree(0.0), 180.0);
    }

    #[test]
    fn speed_mask_preserves_non_speed_flags() {
        let flags = CalcFlags::BUILTIN | CalcFlags::SIDEREAL | CalcFlags::SPEED | CalcFlags::SPEED3;
        assert_eq!(
            without_speed(flags),
            CalcFlags::BUILTIN | CalcFlags::SIDEREAL
        );
        assert_eq!(without_speed(CalcFlags::SPEED), CalcFlags(0));
    }

    #[test]
    fn nearest_result_covers_all_shapes() {
        let jd = |value: &i32| f64::from(*value);
        assert_eq!(nearest_result(Some(1), Some(2), false, jd), Some(1));
        assert_eq!(nearest_result(Some(1), Some(2), true, jd), Some(2));
        assert_eq!(nearest_result(Some(2), Some(1), false, jd), Some(1));
        assert_eq!(nearest_result(Some(2), Some(1), true, jd), Some(2));
        assert_eq!(nearest_result(Some(1), None, false, jd), Some(1));
        assert_eq!(nearest_result(None, Some(2), false, jd), Some(2));
        assert_eq!(nearest_result(None, None, false, jd), None);
        assert_eq!(
            nearest_result(Some((1, 1)), Some((2, 1)), false, |value| {
                f64::from(value.1)
            }),
            Some((2, 1))
        );
    }

    #[test]
    fn search_window_includes_limits() {
        let cases = [
            (12.0, 10.0, 2.0, false, true),
            (12.1, 10.0, 2.0, false, false),
            (8.0, 10.0, 2.0, true, true),
            (7.9, 10.0, 2.0, true, false),
            (100.0, 10.0, 0.0, false, true),
        ];
        for (jd, start, stop, backward, expected) in cases {
            assert_eq!(within_search_window(jd, start, stop, backward), expected);
        }
    }

    #[test]
    fn lower_scan_plan_pins_interval() {
        let (start, steps) = lower_scan_plan(10.0, 1.0 / 720.0);
        assert_eq!(start, 10.0 + 1.0 / 240.0);
        assert_eq!(steps, 717);
    }

    #[test]
    fn angle_interval_is_half_open_and_wrap_aware() {
        assert!(angle_in_interval(10.0, 10.0, 40.0));
        assert!(!angle_in_interval(40.0, 10.0, 40.0));
        assert!(angle_in_interval(350.0, 340.0, 10.0));
        assert!(angle_in_interval(0.0, 340.0, 10.0));
        assert!(!angle_in_interval(10.0, 340.0, 10.0));
        assert!(angle_in_interval(20.0, 10.0, 10.0));
    }
}
