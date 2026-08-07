//! Auto-split from chart.rs — do not edit section headers.

#[allow(unused_imports)]
use crate::functions::time::revjul;
use crate::units::{JulianDay, Longitude};

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 6 — Chinese astrology (Ba Zi / Four Pillars + Solar Terms)
// ═══════════════════════════════════════════════════════════════════════════════

/// One pillar of the Four Pillars (Ba Zi) chart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaZiPillar {
    /// Heavenly Stem index (0–9): Jiǎ, Yǐ, Bǐng, Dīng, Wù, Jǐ, Gēng, Xīn, Rén, Guǐ
    pub stem: u8,
    /// Earthly Branch index (0–11): Rat, Ox, Tiger, Rabbit, Dragon, Snake,
    ///                               Horse, Goat, Monkey, Rooster, Dog, Pig
    pub branch: u8,
    /// Stem name in Pinyin
    pub stem_name: &'static str,
    /// Branch name in English
    pub branch_name: &'static str,
    /// Branch animal name
    pub animal: &'static str,
    /// Element of the stem (Wood, Fire, Earth, Metal, Water)
    pub stem_element: &'static str,
    /// Element of the branch
    pub branch_element: &'static str,
    /// Yin (false) or Yang (true) polarity
    pub yang: bool,
}

/// Heavenly Stems (天干) in traditional order.
pub const HEAVENLY_STEMS: &[(&str, &str, bool)] = &[
    // (Pinyin, Element, Yang?)
    ("Jiǎ", "Wood", true),
    ("Yǐ", "Wood", false),
    ("Bǐng", "Fire", true),
    ("Dīng", "Fire", false),
    ("Wù", "Earth", true),
    ("Jǐ", "Earth", false),
    ("Gēng", "Metal", true),
    ("Xīn", "Metal", false),
    ("Rén", "Water", true),
    ("Guǐ", "Water", false),
];

/// Earthly Branches (地支) with animal, element, and polarity.
pub const EARTHLY_BRANCHES: &[(&str, &str, &str, bool)] = &[
    // (Name, Animal, Element, Yang?)
    ("Zǐ", "Rat", "Water", true),
    ("Chǒu", "Ox", "Earth", false),
    ("Yín", "Tiger", "Wood", true),
    ("Mǎo", "Rabbit", "Wood", false),
    ("Chén", "Dragon", "Earth", true),
    ("Sì", "Snake", "Fire", false),
    ("Wǔ", "Horse", "Fire", true),
    ("Wèi", "Goat", "Earth", false),
    ("Shēn", "Monkey", "Metal", true),
    ("Yǒu", "Rooster", "Metal", false),
    ("Xū", "Dog", "Earth", true),
    ("Hài", "Pig", "Water", false),
];

/// Construct a `BaZiPillar` from stem and branch indices.
#[must_use]
pub fn make_pillar(stem: u8, branch: u8) -> BaZiPillar {
    let s = stem as usize % 10;
    let b = branch as usize % 12;
    BaZiPillar {
        stem,
        branch,
        stem_name: HEAVENLY_STEMS[s].0,
        branch_name: EARTHLY_BRANCHES[b].0,
        animal: EARTHLY_BRANCHES[b].1,
        stem_element: HEAVENLY_STEMS[s].1,
        branch_element: EARTHLY_BRANCHES[b].2,
        yang: HEAVENLY_STEMS[s].2,
    }
}

