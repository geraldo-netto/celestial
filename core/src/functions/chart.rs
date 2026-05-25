//! Complete chart calculation utilities.
//!
//! Functions for computing a full astrological chart in one call,
//! generating aspect tables, secondary progressions, solar/lunar returns,
//! solar arc directions, midpoints, Arabic parts, and planetary stations.

use crate::units::{Degrees, JulianDay, Latitude, Longitude};
use crate::body::{Body, CalcFlags, Calendar, HouseSystem};
use crate::diff_deg_signed;
use crate::error::{Error, Result};
use crate::functions::{calc::calc_ut, houses::houses};

/// Solar arc direction result: (arc_degrees, directed_positions, directed_mc)
pub type SolarArcResult = (f64, Vec<(Body, f64)>, f64);
/// Midpoint table entry: (body1, body2, midpoint_lon, planets_on_midpoint)
pub type MidpointEntry = (Body, Body, f64, Vec<(Body, f64)>);

// ─── Aspect table ──────────────────────────────────────────────────────────────

/// One aspect between two bodies in a chart.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChartAspect {
    /// First body number.
    pub body1: Body,
    /// Second body number.
    pub body2: Body,
    /// Aspect angle (0 = conjunction, 60 = sextile, 90 = square, …).
    pub aspect: f64,
    /// Orb: how many degrees away from exact (always positive).
    pub orb: f64,
    /// `true` if the aspect is applying (planets moving toward exact).
    pub applying: bool,
}

/// Whether an aspect is **applying** (orb shrinking under current motion).
///
/// Derivation:
/// - `d|sep|/dt = sign(signed_sep) · rel_speed`   (separation grows when bodies move apart)
/// - `d(orb)/dt = sign(|sep| − aspect_deg) · d|sep|/dt`
///   (above exact: orb shrinks as |sep| falls; below exact: orb shrinks as |sep| rises)
/// - Applying ⇔ `d(orb)/dt < 0` ⇔ `rel_speed · signed_sep · (|sep| − aspect_deg) < 0`.
///
/// `signed_sep` is the signed angular separation (`diff_deg_signed(lon1, lon2)`, range
/// `(−180, 180]`), `rel_speed = spd1 − spd2` in deg/day, and `aspect_deg` is the
/// exact aspect angle (0, 60, 90, …). Returns `false` at the exact moment when any
/// factor is zero — callers wanting a tri-state should test that explicitly.
#[must_use]
#[inline]
pub fn is_applying(signed_sep: f64, rel_speed: f64, aspect_deg: f64) -> bool {
    rel_speed * signed_sep * (signed_sep.abs() - aspect_deg) < 0.0
}

/// Standard major aspect angles in degrees.
pub const MAJOR_ASPECTS: &[f64] = &[0.0, 60.0, 90.0, 120.0, 180.0];
/// All traditional aspects (major + minor).
pub const ALL_ASPECTS: &[f64] = &[
    0.0,   // Conjunction
    30.0,  // Semi-sextile
    45.0,  // Semi-square
    60.0,  // Sextile
    72.0,  // Quintile
    90.0,  // Square
    120.0, // Trine
    135.0, // Sesquiquadrate
    144.0, // Bi-quintile
    150.0, // Quincunx
    180.0, // Opposition
];

/// Compute all aspects between a list of planets in a chart.
///
/// `positions` — `(body_num, ecliptic_lon, daily_speed)` for each planet.
/// `aspects`   — which aspect angles to check (use [`MAJOR_ASPECTS`] or [`ALL_ASPECTS`]).
/// `orb`       — maximum orb in degrees for an aspect to be included.
///
/// Returns a list of [`ChartAspect`]s sorted by tightness (closest first).
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let jd = 2_451_545.0;
/// let bodies = [Body::SUN, Body::MOON, Body::MERCURY, Body::VENUS, Body::MARS, Body::JUPITER, Body::SATURN];
/// let positions: Vec<_> = bodies.iter().map(|&b| {
///     let p = calc_ut(JulianDay::new(jd), b, CalcFlags::BUILTIN | CalcFlags::SPEED).unwrap();
///     (b, p.lon, p.speed_lon)
/// }).collect();
/// let aspects = calc_chart_aspects(&positions, MAJOR_ASPECTS, 8.0);
/// for a in &aspects {
///     println!("{} /{:.0}° / {} orb={:.2}°", a.body1, a.aspect, a.body2, a.orb);
/// }
/// ```
#[must_use]
pub fn calc_chart_aspects(
    positions: &[(Body, f64, f64)],
    aspects: &[f64],
    orb: f64,
) -> Vec<ChartAspect> {
    // Upper bound: C(n,2) pairs × aspect_kinds (usually < 30 hit per chart).
    let n = positions.len();
    let mut result = Vec::with_capacity(n * n.saturating_sub(1) / 4);
    for i in 0..n {
        for j in (i + 1)..n {
            let (b1, lon1, spd1) = positions[i];
            let (b2, lon2, spd2) = positions[j];
            let signed = diff_deg_signed(lon1, lon2);
            let separation = signed.abs();
            for &asp in aspects {
                let diff = (separation - asp).abs();
                let diff = diff.min(360.0 - diff);
                if diff <= orb {
                    let applying = is_applying(signed, spd1 - spd2, asp);
                    result.push(ChartAspect {
                        body1: b1,
                        body2: b2,
                        aspect: asp,
                        orb: diff,
                        applying,
                    });
                }
            }
        }
    }
    result.sort_by(|a, b| a.orb.total_cmp(&b.orb));
    result
}

// ─── Sign ingress ─────────────────────────────────────────────────────────────

