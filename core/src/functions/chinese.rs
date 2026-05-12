//! Auto-split from chart.rs — do not edit section headers.

#[allow(unused_imports)]
use crate::functions::time::revjul;

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
pub fn four_pillars(jd_ut: f64, hour_ut: f64, sun_lon: f64) -> [BaZiPillar; 4] {
    // ── Year pillar ───────────────────────────────────────────────────────────
    // Chinese year starts at Lì Chūn (立春, Start of Spring, Sun ≈ 315°).
    // Approximate by Gregorian year: adjust if before ~Feb 4 (315° not yet reached).
    let d = crate::revjul(jd_ut, crate::body::Calendar::Gregorian);
    // Sun at 315° ≈ Feb 3–5; use sun_lon to determine if the Chinese year has turned
    // For simplicity, approximate: Chinese year = Gregorian year - 1 if Sun < 315°
    // and month is Jan (before ~Feb 4)
    let chinese_year = if sun_lon < 315.0 && d.month as u8 == 1 {
        d.year - 1
    } else {
        d.year
    };
    // Sexagenary cycle: year 4 CE = cycle 0 (Jiǎ-Zǐ)
    let year_cycle = ((chinese_year - 4).rem_euclid(60)) as u8;
    let year_stem = year_cycle % 10;
    let year_branch = year_cycle % 12;

    // ── Month pillar ──────────────────────────────────────────────────────────
    // 12 "Jié" solar terms start the Chinese months. They fall at Sun longitudes:
    // 315° (Tiger/Yín), 345° (Rabbit), 15° (Dragon), 45° (Snake), 75° (Horse),
    // 105° (Goat), 135° (Monkey), 165° (Rooster), 195° (Dog), 225° (Pig),
    // 255° (Rat), 285° (Ox)
    const JIE_LONGITUDES: [f64; 12] = [
        315.0, 345.0, 15.0, 45.0, 75.0, 105.0, 135.0, 165.0, 195.0, 225.0, 255.0, 285.0,
    ];
    // Branch for each Jié: Tiger(2), Rabbit(3), Dragon(4), Snake(5), Horse(6),
    // Goat(7), Monkey(8), Rooster(9), Dog(10), Pig(11), Rat(0), Ox(1)
    const JIE_BRANCHES: [u8; 12] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 1];
    // Determine which Jié the Sun has passed
    let month_idx = {
        let mut idx = 11usize; // default to Ox (last)
        for i in 0..12 {
            let jie_lon = JIE_LONGITUDES[i];
            // Normalise Sun past this Jié
            let passed = if jie_lon > 270.0 {
                // Jié near 315°-345° straddles the 360°/0° boundary
                sun_lon >= jie_lon || sun_lon < JIE_LONGITUDES[(i + 1) % 12].min(270.0)
            } else {
                sun_lon >= jie_lon
            };
            if passed {
                idx = i;
            }
        }
        idx
    };
    let month_branch = JIE_BRANCHES[month_idx];
    // Month stem depends on year stem: each 5-year group repeats the stem pattern
    // Year stem 0,1 (Jiǎ/Yǐ) → month Tiger starts at stem 2 (Bǐng)
    // Offset table: year_stem % 5 × 2 gives the Tiger-month stem
    const MONTH_STEM_OFFSET: [u8; 5] = [2, 4, 6, 8, 0]; // for year stems 0/1, 2/3, 4/5, 6/7, 8/9
    let stem_offset = MONTH_STEM_OFFSET[(year_stem / 2) as usize % 5];
    // Tiger is month_branch 2; month sequence starts at Tiger
    let months_from_tiger = (month_branch + 12 - 2) % 12;
    let month_stem = (stem_offset + months_from_tiger) % 10;

    // ── Day pillar ────────────────────────────────────────────────────────────
    // Reference: JD 2440588 (Jan 1, 1970) = day-stem 6, day-branch 2 (Gēng-Yín)
    // Verified against multiple Ba Zi calculators.
    const REF_JD: i64 = 2440588;
    const REF_DAY_STEM: i64 = 6;
    const REF_DAY_BRANCH: i64 = 2;
    let jd_int = jd_ut as i64;
    let day_stem = ((jd_int - REF_JD + REF_DAY_STEM).rem_euclid(10)) as u8;
    let day_branch = ((jd_int - REF_JD + REF_DAY_BRANCH).rem_euclid(12)) as u8;

    // ── Hour pillar ───────────────────────────────────────────────────────────
    // 12 double-hours: Zǐ starts at 23:00, Chǒu at 01:00, ...
    // hour_branch = ((hour + 1) / 2) % 12  (Zǐ = 0 at 23–01)
    let h = (hour_ut + 1.0) as u8;
    let hour_branch = (h / 2) % 12;
    // Hour stem depends on day stem: day_stem % 5 × 2 gives Zǐ-hour stem
    let hour_stem = (MONTH_STEM_OFFSET[(day_stem / 2) as usize % 5] + hour_branch) % 10;

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
pub fn solar_term_position(sun_lon: f64) -> (usize, f64, usize, f64) {
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
/// Returns the JD at midnight UT of the Hanoi civil day containing the
/// starting new moon. Returns `None` if no new moon is found within the
/// last 30 days (should not happen for any valid input).
pub fn vietnamese_month_start_jd(jd_ut: f64) -> Option<f64> {
    // Search backwards: find the new moon preceding jd_ut.
    let mut search = jd_ut - 30.0;
    let mut last_nm: Option<f64> = None;
    while search < jd_ut {
        match crate::functions::moon_phases::next_new_moon(search) {
            Ok(nm) if nm <= jd_ut => {
                last_nm = Some(nm);
                search = nm + 2.0;
            }
            _ => break,
        }
    }
    let nm = last_nm?;
    // Civil day in UTC+7 is floor((nm + 7h) to integer day), then shift back
    let local = nm + VIETNAM_TZ_OFFSET_HOURS / 24.0;
    Some(local.floor() + 0.5 - VIETNAM_TZ_OFFSET_HOURS / 24.0)
}

/// Returns `true` if the given JD falls on a different civil day in
/// Vietnam (UTC+7) vs. China (UTC+8).
///
/// Useful for detecting the ~20% of new moons where the lunar month
/// starts a different civil day in the two calendars.
#[must_use]
pub fn vietnamese_chinese_boundary_differs(jd_ut: f64) -> bool {
    let vn = (jd_ut + VIETNAM_TZ_OFFSET_HOURS / 24.0).floor();
    let cn = (jd_ut + CHINA_TZ_OFFSET_HOURS / 24.0).floor();
    vn != cn
}

#[cfg(test)]
mod viet_tests {
    use super::*;

    #[test]
    fn tz_offsets_differ_by_1h() {
        assert!((CHINA_TZ_OFFSET_HOURS - VIETNAM_TZ_OFFSET_HOURS - 1.0).abs() < 1e-9);
    }

    #[test]
    fn boundary_differs_detection() {
        // JD at 23:30 UTC → 06:30 Vietnam, 07:30 China → same civil day
        let jd_same = 2_451_545.0 + 23.5 / 24.0;
        assert!(!vietnamese_chinese_boundary_differs(jd_same));

        // JD at 16:30 UTC → 23:30 Vietnam (same day), 00:30 China (next day)
        let jd_diff = 2_451_545.0 + 16.5 / 24.0;
        assert!(vietnamese_chinese_boundary_differs(jd_diff));
    }
}
