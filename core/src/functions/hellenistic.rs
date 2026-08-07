//! Auto-split from chart.rs — do not edit section headers.

use crate::body::Body;
#[allow(unused_imports)]
use crate::functions::calc::calc_ut;
use crate::functions::chart::{sign_exaltation, sign_ruler, sign_ruler_modern};
use crate::units::{JulianDay, Longitude};

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 5 — Hellenistic / Persian functions
// ═══════════════════════════════════════════════════════════════════════════════

// ─── Sect ─────────────────────────────────────────────────────────────────────

/// Determine whether a chart is a day chart (Sun above horizon).
///
/// In classical Hellenistic astrology, a "day chart" (diurnal) has the Sun
/// in houses 7–12 (above the horizon); "night chart" (nocturnal) has it in
/// houses 1–6 (below the horizon).
///
/// # Arguments
/// * `sun_lon` — Sun's ecliptic longitude (degrees)
/// * `cusps`   — 13-element house cusp array from `houses()` (index 1–12 used)
#[must_use]
pub fn is_day_chart(sun_lon: Longitude, cusps: &[f64; 13]) -> bool {
    let sun_lon: f64 = sun_lon.into();
    // Find which house (1–12) the Sun occupies
    let house = planet_house_number(sun_lon, cusps);
    house >= 7
}

/// Returns the sect benefic/malefic status of a planet for a day or night chart.
///
/// Classical assignment:
/// * Day sect: Sun, Jupiter, Saturn (day benefics/malefics)
/// * Night sect: Moon, Venus, Mars
/// * Mercury and outer planets: treated as sect-neutral because this API does
///   not receive the positional data needed to determine Mercury's orientation
///
/// Returns `true` if the planet is of the *same* sect or is sect-neutral.
#[must_use]
pub fn same_sect(body: Body, is_day: bool) -> bool {
    match body {
        Body::SUN | Body::JUPITER | Body::SATURN => is_day,
        Body::MOON | Body::VENUS | Body::MARS => !is_day,
        _ => true, // Mercury and outer planets are sect-neutral
    }
}

// ─── Essential dignities — terms (bounds) ────────────────────────────────────

/// The five essential dignity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dignity {
    /// Planet ruling its own sign (+5).
    Domicile,
    /// Planet in its sign of exaltation (+4).
    Exaltation,
    /// Planet ruling the sign's triplicity (+3 or +2).
    Triplicity,
    /// Planet ruling the degree's term/bound (+2).
    Term,
    /// Planet ruling the degree's decan/face (+1).
    Decan,
    /// Planet with no essential dignity (0).
    Peregrine,
    /// Planet in the sign opposite its domicile (−5).
    Detriment,
    /// Planet in the sign opposite its exaltation (−4).
    Fall,
}

impl std::fmt::Display for Dignity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Dignity::Domicile => "domicile",
            Dignity::Exaltation => "exaltation",
            Dignity::Triplicity => "triplicity",
            Dignity::Term => "term",
            Dignity::Decan => "decan",
            Dignity::Peregrine => "peregrine",
            Dignity::Detriment => "detriment",
            Dignity::Fall => "fall",
        })
    }
}