/// Find the next time a planet enters a new zodiac sign.
///
/// Returns `(jd, sign_number)` where `sign_number` is 0=Aries … 11=Pisces.
///
/// Uses `solcross_ut` / the crossing engine; works for any body.
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// // When does Saturn next enter a new sign?
/// let (jd, sign) = sign_ingress_ut(Body::SATURN, JulianDay::new(2_451_545.0), CalcFlags::BUILTIN, false).unwrap();
/// let sign_name = ["Aries","Taurus","Gemini","Cancer","Leo","Virgo",
///                  "Libra","Scorpio","Sagittarius","Capricorn","Aquarius","Pisces"];
/// println!("Saturn enters {}: JD {jd:.2}", sign_name[sign as usize]);
/// ```
pub fn sign_ingress_ut(
    body: Body,
    jd_start: JulianDay,
    flags: CalcFlags,
    backward: bool,
) -> Result<(f64, u8)> {
    let jd_start: f64 = jd_start.into();
    // Current position
    let pos = calc_ut(JulianDay::new(jd_start), body, flags)?;
    let current_sign = (pos.lon / 30.0).floor() as i32;

    // Next sign boundary
    // Forward: next cusp = (current_sign + 1) * 30°
    // Backward: previous cusp = current_sign * 30° (the one we already crossed)
    let target_sign = if backward {
        (current_sign.rem_euclid(12)) as f64 * 30.0
    } else {
        ((current_sign + 1) % 12) as f64 * 30.0
    };

    let window = body.ingress_search_window();
    let jd = crate::astronomy::crossings::find_crossing_window(
        body.as_raw(),
        target_sign,
        jd_start,
        !backward,
        flags.as_raw(),
        window,
    )
    .ok_or_else(|| {
        Error::Calc(format!(
            "sign_ingress_ut: body {body} did not change sign within window"
        ))
    })?;

    let sign_num = ((target_sign / 30.0).floor() as i32).rem_euclid(12) as u8;
    Ok((jd, sign_num))
}

// ─── Retrograde stations ──────────────────────────────────────────────────────

/// Retrograde and direct station times for a planet.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Stations {
    /// Julian day of retrograde station (planet turns retrograde).
    pub retrograde: f64,
    /// Julian day of direct station (planet turns direct).
    pub direct: f64,
}

