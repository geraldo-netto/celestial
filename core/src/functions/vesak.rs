//! Buddhist observances — Vesak and Uposatha days.
//!
//! **Vesak** (also Vaisakha, Wesak) is the most sacred day in Theravada Buddhism,
//! commemorating the birth, enlightenment, and parinirvana of Gautama Buddha.
//! It falls on the full moon of the lunar month of Vaisakha (typically April–May).
//!
//! **Uposatha** days are the four monthly observance days corresponding to the
//! new moon, full moon, and the two quarter moons. Practitioners observe precepts
//! and meditation on these days.
//!
//! # Examples
//! ```
//! use celestial_core::{vesak_jd, julday};
//! let jd = vesak_jd(2025);
//! // Vesak 2025 ≈ May 12, 2025
//! assert!(jd > 2460000.0);
//! ```

use crate::body::{Body, CalcFlags, Calendar};
use crate::norm_deg;
use crate::units::JulianDay;
use crate::{calc_ut, julday};

/// Find the next full moon at or after `jd_start`.
///
/// Uses bisection to find when Moon–Sun elongation = 180°.
#[must_use]
pub fn next_full_moon_after(jd_start: JulianDay) -> f64 {
    find_phase(jd_start.into(), 180.0)
}

/// Find the next new moon (elongation = 0°) at or after `jd_start`.
#[must_use]
pub fn next_new_moon_after(jd_start: JulianDay) -> f64 {
    find_phase(jd_start.into(), 0.0)
}

/// Julian day of Vesak for the given Gregorian year.
///
/// Vesak = the full moon in the lunar month of Vaisakha, which falls when the
/// Sun is in Aries or early Taurus (roughly April–May). Per Theravada tradition
/// this is the full moon when the Sun is in the sidereal month of Vaisakha.
/// This implementation finds the first full moon after the Sun enters Aries
/// (vernal equinox), targeting April–May.
#[must_use]
pub fn vesak_jd(year: i32) -> f64 {
    let start = julday(year, 3, 15, 0.0, Calendar::Gregorian);
    let first = next_full_moon_after(JulianDay::new(start));
    let candidate = next_full_moon_after(JulianDay::new(first + 1.0));
    let may1 = julday(year, 5, 1, 0.0, Calendar::Gregorian);
    if candidate >= may1 {
        candidate
    } else {
        next_full_moon_after(JulianDay::new(candidate + 1.0))
    }
}

/// An Uposatha day (Buddhist lunar observance day).
#[derive(Debug, Clone)]
pub struct Uposatha {
    /// Type of Uposatha day.
    pub phase: UposathaPhase,
    /// Julian day.
    pub jd: f64,
    /// Moon–Sun elongation at this moment (degrees).
    pub elongation: f64,
}

/// The four phases of the lunar cycle observed as Uposatha days.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UposathaPhase {
    /// New moon (0°).
    NewMoon,
    /// First quarter (90°).
    FirstQuarter,
    /// Full moon (180°) — the most sacred Uposatha.
    FullMoon,
    /// Last quarter (270°).
    LastQuarter,
}

/// Has the moon's elongation crossed `target` between successive samples?
fn phase_crossed(prev: f64, cur: f64, target: f64) -> bool {
    if target == 0.0 {
        prev > 300.0 && cur < 60.0
    } else {
        prev < target && cur >= target
    }
}

/// Signed distance of `e` from `target`, handling wrap near 0°.
fn elongation_distance(e: f64, target: f64) -> f64 {
    if target == 0.0 {
        if e > 300.0 {
            e - 360.0
        } else {
            e
        }
    } else {
        e - target
    }
}

fn is_before_phase(distance: f64) -> bool {
    distance < 0.0
}

fn is_in_year(jd: f64, start: f64, end: f64) -> bool {
    jd >= start && jd < end
}

fn same_observance(first: f64, second: f64) -> bool {
    (first - second).abs() < 3.0
}