/// Egyptian term (bound) ruler for a given ecliptic longitude.
///
/// Each sign is divided into five unequal sections (terms), each assigned
/// to one of the five non-luminary planets. This uses the Egyptian bounds
/// as compiled by Ptolemy (Tetrabiblos I.20).
///
/// Returns the ruling `Body` for the given longitude.
#[must_use]
pub fn egyptian_terms_ruler(lon: Longitude) -> Body {
    let lon: f64 = lon.into();
    let sign = (lon / 30.0) as usize % 12;
    let deg = lon % 30.0;
    // Each sign: (planet, cumulative_end_degree)
    // Source: Ptolemy / Vettius Valens Egyptian terms
    const TERMS: &[[(Body, u8); 5]; 12] = &[
        // Aries
        [
            (Body::JUPITER, 6),
            (Body::VENUS, 12),
            (Body::MERCURY, 20),
            (Body::MARS, 25),
            (Body::SATURN, 30),
        ],
        // Taurus
        [
            (Body::VENUS, 8),
            (Body::MERCURY, 14),
            (Body::JUPITER, 22),
            (Body::SATURN, 27),
            (Body::MARS, 30),
        ],
        // Gemini
        [
            (Body::MERCURY, 6),
            (Body::JUPITER, 12),
            (Body::VENUS, 17),
            (Body::MARS, 24),
            (Body::SATURN, 30),
        ],
        // Cancer
        [
            (Body::MARS, 7),
            (Body::VENUS, 13),
            (Body::MERCURY, 19),
            (Body::JUPITER, 26),
            (Body::SATURN, 30),
        ],
        // Leo
        [
            (Body::JUPITER, 6),
            (Body::VENUS, 11),
            (Body::SATURN, 18),
            (Body::MERCURY, 24),
            (Body::MARS, 30),
        ],
        // Virgo
        [
            (Body::MERCURY, 7),
            (Body::VENUS, 17),
            (Body::JUPITER, 21),
            (Body::MARS, 28),
            (Body::SATURN, 30),
        ],
        // Libra
        [
            (Body::SATURN, 6),
            (Body::MERCURY, 14),
            (Body::JUPITER, 21),
            (Body::VENUS, 28),
            (Body::MARS, 30),
        ],
        // Scorpio
        [
            (Body::MARS, 7),
            (Body::VENUS, 11),
            (Body::MERCURY, 19),
            (Body::JUPITER, 24),
            (Body::SATURN, 30),
        ],
        // Sagittarius
        [
            (Body::JUPITER, 12),
            (Body::VENUS, 17),
            (Body::MERCURY, 21),
            (Body::SATURN, 26),
            (Body::MARS, 30),
        ],
        // Capricorn
        [
            (Body::MERCURY, 7),
            (Body::JUPITER, 14),
            (Body::VENUS, 22),
            (Body::SATURN, 26),
            (Body::MARS, 30),
        ],
        // Aquarius
        [
            (Body::MERCURY, 7),
            (Body::VENUS, 13),
            (Body::JUPITER, 20),
            (Body::MARS, 25),
            (Body::SATURN, 30),
        ],
        // Pisces
        [
            (Body::VENUS, 12),
            (Body::JUPITER, 16),
            (Body::MERCURY, 19),
            (Body::MARS, 28),
            (Body::SATURN, 30),
        ],
    ];
    for (planet, end_deg) in TERMS[sign] {
        if deg < end_deg as f64 {
            return planet;
        }
    }
    Body::SATURN // fallback (should not reach)
}

/// Decan (face) ruler for an ecliptic longitude.
///
/// Each sign is divided into three 10° faces. The ruler sequence follows
/// the Chaldean order: Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter
/// cycling through the 36 faces.
///
/// Returns the ruling `Body`.
#[must_use]
pub fn decan_ruler(lon: Longitude) -> Body {
    let lon: f64 = lon.into();
    let decan_idx = (lon / 10.0) as usize % 36;
    // Chaldean decan sequence (Firmicus Maternus / Ptolemy)
    const DECAN_RULERS: [Body; 36] = [
        Body::MARS,
        Body::SUN,
        Body::VENUS, // Aries
        Body::MERCURY,
        Body::MOON,
        Body::SATURN, // Taurus
        Body::JUPITER,
        Body::MARS,
        Body::SUN, // Gemini
        Body::VENUS,
        Body::MERCURY,
        Body::MOON, // Cancer
        Body::SATURN,
        Body::JUPITER,
        Body::MARS, // Leo
        Body::SUN,
        Body::VENUS,
        Body::MERCURY, // Virgo
        Body::MOON,
        Body::SATURN,
        Body::JUPITER, // Libra
        Body::MARS,
        Body::SUN,
        Body::VENUS, // Scorpio
        Body::MERCURY,
        Body::MOON,
        Body::SATURN, // Sagittarius
        Body::JUPITER,
        Body::MARS,
        Body::SUN, // Capricorn
        Body::VENUS,
        Body::MERCURY,
        Body::MOON, // Aquarius
        Body::SATURN,
        Body::JUPITER,
        Body::MARS, // Pisces
    ];
    DECAN_RULERS[decan_idx]
}