/// Find the next retrograde and direct stations for a planet.
///
/// A station is when a planet's daily motion passes through zero —
/// either slowing to 0 before going retrograde, or speeding back to 0
/// before going direct.
///
/// Searches at 0.5-day steps from `jd_start`.
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let s = retrograde_station_ut(Body::MARS, JulianDay::new(2_451_545.0), CalcFlags::BUILTIN).unwrap();
/// println!("Mars retrograde: JD {:.2}", s.retrograde);
/// println!("Mars direct:     JD {:.2}", s.direct);
/// ```
/// Bisect to find the JD where a smooth scalar function `f(jd)` crosses zero,
/// given a bracket `[lo, hi]` known to contain a sign change with `f(lo)`
/// having the same sign as `prev_sign`.
///
/// Used by station-finders to locate the exact JD where speed = 0.
/// Returns the midpoint of the final bracket. Converges in ≤40 iterations
/// or when |f(mid)| < `tol`.
fn bisect_zero<F>(mut lo: f64, mut hi: f64, prev_sign: f64, tol: f64, mut f: F) -> f64
where
    F: FnMut(f64) -> f64,
{
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        let mid_v = f(mid);
        if mid_v.abs() < tol {
            return mid;
        }
        if prev_sign.signum() == mid_v.signum() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// Find the next retrograde and direct stations for `body` after `jd_start`.
///
/// Walks forward in 0.5-day steps watching the body's longitudinal speed
/// (`speed_lon` from `calc_ut`); when the speed changes sign, bisects to
/// find the exact zero-crossing. Returns the JDs of both stations as a
/// [`Stations`] struct (retrograde-onset and direct-onset).
///
/// Uses `bisect_zero` internally for the sign-crossing refinement.
///
/// # Search window
/// The function gives up after a planet-specific window:
/// Mars 800d, Jupiter 1500d, Saturn 2000d, Uranus 5000d, Neptune 10000d,
/// Pluto 30000d, others 200d. If neither station is found inside the
/// window, returns an error.
///
/// # Errors
/// - The Swiss Ephemeris dispatch fails (e.g., for `Body::SUN`,
///   `Body::MOON`, `Body::EARTH` — these never go retrograde from Earth's
///   POV)
/// - No station is found within the search window
pub fn retrograde_station_ut(body: Body, jd_start: JulianDay, flags: CalcFlags) -> Result<Stations> {
    let jd_start: f64 = jd_start.into();
    // Step size: 0.5d — stations last hours to a day; this gives good resolution
    let step = 0.5_f64;
    let flags_speed = flags | CalcFlags::SPEED;

    let speed_at =
        |jd: f64| -> Option<f64> { calc_ut(JulianDay::new(jd), body, flags_speed).ok().map(|p| p.speed_lon) };

    let window = body.retrograde_search_window();
    let (retrograde_jd, direct_jd) = scan_retrograde_stations(window, step, jd_start, speed_at);

    match (retrograde_jd, direct_jd) {
        (Some(r), Some(d)) => Ok(Stations {
            retrograde: r,
            direct: d,
        }),
        (Some(r), None) => Err(Error::Calc(format!(
            "retrograde station found at JD {r:.2} but no direct station in window"
        ))),
        _ => Err(Error::Calc(format!(
            "no retrograde station found for body {body} within search window"
        ))),
    }
}

/// Record a detected station into the retrograde/direct slots based on the
/// speed sign transition. First-wins: a slot is filled only once.
fn classify_station(
    prev_speed: f64,
    curr: f64,
    station_jd: f64,
    retrograde_jd: &mut Option<f64>,
    direct_jd: &mut Option<f64>,
) {
    if prev_speed > 0.0 && curr < 0.0 && retrograde_jd.is_none() {
        // Positive → negative: going retrograde
        *retrograde_jd = Some(station_jd);
    } else if prev_speed < 0.0 && curr > 0.0 && direct_jd.is_none() {
        // Negative → positive: going direct
        *direct_jd = Some(station_jd);
    }
}

/// Scan forward from `jd_start` over `window` days in `step` increments,
/// bisecting each speed sign-change to the exact station JD. Returns
/// `(retrograde_jd, direct_jd)`, stopping once both are found.
fn scan_retrograde_stations(
    window: f64,
    step: f64,
    jd_start: f64,
    speed_at: impl Fn(f64) -> Option<f64>,
) -> (Option<f64>, Option<f64>) {
    let mut retrograde_jd: Option<f64> = None;
    let mut direct_jd: Option<f64> = None;
    let mut jd = jd_start;
    let mut prev_speed = speed_at(jd_start).unwrap_or(0.0);

    for _ in 0..(window / step) as i32 + 1 {
        jd += step;
        let curr = speed_at(jd).unwrap_or(prev_speed);

        // Sign change in speed = station — bisect to find the exact zero
        if prev_speed * curr < 0.0 {
            let station_jd = bisect_zero(jd - step, jd, prev_speed, 1e-6, |t| {
                speed_at(t).unwrap_or(0.0)
            });
            classify_station(prev_speed, curr, station_jd, &mut retrograde_jd, &mut direct_jd);
            if retrograde_jd.is_some() && direct_jd.is_some() {
                break;
            }
        }
        prev_speed = curr;
    }

    (retrograde_jd, direct_jd)
}

// ─── Arabic Parts / Lots ─────────────────────────────────────────────────────

/// Compute an Arabic Part (Lot) from three chart factors.
///
/// The classical formula is: `Part = ASC + (body2 − body1)` (mod 360°).
/// For day charts: Lot of Fortune = ASC + Moon − Sun.
/// For night charts: reversed (swap Sun and Moon).
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let chart = houses(JulianDay::new(2_451_545.0), Latitude::new(48.85), Longitude::new(2.35), HouseSystem::PLACIDUS).unwrap();
/// let sun  = calc_ut(JulianDay::new(2_451_545.0), Body::SUN, CalcFlags::BUILTIN).unwrap();
/// let moon = calc_ut(JulianDay::new(2_451_545.0), Body::MOON, CalcFlags::BUILTIN).unwrap();
/// let lot_of_fortune = arabic_part(Degrees::new(chart.ascmc[0]), Longitude::new(moon.lon), Longitude::new(sun.lon));
/// ```
#[inline]
#[must_use]
pub fn arabic_part(asc: Degrees, body2: Longitude, body1: Longitude) -> f64 {
    let asc: f64 = asc.into();
    let body2: f64 = body2.into();
    let body1: f64 = body1.into();
    (asc + body2 - body1).rem_euclid(360.0)
}

/// Standard Arabic Parts with names.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArabicPart {
    /// Name of the part/lot, e.g. `"Part of Fortune"`.
    pub name: &'static str,
    /// Formula used to compute the part, e.g. `"Asc + Moon - Sun"`.
    pub formula: &'static str,
    /// Ecliptic longitude of the part, in degrees [0, 360).
    pub degree: f64,
}

