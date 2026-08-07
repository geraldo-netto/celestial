//! Hindu Panchānga — the five elements of the traditional Hindu almanac.
//!
//! The five elements (Pancha = five, Anga = limb) are:
//! 1. **Tithi** — lunar day (1–30), each 12° of Moon–Sun elongation
//! 2. **Vara** — weekday (0=Sunday … 6=Saturday)
//! 3. **Nakshatra** — lunar mansion (0–26), Moon's position in 27 equal segments
//! 4. **Yoga** — sum of Sun and Moon longitudes / 13.333°, 27 yogas
//! 5. **Karana** — half-tithi (1–60), each 6° of Moon–Sun elongation
//!
//! All calculations use the sidereal (Lahiri) frame, consistent with the
//! existing `long_to_nakshatra` function.
//!
//! # Examples
//! ```
//! # use celestial_core::body::Calendar;
//! use celestial_core::{panchanga, julday, JulianDay};
//! let jd = julday(2025, 3, 20, 6.0, Calendar::Gregorian);
//! let p = panchanga(JulianDay::new(jd));
//! assert!(p.tithi >= 1 && p.tithi <= 30);
//! assert!(p.nakshatra <= 26);
//! ```

use crate::body::{Body, CalcFlags, Calendar};
use crate::calc_ut;
use crate::error::{Error, Result};
use crate::units::JulianDay;
use crate::PlanetPos;

/// The 30 Tithis in order.
pub const TITHI_NAMES: [&str; 30] = [
    "Pratipada",
    "Dwitiya",
    "Tritiya",
    "Chaturthi",
    "Panchami",
    "Shashthi",
    "Saptami",
    "Ashtami",
    "Navami",
    "Dashami",
    "Ekadashi",
    "Dwadashi",
    "Trayodashi",
    "Chaturdashi",
    "Purnima", // Shukla 1-15
    "Pratipada",
    "Dwitiya",
    "Tritiya",
    "Chaturthi",
    "Panchami",
    "Shashthi",
    "Saptami",
    "Ashtami",
    "Navami",
    "Dashami",
    "Ekadashi",
    "Dwadashi",
    "Trayodashi",
    "Chaturdashi",
    "Amavasya", // Krishna 1-15
];

/// The 27 Nakshatras.
pub const NAKSHATRA_NAMES: [&str; 27] = [
    "Ashwini",
    "Bharani",
    "Krittika",
    "Rohini",
    "Mrigashirsha",
    "Ardra",
    "Punarvasu",
    "Pushya",
    "Ashlesha",
    "Magha",
    "Purva Phalguni",
    "Uttara Phalguni",
    "Hasta",
    "Chitra",
    "Swati",
    "Vishakha",
    "Anuradha",
    "Jyeshtha",
    "Mula",
    "Purva Ashadha",
    "Uttara Ashadha",
    "Shravana",
    "Dhanishtha",
    "Shatabhisha",
    "Purva Bhadrapada",
    "Uttara Bhadrapada",
    "Revati",
];

/// The 27 Yogas.
pub const YOGA_NAMES: [&str; 27] = [
    "Vishkambha",
    "Priti",
    "Ayushman",
    "Saubhagya",
    "Shobhana",
    "Atiganda",
    "Sukarman",
    "Dhriti",
    "Shula",
    "Ganda",
    "Vriddhi",
    "Dhruva",
    "Vyaghata",
    "Harshana",
    "Vajra",
    "Siddhi",
    "Vyatipata",
    "Variyana",
    "Parigha",
    "Shiva",
    "Siddha",
    "Sadhya",
    "Shubha",
    "Shukla",
    "Brahma",
    "Indra",
    "Vaidhriti",
];