fn find_phase(jd_start: f64, target: f64) -> f64 {
    let elongation = |jd: f64| -> f64 {
        let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap_or_default();
        let moon = calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN).unwrap_or_default();
        norm_deg(moon.lon - sun.lon)
    };

    let mut jd = jd_start;
    let mut prev = elongation(jd);

    for _ in 0..70 {
        jd += 0.5;
        let cur = elongation(jd);
        if phase_crossed(prev, cur, target) {
            break;
        }
        prev = cur;
    }

    let mut lo = jd - 0.5;
    let mut hi = jd;
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if is_before_phase(elongation_distance(elongation(mid), target)) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// Return all Uposatha days in the given Gregorian year.
#[must_use]
pub fn uposatha_days(year: i32) -> Vec<Uposatha> {
    let elongation_at = |jd: f64| -> f64 {
        let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap_or_default();
        let moon = calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN).unwrap_or_default();
        norm_deg(moon.lon - sun.lon)
    };

    let year_start = julday(year, 1, 1, 0.0, Calendar::Gregorian);
    let end = julday(year + 1, 1, 1, 0.0, Calendar::Gregorian);

    let phases = [
        (0.0, UposathaPhase::NewMoon),
        (90.0, UposathaPhase::FirstQuarter),
        (180.0, UposathaPhase::FullMoon),
        (270.0, UposathaPhase::LastQuarter),
    ];

    // 4 phases × ≈12-13 lunar months × pre-reserved capacity avoids re-growth.
    let mut days = Vec::with_capacity(55);
    for cycle_index in 0..15 {
        let jd = year_start + cycle_index as f64 * 25.0;
        for (target, phase) in &phases {
            let phase_jd = find_phase(jd, *target);
            if is_in_year(phase_jd, year_start, end) {
                let elong = elongation_at(phase_jd);
                days.push(Uposatha {
                    phase: phase.clone(),
                    jd: phase_jd,
                    elongation: elong,
                });
            }
        }
    }

    days.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    days.dedup_by(|a, b| same_observance(a.jd, b.jd));
    days
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julday;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }

    #[test]
    fn phase_search_regressions() {
        let start = julday(2025, 1, 1, 0.0, Calendar::Gregorian);
        let cases = [
            (0.0, 2460705.0256086336),
            (90.0, 2460682.4979206445),
            (180.0, 2460689.43604228),
            (270.0, 2460697.3555077435),
        ];
        for (target, expected) in cases {
            assert_close(find_phase(start, target), expected);
        }
        assert_close(next_new_moon_after(JulianDay::new(start)), cases[0].1);
        assert_close(next_full_moon_after(JulianDay::new(start)), cases[2].1);
    }

    #[test]
    fn phase_crossing_boundaries() {
        let cases = [
            (301.0, 59.0, 0.0, true),
            (300.0, 59.0, 0.0, false),
            (301.0, 60.0, 0.0, false),
            (179.0, 180.0, 180.0, true),
            (180.0, 181.0, 180.0, false),
            (179.0, 179.0, 180.0, false),
        ];
        for (prev, cur, target, expected) in cases {
            assert_eq!(phase_crossed(prev, cur, target), expected);
        }
    }

    #[test]
    fn elongation_distance_boundaries() {
        let cases = [
            (350.0, 0.0, -10.0),
            (300.0, 0.0, 300.0),
            (301.0, 0.0, -59.0),
            (100.0, 90.0, 10.0),
        ];
        for (elongation, target, expected) in cases {
            assert_eq!(elongation_distance(elongation, target), expected);
        }
    }

    #[test]
    fn phase_distance_sign_boundaries() {
        assert!(is_before_phase(-1.0));
        assert!(!is_before_phase(-0.0));
        assert!(!is_before_phase(0.0));
        assert!(!is_before_phase(1.0));
    }

    #[test]
    fn year_membership_boundaries() {
        assert!(is_in_year(1.0, 1.0, 2.0));
        assert!(is_in_year(1.5, 1.0, 2.0));
        assert!(!is_in_year(2.0, 1.0, 2.0));
        assert!(!is_in_year(0.9, 1.0, 2.0));
    }

    #[test]
    fn observance_deduplication_boundaries() {
        assert!(same_observance(4.0, 1.1));
        assert!(!same_observance(4.0, 1.0));
        assert!(!same_observance(4.0, 0.9));
    }

    #[test]
    fn vesak_regressions() {
        for (year, expected) in [
            (2021, 2459360.968251149),
            (2024, 2460454.0787673993),
            (2025, 2460808.2057218654),
        ] {
            assert_close(vesak_jd(year), expected);
        }
    }

    #[test]
    fn uposatha_regressions() {
        let cases = [
            (2021, 49, 2459220.901330012, 2459575.6003538957),
            (2024, 50, 2460313.646782997, 2460675.435761745),
            (2025, 49, 2460682.4979206445, 2461037.2989227148),
        ];
        for (year, count, first, last) in cases {
            let days = uposatha_days(year);
            assert_eq!(days.len(), count);
            assert_close(days[0].jd, first);
            assert_close(days[count - 1].jd, last);
        }
    }

    #[test]
    fn uposatha_elongations_match_phases() {
        for day in uposatha_days(2025) {
            let target = match day.phase {
                UposathaPhase::NewMoon => 0.0,
                UposathaPhase::FirstQuarter => 90.0,
                UposathaPhase::FullMoon => 180.0,
                UposathaPhase::LastQuarter => 270.0,
            };
            let error = norm_deg(day.elongation - target + 180.0) - 180.0;
            assert!(error.abs() < 1e-8, "{}: {}", day.jd, day.elongation);
        }
    }

    #[test]
    fn vesak_2025_in_may() {
        let jd = vesak_jd(2025);
        let apr20 = julday(2025, 4, 20, 0.0, Calendar::Gregorian);
        let jun20 = julday(2025, 6, 20, 0.0, Calendar::Gregorian);
        assert!(
            jd > apr20 && jd < jun20,
            "Vesak 2025 JD={jd} should be Apr20-Jun20"
        );
    }

    #[test]
    fn uposatha_count_per_year() {
        let days = uposatha_days(2025);
        // ~13 lunar cycles × 4 phases = ~52, some may fall outside the year
        assert!(
            days.len() >= 45 && days.len() <= 55,
            "uposatha count={}",
            days.len()
        );
    }

    #[test]
    fn uposatha_phases_all_present() {
        let days = uposatha_days(2025);
        assert!(days.iter().any(|d| d.phase == UposathaPhase::FullMoon));
        assert!(days.iter().any(|d| d.phase == UposathaPhase::NewMoon));
        assert!(days.iter().any(|d| d.phase == UposathaPhase::FirstQuarter));
        assert!(days.iter().any(|d| d.phase == UposathaPhase::LastQuarter));
    }
}