/// Compute all seven traditional Arabic Parts/Lots for a chart.
///
/// The "day chart" rule reverses Fortune and Spirit when the Sun is above the horizon
/// (i.e., houses 7–12 in a diurnal chart). Pass `is_day = true` for day births.
///
/// Returns parts in traditional order:
/// Fortune, Spirit, Love, Necessity, Courage, Victory, Nemesis.
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let jd = 2_451_545.0;
/// let chart = houses(JulianDay::new(jd), Latitude::new(48.85), Longitude::new(2.35), HouseSystem::PLACIDUS).unwrap();
/// let sun  = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap();
/// let moon = calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN).unwrap();
/// let sat  = calc_ut(JulianDay::new(jd), Body::SATURN, CalcFlags::BUILTIN).unwrap();
/// let mar  = calc_ut(JulianDay::new(jd), Body::MARS, CalcFlags::BUILTIN).unwrap();
/// let jup  = calc_ut(JulianDay::new(jd), Body::JUPITER, CalcFlags::BUILTIN).unwrap();
/// let mer  = calc_ut(JulianDay::new(jd), Body::MERCURY, CalcFlags::BUILTIN).unwrap();
/// let ven  = calc_ut(JulianDay::new(jd), Body::VENUS, CalcFlags::BUILTIN).unwrap();
/// let asc  = chart.ascmc[0];
/// let is_day = planet_house_number(sun.lon, &chart.cusps) >= 7;
/// let parts = arabic_parts_seven(Degrees::new(asc), Longitude::new(sun.lon), Longitude::new(moon.lon), Longitude::new(sat.lon), Longitude::new(mar.lon),
///                                Longitude::new(jup.lon), Longitude::new(mer.lon), Longitude::new(ven.lon), is_day);
/// for p in &parts { println!("{}: {:.2}°", p.name, p.degree); }
/// ```
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn arabic_parts_seven(
    asc: Degrees,
    sun: Longitude,
    moon: Longitude,
    sat: Longitude,
    mar: Longitude,
    jup: Longitude,
    mer: Longitude,
    ven: Longitude,
    is_day: bool,
) -> Vec<ArabicPart> {
    let asc: f64 = asc.into();
    let sun: f64 = sun.into();
    let moon: f64 = moon.into();
    let sat: f64 = sat.into();
    let mar: f64 = mar.into();
    let jup: f64 = jup.into();
    let mer: f64 = mer.into();
    let ven: f64 = ven.into();
    // Day chart: Fortune = ASC + Moon - Sun; Spirit = ASC + Sun - Moon
    // Night chart: reversed
    let (fortune_b2, fortune_b1, spirit_b2, spirit_b1) = if is_day {
        (moon, sun, sun, moon)
    } else {
        (sun, moon, moon, sun)
    };

    vec![
        ArabicPart {
            name: "Lot of Fortune",
            formula: if is_day {
                "ASC + Moon − Sun"
            } else {
                "ASC + Sun − Moon"
            },
            degree: arabic_part(Degrees::new(asc), Longitude::new(fortune_b2), Longitude::new(fortune_b1)),
        },
        ArabicPart {
            name: "Lot of Spirit",
            formula: if is_day {
                "ASC + Sun − Moon"
            } else {
                "ASC + Moon − Sun"
            },
            degree: arabic_part(Degrees::new(asc), Longitude::new(spirit_b2), Longitude::new(spirit_b1)),
        },
        ArabicPart {
            name: "Lot of Love",
            formula: "ASC + Venus − Sun",
            degree: arabic_part(Degrees::new(asc), Longitude::new(ven), Longitude::new(sun)),
        },
        ArabicPart {
            name: "Lot of Necessity",
            formula: "ASC + Mercury − Moon",
            degree: arabic_part(Degrees::new(asc), Longitude::new(mer), Longitude::new(moon)),
        },
        ArabicPart {
            name: "Lot of Courage",
            formula: "ASC + Mars − Saturn",
            degree: arabic_part(Degrees::new(asc), Longitude::new(mar), Longitude::new(sat)),
        },
        ArabicPart {
            name: "Lot of Victory",
            formula: "ASC + Jupiter − Saturn",
            degree: arabic_part(Degrees::new(asc), Longitude::new(jup), Longitude::new(sat)),
        },
        ArabicPart {
            name: "Lot of Nemesis",
            formula: "ASC + Saturn − Sun",
            degree: arabic_part(Degrees::new(asc), Longitude::new(sat), Longitude::new(sun)),
        },
    ]
}

// ─── Secondary progressions ──────────────────────────────────────────────────

/// Progressed chart positions via the secondary progression (day-for-a-year) method.
///
/// In secondary progressions, 1 day after birth = 1 year of life.
/// So the "progressed chart" for age `years` is the natal chart calculated
/// at `jd_natal + years` (i.e., the sky positions `years` days after birth).
///
/// # Arguments
/// * `jd_natal`  — Julian Day of birth
/// * `years`     — age in years (can be fractional)
/// * `bodies`    — body numbers to compute (e.g. `&[SUN, Body::MOON, Body::MERCURY, ...]`)
/// * `lat`, `lon` — birth location for progressed angles
/// * `hsys`      — house system for progressed angles
/// * `flags`     — ephemeris flags
///
/// Returns `(progressed_bodies, progressed_chart)` where `progressed_bodies`
/// is a `Vec<(body_num, PlanetPos)>` and the chart has the progressed angles.
pub fn secondary_progressions(
    jd_natal: JulianDay,
    years: f64,
    bodies: &[Body],
    lat: Latitude,
    lon: Longitude,
    hsys: HouseSystem,
    flags: CalcFlags,
) -> Result<(
    Vec<(Body, crate::types::PlanetPos)>,
    crate::functions::houses::HouseResult,
)> {
    let jd_natal: f64 = jd_natal.into();
    let lat: f64 = lat.into();
    let lon: f64 = lon.into();
    // Progressed JD: 1 tropical year = 365.24219 days, but convention is simply
    // progressed_jd = natal_jd + years (one calendar day per year)
    let jd_progressed = jd_natal + years;

    let mut positions = Vec::with_capacity(bodies.len());
    for &body in bodies {
        let pos = calc_ut(JulianDay::new(jd_progressed), body, flags)?;
        positions.push((body, pos));
    }
    let prog_chart = houses(JulianDay::new(jd_progressed), Latitude::new(lat), Longitude::new(lon), hsys)?;
    Ok((positions, prog_chart))
}

// ─── Solar arc directions ─────────────────────────────────────────────────────

/// Solar arc directed positions.
///
/// In solar arc directions, all chart factors are advanced by the same arc —
/// the distance the Sun has moved since birth (approximately 1°/year).
/// The solar arc = `progressed_sun_lon - natal_sun_lon`.
///
/// # Returns
/// `(arc_degrees, directed_positions, directed_chart)`  
/// `arc_degrees` — the solar arc applied  
/// `directed_positions` — each natal longitude + arc (mod 360°)  
/// `directed_chart` — house cusps with MC advanced by arc
pub fn solar_arc_directions(
    jd_natal: JulianDay,
    years: f64,
    natal_positions: &[(Body, f64)], // (body, natal_lon)
    natal_mc: Degrees,
    flags: CalcFlags,
) -> Result<SolarArcResult> {
    let jd_natal: f64 = jd_natal.into();
    let natal_mc: f64 = natal_mc.into();
    // Progressed Sun position (using secondary progression timing)
    let jd_prog = jd_natal + years;
    let prog_sun = calc_ut(JulianDay::new(jd_prog), Body::SUN, flags)?;
    let natal_sun = calc_ut(JulianDay::new(jd_natal), Body::SUN, flags)?;

    // Solar arc = difference between progressed and natal Sun longitude
    let arc = (prog_sun.lon - natal_sun.lon).rem_euclid(360.0);

    // Apply arc to all natal positions
    let directed: Vec<(Body, f64)> = natal_positions
        .iter()
        .map(|&(body, natal_lon)| (body, (natal_lon + arc).rem_euclid(360.0)))
        .collect();

    // Directed MC = natal MC + arc
    let directed_mc = (natal_mc + arc).rem_euclid(360.0);

    Ok((arc, directed, directed_mc))
}

