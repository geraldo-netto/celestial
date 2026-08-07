//! Full Jewish holiday calendar.
//!
//! All dates are computed from the Hebrew calendar engine in `omer.rs`.
//! Holidays are returned as Julian day numbers (at nightfall, ~18:00 local mean time
//! which is represented as JD + 0.25, i.e. 6 hours after midnight UT).
//!
//! # Examples
//! ```
//! use celestial_core::jewish_holidays;
//! let holidays = jewish_holidays(5785);
//! let rh = holidays.iter().find(|h| h.name == "Rosh Hashanah").unwrap();
//! assert_eq!(rh.hebrew_month, 7);
//! assert_eq!(rh.hebrew_day, 1);
//! ```

use super::omer::{
    approx_hebrew_year, hebrew_month_days, hebrew_month_start_jd, is_hebrew_leap_year,
};
use crate::units::JulianDay;

const NIGHTFALL_OFFSET: f64 = 0.25;
const MIN_SUPPORTED_JD: f64 = 347_997.25;
const MAX_SUPPORTED_JD: f64 = 1.0e10;

/// A Jewish holiday with its Hebrew date and Julian day.
#[derive(Debug, Clone)]
pub struct JewishHoliday {
    /// Common English name of the holiday.
    pub name: &'static str,
    /// Hebrew name.
    pub hebrew_name: &'static str,
    /// Hebrew month (1=Nisan … 7=Tishrei … 12/13=Adar II).
    pub hebrew_month: u8,
    /// Hebrew day of the month.
    pub hebrew_day: u8,
    /// Julian day at nightfall (start of the holiday).
    pub jd: f64,
    /// Julian day at nightfall ending the holiday (same as jd for 1-day holidays).
    pub jd_end: f64,
    /// Duration in days.
    pub days: u8,
    /// Category of the holiday.
    pub category: HolidayCategory,
}

/// Category of a Jewish holiday.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolidayCategory {
    /// Major festival (Yom Tov) — Torah-mandated.
    MajorFestival,
    /// Rabbinically ordained holiday.
    RabbinicFestival,
    /// Fast day.
    Fast,
    /// Minor holiday or special Shabbat.
    Minor,
    /// Shabbat that falls on a special date.
    SpecialShabbat,
}

/// Static descriptor for a Jewish holiday entry. Month sentinel `0` = Purim
/// month (12 in common year, 13 in leap year). `extra_end` adds extra days
/// to `jd_end` beyond `start + days` (used for holidays whose ritual day
/// ends at next nightfall but `days` counts only the calendar day).
struct HolidaySpec {
    name: &'static str,
    hebrew_name: &'static str,
    month: u8,
    day: u8,
    days: u8,
    extra_end: f64,
    category: HolidayCategory,
}

