//! Iterative aspect and retrograde search functions.

use crate::body::{Body, CalcFlags, HouseSystem};
use crate::functions::houses::houses;

// ─── Internal helpers ─────────────────────────────────────────────────────────

use crate::{diff_deg, diff_deg_signed, norm_deg};

#[inline]
fn norm360(d: f64) -> f64 {
    norm_deg(d)
}

/// Approximate retrograde station duration (days) for a planet — used as search step.
fn approx_retro_time(body: Body) -> f64 {
    match body.as_raw() {
        0 | 1 | 14 => 0.5, // Sun, Moon, Earth — no retrograde
        2 => 18.0,         // Mercury
        3 => 38.0,         // Venus
        4 => 57.0,         // Mars
        5 => 115.0,        // Jupiter
        6 => 130.0,        // Saturn
        7 => 147.0,        // Uranus
        8 => 154.0,        // Neptune
        9 => 153.0,        // Pluto
        15 => 124.0,       // Chiron
        _ => 0.5,
    }
}

pub struct RetroResult {
    pub jd: f64,
    pub pos: [f64; 6],
}

/// Find the next retrograde or direct station for a planet.
///
/// Returns `None` if the planet cannot go retrograde (Sun, Moon, Earth),
/// or if the station isn't found within the search window.
pub fn next_retro(
    body: Body,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: CalcFlags,
) -> Option<RetroResult> {
    use crate::functions::calc::calc_ut;

    // Bodies that don't retrograde
    if matches!(body.as_raw(), 0 | 1 | 14) {
        return None;
    }

    let step = approx_retro_time(body);
    let dir = if backward { -step } else { step };

    // Bracket search: find sign change in longitude speed
    let speed_at = |jd: f64| -> Option<f64> {
        calc_ut(jd, body, flags | CalcFlags::SPEED)
            .ok()
            .map(|p| p.speed_lon)
    };

    let mut jd = jd_start;
    let max_jd = if stop_days > 0.0 {
        jd_start + if backward { -stop_days } else { stop_days }
    } else {
        jd_start + if backward { -50_000.0 } else { 50_000.0 }
    };

    let mut s0 = speed_at(jd)?;

    loop {
        jd += dir;
        if backward {
            if jd < max_jd {
                return None;
            }
        } else {
            if jd > max_jd {
                return None;
            }
        }

        let s1 = speed_at(jd)?;
        if s0 * s1 < 0.0 {
            // Sign change — bisect
            let (mut ja, mut jb) = (jd - dir, jd);
            for _ in 0..50 {
                let jm = (ja + jb) / 2.0;
                let sm = speed_at(jm)?;
                if sm.abs() < 1e-9 {
                    return Some(RetroResult {
                        jd: jm,
                        pos: calc_ut(jm, body, flags)
                            .ok()
                            .map(|p| [p.lon, p.lat, p.dist, p.speed_lon, p.speed_lat, p.speed_dist])
                            .unwrap_or([0.0; 6]),
                    });
                }
                if s0 * sm < 0.0 {
                    jb = jm;
                } else {
                    ja = jm;
                    s0 = sm;
                }
            }
            let jd_ret = (ja + jb) / 2.0;
            let p = calc_ut(jd_ret, body, flags).ok()?;
            return Some(RetroResult {
                jd: jd_ret,
                pos: [p.lon, p.lat, p.dist, p.speed_lon, p.speed_lat, p.speed_dist],
            });
        }
        s0 = s1;
    }
}

/// Result of an aspect search.
#[derive(Debug, Clone, PartialEq)]
pub struct AspectResult {
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
    let jd = find_crossing(body.as_raw(), target, jd_start, !backward, flags.as_raw())?;