/// Triplicity rulers (day, night, participating) for an ecliptic longitude.
///
/// Fire triplicity (Aries, Leo, Sagittarius):    Sun / Jupiter / Saturn
/// Earth triplicity (Taurus, Virgo, Capricorn):  Venus / Moon / Mars
/// Air triplicity (Gemini, Libra, Aquarius):     Saturn / Mercury / Jupiter
/// Water triplicity (Cancer, Scorpio, Pisces):   Venus / Mars / Moon
///
/// Returns `(day_ruler, night_ruler, participating_ruler)`.
#[must_use]
pub fn triplicity_rulers(lon: Longitude) -> (Body, Body, Body) {
    let lon: f64 = lon.into();
    let sign = (lon / 30.0) as usize;
    match sign % 4 {
        0 => (Body::SUN, Body::JUPITER, Body::SATURN), // fire
        1 => (Body::VENUS, Body::MOON, Body::MARS),    // earth
        2 => (Body::SATURN, Body::MERCURY, Body::JUPITER), // air
        _ => (Body::VENUS, Body::MARS, Body::MOON),    // water (sign % 4 == 3)
    }
}

/// Compute the full dignity score for a planet at a given longitude.
///
/// Returns the highest-ranking `Dignity` and a numeric score:
/// Domicile=5, Exaltation=4, Triplicity=3, Term=2, Decan=1,
/// Peregrine=0, Detriment=−5, Fall=−4.
/// Body rules `sign` (classical or modern).
fn rules_sign(body: Body, sign: u8) -> bool {
    sign_ruler(sign) == body || sign_ruler_modern(sign) == body
}

/// Triplicity score: 3 if same sect, else 2.
fn triplicity_score(body: Body, lon: f64, is_day: bool) -> Option<i8> {
    let (day_r, night_r, part_r) = triplicity_rulers(Longitude::new(lon));
    if body == day_r || body == night_r || body == part_r {
        Some(if same_sect(body, is_day) { 3 } else { 2 })
    } else {
        None
    }
}

/// Highest-ranking essential dignity of `body` at `lon`, with its score.
///
/// Tests domicile, detriment, exaltation, fall, triplicity, term and decan in
/// rank order; returns the first match as `(Dignity, score)`, or peregrine (0).
#[must_use]
pub fn full_dignity(body: Body, lon: Longitude, is_day: bool) -> (Dignity, i8) {
    let lon: f64 = lon.into();
    let sign = (lon / 30.0) as u8 % 12;
    const OPPOSITE_SIGNS: [u8; 12] = [6, 7, 8, 9, 10, 11, 0, 1, 2, 3, 4, 5];
    let opp = OPPOSITE_SIGNS[sign as usize];
    let ex = sign_exaltation(body);

    if rules_sign(body, sign) {
        return (Dignity::Domicile, 5);
    }
    if rules_sign(body, opp) {
        return (Dignity::Detriment, -5);
    }
    if ex >= 0 && ex as u8 == sign {
        return (Dignity::Exaltation, 4);
    }
    if ex >= 0 && (ex as u8 + 6) % 12 == sign {
        return (Dignity::Fall, -4);
    }
    if let Some(score) = triplicity_score(body, lon, is_day) {
        return (Dignity::Triplicity, score);
    }
    if egyptian_terms_ruler(Longitude::new(lon)) == body {
        return (Dignity::Term, 2);
    }
    if decan_ruler(Longitude::new(lon)) == body {
        return (Dignity::Decan, 1);
    }
    (Dignity::Peregrine, 0)
}

fn additive_dignity_score(body: Body, lon: f64, is_day: bool) -> i8 {
    let sign = (lon / 30.0) as u8 % 12;
    let mut score = 0;
    if rules_sign(body, sign) {
        score += 5;
    }
    if sign_exaltation(body) as u8 == sign {
        score += 4;
    }
    if let Some(triplicity) = triplicity_score(body, lon, is_day) {
        score += triplicity;
    }
    if egyptian_terms_ruler(Longitude::new(lon)) == body {
        score += 2;
    }
    if decan_ruler(Longitude::new(lon)) == body {
        score += 1;
    }
    score
}

// ─── Almuten ──────────────────────────────────────────────────────────────────

/// Compute the Almuten (lord of the chart) for a given longitude.
///
/// The Almuten is the planet with the highest sum of dignity scores at a degree.
/// Scores: domicile=5, exaltation=4, triplicity=3/2, term=2, decan=1.
///
/// Returns `(almuten_body, score)`.
#[must_use]
pub fn almuten(lon: Longitude, is_day: bool) -> (Body, i8) {
    let lon: f64 = lon.into();
    let planets = [
        Body::SUN,
        Body::MOON,
        Body::MERCURY,
        Body::VENUS,
        Body::MARS,
        Body::JUPITER,
        Body::SATURN,
    ];
    let mut best_body = Body::SUN;
    let mut best_score = i8::MIN;
    for &body in &planets {
        let score = additive_dignity_score(body, lon, is_day);
        if score > best_score {
            best_score = score;
            best_body = body;
        }
    }
    (best_body, best_score)
}

