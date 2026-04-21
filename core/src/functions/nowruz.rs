//! Nowruz (Persian New Year) and the Bahá'í calendar.
//!
//! **Nowruz** ("New Day") is the Persian/Iranian New Year, celebrated at the
//! exact moment of the vernal equinox (Sun at 0° tropical longitude).
//! It is observed by Iranians, Afghans, Kurds, and many Central Asian peoples.
//!
//! **The Bahá'í calendar** (Badí' calendar) consists of 19 months of 19 days
//! (361 days) plus 4–5 intercalary days (Ayyám-i-Há). The Bahá'í New Year
//! (Naw-Rúz) coincides with the vernal equinox.
//!
//! # Examples
//! ```
//! # use celestial_core::body::Calendar;
//! use celestial_core::{nowruz_jd, julday};
//! let jd = nowruz_jd(2025);
//! // Nowruz 2025 ≈ March 20, 2025
//! let d = celestial_core::revjul(jd, Calendar::Gregorian);
//! assert_eq!(d.month, 3);
//! ```

use crate::body::{CalcFlags, Calendar};
use crate::solcross_ut;
use crate::{julday, revjul};

/// Julian day of Nowruz (vernal equinox) for the given Gregorian year.
///
/// Uses `solcross_ut` to find the exact moment the Sun reaches 0° ecliptic longitude.
pub fn nowruz_jd(year: i32) -> f64 {
    let start = julday(year, 3, 15, 0.0, Calendar::Gregorian);
    solcross_ut(0.0, start, CalcFlags::BUILTIN).unwrap_or(start)
}

/// Convert a Gregorian year to the corresponding Iranian solar (Solar Hijri) year.
///
/// The Solar Hijri calendar starts at Nowruz 622 CE.
pub fn gregorian_to_solar_hijri(gregorian_year: i32) -> i32 {
    // The Solar Hijri year starts at Nowruz
    // Year 1 SH = 622 CE (approximately)
    gregorian_year - 621
}

/// Convert a Solar Hijri year to the Gregorian year (approximate).
pub fn solar_hijri_to_gregorian(solar_hijri_year: i32) -> i32 {
    solar_hijri_year + 621
}

/// A Persian (Solar Hijri) month.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PersianMonth {
    /// Month number (1–12).
    pub month: u8,
    /// Persian name.
    pub name: &'static str,
    /// Season.
    pub season: &'static str,
    /// Number of days.
    pub days: u8,
}

/// The 12 months of the Solar Hijri (Persian) calendar.
#[allow(dead_code)]
pub const PERSIAN_MONTHS: [(&str, &str, u8); 12] = [
    ("Farvardin", "Spring", 31),
    ("Ordibehesht", "Spring", 31),
    ("Khordad", "Spring", 31),
    ("Tir", "Summer", 31),
    ("Mordad", "Summer", 31),
    ("Shahrivar", "Summer", 31),
    ("Mehr", "Autumn", 30),
    ("Aban", "Autumn", 30),
    ("Azar", "Autumn", 30),
    ("Dey", "Winter", 29), // 30 in leap year
    ("Bahman", "Winter", 29),
    ("Esfand", "Winter", 29),
];

// ─── Bahá'í calendar ─────────────────────────────────────────────────────────

/// Epoch of the Bahá'í calendar: Naw-Rúz 1 BE = March 21, 1844 CE.
/// JD of March 21, 1844 ≈ 2394646.5
#[allow(dead_code)]
pub const BAHAI_EPOCH_JD: f64 = 2_394_646.5;

/// The 19 months of the Bahá'í (Badí') calendar.
pub const BAHAI_MONTHS: [(&str, &str); 19] = [
    ("Bahá", "Splendour"),     // 1
    ("Jalál", "Glory"),        // 2
    ("Jamál", "Beauty"),       // 3
    ("'Aẓamat", "Grandeur"),   // 4
    ("Núr", "Light"),          // 5
    ("Raḥmat", "Mercy"),       // 6
    ("Kalimát", "Words"),      // 7
    ("Kamál", "Perfection"),   // 8
    ("Asmá'", "Names"),        // 9
    ("'Izzat", "Might"),       // 10
    ("Mashíyyat", "Will"),     // 11
    ("'Ilm", "Knowledge"),     // 12
    ("Qudrat", "Power"),       // 13
    ("Qawl", "Speech"),        // 14
    ("Masá'il", "Questions"),  // 15
    ("Sharaf", "Honour"),      // 16
    ("Sulṭán", "Sovereignty"), // 17
    ("Mulk", "Dominion"),      // 18
    ("'Alá'", "Loftiness"),    // 19
];