/// The 11 Karanas (half-tithis), cycling through 60 karanas total.
/// The first Karana is fixed (Kimstughna), karanas 2–57 cycle through 7 moveable ones,
/// and the last 3 are fixed (Shakuni, Chatushpada, Naga).
pub const KARANA_NAMES: [&str; 11] = [
    "Bava",
    "Balava",
    "Kaulava",
    "Taitila",
    "Garaja",
    "Vanija",
    "Vishti (Bhadra)", // 7 moveable
    "Shakuni",
    "Chatushpada",
    "Naga",
    "Kimstughna", // Initial fixed Karana
];

/// The 7 Varas (weekdays) starting from Sunday.
pub const VARA_NAMES: [&str; 7] = [
    "Ravivara (Sunday)",
    "Somavara (Monday)",
    "Mangalavara (Tuesday)",
    "Budhavara (Wednesday)",
    "Guruvara (Thursday)",
    "Shukravara (Friday)",
    "Shanivara (Saturday)",
];

/// Whether a Tithi is in the waxing (Shukla) or waning (Krishna) fortnight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Paksha {
    /// Waxing moon (Shukla Paksha), Tithis 1–15.
    Shukla,
    /// Waning moon (Krishna Paksha), Tithis 16–30.
    Krishna,
}

/// Complete Panchānga for a given Julian day.
#[derive(Debug, Clone)]
pub struct Panchanga {
    /// Tithi number (1–30).
    pub tithi: u8,
    /// Tithi name.
    pub tithi_name: &'static str,
    /// Paksha (Shukla or Krishna fortnight).
    pub paksha: Paksha,
    /// Vara (0=Sunday … 6=Saturday).
    pub vara: u8,
    /// Vara name.
    pub vara_name: &'static str,
    /// Nakshatra (0–26).
    pub nakshatra: u8,
    /// Nakshatra name.
    pub nakshatra_name: &'static str,
    /// Pada within Nakshatra (1–4).
    pub nakshatra_pada: u8,
    /// Yoga number (0–26).
    pub yoga: u8,
    /// Yoga name.
    pub yoga_name: &'static str,
    /// Karana number (1–60).
    pub karana: u8,
    /// Karana name.
    pub karana_name: &'static str,
    /// Sun sidereal longitude (degrees).
    pub sun_lon: f64,
    /// Moon sidereal longitude (degrees).
    pub moon_lon: f64,
    /// Moon–Sun elongation (degrees, 0–360).
    pub elongation: f64,
}

use crate::norm_deg;

/// Compute the Karana name for a given karana number (1–60).
///
/// Out-of-range input (`0` or `> 60`) returns `""` rather than panicking on
/// the debug-only `0u8 - 2` underflow that was previously possible.
#[must_use]
pub fn karana_name(karana: u8) -> &'static str {
    match karana {
        1 => KARANA_NAMES[10],
        2..=57 => {
            let idx = ((karana - 2) % 7) as usize;
            KARANA_NAMES[idx]
        }
        58..=60 => KARANA_NAMES[(karana - 51) as usize],
        _ => "",
    }
}

/// Compute the full Panchānga for a given Julian day.
///
/// Uses Lahiri (Chitrapaksha) ayanamsa for sidereal positions.
#[must_use]
pub fn panchanga(jd: JulianDay) -> Panchanga {
    try_panchanga(jd).expect("panchanga Sun/Moon calculation failed")
}

/// Checked Panchānga calculation that surfaces ephemeris failures.
pub fn try_panchanga(jd: JulianDay) -> Result<Panchanga> {
    let jd: f64 = jd.into();
    // DEC-1: compute under Lahiri, but the prior code only *set* the
    // process-global sidereal mode and never restored it — leaking
    // Lahiri into the caller's later `calc_ut`/`ayanamsa`. Save the
    // previous mode and restore on scope exit (RAII → also on early
    // return / panic). The Panchānga result is unchanged (still Lahiri).
    struct SidModeGuard(i32);
    impl Drop for SidModeGuard {
        fn drop(&mut self) {
            crate::set_sid_mode(crate::body::SiderealMode(self.0), 0.0, 0.0);
        }
    }
    let _sid_guard = SidModeGuard(crate::functions::config::current_sid_mode());
    crate::set_sid_mode(crate::body::SiderealMode::LAHIRI, 0.0, 0.0);
    let flags = CalcFlags::BUILTIN | CalcFlags::SIDEREAL | CalcFlags::SPEED;

    panchanga_from_calc_results(
        jd,
        calc_ut(JulianDay::new(jd), Body::SUN, flags),
        calc_ut(JulianDay::new(jd), Body::MOON, flags),
    )
}