// ─── Solar return ─────────────────────────────────────────────────────────────

/// Find the Julian Day of a solar return for a given year.
///
/// A solar return is the exact moment when the transiting Sun returns to its
/// natal longitude. This happens approximately once per year.
///
/// # Arguments
/// * `jd_natal`    — natal Julian Day
/// * `return_year` — the Gregorian year of the desired return
/// * `flags`       — ephemeris flags
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// // Find solar return for year 2025
/// let natal_jd = 2_440_000.0; // some birth date
/// let sr_jd = solar_return_jd(JulianDay::new(natal_jd), 2025, CalcFlags::BUILTIN).unwrap();
/// println!("Solar return 2025: JD {sr_jd:.4}");
/// ```
pub fn solar_return_jd(jd_natal: JulianDay, return_year: i32, flags: CalcFlags) -> Result<f64> {
    let jd_natal: f64 = jd_natal.into();
    // Get natal Sun longitude
    let natal_sun = calc_ut(JulianDay::new(jd_natal), Body::SUN, flags)?;
    let natal_lon = natal_sun.lon;

    // Start searching from ~Jan 1 of return_year
    let jd_start = crate::julday(return_year, 1, 1, 0.0, Calendar::Gregorian);

    // Sun completes one orbit in ~365.25 days; search within ±2 years of start
    crate::astronomy::crossings::find_crossing_window(
        Body::SUN.as_raw(),
        natal_lon,
        jd_start,
        true,
        flags.as_raw(),
        400.0,
    )
    .ok_or_else(|| {
        Error::Calc(format!(
            "solar_return_jd: could not find solar return for year {return_year}"
        ))
    })
}

/// Find the Julian Day of a lunar return for a given month.
///
/// A lunar return is when the transiting Moon returns to its natal longitude
/// (approximately every 27.3 days — the sidereal month).
///
/// # Arguments
/// * `jd_natal`   — natal Julian Day
/// * `jd_start`   — start searching from this date
/// * `flags`      — ephemeris flags
pub fn lunar_return_jd(jd_natal: JulianDay, jd_start: JulianDay, flags: CalcFlags) -> Result<f64> {
    let jd_natal: f64 = jd_natal.into();
    let jd_start: f64 = jd_start.into();
    let natal_moon = calc_ut(JulianDay::new(jd_natal), Body::MOON, flags)?;
    let natal_lon = natal_moon.lon;

    // Offset by 1 day to skip the natal chart itself if jd_start == jd_natal
    let search_from = jd_start + 1.0;
    crate::astronomy::crossings::find_crossing_window(
        Body::MOON.as_raw(),
        natal_lon,
        search_from,
        true,
        flags.as_raw(),
        35.0,
    )
    .ok_or_else(|| Error::Calc("lunar_return_jd: could not find lunar return".into()))
}

// ─── Midpoints ────────────────────────────────────────────────────────────────

/// Compute the midpoint between two ecliptic longitudes.
///
/// Returns the shorter-arc midpoint in [0°, 360°).
/// The midpoint is the degree equidistant between two planets.
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let sun_lon  = 280.0;
/// let moon_lon = 100.0;
/// let mid = midpoint(Longitude::new(sun_lon), Longitude::new(moon_lon)); // 10.0° (shorter arc)
/// ```
#[inline]
#[must_use]
pub fn midpoint(lon1: Longitude, lon2: Longitude) -> f64 {
    let lon1: f64 = lon1.into();
    let lon2: f64 = lon2.into();
    crate::midpoint_deg(lon1, lon2)
}

/// Check if a planet is within `orb` degrees of a midpoint (aspect to midpoint).
///
/// Returns the orb (0.0 if exact) or `None` if outside the orb.
#[inline]
pub fn planet_on_midpoint(planet_lon: Longitude, mid_lon: Longitude, orb: f64) -> Option<f64> {
    let planet_lon: f64 = planet_lon.into();
    let mid_lon: f64 = mid_lon.into();
    let diff = diff_deg_signed(planet_lon, mid_lon).abs();
    let diff = diff.min(180.0 - diff); // also check opposition point
    if diff <= orb {
        Some(diff)
    } else {
        None
    }
}

/// Compute the full midpoint table for a set of positions.
///
/// Returns all `n*(n-1)/2` midpoints between planets, along with any
/// other planet that aspects each midpoint within the given orb.
///
/// `positions` — `(body_num, ecliptic_lon)` for each planet.
/// `orb`       — maximum orb for a planet to be "on" a midpoint.
#[must_use]
pub fn midpoint_table(positions: &[(Body, f64)], orb: f64) -> Vec<MidpointEntry> {
    // Returns: (body1, body2, midpoint_lon, [(planet_on_midpoint, orb)])
    let n = positions.len();
    let mut result = Vec::with_capacity(n * n.saturating_sub(1) / 2);
    for i in 0..n {
        for j in (i + 1)..n {
            let (b1, lon1) = positions[i];
            let (b2, lon2) = positions[j];
            let mid = midpoint(Longitude::new(lon1), Longitude::new(lon2));
            let on_mid: Vec<(Body, f64)> = positions
                .iter()
                .filter(|&&(b, _)| b != b1 && b != b2)
                .filter_map(|&(b, lon)| planet_on_midpoint(Longitude::new(lon), Longitude::new(mid), orb).map(|o| (b, o)))
                .collect();
            result.push((b1, b2, mid, on_mid));
        }
    }
    result
}