/// Compute the Four Pillars (Ba Zi) for a Julian Day and hour.
///
/// Returns `[year_pillar, month_pillar, day_pillar, hour_pillar]`.
///
/// The month pillar is based on solar terms: each Chinese month begins at
/// a "Jié" (节) solar term when the Sun crosses a multiple of 30° longitude
/// (starting from 315° = Lì Chūn / Start of Spring).
///
/// # Arguments
/// * `jd_ut`   — Julian Day (UT)
/// * `hour_ut` — hour of day (0.0–23.99, UT)
/// * `sun_lon` — Sun's ecliptic longitude at `jd_ut` (degrees)
#[must_use]
pub fn four_pillars(jd_ut: JulianDay, hour_ut: f64, sun_lon: Longitude) -> [BaZiPillar; 4] {
    let jd_ut: f64 = jd_ut.into();
    let sun_lon = f64::from(sun_lon).rem_euclid(360.0);
    // ── Year pillar ───────────────────────────────────────────────────────────
    // Chinese year starts at Lì Chūn (立春, Start of Spring, Sun ≈ 315°).
    // Approximate by Gregorian year: adjust if before ~Feb 4 (315° not yet reached).
    let d = crate::revjul(JulianDay::new(jd_ut), crate::body::Calendar::Gregorian);
    // Sun at 315° ≈ Feb 3–5; use sun_lon to determine if the Chinese year has turned
    // For simplicity, use the previous year in January and before Lì Chūn in February.
    let before_li_chun = d.month == 1 || (d.month == 2 && sun_lon < 315.0);
    let chinese_year = if before_li_chun { d.year - 1 } else { d.year };
    // Sexagenary cycle: year 4 CE = cycle 0 (Jiǎ-Zǐ)
    let year_cycle = ((chinese_year - 4).rem_euclid(60)) as u8;
    let year_stem = year_cycle % 10;
    let year_branch = year_cycle % 12;

    // ── Month pillar ──────────────────────────────────────────────────────────
    // 12 "Jié" solar terms start the Chinese months. They fall at Sun longitudes:
    // 315° (Tiger/Yín), 345° (Rabbit), 15° (Dragon), 45° (Snake), 75° (Horse),
    // 105° (Goat), 135° (Monkey), 165° (Rooster), 195° (Dog), 225° (Pig),
    // 255° (Rat), 285° (Ox)
    let months_from_tiger = ((sun_lon - 315.0).rem_euclid(360.0) / 30.0).floor() as u8;
    let month_branch = (2 + months_from_tiger) % 12;
    // Month stem depends on year stem: each 5-year group repeats the stem pattern.
    const MONTH_STEM_OFFSET: [u8; 5] = [2, 4, 6, 8, 0];
    let stem_offset = MONTH_STEM_OFFSET[(year_stem % 5) as usize];
    let month_stem = (stem_offset + months_from_tiger) % 10;

    // ── Day pillar ────────────────────────────────────────────────────────────
    // The civil-date JDN maps to sexagenary index `(JDN + 49) mod 60`.
    let jdn = (jd_ut + 0.5).floor() as i64;
    let day_cycle = ((jdn + 49).rem_euclid(60)) as u8;
    let day_stem = day_cycle % 10;
    let day_branch = day_cycle % 12;

    // ── Hour pillar ───────────────────────────────────────────────────────────
    // 12 double-hours: Zǐ starts at 23:00, Chǒu at 01:00, ...
    // hour_branch = ((hour + 1) / 2) % 12  (Zǐ = 0 at 23–01)
    let h = (hour_ut + 1.0) as u8;
    let hour_branch = (h / 2) % 12;
    // Hour stem depends on day stem: day_stem % 5 × 2 gives Zǐ-hour stem.
    const HOUR_STEM_OFFSET: [u8; 5] = [0, 2, 4, 6, 8];
    let hour_stem = (HOUR_STEM_OFFSET[(day_stem % 5) as usize] + hour_branch) % 10;

    [
        make_pillar(year_stem, year_branch),
        make_pillar(month_stem, month_branch),
        make_pillar(day_stem, day_branch),
        make_pillar(hour_stem, hour_branch),
    ]
}

/// 24 solar terms with their Sun longitude thresholds and names.
pub const SOLAR_TERMS: &[(f64, &str, &str)] = &[
    // (Sun longitude°, Pinyin, English)
    (0.0, "Chūnfēn", "Spring Equinox"),
    (15.0, "Qīngmíng", "Clear and Bright"),
    (30.0, "Gǔyǔ", "Grain Rain"),
    (45.0, "Lìxià", "Start of Summer"),
    (60.0, "Xiǎomǎn", "Grain Buds"),
    (75.0, "Mángzhòng", "Grain in Ear"),
    (90.0, "Xiàzhì", "Summer Solstice"),
    (105.0, "Xiǎoshǔ", "Minor Heat"),
    (120.0, "Dàshǔ", "Major Heat"),
    (135.0, "Lìqiū", "Start of Autumn"),
    (150.0, "Chǔshǔ", "End of Heat"),
    (165.0, "Báilù", "White Dew"),
    (180.0, "Qiūfēn", "Autumn Equinox"),
    (195.0, "Hánlù", "Cold Dew"),
    (210.0, "Shuāngjiàng", "Frost's Descent"),
    (225.0, "Lìdōng", "Start of Winter"),
    (240.0, "Xiǎoxuě", "Minor Snow"),
    (255.0, "Dàxuě", "Major Snow"),
    (270.0, "Dōngzhì", "Winter Solstice"),
    (285.0, "Xiǎohán", "Minor Cold"),
    (300.0, "Dàhán", "Major Cold"),
    (315.0, "Lìchūn", "Start of Spring"),
    (330.0, "Yǔshuǐ", "Rain Water"),
    (345.0, "Jīngzhé", "Awakening of Insects"),
];

