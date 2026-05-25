//! Zoroastrian Fasli (seasonal) calendar.
//!
//! Reformed in 1906 to lock the Zoroastrian New Year (Nowruz) to the
//! astronomical vernal equinox. Structure:
//!
//! - 12 months of 30 days + 5 Gatha (epagomenal) days = 365 days
//! - No calendar-internal leap year rule; the calendar inherits the drift
//!   between successive astronomical equinoxes.
//!
//! This module depends on the engine's [`solcross_ut`] for equinox
//! computation, so it is only available with the default features.

use crate::body::CalcFlags;
use crate::functions::motion::solcross_ut;
use crate::functions::time::{julday, revjul};
use crate::units::{JulianDay, Longitude};

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
    solcross_ut(Longitude::new(0.0), JulianDay::new(jd_approx), CalcFlags::BUILTIN).ok()
}

/// Convert a Julian Day to a Fasli date: `(fasli_year, month_index, day)`.
///
/// - `fasli_year`: 1 corresponds to the reform year, 21 March 1906 CE.
/// - `month_index`: 1..=12 for named months ([`FASLI_MONTHS`]);
///   13 for the Gatha days ([`GATHA_DAYS`]).
/// - `day`: 1..=30 for months 1–12; 1..=5 for month 13.
///
/// Returns `None` if the JD predates the 1906 reform or if the equinox
/// computation fails.
pub fn jd_to_fasli(jd: JulianDay) -> Option<(i32, u8, u8)> {
    let jd: f64 = jd.into();
    let d = revjul(JulianDay::new(jd), crate::body::Calendar::Gregorian);
    if d.year < 1906 {
        return None;
    }

    // Pick Nowruz in this or previous Gregorian year depending on position
    let nowruz_this = fasli_nowruz_jd(d.year)?;
    let (fasli_year_greg, nowruz) = if jd >= nowruz_this {
        (d.year, nowruz_this)
    } else {
        (d.year - 1, fasli_nowruz_jd(d.year - 1)?)
    };

    let days_in = (jd.floor() - nowruz.floor()) as i32;
    let fasli_year = fasli_year_greg - 1905; // CE 1906 = Fasli year 1

    if (0..360).contains(&days_in) {
        Some((
            fasli_year,
            (days_in / 30) as u8 + 1,
            (days_in % 30) as u8 + 1,
        ))
    } else if (360..365).contains(&days_in) {
        // Gatha days: month 13
        Some((fasli_year, 13, (days_in - 359) as u8))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fasli_month_counts() {
        assert_eq!(FASLI_MONTHS.len(), 12);
        assert_eq!(GATHA_DAYS.len(), 5);
    }

    #[test]
    fn fasli_before_1906_returns_none() {
        let jd = julday(1905, 3, 21, 0.0, crate::body::Calendar::Gregorian);
        assert!(jd_to_fasli(JulianDay::new(jd)).is_none());
    }

    #[test]
    fn fasli_nowruz_march_equinox() {
        // Nowruz should always fall in late March
        if let Some(jd) = fasli_nowruz_jd(2024) {
            let d = revjul(JulianDay::new(jd), crate::body::Calendar::Gregorian);
            assert_eq!(d.year, 2024);
            assert_eq!(d.month, 3);
            assert!(
                (19..=21).contains(&d.day),
                "Nowruz 2024 should be March 19-21, got day {}",
                d.day
            );
        }
    }

    #[test]
    fn fasli_year_1_is_1906() {
        // Shortly after Nowruz 1906 = Fasli year 1, month 1
        if let Some(nowruz_1906) = fasli_nowruz_jd(1906) {
            let (fy, m, _d) = jd_to_fasli(JulianDay::new(nowruz_1906 + 0.5)).unwrap();
            assert_eq!(fy, 1);
            assert_eq!(m, 1);
        }
    }
}