// ─── Parallactic angle ────────────────────────────────────────────────────────

/// Parallactic angle at a given ecliptic point for an observer.
///
/// The parallactic angle `q` is the angle between the great circle through a
/// body and the zenith, and the great circle through the body and the north
/// celestial pole. Used in computing position angles of solar eclipses.
///
/// `ha_deg`  — hour angle of the body (degrees)
/// `dec_deg` — declination of the body (degrees)
/// `lat_deg` — observer latitude (degrees)
#[must_use]
pub fn parallactic_angle(ha_deg: Degrees, dec_deg: Degrees, lat_deg: Latitude) -> f64 {
    let ha_deg: f64 = ha_deg.into();
    let dec_deg: f64 = dec_deg.into();
    let lat_deg: f64 = lat_deg.into();
    let ha = ha_deg.to_radians();
    let dec = dec_deg.to_radians();
    let lat = lat_deg.to_radians();
    // q = atan2(sin(ha), tan(lat)*cos(dec) - sin(dec)*cos(ha))
    let (sin_ha, cos_ha) = ha.sin_cos();
    let (sin_dec, cos_dec) = dec.sin_cos();
    sin_ha
        .atan2((-sin_dec).mul_add(cos_ha, lat.tan() * cos_dec))
        .to_degrees()
}

// ─── Profections ─────────────────────────────────────────────────────────────

/// Annual profection house for a given age.
///
/// Profections advance the natal ASC one house per year.
/// Age 0 = 1st house, age 1 = 2nd house, …, age 12 = 1st house again.
///
/// Returns the profected house number (1–12) and the profected ascendant degree.
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let chart = houses(JulianDay::new(2_440_000.0), Latitude::new(48.85), Longitude::new(2.35), HouseSystem::PLACIDUS).unwrap();
/// let (house, degree) = annual_profection(&chart.cusps, 35); // age 35
/// println!("Age 35 profection: house {house}, degree {degree:.2}°");
/// ```
#[must_use]
pub fn annual_profection(cusps: &[f64; 13], age: u32) -> (u8, f64) {
    let house = ((age % 12) + 1) as u8;
    let degree = cusps[house as usize];
    (house, degree)
}

/// Monthly profection: advances 1/12 of a house per month.
///
/// Returns `(profected_house, profected_degree, lord)` where `lord`
/// is a string naming the traditional planetary ruler of that sign.
#[must_use]
pub fn monthly_profection(cusps: &[f64; 13], age_years: u32, age_months: u32) -> (u8, f64) {
    // Total months from birth
    let total_months = age_years * 12 + age_months;
    // Each house spans 30° and takes 12 months to traverse
    // One month = 1 house/12 = 30°/12 = 2.5°
    let house_idx = (total_months / 12 % 12 + 1) as usize;
    let fraction = (total_months % 12) as f64 / 12.0;
    let next_house = (house_idx % 12) + 1;
    let lo = cusps[house_idx];
    let hi = cusps[next_house];
    // Interpolate between current and next house cusp
    let arc = (hi - lo + 360.0).rem_euclid(360.0);
    let degree = (lo + arc * fraction).rem_euclid(360.0);
    (house_idx as u8, degree)
}

// ─── Built-in orb table ────────────────────────────────────────────────────────

/// Default orbs (degrees) for major aspects by body type.
///
/// Base orb (in degrees) for each canonical aspect angle.
/// Rows are `(angle_in_degrees, base_orb)`. Falls back to 2° for minor aspects
/// not in this table.
const ASPECT_BASE_ORBS: &[(i32, f64)] = &[
    (0, 10.0),   // Conjunction
    (30, 2.0),   // Semi-sextile
    (45, 3.0),   // Semi-square
    (60, 6.0),   // Sextile
    (72, 2.0),   // Quintile
    (90, 8.0),   // Square
    (120, 8.0),  // Trine
    (135, 3.0),  // Sesquiquadrate
    (144, 2.0),  // Bi-quintile
    (150, 3.0),  // Quincunx
    (180, 10.0), // Opposition
];

/// Default aspect orb for a pair of bodies and an aspect angle.
///
/// Returns the maximum angular separation from the exact aspect that should
/// still count as "in aspect", based on classical tradition:
///
/// * Wider orbs for luminaries (Sun/Moon): weight 2.0
/// * Standard for personal planets (Mercury/Venus/Mars): weight 1.5
/// * Narrower for social planets (Jupiter/Saturn): weight 1.0
/// * Tightest for outer planets, asteroids, nodes, Chiron: weight 0.75
///
/// The per-aspect base orb comes from the `ASPECT_BASE_ORBS` table — major aspects
/// (conjunction, opposition) get 10°, sextile and square 6–8°, minor aspects
/// 2–3°. The returned orb is `base * average_weight / 1.75` so a luminary
/// pair gets the full base orb and other pairs scale proportionally.
///
/// Callers who need custom orb rules should pass an explicit orb to
/// [`calc_chart_aspects`] instead.
#[must_use]
pub fn default_orb(body1: Body, body2: Body, aspect: f64) -> f64 {
    let w = (body1.orb_weight() + body2.orb_weight()) / 2.0;
    let base = ASPECT_BASE_ORBS
        .iter()
        .find(|(a, _)| *a == aspect as i32)
        .map_or(2.0, |(_, orb)| *orb);
    base * w / 1.75 // normalise: luminaries w=2.0 → factor 1.0; others scale down
}