fn panchanga_from_calc_results(
    jd: f64,
    sun: Result<PlanetPos>,
    moon: Result<PlanetPos>,
) -> Result<Panchanga> {
    let sun = sun.map_err(|e| panchanga_calc_error("Sun", e))?;
    let moon = moon.map_err(|e| panchanga_calc_error("Moon", e))?;
    Ok(panchanga_from_positions(jd, sun, moon))
}

fn panchanga_calc_error(body: &str, err: Error) -> Error {
    Error::Calc(format!("panchanga {body} calculation failed: {err}"))
}

fn panchanga_from_positions(jd: f64, sun: PlanetPos, moon: PlanetPos) -> Panchanga {
    let sun_lon = norm_deg(sun.lon);
    let moon_lon = norm_deg(moon.lon);
    let elongation = norm_deg(moon_lon - sun_lon);

    // ── Tithi ─────────────────────────────────────────────────────────────
    let tithi_raw = elongation / 12.0;
    let tithi = (tithi_raw.floor() as u8 % 30) + 1;
    let paksha = if tithi <= 15 {
        Paksha::Shukla
    } else {
        Paksha::Krishna
    };

    // ── Nakshatra ─────────────────────────────────────────────────────────
    // `rem_euclid(360.0)` returns `[0, 360)` but fp rounding around the
    // boundary (e.g. `(360.0 / (360.0/27.0)).floor()`) can still produce 27.
    // Clamp every index to its valid range to guarantee in-bounds slot lookup.
    let nak_size = 360.0 / 27.0; // 13.333...°
    let nak_idx = ((moon_lon / nak_size).floor() as u8).min(26);
    let nak_pada_raw = (moon_lon % nak_size) / (nak_size / 4.0);
    let nak_pada = ((nak_pada_raw.floor() as u8).min(3)) + 1;

    // ── Yoga ──────────────────────────────────────────────────────────────
    let yoga_sum = norm_deg(sun_lon + moon_lon);
    let yoga = ((yoga_sum / nak_size).floor() as u8) % 27;

    // ── Karana ────────────────────────────────────────────────────────────
    let karana_raw = elongation / 6.0;
    let karana = ((karana_raw.floor() as u8).min(59)) + 1; // 1-60

    // ── Vara ──────────────────────────────────────────────────────────────
    // JD 0.0 = Monday, so day_of_week = (jd + 1.5) % 7, 0=Sunday
    let vara = ((jd + 1.5).floor() as i64).rem_euclid(7) as u8;

    Panchanga {
        tithi,
        tithi_name: TITHI_NAMES[(tithi - 1) as usize],
        paksha,
        vara,
        vara_name: VARA_NAMES[vara as usize],
        nakshatra: nak_idx,
        nakshatra_name: NAKSHATRA_NAMES[nak_idx as usize],
        nakshatra_pada: nak_pada,
        yoga,
        yoga_name: YOGA_NAMES[yoga as usize],
        karana,
        karana_name: karana_name(karana),
        sun_lon,
        moon_lon,
        elongation,
    }
}

/// Major Hindu festivals for a Gregorian year (approximate dates via Tithi/Nakshatra).
///
/// Returns festival names, descriptions, and Julian days.
/// Note: Hindu festival dates shift year to year; these are algorithmic approximations
/// based on the Tithi at solar noon for each day. Exact observance may vary by tradition.
#[derive(Debug, Clone)]
pub struct HinduFestival {
    /// Name of the festival.
    pub name: &'static str,
    /// Short description of the festival.
    pub description: &'static str,
    /// Julian day of the festival.
    pub jd: f64,
}