    // Check stop limit
    if stop_days > 0.0 {
        if !backward && jd > jd_start + stop_days {
            return None;
        }
        if backward && jd < jd_start - stop_days {
            return None;
        }
    }
    let p = calc_ut(jd, body, flags).ok()?;
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
    if asp_n == 0.0 || asp_n == 180.0 {
        return r1;
    }
    let r2 = next_aspect(body, -asp_n, fixed_pt, jd_start, backward, stop_days, flags);
    match (r1, r2) {
        (Some(a), Some(b)) => Some(if (!backward && a.jd < b.jd) || (backward && a.jd > b.jd) {
            a
        } else {
            b
        }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Find next exact aspect between two moving planets.
/// `aspect` in `[0, 360)`.
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
    let dir = if backward { -step } else { step };

    let diff_at = |jd: f64| -> Option<f64> {
        let p1 = calc_ut(jd, body, flags).ok()?.lon;
        let p2 = calc_ut(jd, other, flags).ok()?.lon;
        Some(diff_deg_signed(p1 + aspect, p2))
    };

    let max_jd = if stop_days > 0.0 {
        jd_start + if backward { -stop_days } else { stop_days }
    } else {
        jd_start + if backward { -100_000.0 } else { 100_000.0 }
    };

    let mut jd = jd_start;
    let mut d0 = diff_at(jd)?;

    loop {
        jd += dir;
        if backward {
            if jd < max_jd {
                return None;
            }
        } else {
            if jd > max_jd {
                return None;
            }
        }

        let d1 = diff_at(jd)?;
        if d0 * d1 <= 0.0 && (d1 - d0).abs() < 180.0 {
            // Bisect
            let (mut ja, mut jb) = (jd - dir, jd);
            let (mut da, _) = (d0, d1);
            for _ in 0..60 {
                let jm = (ja + jb) / 2.0;
                let dm = diff_at(jm)?;
                if dm.abs() < 1e-8 {
                    break;
                }
                if da * dm <= 0.0 {
                    jb = jm;
                } else {
                    ja = jm;
                    da = dm;
                }
            }
            let jd_ret = (ja + jb) / 2.0;
            let p1 = calc_ut(jd_ret, body, flags).ok()?;
            let p2 = calc_ut(jd_ret, other, flags).ok()?;
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
    if asp_n == 0.0 || asp_n == 180.0 {
        return r1;
    }
    let r2 = next_aspect_with(body, -asp_n, other, jd_start, backward, stop_days, flags);
    match (r1, r2) {
        (Some(a), Some(b)) => Some(if (!backward && a.jd < b.jd) || (backward && a.jd > b.jd) {
            a
        } else {
            b
        }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Result of an aspect-to-cusp search.
#[derive(Debug, Clone, PartialEq)]
pub struct AspectCuspResult {
    pub jd: f64,
    pub pos: [f64; 6],
    pub cusps: [f64; 13],
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
    let dir = if backward { -step } else { step };

    let diff_at = |jd: f64| -> Option<f64> {
        let p = calc_ut(jd, body, flags).ok()?.lon;
        let hr = houses(jd, lat, lon, hsys).ok()?;
        Some(diff_deg_signed(p + aspect, hr.cusps[cusp]))
    };

    let max_jd = jd_start + if backward { -400.0 } else { 400.0 };
    let mut jd = jd_start;
    let mut d0 = diff_at(jd)?;

    loop {
        jd += dir;
        if backward {
            if jd < max_jd {
                return None;
            }
        } else {
            if jd > max_jd {
                return None;
            }
        }

        let d1 = diff_at(jd)?;
        if d0 * d1 <= 0.0 && (d1 - d0).abs() < 180.0 {
            let (mut ja, mut jb) = (jd - dir, jd);
            let (mut da, _) = (d0, d1);
            for _ in 0..60 {
                let jm = (ja + jb) / 2.0;
                let dm = diff_at(jm)?;
                if dm.abs() < 1e-8 {
                    break;
                }
                if da * dm <= 0.0 {
                    jb = jm;
                } else {
                    ja = jm;
                    da = dm;
                }
            }
            let jd_ret = (ja + jb) / 2.0;
            let p = calc_ut(jd_ret, body, flags).ok()?;
            let hr = houses(jd_ret, lat, lon, hsys).ok()?;
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
    if asp_n == 0.0 || asp_n == 180.0 {
        return r1;
    }
    let r2 = next_aspect_cusp(
        body, -asp_n, cusp, jd_start, lat, lon, hsys, backward, flags,
    );
    match (r1, r2) {
        (Some(a), Some(b)) => Some(if (!backward && a.jd < b.jd) || (backward && a.jd > b.jd) {
            a
        } else {
            b
        }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Number of astrological years between two Julian days.
/// An "astrological year" = one solar revolution.
pub fn years_diff(jd1: f64, jd2: f64, flags: CalcFlags) -> crate::Result<f64> {
    use crate::functions::calc::calc_ut;
    let sun1 = calc_ut(jd1, Body::SUN, flags)?.lon;
    let sun2 = calc_ut(jd2, Body::SUN, flags)?.lon;
    let mut years = 0.0_f64;

    if jd1 < jd2 {
        let dec = diff_deg(sun2, sun1) / 360.0;
        let mut jd = jd1;
        loop {
            let r = crate::functions::motion::solcross(sun1, jd + 1e-5, CalcFlags::BUILTIN)
                .map_err(|e| crate::error::Error::Calc(e.to_string()))?;
            if r <= jd2 {
                years += 1.0;
                jd = r;
            } else {
                break;
            }
        }
        years += dec;
    } else if jd1 > jd2 {
        let dec = diff_deg(sun1, sun2) / 360.0;
        let mut jd = jd1;
        loop {
            let _r = crate::functions::motion::solcross(sun1, jd - 1e-5, CalcFlags::BUILTIN)
                .map_err(|e| crate::error::Error::Calc(e.to_string()))?;
            // backward crossing returns future JD when called with backward=false
            // so we need to search backward
            let rb = crate::astronomy::crossings::find_crossing(
                0,
                sun1,
                jd - 1e-5,
                false,
                flags.as_raw(),
            )
            .ok_or_else(|| crate::error::Error::Calc("no crossing".into()))?;
            if rb >= jd2 {
                years -= 1.0;
                jd = rb;
            } else {
                break;
            }
        }
        years -= dec;
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
    let window_days: f64 = match body.as_raw() {
        0..=3 => 400.0, // Sun, Moon, Mercury, Venus — fast movers
        4 => 750.0,     // Mars — 687 day orbital period
        5 => 4500.0,    // Jupiter
        6 => 11000.0,   // Saturn
        7 => 31000.0,   // Uranus
        8 => 61000.0,   // Neptune
        9 => 95000.0,   // Pluto — 248 year orbit
        _ => 800.0,     // Nodes, Chiron, etc.
    };
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
    let chart = crate::functions::houses::houses(jd_natal, lat, lon, hsys)
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
    let chart = crate::functions::houses::houses(jd_natal, lat, lon, hsys)
        .map_err(|e| crate::Error::Calc(format!("ic_transit_ut: {e}")))?;
    let natal_ic = (chart.ascmc[1] + 180.0).rem_euclid(360.0);
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
    let chart = crate::functions::houses::houses(jd_natal, lat, lon, hsys)
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
    let chart = crate::functions::houses::houses(jd_natal, lat, lon, hsys)
        .map_err(|e| crate::Error::Calc(format!("dsc_transit_ut: {e}")))?;
    let natal_dsc = (chart.ascmc[0] + 180.0).rem_euclid(360.0);
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
/// let chart = houses(jd, 48.85, 2.35, HouseSystem::PLACIDUS).unwrap();
/// let sun = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN).unwrap();
/// let house = planet_house_number(sun.lon, &chart.cusps);
/// println!("Sun is in house {house}");
/// ```
pub fn planet_house_number(planet_lon: f64, cusps: &[f64; 13]) -> u8 {
    // Cusps are in [0,360) in ascending order for most systems.
    // Walk from cusp 1; the planet is in house h when it is between cusps[h] and cusps[h+1].
    // For the 12th house the upper bound wraps to cusps[1].
    for h in 1usize..=12 {
        let lo = cusps[h];
        let hi = if h == 12 { cusps[1] } else { cusps[h + 1] };
        let lon = planet_lon.rem_euclid(360.0);
        // Handle wrap-around (e.g. cusp 12 = 350°, cusp 1 = 10°)
        if lo < hi {
            if lon >= lo && lon < hi {
                return h as u8;
            }
        } else {
            // Wraps through 0°
            if lon >= lo || lon < hi {
                return h as u8;
            }
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
pub fn distance_to_mc(planet_lon: f64, mc_lon: f64) -> f64 {
    // Positive = MC is ahead of planet (planet approaching MC)
    // Negative = planet has already passed MC
    crate::functions::utils::diff_deg_signed(mc_lon, planet_lon)
}

/// Is a planet within `orb` degrees of the Midheaven?
///
/// A shorthand for `distance_to_mc(planet_lon, mc_lon).abs() <= orb`.
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
        jd_start,
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
    let flags_pos = flags & !CalcFlags::SPEED;

    let ha_at = |jd: f64| -> f64 {
        let pos = match crate::calc_ut(jd, body, flags_pos) {
            Ok(p) => p,
            Err(_) => return 0.0, // treat as HA=0 if calc fails
        };
        // Convert ecliptic lon/lat → RA
        let eps = crate::true_obliquity(jd).to_radians();
        let lon_r = pos.lon.to_radians();
        let lat_r = pos.lat.to_radians();
        // Meeus eq.13.3: RA = atan2(sin(lon)*cos(eps) - tan(lat)*sin(eps), cos(lon))
        let ra = (lon_r.sin() * eps.cos() - lat_r.tan() * eps.sin())
            .atan2(lon_r.cos())
            .to_degrees()
            .rem_euclid(360.0);
        let gast = crate::sidtime(jd) * 15.0; // hours → degrees
        (gast + lon - ra).rem_euclid(360.0)
    };

    // Signed distance from 180° in (-180, +180]
    let diff = |jd: f64| -> f64 {
        let ha = ha_at(jd);
        let d = ha - 180.0;
        if d > 180.0 {
            d - 360.0
        } else if d <= -180.0 {
            d + 360.0
        } else {
            d
        }
    };

    // Start from upper transit to find the NEXT lower transit (~12h later)
    let upper = crate::functions::motion::rise_trans(
        jd_start,
        body,
        None,
        flags,
        crate::CALC_MTRANSIT,
        geopos,
        1013.25,
        15.0,
    )?;
    let mut jd = upper.tret + 0.1 / 24.0; // just after upper transit
    let limit = upper.tret + 1.0; // search 1 day ahead
    let mut d0 = diff(jd);

    while jd < limit {
        jd += step;
        let d1 = diff(jd);
        if d0 * d1 < 0.0 && (d1 - d0).abs() < 180.0 {
            // Bisect to refine
            let (mut lo, mut hi) = (jd - step, jd);
            let mut dlo = d0;
            for _ in 0..40 {
                let mid = (lo + hi) / 2.0;
                let dm = diff(mid);
                if dm.abs() < 1e-8 {
                    lo = mid;
                    break;
                }
                if dlo * dm <= 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                    dlo = dm;
                }
            }
            return Ok(RiseTransResult {
                ret_flags: 0,
                tret: (lo + hi) / 2.0,
            });
        }
        d0 = d1;
    }
    Err(crate::Error::RiseTrans(
        "lower meridian transit not found within 2 days".into(),
    ))
}

// ── SearchOptions builder ─────────────────────────────────────────────────────

/// Builder for aspect and angle transit searches.
///
/// Replaces `next_aspect_cusp`, `next_aspect_cusp2`, `mc_transit_ut`,
/// `ic_transit_ut`, `asc_transit_ut`, and `dsc_transit_ut` with a single
/// discoverable, forward-compatible API.
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