/// Like [`calc_chart_aspects`] but uses the built-in orb table.
///
/// Each pair of bodies gets an orb appropriate to their significance
/// (luminaries get wider orbs than outer planets).
#[must_use]
pub fn calc_chart_aspects_auto(
    positions: &[(Body, f64, f64)],
    aspects: &[f64],
) -> Vec<ChartAspect> {
    let n = positions.len();
    let mut result = Vec::with_capacity(n * n.saturating_sub(1) / 4);
    for i in 0..n {
        for j in (i + 1)..n {
            let (b1, lon1, spd1) = positions[i];
            let (b2, lon2, spd2) = positions[j];
            let signed = diff_deg_signed(lon1, lon2);
            let separation = signed.abs();
            for &asp in aspects {
                let orb = default_orb(b1, b2, asp);
                let diff = (separation - asp).abs();
                let diff = diff.min(360.0 - diff);
                if diff <= orb {
                    let applying = is_applying(signed, spd1 - spd2, asp);
                    result.push(ChartAspect {
                        body1: b1,
                        body2: b2,
                        aspect: asp,
                        orb: diff,
                        applying,
                    });
                }
            }
        }
    }
    result.sort_by(|a, b| a.orb.total_cmp(&b.orb));
    result
}

// ─── Local Apparent Solar Time ────────────────────────────────────────────────

/// Local Apparent Solar Time (LAST) at a given UT and geographic longitude.
///
/// LAST is the time shown by a sundial. It differs from clock time by:
/// - the longitude offset from the time zone meridian
/// - the equation of time (orbital eccentricity + axial tilt)
///
/// Returns LAST in decimal hours (0–24).
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// // What does the sundial show in Paris (lon=2.35°E) at J2000?
/// let last = local_apparent_solar_time(JulianDay::new(2_451_545.0), Longitude::new(2.35)).unwrap();
/// println!("LAST Paris: {:.2}h", last);
/// ```
pub fn local_apparent_solar_time(jd_ut: JulianDay, geolon_deg: Longitude) -> crate::Result<f64> {
    let jd_ut: f64 = jd_ut.into();
    let geolon_deg: f64 = geolon_deg.into();
    // Extract UTC hour-of-day from JD:
    // JD epoch is noon (12:00 UT), so fractional part 0.0 = noon.
    // Time of day (hours) = ((jd_ut + 0.5).fract()) * 24.0
    let utc_hours = ((jd_ut + 0.5).fract() * 24.0).rem_euclid(24.0);
    // Local Mean Solar Time = UTC + longitude_offset
    let lmt = (utc_hours + geolon_deg / 15.0).rem_euclid(24.0);
    // Equation of time (hours): positive when sundial is ahead of clock
    let eot = crate::functions::time::time_equ(JulianDay::new(jd_ut))?;
    // Local Apparent Solar Time = LMT + EoT
    let last = (lmt + eot).rem_euclid(24.0);
    Ok(last)
}

// ─── Sign rulerships ──────────────────────────────────────────────────────────

/// Traditional planetary ruler of a zodiac sign (0=Aries … 11=Pisces).
///
/// Uses the classical 7-planet rulership scheme (pre-Uranus/Neptune/Pluto):
/// Aries→Mars, Taurus→Venus, …, Pisces→Jupiter.
///
/// Returns the body number or -1 for unknown.
#[must_use]
pub fn sign_ruler(sign: u8) -> Body {
    // 0=Aries, 1=Taurus, 2=Gemini, 3=Cancer, 4=Leo, 5=Virgo,
    // 6=Libra, 7=Scorpio, 8=Sagittarius, 9=Capricorn, 10=Aquarius, 11=Pisces
    match sign % 12 {
        0 => Body::MARS,        // Aries
        1 => Body::VENUS,       // Taurus
        2 | 5 => Body::MERCURY, // Gemini, Virgo
        3 => Body::MOON,        // Cancer
        4 => Body::SUN,         // Leo
        6 => Body::VENUS,       // Libra
        7 => Body::MARS,        // Scorpio (traditional)
        8 => Body::JUPITER,     // Sagittarius
        9 | 10 => Body::SATURN, // Capricorn, Aquarius (traditional)
        _ => Body::JUPITER,     // Pisces (sign % 12 == 11)
    }
}

/// Modern planetary ruler (assigns Uranus→Aquarius, Neptune→Pisces, Pluto→Scorpio).
#[must_use]
pub fn sign_ruler_modern(sign: u8) -> Body {
    match sign % 12 {
        7 => Body::PLUTO,    // Scorpio → Pluto
        10 => Body::URANUS,  // Aquarius → Uranus
        11 => Body::NEPTUNE, // Pisces → Neptune
        s => sign_ruler(s),
    }
}

/// The sign (0–11) a planet is exalted in, or -1 if none.
#[must_use]
pub fn sign_exaltation(body: Body) -> i8 {
    match body.as_raw() {
        0 => 0,  // Sun exalted in Aries
        1 => 1,  // Moon exalted in Taurus
        2 => 6,  // Mercury exalted in Virgo (or Aquarius)
        3 => 11, // Venus exalted in Pisces
        4 => 9,  // Mars exalted in Capricorn
        5 => 3,  // Jupiter exalted in Cancer
        6 => 6,  // Saturn exalted in Libra
        _ => -1,
    }
}