const HOLIDAY_TABLE: &[HolidaySpec] = &[
    // Tishrei
    HolidaySpec {
        name: "Rosh Hashanah",
        hebrew_name: "ראש השנה",
        month: 7,
        day: 1,
        days: 2,
        extra_end: 0.0,
        category: HolidayCategory::MajorFestival,
    },
    HolidaySpec {
        name: "Tzom Gedaliah",
        hebrew_name: "צום גדליה",
        month: 7,
        day: 3,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Fast,
    },
    HolidaySpec {
        name: "Yom Kippur",
        hebrew_name: "יום כיפור",
        month: 7,
        day: 10,
        days: 1,
        extra_end: 0.0,
        category: HolidayCategory::MajorFestival,
    },
    HolidaySpec {
        name: "Sukkot",
        hebrew_name: "סוכות",
        month: 7,
        day: 15,
        days: 7,
        extra_end: 0.0,
        category: HolidayCategory::MajorFestival,
    },
    HolidaySpec {
        name: "Shemini Atzeret",
        hebrew_name: "שמיני עצרת",
        month: 7,
        day: 22,
        days: 1,
        extra_end: 0.0,
        category: HolidayCategory::MajorFestival,
    },
    HolidaySpec {
        name: "Simchat Torah",
        hebrew_name: "שמחת תורה",
        month: 7,
        day: 23,
        days: 1,
        extra_end: 0.0,
        category: HolidayCategory::MajorFestival,
    },
    // Kislev / Tevet / Shevat
    HolidaySpec {
        name: "Hanukkah",
        hebrew_name: "חנוכה",
        month: 9,
        day: 25,
        days: 8,
        extra_end: 0.0,
        category: HolidayCategory::RabbinicFestival,
    },
    HolidaySpec {
        name: "Tzom Tevet (10 Tevet)",
        hebrew_name: "עשרה בטבת",
        month: 10,
        day: 10,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Fast,
    },
    HolidaySpec {
        name: "Tu BiShvat",
        hebrew_name: "ט\"ו בשבט",
        month: 11,
        day: 15,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Minor,
    },
    // Purim month (sentinel 0 → 12 or 13)
    HolidaySpec {
        name: "Ta'anit Esther",
        hebrew_name: "תענית אסתר",
        month: 0,
        day: 13,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Fast,
    },
    HolidaySpec {
        name: "Purim",
        hebrew_name: "פורים",
        month: 0,
        day: 14,
        days: 2,
        extra_end: 0.0,
        category: HolidayCategory::RabbinicFestival,
    },
    // Nisan
    HolidaySpec {
        name: "Ta'anit Bechorot (Fast of the Firstborn)",
        hebrew_name: "תענית בכורות",
        month: 1,
        day: 14,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Fast,
    },
    HolidaySpec {
        name: "Passover (Pesach)",
        hebrew_name: "פסח",
        month: 1,
        day: 15,
        days: 8,
        extra_end: 0.0,
        category: HolidayCategory::MajorFestival,
    },
    HolidaySpec {
        name: "Yom HaShoah",
        hebrew_name: "יום השואה",
        month: 1,
        day: 27,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Minor,
    },
    // Iyyar
    HolidaySpec {
        name: "Yom HaZikaron",
        hebrew_name: "יום הזיכרון",
        month: 2,
        day: 4,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Minor,
    },
    HolidaySpec {
        name: "Yom HaAtzmaut",
        hebrew_name: "יום העצמאות",
        month: 2,
        day: 5,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Minor,
    },
    HolidaySpec {
        name: "Lag Ba'Omer",
        hebrew_name: "ל\"ג בעומר",
        month: 2,
        day: 18,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Minor,
    },
    HolidaySpec {
        name: "Yom Yerushalayim",
        hebrew_name: "יום ירושלים",
        month: 2,
        day: 28,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Minor,
    },
    // Sivan
    HolidaySpec {
        name: "Shavuot",
        hebrew_name: "שבועות",
        month: 3,
        day: 6,
        days: 2,
        extra_end: 0.0,
        category: HolidayCategory::MajorFestival,
    },
    // Tammuz
    HolidaySpec {
        name: "Shiva Asar B'Tammuz",
        hebrew_name: "שבעה עשר בתמוז",
        month: 4,
        day: 17,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Fast,
    },
    // Av
    HolidaySpec {
        name: "Tisha B'Av",
        hebrew_name: "תשעה באב",
        month: 5,
        day: 9,
        days: 1,
        extra_end: 0.0,
        category: HolidayCategory::Fast,
    },
    HolidaySpec {
        name: "Tu B'Av",
        hebrew_name: "ט\"ו באב",
        month: 5,
        day: 15,
        days: 1,
        extra_end: -1.0,
        category: HolidayCategory::Minor,
    },
];

#[inline]
fn nightfall_jd(hebrew_year: i32, month: u8, day: u8) -> f64 {
    hebrew_month_start_jd(hebrew_year, month as i32) as f64 + (day as f64 - 1.0) + NIGHTFALL_OFFSET
}

fn bounded_jd(jd: f64) -> f64 {
    if jd.is_finite() {
        jd.clamp(MIN_SUPPORTED_JD, MAX_SUPPORTED_JD)
    } else {
        MIN_SUPPORTED_JD
    }
}

fn materialize_holiday(spec: &HolidaySpec, hebrew_year: i32, purim_month: u8) -> JewishHoliday {
    let month = if spec.month == 0 {
        purim_month
    } else {
        spec.month
    };
    let start = nightfall_jd(hebrew_year, month, spec.day);
    let end = start + spec.days as f64 + spec.extra_end;
    JewishHoliday {
        name: spec.name,
        hebrew_name: spec.hebrew_name,
        hebrew_month: month,
        hebrew_day: spec.day,
        jd: start,
        jd_end: end,
        days: spec.days,
        category: spec.category.clone(),
    }
}

/// Compute all major Jewish holidays for the given Hebrew year.
///
/// Returns holidays sorted by Julian day (chronological).
#[must_use]
pub fn jewish_holidays(hebrew_year: i32) -> Vec<JewishHoliday> {
    let purim_month = if is_hebrew_leap_year(hebrew_year) {
        13u8
    } else {
        12u8
    };
    let mut out: Vec<JewishHoliday> = HOLIDAY_TABLE
        .iter()
        .map(|spec| materialize_holiday(spec, hebrew_year, purim_month))
        .collect();
    out.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    out
}

/// Return the JD of a specific Jewish holiday in the given Hebrew year.
/// Returns `None` if the holiday name is not found.
pub fn jewish_holiday_jd(hebrew_year: i32, name: &str) -> Option<f64> {
    jewish_holidays(hebrew_year)
        .into_iter()
        .find(|h| h.name == name)
        .map(|h| h.jd)
}