/// Return the current and next solar term for a given Sun longitude.
///
/// Returns `(current_term_index, degrees_into_term, next_term_index, degrees_to_next)`.
#[must_use]
pub fn solar_term_position(sun_lon: Longitude) -> (usize, f64, usize, f64) {
    let sun_lon: f64 = sun_lon.into();
    let lon = sun_lon.rem_euclid(360.0);
    let current = SOLAR_TERMS
        .iter()
        .rposition(|(threshold, ..)| lon >= *threshold)
        .unwrap_or(SOLAR_TERMS.len() - 1);
    let next = (current + 1) % SOLAR_TERMS.len();
    let current_lon = SOLAR_TERMS[current].0;
    let next_lon = if next == 0 {
        360.0
    } else {
        SOLAR_TERMS[next].0
    };
    let degrees_into = lon - current_lon;
    let degrees_to = next_lon - lon;
    (current, degrees_into, next, degrees_to)
}

/// 60-year sexagenary cycle name (Jiǎ-Zǐ, Yǐ-Chǒu, …).
#[must_use]
pub fn sexagenary_name(cycle_index: u8) -> (&'static str, &'static str) {
    let s = cycle_index as usize % 10;
    let b = cycle_index as usize % 12;
    (HEAVENLY_STEMS[s].0, EARTHLY_BRANCHES[b].1)
}

// ═══════════════════════════════════════════════════════════════════════════
// Vietnamese Âm Lịch (lunar calendar)
// ═══════════════════════════════════════════════════════════════════════════
//
// Structurally similar to the Chinese calendar, but uses Indochina Time
// (UTC+7) for lunar-month boundary determination. This causes roughly 20%
// of months to begin one civil day before or after their Chinese
// equivalent. Vietnam has used UTC+7 officially since 1967.

/// Vietnam / Indochina Time offset from UTC (hours).
pub const VIETNAM_TZ_OFFSET_HOURS: f64 = 7.0;

/// China Standard Time offset from UTC, for comparison (hours).
pub const CHINA_TZ_OFFSET_HOURS: f64 = 8.0;

/// Civil day (Vietnam UTC+7) containing the new moon that starts the
/// Vietnamese lunar month containing `jd_ut`.
///
/// Returns the Hanoi civil day's start expressed as a UT Julian Day.
/// Returns `None` if no new moon is found within the last 30 days.
pub fn vietnamese_month_start_jd(jd_ut: JulianDay) -> Option<f64> {
    let jd_ut: f64 = jd_ut.into();
    // Search backwards: find the new moon preceding jd_ut.
    let mut search = jd_ut - 30.0;
    let mut last_nm: Option<f64> = None;
    for _ in 0..2 {
        match crate::functions::moon_phases::next_new_moon(JulianDay::new(search)) {
            Ok(nm) if nm <= jd_ut => {
                last_nm = Some(nm);
                search = nm + 2.0;
            }
            _ => break,
        }
    }
    let nm = last_nm?;
    // Convert the new moon to its Hanoi civil day, then express local midnight in UT.
    let local = nm + VIETNAM_TZ_OFFSET_HOURS / 24.0;
    Some((local - 0.5).floor() + 0.5 - VIETNAM_TZ_OFFSET_HOURS / 24.0)
}