/// Scan a Gregorian year and return major Hindu festivals.
///
/// One row of the Hindu-festival lookup table used by [`hindu_festivals`].
///
/// A festival candidate matches `tithi` and `paksha` inside the
/// `(month_start, month_end)` window. The candidate nearest the seasonal
/// target date is selected so adjacent lunar months cannot win by scan order.
/// Each rule chooses the civil-day observation time relevant to its rite.
struct FestivalRule {
    name: &'static str,
    description: &'static str,
    tithi: u8,
    paksha: Paksha,
    month_start: i32,
    month_end: i32,
    target_month: i32,
    target_day: i32,
    observance: FestivalObservance,
}

#[derive(Clone, Copy)]
enum FestivalObservance {
    Daytime,
    IstMidnight,
}

const IST_MIDNIGHT_OFFSET_FROM_06_UT_DAYS: f64 = 12.5 / 24.0;

/// The fixed table of festivals checked by [`hindu_festivals`].
const FESTIVAL_RULES: &[FestivalRule] = &[
    FestivalRule {
        name: "Naraka Chaturdashi (Choti Diwali)",
        description: "Eve of Diwali, Krishna Chaturdashi of Kartik",
        tithi: 29,
        paksha: Paksha::Krishna,
        month_start: 10,
        month_end: 12,
        target_month: 11,
        target_day: 1,
        observance: FestivalObservance::Daytime,
    },
    FestivalRule {
        name: "Diwali (Lakshmi Puja)",
        description: "Festival of Lights, Amavasya of Kartik",
        tithi: 30,
        paksha: Paksha::Krishna,
        month_start: 10,
        month_end: 12,
        target_month: 11,
        target_day: 1,
        observance: FestivalObservance::Daytime,
    },
    FestivalRule {
        name: "Holi (Holika Dahan)",
        description: "Festival of Colors, Purnima of Phalguna",
        tithi: 15,
        paksha: Paksha::Shukla,
        month_start: 2,
        month_end: 4,
        target_month: 3,
        target_day: 10,
        observance: FestivalObservance::Daytime,
    },
    FestivalRule {
        name: "Maha Shivaratri",
        description: "Great Night of Shiva, Krishna Chaturdashi of Phalguna",
        tithi: 29,
        paksha: Paksha::Krishna,
        month_start: 2,
        month_end: 4,
        target_month: 3,
        target_day: 1,
        observance: FestivalObservance::IstMidnight,
    },
    FestivalRule {
        name: "Raksha Bandhan",
        description: "Bond of Protection, Purnima of Shravana",
        tithi: 15,
        paksha: Paksha::Shukla,
        month_start: 7,
        month_end: 9,
        target_month: 8,
        target_day: 20,
        observance: FestivalObservance::Daytime,
    },
    FestivalRule {
        name: "Janmashtami (Krishna Jayanti)",
        description: "Birth of Lord Krishna, Krishna Ashtami of Bhadrapada",
        tithi: 23,
        paksha: Paksha::Krishna,
        month_start: 8,
        month_end: 10,
        target_month: 8,
        target_day: 25,
        observance: FestivalObservance::Daytime,
    },
    FestivalRule {
        name: "Navratri (Sharada) begins",
        description: "Nine nights of Goddess Durga, Ashwin Shukla Pratipada",
        tithi: 1,
        paksha: Paksha::Shukla,
        month_start: 9,
        month_end: 11,
        target_month: 10,
        target_day: 1,
        observance: FestivalObservance::Daytime,
    },
];

fn festival_rule_matches(
    rule: &FestivalRule,
    panchanga: &Panchanga,
    jd: f64,
    month_jd: &[f64; 13],
) -> bool {
    panchanga.tithi == rule.tithi
        && panchanga.paksha == rule.paksha
        && jd > month_jd[rule.month_start as usize]
        && jd < month_jd[rule.month_end as usize]
}

