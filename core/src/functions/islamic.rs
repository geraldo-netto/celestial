//! Islamic (Hijri) calendar and major observances.
//!
//! The Hijri calendar is a purely lunar calendar of 12 months, each beginning
//! with the sighting of the crescent moon. The **tabular** (calculated) Hijri
//! calendar is used here, which approximates the observed calendar within 1–2 days.
//!
//! # Month numbering
//! 1=Muharram, 2=Safar, 3=Rabi al-Awwal, 4=Rabi al-Thani, 5=Jumada al-Awwal,
//! 6=Jumada al-Thani, 7=Rajab, 8=Sha'ban, 9=Ramadan, 10=Shawwal,
//! 11=Dhu al-Qi'dah, 12=Dhu al-Hijjah
//!
//! # Examples
//! ```
//! use celestial_core::{hijri_from_jd, JulianDay};
//! let (y, m, d) = hijri_from_jd(JulianDay::new(2451545.0)); // J2000 = Jan 1, 2000
//! assert_eq!(y, 1420);
//! assert_eq!(m, 9); // Ramadan
//! ```

use crate::body::Calendar;
use crate::units::JulianDay;

/// Islamic Hijri epoch: 1 Muharram 1 AH = July 16, 622 CE = JD 1948439.5
pub const HIJRI_EPOCH: f64 = 1_948_439.5;

/// Month names in English.
pub const HIJRI_MONTH_NAMES: [&str; 12] = [
    "Muharram",
    "Safar",
    "Rabi al-Awwal",
    "Rabi al-Thani",
    "Jumada al-Awwal",
    "Jumada al-Thani",
    "Rajab",
    "Sha'ban",
    "Ramadan",
    "Shawwal",
    "Dhu al-Qi'dah",
    "Dhu al-Hijjah",
];

/// Month names in Arabic.
pub const HIJRI_MONTH_NAMES_AR: [&str; 12] = [
    "محرم",
    "صفر",
    "ربيع الأول",
    "ربيع الثاني",
    "جمادى الأولى",
    "جمادى الآخرة",
    "رجب",
    "شعبان",
    "رمضان",
    "شوال",
    "ذو القعدة",
    "ذو الحجة",
];

/// Is the given Hijri year a leap year? (Leap years have 355 days.)
/// Uses the common 30-year tabular leap cycle.
#[must_use]
pub fn is_hijri_leap_year(year: i32) -> bool {
    // i64 multiplication avoids overflow at year ≈ i32::MAX/11.
    (11_i64 * i64::from(year) + 14) % 30 < 11
}

/// Number of days in a Hijri month.
#[must_use]
pub fn hijri_month_days(year: i32, month: u8) -> u8 {
    if month % 2 == 1 || (month == 12 && is_hijri_leap_year(year)) {
        30
    } else {
        29
    }
}

/// Julian day of 1 Muharram of the given Hijri year.
#[must_use]
pub fn hijri_new_year_jd(year: i32) -> f64 {
    let y = f64::from(year);
    HIJRI_EPOCH + (y - 1.0) * 354.0 + ((11.0 * y + 3.0) / 30.0).floor()
}

/// Julian day of the first day of a given Hijri month.
#[must_use]
pub fn hijri_month_start_jd(year: i32, month: u8) -> f64 {
    let mut jd = hijri_new_year_jd(year);
    for m in 1u8..month {
        jd += hijri_month_days(year, m) as f64;
    }
    jd
}

/// Convert a Julian day to a Hijri date (year, month, day).
///
/// Returns `(1, 1, 1)` for non-finite input or dates before the Hijri epoch.
#[must_use]
pub fn hijri_from_jd(jd: JulianDay) -> (i32, u8, u8) {
    let jd: f64 = jd.into();
    if !jd.is_finite() {
        return (1, 1, 1);
    }
    let jd = jd.max(HIJRI_EPOCH);
    let year = ((30.0 * (jd - HIJRI_EPOCH) + 10_646.0) / 10_631.0).floor();
    if year > f64::from(i32::MAX) {
        return (1, 1, 1);
    }
    let year = year as i32;

    let mut month = 1u8;
    let mut remaining = (jd - hijri_new_year_jd(year)).floor() as i32;
    for candidate in 1u8..12 {
        let month_days = hijri_month_days(year, candidate) as i32;
        if remaining < month_days {
            break;
        }
        remaining -= month_days;
        month += 1;
    }
    let day = (remaining + 1) as u8;
    (year, month, day)
}

/// Convert a Hijri date to a Julian day.
#[must_use]
pub fn hijri_to_jd(year: i32, month: u8, day: u8) -> f64 {
    hijri_month_start_jd(year, month) + (day as f64 - 1.0)
}