/// Intercalary days preceding the 19th month ('Alá').
pub const AYYAM_I_HA: (&str, &str) = ("Ayyám-i-Há", "Days of Há");

/// A Bahá'í date.
#[derive(Debug, Clone)]
pub struct BahaiDate {
    /// Bahá'í Era year (1 BE = 1844 CE).
    pub year: i32,
    /// Month number (1–19), or 0 for Ayyám-i-Há.
    pub month: u8,
    /// Day within the month (1–19), or day of Ayyám-i-Há (1–4/5).
    pub day: u8,
    /// Month name.
    pub month_name: &'static str,
    /// Julian day.
    pub jd: f64,
}

/// Is the given Bahá'í year a leap year?
/// Bahá'í leap years correspond to Gregorian leap years.
pub fn is_bahai_leap_year(bahai_year: i32) -> bool {
    let gregorian_year = bahai_year + 1843;
    gregorian_year % 4 == 0 && (gregorian_year % 100 != 0 || gregorian_year % 400 == 0)
}

/// Julian day of Naw-Rúz (Bahá'í New Year) for the given Bahá'í year.
pub fn naw_ruz_jd(bahai_year: i32) -> f64 {
    let gregorian_year = bahai_year + 1843;
    nowruz_jd(gregorian_year)
}

/// Convert a Julian day to a Bahá'í date.
pub fn jd_to_bahai(jd: f64) -> BahaiDate {
    // Find the Bahá'í year
    let gregorian_year = {
        let d = revjul(jd, Calendar::Gregorian);
        d.year
    };
    let mut bahai_year = gregorian_year - 1843;

    // Adjust: if before Naw-Ruz, it's the previous Bahá'í year
    if jd < naw_ruz_jd(bahai_year) {
        bahai_year -= 1;
    }

    let year_start = naw_ruz_jd(bahai_year);
    let day_of_year = (jd - year_start).floor() as u32;

    // Months 1-18 have 19 days each (342 days), then Ayyám-i-Há (4/5 days), then month 19
    let ayyam_days = if is_bahai_leap_year(bahai_year) {
        5u32
    } else {
        4
    };

    if day_of_year < 342 {
        let month = (day_of_year / 19 + 1) as u8;
        let day = (day_of_year % 19 + 1) as u8;
        BahaiDate {
            year: bahai_year,
            month,
            day,
            month_name: BAHAI_MONTHS[(month - 1) as usize].0,
            jd,
        }
    } else if day_of_year < 342 + ayyam_days {
        let day = (day_of_year - 342 + 1) as u8;
        BahaiDate {
            year: bahai_year,
            month: 0, // Ayyám-i-Há
            day,
            month_name: AYYAM_I_HA.0,
            jd,
        }
    } else {
        let day = (day_of_year - 342 - ayyam_days + 1) as u8;
        BahaiDate {
            year: bahai_year,
            month: 19,
            day,
            month_name: BAHAI_MONTHS[18].0,
            jd,
        }
    }
}

/// A Bahá'í holy day.
#[derive(Debug, Clone)]
pub struct BahaiHolyDay {
    pub name: &'static str,
    pub description: &'static str,
    /// Bahá'í month (1–19, 0=Ayyám-i-Há).
    pub bahai_month: u8,
    /// Day within the month.
    pub bahai_day: u8,
    /// Julian day.
    pub jd: f64,
}

