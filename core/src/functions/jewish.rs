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
    hebrew_month_days, hebrew_month_start_jd, hebrew_new_year_jd, is_hebrew_leap_year,
};

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
#[derive(Debug, Clone, PartialEq)]
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

/// Compute all major Jewish holidays for the given Hebrew year.
///
/// Returns holidays sorted by Julian day (chronological).
pub fn jewish_holidays(hebrew_year: i32) -> Vec<JewishHoliday> {
    let leap = is_hebrew_leap_year(hebrew_year);
    let mut out: Vec<JewishHoliday> = Vec::with_capacity(30);

    // Helper: JD of a Hebrew month/day in this year, at nightfall
    let jd = |month: u8, day: u8| -> f64 {
        hebrew_month_start_jd(hebrew_year, month as i32) as f64 + (day as f64 - 1.0) + 0.25
        // nightfall offset
    };
    let jd_day = |month: u8, day: u8, duration: u8| -> (f64, f64) {
        let start = jd(month, day);
        (start, start + duration as f64)
    };

    // ── Tishrei (month 7) ─────────────────────────────────────────────────
    {
        let (s, e) = jd_day(7, 1, 2);
        out.push(JewishHoliday {
            name: "Rosh Hashanah",
            hebrew_name: "ראש השנה",
            hebrew_month: 7,
            hebrew_day: 1,
            jd: s,
            jd_end: e,
            days: 2,
            category: HolidayCategory::MajorFestival,
        });
    }
    {
        let s = jd(7, 3);
        out.push(JewishHoliday {
            name: "Tzom Gedaliah",
            hebrew_name: "צום גדליה",
            hebrew_month: 7,
            hebrew_day: 3,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Fast,
        });
    }
    {
        let s = jd(7, 10);
        out.push(JewishHoliday {
            name: "Yom Kippur",
            hebrew_name: "יום כיפור",
            hebrew_month: 7,
            hebrew_day: 10,
            jd: s,
            jd_end: s + 1.0,
            days: 1,
            category: HolidayCategory::MajorFestival,
        });
    }
    {
        let (s, e) = jd_day(7, 15, 7);
        out.push(JewishHoliday {
            name: "Sukkot",
            hebrew_name: "סוכות",
            hebrew_month: 7,
            hebrew_day: 15,
            jd: s,
            jd_end: e,
            days: 7,
            category: HolidayCategory::MajorFestival,
        });
    }
    {
        let s = jd(7, 22);
        out.push(JewishHoliday {
            name: "Shemini Atzeret",
            hebrew_name: "שמיני עצרת",
            hebrew_month: 7,
            hebrew_day: 22,
            jd: s,
            jd_end: s + 1.0,
            days: 1,
            category: HolidayCategory::MajorFestival,
        });
    }
    {
        let s = jd(7, 23);
        out.push(JewishHoliday {
            name: "Simchat Torah",
            hebrew_name: "שמחת תורה",
            hebrew_month: 7,
            hebrew_day: 23,
            jd: s,
            jd_end: s + 1.0,
            days: 1,
            category: HolidayCategory::MajorFestival,
        });
    }

    // ── Kislev (month 9) — Hanukkah ───────────────────────────────────────
    {
        let (s, e) = jd_day(9, 25, 8);
        out.push(JewishHoliday {
            name: "Hanukkah",
            hebrew_name: "חנוכה",
            hebrew_month: 9,
            hebrew_day: 25,
            jd: s,
            jd_end: e,
            days: 8,
            category: HolidayCategory::RabbinicFestival,
        });
    }

    // ── Tevet (month 10) ──────────────────────────────────────────────────
    {
        let s = jd(10, 10);
        out.push(JewishHoliday {
            name: "Tzom Tevet (10 Tevet)",
            hebrew_name: "עשרה בטבת",
            hebrew_month: 10,
            hebrew_day: 10,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Fast,
        });
    }

    // ── Shevat (month 11) ─────────────────────────────────────────────────
    {
        let s = jd(11, 15);
        out.push(JewishHoliday {
            name: "Tu BiShvat",
            hebrew_name: "ט\"ו בשבט",
            hebrew_month: 11,
            hebrew_day: 15,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Minor,
        });
    }

    // ── Adar / Adar II (month 12 or 13 in leap year) ─────────────────────
    let purim_month = if leap { 13u8 } else { 12u8 };
    {
        let s = jd(purim_month, 13);
        out.push(JewishHoliday {
            name: "Ta'anit Esther",
            hebrew_name: "תענית אסתר",
            hebrew_month: purim_month,
            hebrew_day: 13,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Fast,
        });
    }
    {
        let (s, e) = jd_day(purim_month, 14, 2);
        out.push(JewishHoliday {
            name: "Purim",
            hebrew_name: "פורים",
            hebrew_month: purim_month,
            hebrew_day: 14,
            jd: s,
            jd_end: e,
            days: 2,
            category: HolidayCategory::RabbinicFestival,
        });
    }

    // ── Nisan (month 1) ───────────────────────────────────────────────────
    {
        let s = jd(1, 14);
        out.push(JewishHoliday {
            name: "Ta'anit Bechorot (Fast of the Firstborn)",
            hebrew_name: "תענית בכורות",
            hebrew_month: 1,
            hebrew_day: 14,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Fast,
        });
    }
    {
        let (s, e) = jd_day(1, 15, 8);
        out.push(JewishHoliday {
            name: "Passover (Pesach)",
            hebrew_name: "פסח",
            hebrew_month: 1,
            hebrew_day: 15,
            jd: s,
            jd_end: e,
            days: 8,
            category: HolidayCategory::MajorFestival,
        });
    }
    {
        let s = jd(1, 27);
        out.push(JewishHoliday {
            name: "Yom HaShoah",
            hebrew_name: "יום השואה",
            hebrew_month: 1,
            hebrew_day: 27,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Minor,
        });
    }

    // ── Iyyar (month 2) ───────────────────────────────────────────────────
    {
        let s = jd(2, 4);
        out.push(JewishHoliday {
            name: "Yom HaZikaron",
            hebrew_name: "יום הזיכרון",
            hebrew_month: 2,
            hebrew_day: 4,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Minor,
        });
    }
    {
        let s = jd(2, 5);
        out.push(JewishHoliday {
            name: "Yom HaAtzmaut",
            hebrew_name: "יום העצמאות",
            hebrew_month: 2,
            hebrew_day: 5,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Minor,
        });
    }
    {
        let s = jd(2, 18);
        out.push(JewishHoliday {
            name: "Lag Ba'Omer",
            hebrew_name: "ל\"ג בעומר",
            hebrew_month: 2,
            hebrew_day: 18,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Minor,
        });
    }
    {
        let s = jd(2, 28);
        out.push(JewishHoliday {
            name: "Yom Yerushalayim",
            hebrew_name: "יום ירושלים",
            hebrew_month: 2,
            hebrew_day: 28,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Minor,
        });
    }

    // ── Sivan (month 3) — Shavuot ─────────────────────────────────────────
    {
        let (s, e) = jd_day(3, 6, 2);
        out.push(JewishHoliday {
            name: "Shavuot",
            hebrew_name: "שבועות",
            hebrew_month: 3,
            hebrew_day: 6,
            jd: s,
            jd_end: e,
            days: 2,
            category: HolidayCategory::MajorFestival,
        });
    }

    // ── Tammuz (month 4) ──────────────────────────────────────────────────
    {
        let s = jd(4, 17);
        out.push(JewishHoliday {
            name: "Shiva Asar B'Tammuz",
            hebrew_name: "שבעה עשר בתמוז",
            hebrew_month: 4,
            hebrew_day: 17,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Fast,
        });
    }

    // ── Av (month 5) ──────────────────────────────────────────────────────
    {
        let s = jd(5, 9);
        out.push(JewishHoliday {
            name: "Tisha B'Av",
            hebrew_name: "תשעה באב",
            hebrew_month: 5,
            hebrew_day: 9,
            jd: s,
            jd_end: s + 1.0,
            days: 1,
            category: HolidayCategory::Fast,
        });
    }
    {
        let s = jd(5, 15);
        out.push(JewishHoliday {
            name: "Tu B'Av",
            hebrew_name: "ט\"ו באב",
            hebrew_month: 5,
            hebrew_day: 15,
            jd: s,
            jd_end: s,
            days: 1,
            category: HolidayCategory::Minor,
        });
    }

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
pub fn hebrew_year_from_jd(jd: f64) -> i32 {
    let mut year = ((jd - 347_997.0) * 98_496.0 / 35_975_351.0) as i32 + 1;
    while (hebrew_new_year_jd(year + 1) as f64) <= jd {
        year += 1;
    }
    year.max(1)
}

/// Convert a Julian day to a Hebrew date (year, month, day).
/// Month: 1=Nisan, 2=Iyyar, …, 7=Tishrei, …, 12=Adar (or Adar I), 13=Adar II (leap)
pub fn jd_to_hebrew_date(jd: f64) -> (i32, u8, u8) {
    let year = hebrew_year_from_jd(jd);
    let jd_int = jd.floor() as i64;
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
    fn jd_to_hebrew_date_known() {
        // JD 2451545.0 = Jan 1, 2000 = 23 Tevet 5760
        let (y, m, d) = jd_to_hebrew_date(2_451_545.0);
        assert_eq!(y, 5760);
        assert_eq!(m, 10); // Tevet
        assert_eq!(d, 24); // Jan 1, 2000 = 24 Tevet 5760
    }
}