/// Name of a Hijri month (1–12).
#[must_use]
pub fn hijri_month_name(month: u8) -> &'static str {
    if (1..=12).contains(&month) {
        HIJRI_MONTH_NAMES[(month - 1) as usize]
    } else {
        "Unknown"
    }
}

/// An Islamic observance with its Hijri date and Julian day.
#[derive(Debug, Clone)]
pub struct IslamicObservance {
    /// English name.
    pub name: &'static str,
    /// Arabic name.
    pub arabic_name: &'static str,
    /// Hijri month (1–12).
    pub hijri_month: u8,
    /// Hijri day.
    pub hijri_day: u8,
    /// Julian day at sunset (start of the Islamic day).
    pub jd: f64,
    /// Duration in days.
    pub days: u8,
}

/// Return all major Islamic observances for the given Hijri year.
#[must_use]
pub fn islamic_observances(hijri_year: i32) -> Vec<IslamicObservance> {
    let jd_start = |m: u8, d: u8| hijri_to_jd(hijri_year, m, d) + 0.25; // ~sunset

    vec![
        IslamicObservance {
            name: "Islamic New Year (Hijri New Year)",
            arabic_name: "رأس السنة الهجرية",
            hijri_month: 1,
            hijri_day: 1,
            jd: jd_start(1, 1),
            days: 1,
        },
        IslamicObservance {
            name: "Ashura",
            arabic_name: "عاشوراء",
            hijri_month: 1,
            hijri_day: 10,
            jd: jd_start(1, 10),
            days: 1,
        },
        IslamicObservance {
            name: "Mawlid al-Nabi (Prophet's Birthday)",
            arabic_name: "المولد النبوي",
            hijri_month: 3,
            hijri_day: 12,
            jd: jd_start(3, 12),
            days: 1,
        },
        IslamicObservance {
            name: "Isra and Mi'raj (Night Journey)",
            arabic_name: "الإسراء والمعراج",
            hijri_month: 7,
            hijri_day: 27,
            jd: jd_start(7, 27),
            days: 1,
        },
        IslamicObservance {
            name: "Laylat al-Bara'at (Night of Forgiveness)",
            arabic_name: "ليلة البراءة",
            hijri_month: 8,
            hijri_day: 15,
            jd: jd_start(8, 15),
            days: 1,
        },
        IslamicObservance {
            name: "Ramadan (start)",
            arabic_name: "رمضان",
            hijri_month: 9,
            hijri_day: 1,
            jd: jd_start(9, 1),
            days: 29,
        },
        IslamicObservance {
            name: "Laylat al-Qadr (Night of Power)",
            arabic_name: "ليلة القدر",
            hijri_month: 9,
            hijri_day: 27,
            jd: jd_start(9, 27),
            days: 1,
        },
        IslamicObservance {
            name: "Eid al-Fitr",
            arabic_name: "عيد الفطر",
            hijri_month: 10,
            hijri_day: 1,
            jd: jd_start(10, 1),
            days: 3,
        },
        IslamicObservance {
            name: "Day of Arafah",
            arabic_name: "يوم عرفة",
            hijri_month: 12,
            hijri_day: 9,
            jd: jd_start(12, 9),
            days: 1,
        },
        IslamicObservance {
            name: "Eid al-Adha",
            arabic_name: "عيد الأضحى",
            hijri_month: 12,
            hijri_day: 10,
            jd: jd_start(12, 10),
            days: 4,
        },
    ]
}

/// Return the Islamic observances for the Hijri year that contains the given JD.
#[must_use]
pub fn islamic_observances_for_jd(jd: JulianDay) -> Vec<IslamicObservance> {
    let jd: f64 = jd.into();
    let (year, _, _) = hijri_from_jd(JulianDay::new(jd));
    islamic_observances(year)
}