/// Return all Bahá'í holy days for the given Bahá'í year.
pub fn bahai_holy_days(bahai_year: i32) -> Vec<BahaiHolyDay> {
    let year_start = naw_ruz_jd(bahai_year);
    let ayyam_days = if is_bahai_leap_year(bahai_year) {
        5u32
    } else {
        4
    };

    // Compute JD of a Bahá'í month/day
    let jd_of = |month: u8, day: u8| -> f64 {
        if month == 0 {
            year_start + 342.0 + (day as f64 - 1.0)
        } else if month <= 18 {
            year_start + (month as f64 - 1.0) * 19.0 + (day as f64 - 1.0)
        } else {
            year_start + 342.0 + ayyam_days as f64 + (day as f64 - 1.0)
        }
    };

    vec![
        BahaiHolyDay {
            name: "Naw-Rúz (Bahá'í New Year)",
            description: "First day of the Bahá'í year, coincides with vernal equinox",
            bahai_month: 1,
            bahai_day: 1,
            jd: jd_of(1, 1),
        },
        BahaiHolyDay {
            name: "First Day of Riḍván",
            description: "Declaration of Bahá'u'lláh, most holy Bahá'í festival (12 days)",
            bahai_month: 2,
            bahai_day: 13,
            jd: jd_of(2, 13),
        },
        BahaiHolyDay {
            name: "Ninth Day of Riḍván",
            description: "Bahá'u'lláh's family joins in the Garden of Riḍván",
            bahai_month: 3,
            bahai_day: 2,
            jd: jd_of(3, 2),
        },
        BahaiHolyDay {
            name: "Twelfth Day of Riḍván",
            description: "Conclusion of the Riḍván festival",
            bahai_month: 3,
            bahai_day: 5,
            jd: jd_of(3, 5),
        },
        BahaiHolyDay {
            name: "Declaration of the Báb",
            description: "Anniversary of the Báb's declaration (May 23, 1844)",
            bahai_month: 4,
            bahai_day: 8,
            jd: jd_of(4, 8),
        },
        BahaiHolyDay {
            name: "Ascension of Bahá'u'lláh",
            description: "Passing of Bahá'u'lláh (May 29, 1892)",
            bahai_month: 4,
            bahai_day: 14,
            jd: jd_of(4, 14),
        },
        BahaiHolyDay {
            name: "Martyrdom of the Báb",
            description: "Execution of the Báb (July 9, 1850)",
            bahai_month: 6,
            bahai_day: 17,
            jd: jd_of(6, 17),
        },
        BahaiHolyDay {
            name: "Birth of the Báb",
            description: "Anniversary of the Báb's birth (Oct 20, 1819)",
            bahai_month: 13,
            bahai_day: 1,
            jd: jd_of(13, 1),
        },
        BahaiHolyDay {
            name: "Birth of Bahá'u'lláh",
            description: "Anniversary of Bahá'u'lláh's birth (Nov 12, 1817)",
            bahai_month: 13,
            bahai_day: 2,
            jd: jd_of(13, 2),
        },
        BahaiHolyDay {
            name: "Ayyám-i-Há (Intercalary Days)",
            description: "Days of gift-giving and charitable deeds before the Fast",
            bahai_month: 0,
            bahai_day: 1,
            jd: jd_of(0, 1),
        },
        BahaiHolyDay {
            name: "Fast of 'Alá' begins",
            description: "19-day period of fasting (sunrise to sunset)",
            bahai_month: 19,
            bahai_day: 1,
            jd: jd_of(19, 1),
        },
        BahaiHolyDay {
            name: "Day of the Covenant",
            description: "Celebration of 'Abdu'l-Bahá as Centre of the Covenant",
            bahai_month: 14,
            bahai_day: 4,
            jd: jd_of(14, 4),
        },
        BahaiHolyDay {
            name: "Ascension of 'Abdu'l-Bahá",
            description: "Passing of 'Abdu'l-Bahá (Nov 28, 1921)",
            bahai_month: 14,
            bahai_day: 6,
            jd: jd_of(14, 6),
        },
    ]
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nowruz_2025_in_march() {
        let jd = nowruz_jd(2025);
        let d = revjul(jd, Calendar::Gregorian);
        assert_eq!(d.year, 2025);
        assert_eq!(d.month, 3);
        assert!(d.day == 20 || d.day == 21, "day={}", d.day);
    }

    #[test]
    fn bahai_year_from_jd() {
        // Jan 1, 2025 should be in Bahá'í year 181
        let jd = julday(2025, 1, 1, 12.0, Calendar::Gregorian);
        let bd = jd_to_bahai(jd);
        assert_eq!(bd.year, 181, "year={}", bd.year);
    }

    #[test]
    fn naw_ruz_2025() {
        // Naw-Rúz 182 BE = Nowruz 2025 ≈ March 20, 2025
        let jd = naw_ruz_jd(182);
        let d = revjul(jd, Calendar::Gregorian);
        assert_eq!(d.month, 3);
        assert!(d.day == 20 || d.day == 21);
    }

    #[test]
    fn bahai_holy_days_count() {
        let holy_days = bahai_holy_days(182);
        assert!(holy_days.len() >= 12);
    }

    #[test]
    fn solar_hijri_conversion() {
        assert_eq!(gregorian_to_solar_hijri(2025), 1404);
        assert_eq!(solar_hijri_to_gregorian(1404), 2025);
    }

    #[test]
    fn bahai_months_count() {
        assert_eq!(BAHAI_MONTHS.len(), 19);
    }
}