// ─── Firdaria ─────────────────────────────────────────────────────────────────

/// A single Firdaria period.
#[derive(Debug, Clone, PartialEq)]
pub struct FirdariaPeriod {
    /// The major lord (primary planet).
    pub major_lord: Body,
    /// The minor (sub-period) lord.
    pub minor_lord: Body,
    /// Start Julian Day (UT).
    pub start: f64,
    /// End Julian Day (UT).
    pub end: f64,
    /// Duration in years.
    pub years: f64,
}

/// Compute Firdaria periods for a chart (Persian planetary period system).
///
/// Each of the 7 planets rules for a fixed number of years in a fixed sequence.
/// Day charts start with the Sun; night charts start with the Moon.
/// Each major period is subdivided into 7 minor periods.
///
/// The sequence and durations (from Abū Maʿshar):
/// Sun=10, Venus=8, Mercury=13, Moon=9, Saturn=11, Jupiter=12, Mars=7  
/// Then North Node=3, South Node=2 (for a 75-year cycle).
///
/// # Arguments
/// * `jd_birth` — Julian Day of birth
/// * `is_day`   — true for day chart (Sun above horizon)
/// * `span`     — how many years forward to generate periods
#[must_use]
pub fn firdaria(jd_birth: JulianDay, is_day: bool, span: f64) -> Vec<FirdariaPeriod> {
    let jd_birth: f64 = jd_birth.into();
    if span <= 0.0 {
        return Vec::new();
    }
    // Major period durations (years)
    const DAY_SEQ: &[(Body, f64)] = &[
        (Body::SUN, 10.0),
        (Body::VENUS, 8.0),
        (Body::MERCURY, 13.0),
        (Body::MOON, 9.0),
        (Body::SATURN, 11.0),
        (Body::JUPITER, 12.0),
        (Body::MARS, 7.0),
        (Body::MEAN_NODE, 3.0), // North Node
        (Body::TRUE_NODE, 2.0), // South Node (Ketu proxy)
    ];
    const NIGHT_SEQ: &[(Body, f64)] = &[
        (Body::MOON, 9.0),
        (Body::SATURN, 11.0),
        (Body::MERCURY, 13.0),
        (Body::VENUS, 8.0),
        (Body::JUPITER, 12.0),
        (Body::MARS, 7.0),
        (Body::SUN, 10.0),
        (Body::MEAN_NODE, 3.0),
        (Body::TRUE_NODE, 2.0),
    ];
    let seq = if is_day { DAY_SEQ } else { NIGHT_SEQ };
    const DAYS_PER_YEAR: f64 = 365.25;

    // 7 minor periods per major lord; ~12 major lords covered in a 75-year span.
    let mut periods = Vec::new();
    let mut jd = jd_birth;
    let jd_end = jd_birth + span * DAYS_PER_YEAR;

    'outer: for &(major_lord, major_years) in seq.iter().cycle().take(span.ceil() as usize) {
        let major_end = jd + major_years * DAYS_PER_YEAR;
        let minor_dur = major_years / 7.0;
        // Sub-periods: same planet sequence, starting from major lord
        let start_idx = seq.iter().position(|(b, _)| *b == major_lord).unwrap_or(0);
        for i in 0..7 {
            let minor_lord = seq[(start_idx + i) % seq.len()].0;
            let period_start = jd + i as f64 * minor_dur * DAYS_PER_YEAR;
            let period_end = (period_start + minor_dur * DAYS_PER_YEAR).min(major_end);
            periods.push(FirdariaPeriod {
                major_lord,
                minor_lord,
                start: period_start,
                end: period_end,
                years: minor_dur,
            });
            if period_end >= jd_end {
                break 'outer;
            }
        }
        jd = major_end;
        if jd >= jd_end {
            break;
        }
    }
    periods
}

// ─── Helper (used internally) ─────────────────────────────────────────────────

