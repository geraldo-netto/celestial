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

use crate::units::JulianDay;
use crate::body::{Body, CalcFlags, Calendar};
use crate::norm_deg;
use crate::{calc_ut, julday};

/// Find the next full moon at or after `jd_start`.
///
/// Uses bisection to find when Moon–Sun elongation = 180°.
#[must_use]
pub fn next_full_moon_after(jd_start: JulianDay) -> f64 {
    let jd_start: f64 = jd_start.into();
    // Step forward in ~1-day steps to find the lunation
    let mut jd = jd_start;
    let elongation = |jd: f64| -> f64 {
        let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap_or_default();
        let moon = calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN).unwrap_or_default();
        norm_deg(moon.lon - sun.lon)
    };

    // Find crossing of 180° (full moon)
    let mut prev_elong = elongation(jd);
    loop {
        jd += 0.5;
        let cur_elong = elongation(jd);
        // Detect crossing of 180°
        if (prev_elong < 180.0 && cur_elong >= 180.0)
            || (prev_elong > 200.0 && cur_elong < 160.0 && cur_elong > 100.0)
        {
            break;
        }
        // Also detect wrapped crossing (prev ~359, cur ~1 after being near 180)
        prev_elong = cur_elong;
        if jd > jd_start + 35.0 {
            break;
        } // safety
    }

    // Bisect to sub-hour precision
    let mut lo = jd - 1.0;
    let mut hi = jd;
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        let e = elongation(mid);
        if e < 180.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// Find the next new moon (elongation = 0°) at or after `jd_start`.
#[must_use]
pub fn next_new_moon_after(jd_start: JulianDay) -> f64 {
    let jd_start: f64 = jd_start.into();
    let elongation = |jd: f64| -> f64 {
        let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap_or_default();
        let moon = calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN).unwrap_or_default();
        norm_deg(moon.lon - sun.lon)
    };

    let mut jd = jd_start + 1.0;
    let mut prev = elongation(jd);
    loop {
        jd += 0.5;
        let cur = elongation(jd);
        // New moon: elongation crosses 0° from below (wrapping 359→0)
        if prev > 300.0 && cur < 60.0 {
            break;
        }
        prev = cur;
        if jd > jd_start + 35.0 {
            break;
        }
    }

    let mut lo = jd - 1.0;
    let mut hi = jd;
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        let e = elongation(mid);
        if e > 10.0 && e < 350.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
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
    // Start search from March 15 of the given year
    let start = julday(year, 3, 15, 0.0, Calendar::Gregorian);
    // Find first full moon after the equinox
    let fm1 = next_full_moon_after(JulianDay::new(start));
    // Vesak is typically the second full moon after the equinox (in Vaisakha)
    // If the first full moon is in April, the second is Vesak; if in May, it might be the first
    let fm2 = next_full_moon_after(JulianDay::new(fm1 + 1.0));

    // Vesak per Theravada / Sri Lanka / Thailand / India Buddha Purnima
    // is the May full moon (full moon when Sun is in sidereal Taurus,
    // = tropical longitude ≳ Taurus mid-sign in modern era). The Apr 20
    // cut-off previously used here picked the April full moon for
    // years like 2021/2024 where the canonical date is the May one.
    //
    // Empirical fix: skip any full moon before May 1. The first
    // full moon ≥ May 1 and ≤ Jun 20 is Vesak.
    let may1 = julday(year, 5, 1, 0.0, Calendar::Gregorian);
    let jun20 = julday(year, 6, 20, 0.0, Calendar::Gregorian);
    if fm1 >= may1 && fm1 < jun20 {
        fm1
    } else if fm2 >= may1 && fm2 < jun20 {
        fm2
    } else {
        // Find one more FM
        next_full_moon_after(JulianDay::new(fm2 + 1.0))
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
    if !(30.0..=330.0).contains(&target) {
        prev > 300.0 && cur < 60.0
    } else {
        prev < target && cur >= target
    }
}

/// Signed distance of `e` from `target`, handling wrap near 0°.
fn elongation_distance(e: f64, target: f64) -> f64 {
    if target < 30.0 {
        let e2 = if e > 300.0 { e - 360.0 } else { e };
        e2 - target
    } else {
        e - target
    }
}

fn find_phase(jd_start: f64, target: f64) -> f64 {
    let elongation = |jd: f64| -> f64 {
        let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap_or_default();
        let moon = calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN).unwrap_or_default();
        norm_deg(moon.lon - sun.lon)
    };

    let mut jd = jd_start + 0.5;
    let mut prev = elongation(jd);

    loop {
        jd += 0.5;
        let cur = elongation(jd);
        if phase_crossed(prev, cur, target) || jd > jd_start + 35.0 {
            break;
        }
        prev = cur;
    }

    let mut lo = jd - 1.0;
    let mut hi = jd;
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if elongation_distance(elongation(mid), target) < 0.0 {
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
    let mut jd = year_start;

    while jd < end {
        for (target, phase) in &phases {
            let phase_jd = find_phase(jd, *target);
            if phase_jd >= year_start && phase_jd < end && phase_jd > jd - 0.5 {
                let elong = elongation_at(phase_jd);
                days.push(Uposatha {
                    phase: phase.clone(),
                    jd: phase_jd,
                    elongation: elong,
                });
            }
        }
        jd += 25.0; // advance ~one lunar cycle
    }

    days.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    days.dedup_by(|a, b| (a.jd - b.jd).abs() < 3.0);
    days
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julday;

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