/// Name of a zodiac sign number (0=Aries … 11=Pisces).
/// Always returns a valid name (wraps mod 12).
/// See also [`sign_name`](crate::sign_name) in geoformat (takes i32, returns Option).
#[must_use]
pub fn zodiac_sign_name(sign: u8) -> &'static str {
    const NAMES: [&str; 12] = [
        "Aries",
        "Taurus",
        "Gemini",
        "Cancer",
        "Leo",
        "Virgo",
        "Libra",
        "Scorpio",
        "Sagittarius",
        "Capricorn",
        "Aquarius",
        "Pisces",
    ];
    NAMES[(sign % 12) as usize]
}

/// Sign number and degree for an ecliptic longitude.
///
/// Returns `(sign_number, degrees_in_sign)` where sign is 0–11
/// and degrees_in_sign is 0.0–29.99…
#[inline]
#[must_use]
pub fn lon_to_sign(lon: f64) -> (u8, f64) {
    let lon = lon.rem_euclid(360.0);
    let sign = (lon / 30.0).floor() as u8;
    let deg = lon % 30.0;
    (sign, deg)
}

// ─── Vimshottari Dasha ────────────────────────────────────────────────────────

/// One level of the Vimshottari dasha system.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DashaLevel {
    /// Ruling planet (body number).
    pub body: Body,
    /// Start Julian Day.
    pub start: f64,
    /// End Julian Day.
    pub end: f64,
    /// Duration in years.
    pub years: f64,
}

/// Vimshottari dasha sequence and durations (120-year cycle).
/// Planets in order: Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury.
pub const DASHA_SEQUENCE: &[(Body, f64)] = &[
    (Body::TRUE_NODE, 7.0),  // Ketu (South Node) — 7 years
    (Body::VENUS, 20.0),     // Venus  — 20 years
    (Body::SUN, 6.0),        // Sun    — 6 years
    (Body::MOON, 10.0),      // Moon   — 10 years
    (Body::MARS, 7.0),       // Mars   — 7 years
    (Body::MEAN_NODE, 18.0), // Rahu (North Node) — 18 years
    (Body::JUPITER, 16.0),   // Jupiter — 16 years
    (Body::SATURN, 19.0),    // Saturn  — 19 years
    (Body::MERCURY, 17.0),   // Mercury — 17 years
];

/// Compute the Vimshottari dasha periods from birth.
///
/// The starting dasha is determined by the Moon's position in the 27 nakshatras.
/// Each nakshatra is 13°20' wide; each has a ruling planet.
///
/// Returns a `Vec<DashaLevel>` of all major dasha periods from birth
/// through `years_ahead` years.
///
/// # Example
/// ```no_run
/// use celestial_core::*;
/// use celestial_core::body::{Body, CalcFlags, HouseSystem};
/// let jd_birth = 2_440_000.0;
/// let moon_lon = calc_ut(JulianDay::new(jd_birth), Body::MOON, CalcFlags::BUILTIN | CalcFlags::SIDEREAL).unwrap().lon;
/// let dashas = vimshottari_dasha(JulianDay::new(jd_birth), Longitude::new(moon_lon), 120.0);
/// for d in &dashas {
///     println!("{}: {:.2} years", d.body.name(), d.years);
/// }
/// ```
#[must_use]
pub fn vimshottari_dasha(
    jd_birth: JulianDay,
    moon_lon_sidereal: Longitude,
    years_ahead: f64,
) -> Vec<DashaLevel> {
    let jd_birth: f64 = jd_birth.into();
    let moon_lon_sidereal: f64 = moon_lon_sidereal.into();
    const YEAR_DAYS: f64 = 365.25;
    const NAKSHATRA_DEG: f64 = 13.333_333; // 360/27

    // Determine nakshatra (0–26) from Moon's sidereal longitude
    let nak = ((moon_lon_sidereal / NAKSHATRA_DEG).floor() as usize) % 27;
    // Each nakshatra maps to a dasha lord; cycle is 9-planet, 3 full cycles = 27
    let start_dasha_idx = nak % 9;
    // Fraction already elapsed in the starting dasha
    let nak_fraction = (moon_lon_sidereal % NAKSHATRA_DEG) / NAKSHATRA_DEG;
    let (_, start_years) = DASHA_SEQUENCE[start_dasha_idx];
    let elapsed = start_years * nak_fraction;

    // Vimshottari spans 120 years; 9 dasha lords; at most ~10 periods active.
    let mut dashas = Vec::with_capacity(9);
    let mut jd = jd_birth - elapsed * YEAR_DAYS;
    let jd_end = jd_birth + years_ahead * YEAR_DAYS;

    let mut idx = start_dasha_idx;
    loop {
        let (body, years) = DASHA_SEQUENCE[idx];
        let jd_period_end = jd + years * YEAR_DAYS;
        if jd >= jd_birth - 0.1 || jd_period_end > jd_birth {
            // Only include if overlaps with birth → end window
            let start = jd.max(jd_birth);
            if start < jd_end {
                dashas.push(DashaLevel {
                    body,
                    start,
                    end: jd_period_end,
                    years: (jd_period_end - start) / YEAR_DAYS,
                });
            }
        }
        jd = jd_period_end;
        if jd >= jd_end {
            break;
        }
        idx = (idx + 1) % 9;
    }
    dashas
}

// ── Phase 5–8 functions live in sibling modules (see functions/mod.rs) ─────────
// Re-exported here so existing code using celestial_core::full_dignity etc. still works.
pub use crate::functions::chinese::*;
pub use crate::functions::hellenistic::*;
pub use crate::functions::indigenous::*;
pub use crate::functions::mesoamerican::*;
