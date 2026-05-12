//! Coptic and Ethiopic calendars.
//!
//! Two closely related Christian calendars sharing the same structure:
//! 12 months of 30 days plus a short 13th month of 5 or 6 epagomenal days.
//!
//! Leap year rule: `year mod 4 == 3`.
//!
//! - **Coptic** (Anno Martyrum, AM): epoch 29 Aug 284 Julian AD = JD 1 825 029.5
//!   — Coptic Orthodox Church.
//! - **Ethiopic** (Amätä Mihrät / Ethiopian Era, EE): epoch 29 Aug 8 Julian AD
//!   (276 Coptic years before AM) — Ethiopian Orthodox Church.

/// Julian Day number of 1 Thout AM 1 (Coptic Year 1, Day 1) at noon.
pub const COPTIC_EPOCH_JD: i64 = 1_825_030;

/// Julian Day number of 1 Maskaram EE 1 (Ethiopic Year 1, Day 1) at noon.
pub const ETHIOPIC_EPOCH_JD: i64 = 1_724_221;

/// The 13 Coptic month names (transliterated from Coptic).
pub const COPTIC_MONTHS: [&str; 13] = [
    "Thout",
    "Paopi",
    "Hathor",
    "Koiak",
    "Tobi",
    "Meshir",
    "Paremhat",
    "Parmouti",
    "Pashons",
    "Paoni",
    "Epip",
    "Mesori",
    "Pi Kogi Enavot",
];

/// The 13 Ethiopic month names (Ge'ez).
pub const ETHIOPIC_MONTHS: [&str; 13] = [
    "Maskaram", "Teqemt", "Hedar", "Tahsas", "Ter", "Yakatit", "Magabit", "Miyazya", "Genbot",
    "Sene", "Hamle", "Nahase", "Pagume",
];

/// True if the given Coptic/Ethiopic year is a leap year.
///
/// Leap years occur when `year mod 4 == 3` (i.e. 3, 7, 11, ... AM/EE).
#[must_use]
pub fn is_coptic_leap_year(year: i32) -> bool {
    year.rem_euclid(4) == 3
}

/// Number of days in a given Coptic/Ethiopic month (1..=13).
///
/// Months 1–12 have 30 days; month 13 (epagomenal) has 5 days
/// (or 6 in leap years). Returns 0 for invalid month numbers.
#[must_use]
pub fn coptic_month_days(year: i32, month: u32) -> u32 {
    match month {
        1..=12 => 30,
        13 => {
            if is_coptic_leap_year(year) {
                6
            } else {
                5
            }
        }
        _ => 0,
    }
}

// ── Coptic conversions ────────────────────────────────────────────────────────

/// Coptic (AM) date → Julian Day (start of civil day, JD .5).
#[must_use]
pub fn coptic_to_jd(year: i32, month: u32, day: u32) -> f64 {
    coptic_like_to_jd(COPTIC_EPOCH_JD, year, month, day)
}

/// Julian Day → Coptic (AM) date `(year, month, day)`.
#[must_use]
pub fn jd_to_coptic(jd: f64) -> (i32, u32, u32) {
    jd_to_coptic_like(COPTIC_EPOCH_JD, jd)
}

// ── Ethiopic conversions ──────────────────────────────────────────────────────

/// Ethiopic (EE) date → Julian Day (start of civil day).
#[must_use]
pub fn ethiopic_to_jd(year: i32, month: u32, day: u32) -> f64 {
    coptic_like_to_jd(ETHIOPIC_EPOCH_JD, year, month, day)
}

/// Julian Day → Ethiopic (EE) date `(year, month, day)`.
#[must_use]
pub fn jd_to_ethiopic(jd: f64) -> (i32, u32, u32) {
    jd_to_coptic_like(ETHIOPIC_EPOCH_JD, jd)
}

// ── Shared core arithmetic ────────────────────────────────────────────────────

fn coptic_like_to_jd(epoch: i64, year: i32, month: u32, day: u32) -> f64 {
    let n = epoch - 1
        + 365 * (year - 1) as i64
        + (year as i64).div_euclid(4)
        + 30 * (month - 1) as i64
        + day as i64;
    n as f64 - 0.5
}

fn jd_to_coptic_like(epoch: i64, jd: f64) -> (i32, u32, u32) {
    let n = (jd + 0.5).floor() as i64 - epoch + 1;
    let year = ((4 * n + 1463).div_euclid(1461)) as i32;
    let days_prev = 365 * (year - 1) as i64 + (year as i64).div_euclid(4);
    let d_in_year = n - days_prev;
    let month = (((d_in_year - 1) / 30) + 1) as u32;
    let day = (d_in_year - 30 * (month - 1) as i64) as u32;
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coptic_epoch_roundtrip() {
        // 1 Thout AM 1 = JD 1825029.5
        let jd = coptic_to_jd(1, 1, 1);
        assert!((jd - 1_825_029.5).abs() < 1e-6, "got jd = {jd}");
        assert_eq!(jd_to_coptic(jd), (1, 1, 1));
    }

    #[test]
    fn coptic_leap_year_rule() {
        assert!(is_coptic_leap_year(3));
        assert!(!is_coptic_leap_year(4));
        assert!(is_coptic_leap_year(1739));
        assert_eq!(coptic_month_days(3, 13), 6);
        assert_eq!(coptic_month_days(4, 13), 5);
    }

    #[test]
    fn coptic_invalid_month_returns_zero() {
        assert_eq!(coptic_month_days(1, 0), 0);
        assert_eq!(coptic_month_days(1, 14), 0);
    }

    #[test]
    fn coptic_month_count() {
        assert_eq!(COPTIC_MONTHS.len(), 13);
        assert_eq!(ETHIOPIC_MONTHS.len(), 13);
    }

    #[test]
    fn ethiopic_epoch_roundtrip() {
        // 1 Maskaram EE 1 = JD 1724220.5
        let jd = ethiopic_to_jd(1, 1, 1);
        assert!((jd - 1_724_220.5).abs() < 1e-6, "got jd = {jd}");
        assert_eq!(jd_to_ethiopic(jd), (1, 1, 1));
    }

    #[test]
    fn coptic_within_month_arithmetic() {
        // 15 Thout AM 1 should be 14 days after 1 Thout AM 1
        let a = coptic_to_jd(1, 1, 1);
        let b = coptic_to_jd(1, 1, 15);
        assert!((b - a - 14.0).abs() < 1e-6);
    }

    #[test]
    fn coptic_month_boundary() {
        // 30 Thout + 1 day = 1 Paopi
        let (y, m, d) = jd_to_coptic(coptic_to_jd(1, 1, 30) + 1.0);
        assert_eq!((y, m, d), (1, 2, 1));
    }
}
