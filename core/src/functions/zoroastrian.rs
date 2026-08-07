//! Zoroastrian Fasli (seasonal) calendar.
//!
//! Reformed in 1906 to lock the Zoroastrian New Year (Nowruz) to the
//! astronomical vernal equinox. Structure:
//!
//! - 12 months of 30 days + 5 Gatha (epagomenal) days = 365 days
//! - A sixth intercalary day is inserted when successive equinox dates span
//!   366 days.
//!
//! This module depends on the engine's [`solcross_ut`] for equinox
//! computation, so it is only available with the default features.

use crate::body::CalcFlags;
use crate::functions::motion::solcross_ut;
use crate::functions::time::{julday, revjul};
use crate::units::{JulianDay, Longitude};

fn utc_day_number(jd: f64) -> i64 {
    jd.round() as i64
}

/// The 12 Fasli months, named after Yazatas (divinities).
pub const FASLI_MONTHS: [&str; 12] = [
    "Farvardin",
    "Ardibehesht",
    "Khordad",
    "Tir",
    "Amardad",
    "Shahrivar",
    "Mehr",
    "Aban",
    "Azar",
    "Dae",
    "Bahman",
    "Spenta",
];

/// The 5 Gatha days (epagomenae), appended after Spenta.
pub const GATHA_DAYS: [&str; 5] = [
    "Ahunavad",
    "Ushtavad",
    "Spentomad",
    "Vohukhshatra",
    "Vahishto-Ishti",
];

/// Julian Day of Fasli Nowruz (astronomical vernal equinox, Sun at 0° Aries)
/// for a given Gregorian year.
///
/// Returns `None` if the astronomy engine fails to converge (should not
/// happen for any reasonable year).
pub fn fasli_nowruz_jd(gregorian_year: i32) -> Option<f64> {
    // Search from March 19 — the equinox falls within a few days
    let jd_approx = julday(gregorian_year, 3, 19, 0.0, crate::body::Calendar::Gregorian);
    solcross_ut(
        Longitude::new(0.0),
        JulianDay::new(jd_approx),
        CalcFlags::BUILTIN,
    )
    .ok()
}

/// Convert a Julian Day to a Fasli date: `(fasli_year, month_index, day)`.
///
/// - `fasli_year`: 1 corresponds to the reform year, 21 March 1906 CE.
/// - `month_index`: 1..=12 for named months ([`FASLI_MONTHS`]);
///   13 for the Gatha days ([`GATHA_DAYS`]).
/// - `day`: 1..=30 for months 1–12; 1..=5 for month 13, or 6 in an
///   intercalary year.
///
/// Returns `None` if the JD predates the 1906 reform or if the equinox
/// computation fails.
pub fn jd_to_fasli(jd: JulianDay) -> Option<(i32, u8, u8)> {
    let jd: f64 = jd.into();
    let d = revjul(JulianDay::new(jd), crate::body::Calendar::Gregorian);
    if d.year < 1906 {
        return None;
    }

    let jd_day = utc_day_number(jd);
    let nowruz_this = utc_day_number(fasli_nowruz_jd(d.year)?);
    if d.year == 1906 && jd_day < nowruz_this {
        return None;
    }

    let (fasli_year_greg, nowruz) = if jd_day >= nowruz_this {
        (d.year, nowruz_this)
    } else {
        let nowruz_previous = utc_day_number(fasli_nowruz_jd(d.year - 1)?);
        (d.year - 1, nowruz_previous)
    };

    let days_in = (jd_day - nowruz) as i32;
    let fasli_year = fasli_year_greg - 1905;

    if (0..360).contains(&days_in) {
        Some((
            fasli_year,
            (days_in / 30) as u8 + 1,
            (days_in % 30) as u8 + 1,
        ))
    } else if days_in >= 360 {
        Some((fasli_year, 13, (days_in - 359) as u8))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_of_utc_day(jd: f64) -> f64 {
        (jd + 0.5).floor() - 0.5
    }

    fn fasli_date(year: i32, day_offset: i32) -> Option<(i32, u8, u8)> {
        let nowruz = fasli_nowruz_jd(year).expect("Nowruz should converge");
        let jd = start_of_utc_day(nowruz) + f64::from(day_offset) + 0.25;
        jd_to_fasli(JulianDay::new(jd))
    }

    #[test]
    fn fasli_month_counts() {
        assert_eq!(FASLI_MONTHS.len(), 12);
        assert_eq!(GATHA_DAYS.len(), 5);
    }

    #[test]
    fn fasli_before_1906_returns_none() {
        let dates = [(1905, 1, 1), (1905, 3, 21), (1906, 1, 1)];
        for (year, month, day) in dates {
            let jd = julday(year, month, day, 0.0, crate::body::Calendar::Gregorian);
            assert_eq!(jd_to_fasli(JulianDay::new(jd)), None);
        }
    }

    #[test]
    fn fasli_nowruz_march_equinox() {
        let jd = fasli_nowruz_jd(2024).expect("Nowruz should converge");
        let expected = julday(2024, 3, 20, 3.1, crate::body::Calendar::Gregorian);
        assert!((jd - expected).abs() < 0.02, "unexpected Nowruz JD {jd}");
    }

    #[test]
    fn fasli_year_1_is_1906() {
        let nowruz = fasli_nowruz_jd(1906).expect("Nowruz should converge");
        let start = start_of_utc_day(nowruz);
        assert_eq!(jd_to_fasli(JulianDay::new(start - 0.25)), None);
        assert_eq!(jd_to_fasli(JulianDay::new(start + 0.25)), Some((1, 1, 1)));
    }

    #[test]
    fn fasli_month_and_gatha_boundaries() {
        let cases = [
            (0, (119, 1, 1)),
            (29, (119, 1, 30)),
            (30, (119, 2, 1)),
            (359, (119, 12, 30)),
            (360, (119, 13, 1)),
            (364, (119, 13, 5)),
        ];
        for (offset, expected) in cases {
            assert_eq!(fasli_date(2024, offset), Some(expected));
        }
    }

    #[test]
    fn fasli_previous_gregorian_year_branch() {
        assert_eq!(fasli_date(2023, 300), Some((118, 11, 1)));
    }

    #[test]
    fn fasli_intercalary_day_and_year_transition() {
        let start_2023 = start_of_utc_day(fasli_nowruz_jd(2023).unwrap());
        let start_2024 = start_of_utc_day(fasli_nowruz_jd(2024).unwrap());
        assert_eq!(start_2024 - start_2023, 366.0);
        assert_eq!(fasli_date(2023, 365), Some((118, 13, 6)));
        assert_eq!(fasli_date(2023, 366), Some((119, 1, 1)));
    }

    #[test]
    fn fasli_date_does_not_roll_at_julian_noon() {
        let start = start_of_utc_day(fasli_nowruz_jd(2024).unwrap());
        let morning = jd_to_fasli(JulianDay::new(start + 0.25));
        let evening = jd_to_fasli(JulianDay::new(start + 0.75));
        assert_eq!(morning, Some((119, 1, 1)));
        assert_eq!(evening, morning);
    }
}