/// Gregorian year → Hijri years that overlap it (usually 2).
#[must_use]
pub fn gregorian_to_hijri_years(gregorian_year: i32) -> (i32, i32) {
    let jan1 = crate::functions::time::julday(gregorian_year, 1, 1, 0.0, Calendar::Gregorian);
    let dec31 = crate::functions::time::julday(gregorian_year, 12, 31, 0.0, Calendar::Gregorian);
    let (y1, _, _) = hijri_from_jd(JulianDay::new(jan1));
    let (y2, _, _) = hijri_from_jd(JulianDay::new(dec31));
    (y1, y2)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const LEAP_YEARS_IN_CYCLE: [i32; 11] = [2, 5, 7, 10, 13, 16, 18, 21, 24, 26, 29];

    #[test]
    fn hijri_from_jd_j2000() {
        // JD 2451545.0 = Jan 1, 2000 CE = 24 Ramadan 1420 AH
        let (y, m, d) = hijri_from_jd(JulianDay::new(2_451_545.0));
        assert_eq!(y, 1420);
        assert_eq!(m, 9); // Ramadan
        assert_eq!(d, 24);
    }

    #[test]
    fn hijri_to_jd_roundtrip() {
        let dates = [
            (1, 1, 1),
            (1, 12, 29),
            (2, 12, 30),
            (30, 6, 15),
            (31, 1, 1),
            (1446, 9, 1),
        ];
        for date in dates {
            let jd = hijri_to_jd(date.0, date.1, date.2);
            assert_eq!(hijri_from_jd(JulianDay::new(jd)), date);
        }
    }

    #[test]
    fn hijri_epoch_and_year_boundaries_are_exact() {
        assert_eq!(hijri_new_year_jd(1), HIJRI_EPOCH);
        assert_eq!(hijri_new_year_jd(31), HIJRI_EPOCH + 10_631.0);
        for year in 1..=30 {
            let expected = if LEAP_YEARS_IN_CYCLE.contains(&year) {
                355.0
            } else {
                354.0
            };
            assert_eq!(
                hijri_new_year_jd(year + 1) - hijri_new_year_jd(year),
                expected
            );
        }
    }

    #[test]
    fn hijri_from_jd_handles_supported_limits() {
        for jd in [f64::NAN, f64::INFINITY, HIJRI_EPOCH - 0.25] {
            assert_eq!(hijri_from_jd(JulianDay::new(jd)), (1, 1, 1));
        }
        assert_eq!(hijri_from_jd(JulianDay::new(HIJRI_EPOCH)), (1, 1, 1));

        let max_start = hijri_new_year_jd(i32::MAX);
        assert_eq!(hijri_from_jd(JulianDay::new(max_start)), (i32::MAX, 1, 1));
        assert_eq!(hijri_from_jd(JulianDay::new(max_start + 355.0)), (1, 1, 1));
    }

    #[test]
    fn hijri_month_lengths_cover_common_and_leap_years() {
        let common: Vec<_> = (1..=12).map(|month| hijri_month_days(1, month)).collect();
        assert_eq!(common, [30, 29, 30, 29, 30, 29, 30, 29, 30, 29, 30, 29]);
        assert_eq!(hijri_month_days(2, 12), 30);
    }

    #[test]
    fn hijri_month_names_cover_valid_and_invalid_numbers() {
        for (index, expected) in HIJRI_MONTH_NAMES.iter().enumerate() {
            assert_eq!(hijri_month_name(index as u8 + 1), *expected);
        }
        assert_eq!(hijri_month_name(0), "Unknown");
        assert_eq!(hijri_month_name(13), "Unknown");
    }

    #[test]
    fn hijri_day_offsets_are_exact() {
        assert_eq!(hijri_to_jd(1, 1, 2), HIJRI_EPOCH + 1.0);
        assert_eq!(hijri_month_start_jd(1, 12), HIJRI_EPOCH + 325.0);
    }

    #[test]
    fn ramadan_1446_gregorian() {
        // Ramadan 1446 AH starts ~March 1, 2025 CE
        let jd = hijri_to_jd(1446, 9, 1);
        // JD of March 1, 2025 ≈ 2460735.5
        assert!((jd - 2_460_735.5).abs() < 3.0, "Ramadan 1446 JD: {jd}");
    }

    #[test]
    fn eid_al_fitr_1446() {
        // Eid al-Fitr 1446 ≈ March 30, 2025
        let jd = hijri_to_jd(1446, 10, 1);
        assert!((jd - 2_460_764.5).abs() < 3.0, "Eid al-Fitr 1446 JD: {jd}");
    }

    #[test]
    fn observances_count() {
        let obs = islamic_observances(1446);
        assert_eq!(obs.len(), 10);
        let ramadan = &obs[5];
        assert_eq!(ramadan.name, "Ramadan (start)");
        assert_eq!(ramadan.arabic_name, "رمضان");
        assert_eq!((ramadan.hijri_month, ramadan.hijri_day), (9, 1));
        assert_eq!(ramadan.jd, hijri_to_jd(1446, 9, 1) + 0.25);
        assert_eq!(ramadan.days, 29);

        let from_jd = islamic_observances_for_jd(JulianDay::new(hijri_to_jd(1446, 6, 1)));
        assert_eq!(from_jd.len(), 10);
        assert_eq!(from_jd[0].hijri_month, 1);
        assert_eq!(from_jd[0].jd, hijri_to_jd(1446, 1, 1) + 0.25);
    }

    #[test]
    fn gregorian_to_hijri_2025() {
        let (y1, y2) = gregorian_to_hijri_years(2025);
        assert_eq!(y1, 1446);
        assert_eq!(y2, 1447);
    }

    #[test]
    fn leap_year_detection() {
        for year in 1..=30 {
            assert_eq!(
                is_hijri_leap_year(year),
                LEAP_YEARS_IN_CYCLE.contains(&year)
            );
        }
    }
}