/// Returns `true` if the given JD falls on a different civil day in
/// Vietnam (UTC+7) vs. China (UTC+8).
///
/// Useful for detecting the ~20% of new moons where the lunar month
/// starts a different civil day in the two calendars.
#[must_use]
pub fn vietnamese_chinese_boundary_differs(jd_ut: JulianDay) -> bool {
    let jd_ut: f64 = jd_ut.into();
    let vn = (jd_ut + VIETNAM_TZ_OFFSET_HOURS / 24.0 + 0.5).floor();
    let cn = (jd_ut + CHINA_TZ_OFFSET_HOURS / 24.0 + 0.5).floor();
    vn != cn
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jd(year: i32, month: i32, day: i32, hour: f64) -> f64 {
        crate::julday(year, month, day, hour, crate::body::Calendar::Gregorian)
    }

    fn pillar_indices(jd_ut: f64, hour_ut: f64, sun_lon: f64) -> [(u8, u8); 4] {
        let pillars = four_pillars(JulianDay::new(jd_ut), hour_ut, Longitude::new(sun_lon));
        pillars.map(|pillar| (pillar.stem, pillar.branch))
    }

    #[test]
    fn make_pillar_wraps_metadata_but_preserves_indices() {
        let pillar = make_pillar(21, 25);
        assert_eq!((pillar.stem, pillar.branch), (21, 25));
        assert_eq!((pillar.stem_name, pillar.stem_element), ("Yǐ", "Wood"));
        assert_eq!((pillar.branch_name, pillar.animal), ("Chǒu", "Ox"));
        assert_eq!(pillar.branch_element, "Earth");
        assert!(!pillar.yang);
    }

    #[test]
    fn year_pillar_changes_at_li_chun() {
        let cases = [
            (2000, 1, 1, 280.0, (5, 3)),
            (2000, 2, 3, 314.9, (5, 3)),
            (2000, 2, 4, 315.0, (6, 4)),
            (2000, 3, 1, 314.9, (6, 4)),
        ];
        for (year, month, day, sun_lon, expected) in cases {
            assert_eq!(
                pillar_indices(jd(year, month, day, 0.0), 0.0, sun_lon)[0],
                expected
            );
        }
    }

    #[test]
    fn month_pillar_follows_wrapped_jie_sectors() {
        let cases = [
            (314.999, (5, 1)),
            (315.0, (4, 2)),
            (344.999, (4, 2)),
            (345.0, (5, 3)),
            (14.999, (5, 3)),
            (15.0, (6, 4)),
            (-15.0, (5, 3)),
            (375.0, (6, 4)),
        ];
        for (sun_lon, expected) in cases {
            assert_eq!(
                pillar_indices(jd(2000, 3, 1, 0.0), 0.0, sun_lon)[1],
                expected
            );
        }
        assert_eq!(pillar_indices(jd(2001, 3, 1, 0.0), 0.0, 315.0)[1], (6, 2));
    }

    #[test]
    fn day_pillar_uses_civil_date_sexagenary_cycle() {
        let cases = [
            (jd(1970, 1, 1, 0.0), (7, 5)),
            (jd(1970, 1, 1, 11.99), (7, 5)),
            (jd(1970, 1, 2, 0.0), (8, 6)),
            (jd(2000, 1, 1, 0.0), (4, 6)),
        ];
        for (date_jd, expected) in cases {
            assert_eq!(pillar_indices(date_jd, 0.0, 315.0)[2], expected);
        }
    }

    #[test]
    fn hour_pillar_changes_at_double_hour_boundaries() {
        let date_jd = jd(1970, 1, 1, 0.0);
        let cases = [
            (0.0, (4, 0)),
            (0.999, (4, 0)),
            (1.0, (5, 1)),
            (2.999, (5, 1)),
            (3.0, (6, 2)),
            (22.999, (5, 11)),
            (23.0, (4, 0)),
        ];
        for (hour, expected) in cases {
            assert_eq!(pillar_indices(date_jd, hour, 315.0)[3], expected);
        }
    }

    #[test]
    fn solar_term_position_reports_boundaries_and_wrap() {
        let cases = [
            (0.0, (0, 0.0, 1, 15.0)),
            (14.25, (0, 14.25, 1, 0.75)),
            (15.0, (1, 0.0, 2, 15.0)),
            (344.0, (22, 14.0, 23, 1.0)),
            (345.0, (23, 0.0, 0, 15.0)),
            (359.0, (23, 14.0, 0, 1.0)),
            (-1.0, (23, 14.0, 0, 1.0)),
            (360.0, (0, 0.0, 1, 15.0)),
        ];
        for (sun_lon, expected) in cases {
            assert_eq!(solar_term_position(Longitude::new(sun_lon)), expected);
        }
    }

    #[test]
    fn solar_term_position_handles_non_finite_longitude() {
        let (current, degrees_into, next, degrees_to) =
            solar_term_position(Longitude::new(f64::NAN));
        assert_eq!((current, next), (23, 0));
        assert!(degrees_into.is_nan());
        assert!(degrees_to.is_nan());
    }

    #[test]
    fn sexagenary_name_wraps_both_cycles() {
        let cases = [
            (0, ("Jiǎ", "Rat")),
            (11, ("Yǐ", "Pig")),
            (59, ("Guǐ", "Pig")),
            (60, ("Jiǎ", "Rat")),
            (121, ("Yǐ", "Ox")),
        ];
        for (cycle_index, expected) in cases {
            assert_eq!(sexagenary_name(cycle_index), expected);
        }
    }

    #[test]
    fn vietnamese_month_start_is_hanoi_midnight() {
        let cases = [
            ((2025, 4, 30), (2025, 4, 27)),
            ((2025, 6, 26), (2025, 6, 24)),
            ((2025, 6, 28), (2025, 6, 24)),
        ];
        for ((year, month, day), (start_year, start_month, start_day)) in cases {
            let actual = vietnamese_month_start_jd(JulianDay::new(jd(year, month, day, 0.0)))
                .expect("new moon should be found");
            let expected = jd(start_year, start_month, start_day, 17.0);
            assert!((actual - expected).abs() < 1e-9, "actual={actual}");
        }
    }

    #[test]
    fn tz_offsets_differ_by_1h() {
        assert!((CHINA_TZ_OFFSET_HOURS - VIETNAM_TZ_OFFSET_HOURS - 1.0).abs() < 1e-9);
    }

    #[test]
    fn boundary_differs_detection() {
        let cases = [(15.5, false), (16.5, true), (17.5, false), (23.5, false)];
        for (hour, expected) in cases {
            assert_eq!(
                vietnamese_chinese_boundary_differs(JulianDay::new(jd(2000, 1, 1, hour))),
                expected
            );
        }
    }
}