fn festival_panchanga<'a>(
    rule: &FestivalRule,
    daytime: &'a Panchanga,
    ist_midnight: &'a Panchanga,
) -> &'a Panchanga {
    match rule.observance {
        FestivalObservance::Daytime => daytime,
        FestivalObservance::IstMidnight => ist_midnight,
    }
}

fn update_festival_candidate(
    candidate: &mut Option<(f64, HinduFestival)>,
    rule: &FestivalRule,
    jd: f64,
    target_jd: f64,
) {
    let distance = (jd - target_jd).abs();
    if candidate
        .as_ref()
        .is_none_or(|(best_distance, _)| distance < *best_distance)
    {
        *candidate = Some((
            distance,
            HinduFestival {
                name: rule.name,
                description: rule.description,
                jd,
            },
        ));
    }
}

/// Key Hindu festival dates for the given Gregorian year.
///
/// Iterates day-by-day through the year, matching each day's Panchānga against
/// the `FESTIVAL_RULES` table. Returns festivals in the order they occur.
#[must_use]
pub fn hindu_festivals(gregorian_year: i32) -> Vec<HinduFestival> {
    use crate::julday;

    let start_jd = julday(gregorian_year, 1, 1, 6.0, Calendar::Gregorian);

    let mut month_jd = [0.0_f64; 13];
    for m in 1..=12 {
        month_jd[m as usize] = julday(gregorian_year, m, 1, 0.0, Calendar::Gregorian);
    }

    let targets: Vec<f64> = FESTIVAL_RULES
        .iter()
        .map(|rule| {
            julday(
                gregorian_year,
                rule.target_month,
                rule.target_day,
                6.0,
                Calendar::Gregorian,
            )
        })
        .collect();
    let mut candidates = vec![None; FESTIVAL_RULES.len()];
    for day_offset in 0..366 {
        let jd = start_jd + f64::from(day_offset);
        let daytime = panchanga(JulianDay::new(jd));
        let ist_midnight = panchanga(JulianDay::new(jd + IST_MIDNIGHT_OFFSET_FROM_06_UT_DAYS));
        for (index, rule) in FESTIVAL_RULES.iter().enumerate() {
            let p = festival_panchanga(rule, &daytime, &ist_midnight);
            if festival_rule_matches(rule, p, jd, &month_jd) {
                update_festival_candidate(&mut candidates[index], rule, jd, targets[index]);
            }
        }
    }

    let mut festivals: Vec<_> = candidates
        .into_iter()
        .flatten()
        .map(|(_, festival)| festival)
        .collect();
    festivals.sort_by(|left, right| left.jd.total_cmp(&right.jd));
    festivals
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julday;

    struct PositionCase {
        jd: f64,
        sun: f64,
        moon: f64,
        values: (u8, Paksha, u8, u8, u8, u8, u8),
        names: (&'static str, &'static str, &'static str, &'static str),
    }

    fn planet_at(lon: f64) -> PlanetPos {
        PlanetPos {
            lon,
            ..PlanetPos::default()
        }
    }

    fn assert_position_case(case: PositionCase) {
        let p = panchanga_from_positions(case.jd, planet_at(case.sun), planet_at(case.moon));
        assert_eq!(
            (
                p.tithi,
                p.paksha,
                p.vara,
                p.nakshatra,
                p.nakshatra_pada,
                p.yoga,
                p.karana,
            ),
            case.values
        );
        assert_eq!(
            (p.tithi_name, p.vara_name, p.yoga_name, p.karana_name),
            case.names
        );
        assert_eq!((p.sun_lon, p.moon_lon), (case.sun, case.moon));
        assert_eq!(p.elongation, norm_deg(case.moon - case.sun));
    }

    fn festival_dates(year: i32) -> Vec<(&'static str, i32, i32)> {
        hindu_festivals(year)
            .into_iter()
            .map(|festival| {
                let date = crate::revjul(JulianDay::new(festival.jd), Calendar::Gregorian);
                (festival.name, date.month, date.day)
            })
            .collect()
    }

    #[test]
    fn panchanga_ranges() {
        let jd = julday(2025, 3, 20, 6.0, Calendar::Gregorian);
        let p = panchanga(JulianDay::new(jd));
        assert!(p.tithi >= 1 && p.tithi <= 30, "tithi={}", p.tithi);
        assert!(p.nakshatra <= 26, "nak={}", p.nakshatra);
        assert!(p.yoga <= 26, "yoga={}", p.yoga);
        assert!(p.vara <= 6, "vara={}", p.vara);
        assert!(p.karana >= 1 && p.karana <= 60, "karana={}", p.karana);
        assert!(p.elongation >= 0.0 && p.elongation < 360.0);
    }

    #[test]
    fn panchanga_surfaces_sun_calc_error() {
        let err = panchanga_from_calc_results(
            2_451_545.0,
            Err(Error::Calc("synthetic failure".into())),
            Ok(PlanetPos::default()),
        )
        .unwrap_err();

        assert!(err.to_string().contains("Sun calculation failed"));
    }

    #[test]
    fn positions_map_to_exact_panchanga_elements() {
        let cases = [
            PositionCase {
                jd: 2_451_545.0,
                sun: 10.0,
                moon: 21.0,
                values: (1, Paksha::Shukla, 6, 1, 3, 2, 2),
                names: ("Pratipada", "Shanivara (Saturday)", "Ayushman", "Bava"),
            },
            PositionCase {
                jd: -1.75,
                sun: 350.0,
                moon: 10.0,
                values: (2, Paksha::Shukla, 6, 0, 4, 0, 4),
                names: ("Dwitiya", "Shanivara (Saturday)", "Vishkambha", "Kaulava"),
            },
            PositionCase {
                jd: 0.0,
                sun: 15.0,
                moon: 355.0,
                values: (29, Paksha::Krishna, 1, 26, 3, 0, 57),
                names: (
                    "Chaturdashi",
                    "Somavara (Monday)",
                    "Vishkambha",
                    "Vishti (Bhadra)",
                ),
            },
            PositionCase {
                jd: 1.0,
                sun: 0.0,
                moon: 359.0,
                values: (30, Paksha::Krishna, 2, 26, 4, 26, 60),
                names: ("Amavasya", "Mangalavara (Tuesday)", "Vaidhriti", "Naga"),
            },
        ];
        for case in cases {
            assert_position_case(case);
        }
    }

    #[test]
    fn panchanga_uses_lahiri_sidereal_positions() {
        use crate::body::SiderealMode;

        let jd = JulianDay::new(2_451_545.0);
        crate::set_sid_mode(SiderealMode::LAHIRI, 0.0, 0.0);
        let flags = CalcFlags::BUILTIN | CalcFlags::SIDEREAL | CalcFlags::SPEED;
        let expected = panchanga_from_calc_results(
            jd.into(),
            calc_ut(jd, Body::SUN, flags),
            calc_ut(jd, Body::MOON, flags),
        )
        .unwrap();

        crate::set_sid_mode(SiderealMode::RAMAN, 0.0, 0.0);
        let actual = try_panchanga(jd).unwrap();
        crate::set_sid_mode(SiderealMode::FAGAN_BRADLEY, 0.0, 0.0);

        assert_eq!(actual.sun_lon, expected.sun_lon);
        assert_eq!(actual.moon_lon, expected.moon_lon);
    }

    #[test]
    fn panchanga_full_moon() {
        // At full moon, elongation ≈ 180°, tithi should be ~15 (Purnima)
        let jd = julday(2025, 1, 13, 22.0, Calendar::Gregorian); // Full moon Jan 13, 2025
        let p = panchanga(JulianDay::new(jd));
        // Elongation near 180°
        assert!(
            p.elongation > 140.0 && p.elongation < 220.0,
            "elongation={}",
            p.elongation
        );
        assert!(
            p.paksha == Paksha::Shukla || p.tithi >= 14,
            "tithi={}",
            p.tithi
        );
    }

    #[test]
    fn tithi_names_count() {
        assert_eq!(TITHI_NAMES.len(), 30);
        assert_eq!(NAKSHATRA_NAMES.len(), 27);
        assert_eq!(YOGA_NAMES.len(), 27);
    }

    #[test]
    fn vara_sunday_known() {
        // J2000.0 = Jan 1.5, 2000 = Saturday
        let jd = 2_451_545.0;
        let p = panchanga(JulianDay::new(jd));
        assert_eq!(p.vara, 6, "J2000 should be Saturday (6)");
    }

    #[test]
    fn karana_name_in_range() {
        // Four fixed half-tithis occupy positions 1 and 58–60.
        for k in 1..=60u8 {
            let nm = karana_name(k);
            assert!(!nm.is_empty(), "karana {k} has empty name");
        }
    }

    #[test]
    fn paksha_flips_across_new_moon() {
        // Several days after new moon we must be Shukla (waxing).
        let jd = julday(2025, 2, 3, 12.0, Calendar::Gregorian); // ~5 days post-new-moon
        let p = panchanga(JulianDay::new(jd));
        assert_eq!(p.paksha, Paksha::Shukla);
        // And several days after full moon, Krishna (waning).
        let jd2 = julday(2025, 2, 17, 12.0, Calendar::Gregorian);
        let p2 = panchanga(JulianDay::new(jd2));
        assert_eq!(p2.paksha, Paksha::Krishna);
    }

    #[test]
    fn hindu_festivals_year_returns_some() {
        // Festival list for 2025 should include common entries
        let festivals = hindu_festivals(2025);
        assert!(
            !festivals.is_empty(),
            "expected at least one festival in 2025"
        );
        // Each festival has a valid JD and non-empty name
        for f in &festivals {
            assert!(f.jd > 2_400_000.0 && f.jd < 2_600_000.0, "jd={}", f.jd);
            assert!(!f.name.is_empty());
        }
    }

    #[test]
    fn hindu_festivals_choose_the_seasonal_lunation() {
        let cases = [
            (
                2024,
                vec![
                    ("Maha Shivaratri", 3, 8),
                    ("Holi (Holika Dahan)", 3, 24),
                    ("Raksha Bandhan", 8, 19),
                    ("Janmashtami (Krishna Jayanti)", 8, 26),
                    ("Navratri (Sharada) begins", 10, 3),
                    ("Naraka Chaturdashi (Choti Diwali)", 10, 31),
                    ("Diwali (Lakshmi Puja)", 11, 1),
                ],
            ),
            (
                2025,
                vec![
                    ("Maha Shivaratri", 2, 26),
                    ("Holi (Holika Dahan)", 3, 13),
                    ("Raksha Bandhan", 8, 9),
                    ("Janmashtami (Krishna Jayanti)", 8, 16),
                    ("Navratri (Sharada) begins", 9, 22),
                    ("Naraka Chaturdashi (Choti Diwali)", 10, 20),
                    ("Diwali (Lakshmi Puja)", 10, 21),
                ],
            ),
            (
                2026,
                vec![
                    ("Maha Shivaratri", 2, 15),
                    ("Holi (Holika Dahan)", 3, 3),
                    ("Raksha Bandhan", 8, 27),
                    ("Janmashtami (Krishna Jayanti)", 9, 4),
                    ("Navratri (Sharada) begins", 10, 11),
                    ("Naraka Chaturdashi (Choti Diwali)", 11, 7),
                    ("Diwali (Lakshmi Puja)", 11, 8),
                ],
            ),
        ];
        for (year, expected) in cases {
            assert_eq!(festival_dates(year), expected, "year={year}");
        }
    }

    #[test]
    fn festival_match_window_is_exclusive() {
        let rule = FestivalRule {
            name: "Test",
            description: "Test",
            tithi: 1,
            paksha: Paksha::Shukla,
            month_start: 2,
            month_end: 4,
            target_month: 3,
            target_day: 1,
            observance: FestivalObservance::Daytime,
        };
        let matching = panchanga_from_positions(0.0, planet_at(0.0), planet_at(1.0));
        let mut month_jd = [0.0; 13];
        month_jd[2] = 10.0;
        month_jd[4] = 20.0;
        let cases = [(10.0, false), (10.5, true), (20.0, false), (20.5, false)];
        for (jd, expected) in cases {
            assert_eq!(
                festival_rule_matches(&rule, &matching, jd, &month_jd),
                expected,
                "jd={jd}"
            );
        }

        let wrong = panchanga_from_positions(0.0, planet_at(0.0), planet_at(181.0));
        assert!(!festival_rule_matches(&rule, &wrong, 10.5, &month_jd));
    }

    #[test]
    fn closest_festival_candidate_wins_with_earlier_tie_break() {
        let rule = &FESTIVAL_RULES[0];
        let target = 100.0;
        let mut candidate = None;
        for jd in [98.0, 99.0, 101.0] {
            update_festival_candidate(&mut candidate, rule, jd, target);
        }
        let (_, festival) = candidate.unwrap();
        assert_eq!(festival.jd, 99.0);
        assert_eq!(festival.name, rule.name);
        assert_eq!(festival.description, rule.description);
    }

    #[test]
    fn vara_cycles_seven_days() {
        // Vara (weekday) cycles 0..7 over consecutive days
        let jd0 = julday(2025, 1, 1, 12.0, Calendar::Gregorian);
        let mut seen = [false; 7];
        for d in 0..7 {
            let p = panchanga(JulianDay::new(jd0 + d as f64));
            seen[p.vara as usize] = true;
        }
        assert!(seen.iter().all(|&b| b), "missed weekday: {seen:?}");
    }

    #[test]
    fn tithi_advances_monotonically_intra_lunation() {
        // Tithi 1..30 cycles every ~29.5 days; sample 1-day steps and confirm
        // the difference is small/positive (allowing for end-of-cycle wrap).
        let jd0 = julday(2025, 1, 1, 12.0, Calendar::Gregorian);
        let prev = panchanga(JulianDay::new(jd0)).tithi;
        let next = panchanga(JulianDay::new(jd0 + 1.0)).tithi;
        let diff = (next as i32 - prev as i32).rem_euclid(30);
        assert!((0..=2).contains(&diff), "tithi step too large: {diff}");
    }

    /// DEC-1 regression: `panchanga` must NOT leak its internal Lahiri
    /// sidereal mode into the process-global state — the caller's mode
    /// must be exactly as it was before the call.
    #[test]
    fn panchanga_restores_caller_sidereal_mode() {
        use crate::body::SiderealMode;
        use crate::functions::config::current_sid_mode;

        // Caller picks Raman; panchanga internally switches to Lahiri.
        crate::set_sid_mode(SiderealMode::RAMAN, 0.0, 0.0);
        assert_eq!(current_sid_mode(), SiderealMode::RAMAN.as_raw());
        let p = panchanga(JulianDay::new(julday(
            2000,
            1,
            1,
            12.0,
            crate::body::Calendar::Gregorian,
        )));
        assert!(p.tithi >= 1 && p.tithi <= 30, "panchanga still valid");
        assert_eq!(
            current_sid_mode(),
            SiderealMode::RAMAN.as_raw(),
            "panchanga leaked its Lahiri mode into the caller"
        );

        // Also from the default (Fagan-Bradley = 0).
        crate::set_sid_mode(SiderealMode::FAGAN_BRADLEY, 0.0, 0.0);
        let _ = panchanga(JulianDay::new(julday(
            1986,
            5,
            30,
            9.0,
            crate::body::Calendar::Gregorian,
        )));
        assert_eq!(current_sid_mode(), SiderealMode::FAGAN_BRADLEY.as_raw());
    }
}