/// Which house (1–12) does a planet longitude fall in given house cusps?
/// Returns 1 if not determinable.
fn planet_house_number(lon: f64, cusps: &[f64; 13]) -> usize {
    for h in 1..=12usize {
        let lo = cusps[h];
        let hi = cusps[if h == 12 { 1 } else { h + 1 }];
        let contained = if lo <= hi {
            (lo..hi).contains(&lon)
        } else {
            (lo..).contains(&lon) || (..hi).contains(&lon)
        };
        if contained {
            return h;
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUSPS: [f64; 13] = [
        0.0, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
    ];
    const ROTATED_CUSPS: [f64; 13] = [
        0.0, 10.0, 40.0, 70.0, 100.0, 130.0, 160.0, 190.0, 220.0, 250.0, 280.0, 310.0, 340.0,
    ];

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }

    #[test]
    fn house_lookup_and_sect_cover_boundaries() {
        let cases = [
            (0.0, 1, false),
            (29.999, 1, false),
            (30.0, 2, false),
            (179.999, 6, false),
            (180.0, 7, true),
            (329.999, 11, true),
            (330.0, 12, true),
            (359.999, 12, true),
        ];
        for (lon, house, is_day) in cases {
            assert_eq!(planet_house_number(lon, &CUSPS), house);
            assert_eq!(is_day_chart(Longitude::new(lon), &CUSPS), is_day);
        }
        assert_eq!(planet_house_number(5.0, &ROTATED_CUSPS), 12);
        assert!(is_day_chart(Longitude::new(5.0), &ROTATED_CUSPS));
    }

    #[test]
    fn same_sect_covers_day_night_and_neutral_bodies() {
        let cases = [
            (Body::SUN, true, true),
            (Body::JUPITER, false, false),
            (Body::MOON, false, true),
            (Body::MARS, true, false),
            (Body::MERCURY, true, true),
            (Body::MERCURY, false, true),
        ];
        for (body, is_day, expected) in cases {
            assert_eq!(same_sect(body, is_day), expected);
        }
    }

    #[test]
    fn dignity_display_names_every_variant() {
        let cases = [
            (Dignity::Domicile, "domicile"),
            (Dignity::Exaltation, "exaltation"),
            (Dignity::Triplicity, "triplicity"),
            (Dignity::Term, "term"),
            (Dignity::Decan, "decan"),
            (Dignity::Peregrine, "peregrine"),
            (Dignity::Detriment, "detriment"),
            (Dignity::Fall, "fall"),
        ];
        for (dignity, expected) in cases {
            assert_eq!(dignity.to_string(), expected);
        }
    }

    #[test]
    fn egyptian_terms_cover_boundaries_and_wrapping() {
        let cases = [
            (0.0, Body::JUPITER),
            (5.999, Body::JUPITER),
            (6.0, Body::VENUS),
            (12.0, Body::MERCURY),
            (25.0, Body::SATURN),
            (30.0, Body::VENUS),
            (42.0, Body::MERCURY),
            (59.999, Body::MARS),
            (180.0, Body::SATURN),
            (360.0, Body::JUPITER),
        ];
        for (lon, expected) in cases {
            assert_eq!(egyptian_terms_ruler(Longitude::new(lon)), expected);
        }
    }

    #[test]
    fn decans_cover_boundaries_and_wrapping() {
        let cases = [
            (0.0, Body::MARS),
            (9.999, Body::MARS),
            (10.0, Body::SUN),
            (20.0, Body::VENUS),
            (30.0, Body::MERCURY),
            (110.0, Body::MOON),
            (350.0, Body::MARS),
            (360.0, Body::MARS),
        ];
        for (lon, expected) in cases {
            assert_eq!(decan_ruler(Longitude::new(lon)), expected);
        }
    }

    #[test]
    fn triplicities_cover_each_element_and_wrapping() {
        let cases = [
            (0.0, (Body::SUN, Body::JUPITER, Body::SATURN)),
            (30.0, (Body::VENUS, Body::MOON, Body::MARS)),
            (60.0, (Body::SATURN, Body::MERCURY, Body::JUPITER)),
            (90.0, (Body::VENUS, Body::MARS, Body::MOON)),
            (360.0, (Body::SUN, Body::JUPITER, Body::SATURN)),
        ];
        for (lon, expected) in cases {
            assert_eq!(triplicity_rulers(Longitude::new(lon)), expected);
        }
    }

    #[test]
    fn rulership_and_triplicity_helpers_cover_each_path() {
        assert!(rules_sign(Body::MARS, 0));
        assert!(rules_sign(Body::PLUTO, 7));
        assert!(!rules_sign(Body::SUN, 0));
        assert_eq!(triplicity_score(Body::SUN, 0.0, true), Some(3));
        assert_eq!(triplicity_score(Body::SUN, 0.0, false), Some(2));
        assert_eq!(triplicity_score(Body::JUPITER, 0.0, true), Some(3));
        assert_eq!(triplicity_score(Body::SATURN, 0.0, true), Some(3));
        assert_eq!(triplicity_score(Body::VENUS, 0.0, true), None);
    }

    #[test]
    fn full_dignity_covers_every_rank() {
        let cases = [
            (Body::SUN, 120.0, true, (Dignity::Domicile, 5)),
            (Body::PLUTO, 210.0, true, (Dignity::Domicile, 5)),
            (Body::SUN, 300.0, true, (Dignity::Detriment, -5)),
            (Body::SUN, 360.0, true, (Dignity::Exaltation, 4)),
            (Body::SUN, 180.0, true, (Dignity::Fall, -4)),
            (Body::JUPITER, 125.0, true, (Dignity::Triplicity, 3)),
            (Body::JUPITER, 125.0, false, (Dignity::Triplicity, 2)),
            (Body::MERCURY, 12.0, true, (Dignity::Term, 2)),
            (Body::MERCURY, 30.0, true, (Dignity::Decan, 1)),
            (Body::URANUS, 0.0, true, (Dignity::Peregrine, 0)),
        ];
        for (body, lon, is_day, expected) in cases {
            assert_eq!(full_dignity(body, Longitude::new(lon), is_day), expected);
        }
    }

    #[test]
    fn almuten_sums_stacked_dignities_and_keeps_first_tie() {
        assert_eq!(additive_dignity_score(Body::SUN, 0.0, true), 7);
        assert_eq!(additive_dignity_score(Body::MARS, 0.0, true), 6);
        assert_eq!(additive_dignity_score(Body::JUPITER, 0.0, true), 5);
        assert_eq!(almuten(Longitude::new(0.0), true), (Body::SUN, 7));
        assert_eq!(almuten(Longitude::new(0.0), false), (Body::SUN, 6));
        assert_eq!(almuten(Longitude::new(30.0), true), (Body::VENUS, 9));
    }

    #[test]
    fn firdaria_rejects_non_positive_spans() {
        assert!(firdaria(JulianDay::new(2_451_545.0), true, 0.0).is_empty());
        assert!(firdaria(JulianDay::new(2_451_545.0), false, -1.0).is_empty());
    }

    #[test]
    fn firdaria_day_subperiods_are_exact() {
        let jd = 2_451_545.0;
        let periods = firdaria(JulianDay::new(jd), true, 10.0);
        let lords = [
            Body::SUN,
            Body::VENUS,
            Body::MERCURY,
            Body::MOON,
            Body::SATURN,
            Body::JUPITER,
            Body::MARS,
        ];
        assert_eq!(periods.len(), 7);
        for (index, period) in periods.iter().enumerate() {
            let years = 10.0 / 7.0;
            assert_eq!(period.major_lord, Body::SUN);
            assert_eq!(period.minor_lord, lords[index]);
            assert_close(period.start, jd + index as f64 * years * 365.25);
            assert_close(period.end, jd + (index + 1) as f64 * years * 365.25);
            assert_close(period.years, years);
        }
    }

    #[test]
    fn firdaria_night_and_next_major_sequences_are_used() {
        let jd = 2_451_545.0;
        let night = firdaria(JulianDay::new(jd), false, 10.0);
        assert_eq!(
            (night[0].major_lord, night[0].minor_lord),
            (Body::MOON, Body::MOON)
        );
        assert_eq!(
            (night[7].major_lord, night[7].minor_lord),
            (Body::SATURN, Body::SATURN)
        );
        assert_close(night[7].start, jd + 9.0 * 365.25);

        let day = firdaria(JulianDay::new(jd), true, 18.0);
        assert_eq!(day.len(), 14);
        assert_eq!(
            (day[7].major_lord, day[7].minor_lord),
            (Body::VENUS, Body::VENUS)
        );
        assert_close(day[7].start, jd + 10.0 * 365.25);
        assert_close(day[13].end, jd + 18.0 * 365.25);
    }
}