/// Return the current Hebrew year for a given Julian day.
///
/// Finite values are clamped to 347997.25..=1e10. Non-finite values map to year 1.
#[must_use]
pub fn hebrew_year_from_jd(jd: JulianDay) -> i32 {
    let jd = bounded_jd(jd.into());
    let max_year = approx_hebrew_year(JulianDay::new(MAX_SUPPORTED_JD)).saturating_add(1);
    let year = approx_hebrew_year(JulianDay::new(jd)).clamp(1, max_year);
    // The mean-year estimate differs by at most one year over the supported JD range.
    if nightfall_jd(year, 7, 1) > jd {
        return year.saturating_sub(1).max(1);
    }
    let next_year = year.saturating_add(1);
    if nightfall_jd(next_year, 7, 1) <= jd {
        next_year.min(max_year)
    } else {
        year
    }
}

/// Convert a Julian day to a Hebrew date (year, month, day).
/// Month: 1=Nisan, 2=Iyyar, …, 7=Tishrei, …, 12=Adar (or Adar I), 13=Adar II (leap)
///
/// Finite values are clamped to 347997.25..=1e10. Non-finite values map to 1 Tishrei 1.
#[must_use]
pub fn jd_to_hebrew_date(jd: JulianDay) -> (i32, u8, u8) {
    let jd = bounded_jd(jd.into());
    let year = hebrew_year_from_jd(JulianDay::new(jd));
    let jd_int = (jd - NIGHTFALL_OFFSET).floor() as i64;
    // Find month by walking from Tishrei
    let months: Vec<i32> = {
        let n = if is_hebrew_leap_year(year) { 13 } else { 12 };
        (7..=n).chain(1..7).collect()
    };
    let rem = jd_int;
    for &m in &months {
        let start = hebrew_month_start_jd(year, m);
        let days = hebrew_month_days(year, m);
        if rem < start + days {
            let day = (rem - start + 1).max(1) as u8;
            return (year, m as u8, day);
        }
    }
    (year, 1, 1)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holidays_5785_rosh_hashanah() {
        // Rosh Hashanah 5785 = Oct 2-3, 2024
        let h = jewish_holiday_jd(5785, "Rosh Hashanah").unwrap();
        // JD of Oct 2, 2024 nightfall ≈ JD 2460585.75
        assert!((h - 2_460_585.75).abs() < 1.5, "Rosh Hashanah JD {h}");
    }

    #[test]
    fn holidays_5785_passover() {
        // Passover 5785 begins 15 Nisan = April 12, 2025 nightfall
        let h = jewish_holiday_jd(5785, "Passover (Pesach)").unwrap();
        assert!((h - 2_460_778.25).abs() < 1.5, "Passover JD {h}");
    }

    #[test]
    fn holidays_5785_count() {
        let holidays = jewish_holidays(5785);
        assert!(holidays.len() >= 20);
    }

    #[test]
    fn holidays_sorted_chronologically() {
        let h = jewish_holidays(5785);
        for i in 1..h.len() {
            assert!(h[i].jd >= h[i - 1].jd);
        }
    }

    #[test]
    fn holidays_count_stable_common_year() {
        let h = jewish_holidays(5785);
        assert_eq!(h.len(), HOLIDAY_TABLE.len());
    }

    #[test]
    fn holidays_count_stable_leap_year() {
        // 5787 is leap (13 months); count must equal table length.
        let h = jewish_holidays(5787);
        assert_eq!(h.len(), HOLIDAY_TABLE.len());
    }

    #[test]
    fn purim_month_shifts_in_leap_year() {
        // Purim in Adar II (13) for leap year, Adar (12) for common year.
        let common = jewish_holidays(5785);
        let leap = jewish_holidays(5787);
        let purim_common = common.iter().find(|h| h.name == "Purim").unwrap();
        let purim_leap = leap.iter().find(|h| h.name == "Purim").unwrap();
        assert_eq!(purim_common.hebrew_month, 12);
        assert_eq!(purim_leap.hebrew_month, 13);
    }

    #[test]
    fn holidays_end_after_start() {
        for spec_year in [5783, 5784, 5785, 5786, 5787, 5788] {
            for h in jewish_holidays(spec_year) {
                assert!(
                    h.jd_end >= h.jd,
                    "{}: jd_end {} < jd {}",
                    h.name,
                    h.jd_end,
                    h.jd
                );
            }
        }
    }

    #[test]
    fn yom_kippur_one_day_jd_end_offset() {
        // Yom Kippur is days=1 but ends at next nightfall: jd_end = jd + 1.0
        let h = jewish_holidays(5785);
        let yk = h.iter().find(|h| h.name == "Yom Kippur").unwrap();
        assert!((yk.jd_end - yk.jd - 1.0).abs() < 1e-9);
        assert_eq!(yk.days, 1);
    }

    #[test]
    fn one_day_minor_jd_end_equals_jd() {
        // Tu BiShvat is a 1-day minor holiday → jd_end == jd
        let h = jewish_holidays(5785);
        let tb = h.iter().find(|h| h.name == "Tu BiShvat").unwrap();
        assert!((tb.jd_end - tb.jd).abs() < 1e-9);
    }

    #[test]
    fn daytime_holidays_have_zero_length_jd_range() {
        let holidays = jewish_holidays(5785);
        let names = [
            "Tzom Gedaliah",
            "Tzom Tevet (10 Tevet)",
            "Tu BiShvat",
            "Ta'anit Esther",
            "Ta'anit Bechorot (Fast of the Firstborn)",
            "Yom HaShoah",
            "Yom HaZikaron",
            "Yom HaAtzmaut",
            "Lag Ba'Omer",
            "Yom Yerushalayim",
            "Shiva Asar B'Tammuz",
            "Tu B'Av",
        ];
        for name in names {
            let holiday = holidays
                .iter()
                .find(|holiday| holiday.name == name)
                .unwrap();
            assert_eq!(holiday.jd_end, holiday.jd, "{name}");
        }
    }

    #[test]
    fn hebrew_date_rolls_over_at_nightfall() {
        let start = nightfall_jd(5785, 7, 1);
        let cases = [
            (start - 0.01, (5784, 6, 29)),
            (start, (5785, 7, 1)),
            (start + 0.99, (5785, 7, 1)),
            (start + 1.0, (5785, 7, 2)),
        ];
        for (jd, expected) in cases {
            assert_eq!(jd_to_hebrew_date(JulianDay::new(jd)), expected);
            assert_eq!(hebrew_year_from_jd(JulianDay::new(jd)), expected.0);
        }
    }

    #[test]
    fn hebrew_month_rolls_over_at_nightfall() {
        let start = nightfall_jd(5785, 10, 1);
        let kislev_days = hebrew_month_days(5785, 9) as u8;
        assert_eq!(
            jd_to_hebrew_date(JulianDay::new(start - 0.01)),
            (5785, 9, kislev_days)
        );
        assert_eq!(jd_to_hebrew_date(JulianDay::new(start)), (5785, 10, 1));
    }

    #[test]
    fn hebrew_year_adjusts_underestimated_year() {
        let start = nightfall_jd(5000, 7, 1);
        assert_eq!(approx_hebrew_year(JulianDay::new(start)), 4999);
        for jd in [start, start + 0.1] {
            assert_eq!(approx_hebrew_year(JulianDay::new(jd)), 4999);
            assert_eq!(hebrew_year_from_jd(JulianDay::new(jd)), 5000);
        }
    }

    #[test]
    fn invalid_and_extreme_jds_are_bounded() {
        for jd in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -f64::MAX] {
            assert_eq!(hebrew_year_from_jd(JulianDay::new(jd)), 1);
            assert_eq!(jd_to_hebrew_date(JulianDay::new(jd)), (1, 7, 1));
        }
        assert!(hebrew_year_from_jd(JulianDay::new(f64::MAX)) > 1);
    }

    #[test]
    fn fuzz_random_years_no_panic_and_sorted() {
        // Deterministic xorshift-ish year sweep across plausible Hebrew years.
        let mut s: u64 = 0xC0FFEE;
        for _ in 0..256 {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let year = (s % 9000) as i32 + 1; // 1..9000
            let h = jewish_holidays(year);
            assert_eq!(h.len(), HOLIDAY_TABLE.len(), "year {year}");
            for w in h.windows(2) {
                assert!(w[1].jd >= w[0].jd, "year {year} unsorted");
            }
        }
    }

    #[test]
    fn jd_to_hebrew_date_known() {
        // JD 2451545.0 = Jan 1, 2000 = 23 Tevet 5760
        let (y, m, d) = jd_to_hebrew_date(JulianDay::new(2_451_545.0));
        assert_eq!(y, 5760);
        assert_eq!(m, 10); // Tevet
        assert_eq!(d, 23);
    }

    #[test]
    fn hebrew_year_before_rosh_hashanah_handles_estimate_overshoot() {
        let jd = crate::julday(1853, 10, 1, 12.0, crate::body::Calendar::Gregorian);
        assert_eq!(approx_hebrew_year(JulianDay::new(jd)), 5614);
        assert_eq!(hebrew_year_from_jd(JulianDay::new(jd)), 5613);
        assert_eq!(jd_to_hebrew_date(JulianDay::new(jd)).0, 5613);
    }
}
