//! Moon phase calculations.
//!
//! Provides the current phase, illumination, and exact timing of all four
//! principal phases (new moon, first quarter, full moon, last quarter)
//! using the same high-accuracy bisection + Newton's method used by the
//! esbats engine.
//!
//! # Phase definitions
//!
//! | Phase | Elongation |
//! |---|---|
//! | New moon | 0° (Moon between Earth and Sun) |
//! | First quarter | 90° (waxing) |
//! | Full moon | 180° (Moon opposite Sun) |
//! | Last quarter | 270° (waning) |
//!
//! # Examples
//! ```
//! # use celestial_core::body::Calendar;
//! use celestial_core::{moon_phase, moon_illumination, MoonPhase, julday};
//!
//! let jd = julday(2025, 4, 27, 20.0, Calendar::Gregorian); // near new moon
//! let phase = moon_phase(jd).unwrap();
//! let illum = moon_illumination(jd).unwrap();
//! assert!(illum < 0.15); // near new moon, low illumination
//! ```

use crate::body::{Body, CalcFlags, Calendar};
use crate::calc_ut;
use crate::error::{Error, Result};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Mean synodic month (new moon to new moon), in days.
pub const SYNODIC_MONTH: f64 = 29.530_588_853;

/// Known new moon epoch: JD 2_451_550.1 ≈ Jan 6, 2000 18:14 UT.
const EPOCH_NEW_MOON: f64 = 2_451_550.1;

// ── Phase enum ────────────────────────────────────────────────────────────────

/// The eight named Moon phases based on elongation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoonPhase {
    /// Elongation 0° ± 22.5° — Moon not visible.
    NewMoon,
    /// Elongation 22.5°–67.5° — less than half illuminated, waxing.
    WaxingCrescent,
    /// Elongation 90° ± 22.5° — exactly half illuminated, waxing.
    FirstQuarter,
    /// Elongation 112.5°–157.5° — more than half illuminated, waxing.
    WaxingGibbous,
    /// Elongation 180° ± 22.5° — fully illuminated.
    FullMoon,
    /// Elongation 202.5°–247.5° — more than half illuminated, waning.
    WaningGibbous,
    /// Elongation 270° ± 22.5° — exactly half illuminated, waning.
    LastQuarter,
    /// Elongation 292.5°–337.5° — less than half illuminated, waning.
    WaningCrescent,
}

impl MoonPhase {
    /// Return the display name of the phase.
    pub fn name(&self) -> &'static str {
        match self {
            Self::NewMoon => "New Moon",
            Self::WaxingCrescent => "Waxing Crescent",
            Self::FirstQuarter => "First Quarter",
            Self::WaxingGibbous => "Waxing Gibbous",
            Self::FullMoon => "Full Moon",
            Self::WaningGibbous => "Waning Gibbous",
            Self::LastQuarter => "Last Quarter",
            Self::WaningCrescent => "Waning Crescent",
        }
    }

    /// Return the target elongation (degrees) of the nearest principal phase.
    pub fn target_elongation(&self) -> f64 {
        match self {
            Self::NewMoon | Self::WaxingCrescent | Self::WaningCrescent => 0.0,
            Self::FirstQuarter | Self::WaxingGibbous => 90.0,
            Self::FullMoon | Self::WaningGibbous => 180.0,
            Self::LastQuarter => 270.0,
        }
    }
}

/// The four principal Moon phases used for `next_phase` and `moon_phases_for_month`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalPhase {
    NewMoon,
    FirstQuarter,
    FullMoon,
    LastQuarter,
}

impl PrincipalPhase {
    /// Target elongation for this principal phase.
    pub fn elongation_target(self) -> f64 {
        match self {
            Self::NewMoon => 0.0,
            Self::FirstQuarter => 90.0,
            Self::FullMoon => 180.0,
            Self::LastQuarter => 270.0,
        }
    }

    /// Human-readable name of this principal phase (e.g. `"Full Moon"`).
    pub fn name(self) -> &'static str {
        match self {
            Self::NewMoon => "New Moon",
            Self::FirstQuarter => "First Quarter",
            Self::FullMoon => "Full Moon",
            Self::LastQuarter => "Last Quarter",
        }
    }
}

/// A principal Moon phase event with its exact Julian day.
#[derive(Debug, Clone)]
pub struct PhaseEvent {
    /// Which principal phase.
    pub phase: PrincipalPhase,
    /// Julian day of the exact phase moment.
    pub jd: f64,
    /// Moon–Sun elongation at this moment (should be very close to the target).
    pub elongation: f64,
}

// ── Core helpers ──────────────────────────────────────────────────────────────

/// Moon–Sun elongation in [0°, 360°) at the given Julian day.
pub fn moon_elongation(jd: f64) -> Result<f64> {
    let sun = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN)?;
    let moon = calc_ut(jd, Body::MOON, CalcFlags::BUILTIN)?;
    Ok((moon.lon - sun.lon).rem_euclid(360.0))
}

/// Signed distance of elongation from a target, in (−180°, +180°].
fn signed_dist(jd: f64, target: f64) -> Result<f64> {
    let e = moon_elongation(jd)?;
    let d = e - target;
    // Wrap to (−180, +180]
    Ok(if d > 180.0 {
        d - 360.0
    } else if d <= -180.0 {
        d + 360.0
    } else {
        d
    })
}

/// Find the exact moment when the Moon–Sun elongation equals `target` degrees,
/// bracketed in `[jd_lo, jd_hi]`.
///
/// Uses Newton's method (≈5 iterations) with bisection fallback.
fn bisect_phase(jd_lo: f64, jd_hi: f64, target: f64) -> Result<f64> {
    let mut jd = (jd_lo + jd_hi) / 2.0;
    let h = 0.01; // 0.01 d ≈ 14 min for numerical derivative

    for _ in 0..15 {
        let f = signed_dist(jd, target)?;
        if f.abs() < 1.0 / 3_600.0 {
            break; // sub-arcsecond
        }
        let fp = (signed_dist(jd + h, target)? - signed_dist(jd - h, target)?) / (2.0 * h);
        if fp.abs() < 0.1 {
            break;
        }
        let step = (f / fp).clamp(-3.0, 3.0);
        jd -= step;
        jd = jd.clamp(jd_lo - 0.5, jd_hi + 0.5);
    }

    // Validate result
    let e = moon_elongation(jd)?;
    let residual = signed_dist(jd, target)?.abs();
    if residual > 0.5 {
        return Err(Error::Calc(format!(
            "phase bisection did not converge: target={target:.1}° got e={e:.3}°"
        )));
    }
    Ok(jd)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Current Moon phase at the given Julian day.
///
/// The phase is determined by the Moon–Sun elongation:
/// each octant (45°) maps to one of the eight named phases.
pub fn moon_phase(jd: f64) -> Result<MoonPhase> {
    let e = moon_elongation(jd)?;
    Ok(match e {
        e if !(22.5..337.5).contains(&e) => MoonPhase::NewMoon,
        e if e < 67.5 => MoonPhase::WaxingCrescent,
        e if e < 112.5 => MoonPhase::FirstQuarter,
        e if e < 157.5 => MoonPhase::WaxingGibbous,
        e if e < 202.5 => MoonPhase::FullMoon,
        e if e < 247.5 => MoonPhase::WaningGibbous,
        e if e < 292.5 => MoonPhase::LastQuarter,
        _ => MoonPhase::WaningCrescent,
    })
}

/// Fraction of the Moon's disk that is illuminated at the given Julian day.
///
/// Returns a value in `[0.0, 1.0]` where 0 = new moon, 1 = full moon.
/// Uses the standard formula: `(1 − cos(elongation)) / 2`.
pub fn moon_illumination(jd: f64) -> Result<f64> {
    let e = moon_elongation(jd)?.to_radians();
    Ok((1.0 - e.cos()) / 2.0)
}

/// Moon–Sun phase angle at the given Julian day (degrees, [0°, 360°)).
///
/// Identical to the elongation for the purposes of phase; 0° = new, 180° = full.
pub fn moon_phase_angle(jd: f64) -> Result<f64> {
    moon_elongation(jd)
}

/// Find the next new moon at or after `jd_from` (elongation = 0°).
///
/// Uses the synodic-month grid to seed a bracketed Newton/bisection solve.
pub fn next_new_moon(jd_from: f64) -> Result<f64> {
    next_principal_phase(jd_from, PrincipalPhase::NewMoon).map(|e| e.jd)
}

/// Find the next first-quarter moon at or after `jd_from` (elongation = 90°).
pub fn next_first_quarter(jd_from: f64) -> Result<f64> {
    next_principal_phase(jd_from, PrincipalPhase::FirstQuarter).map(|e| e.jd)
}

/// Find the next full moon at or after `jd_from` (elongation = 180°).
///
/// This is a higher-level wrapper over the esbats bisection engine.
/// For named full moons (Celtic tradition), use [`crate::next_full_moon`] instead.
pub fn next_full_moon_phase(jd_from: f64) -> Result<f64> {
    next_principal_phase(jd_from, PrincipalPhase::FullMoon).map(|e| e.jd)
}

/// Find the next last-quarter moon at or after `jd_from` (elongation = 270°).
pub fn next_last_quarter(jd_from: f64) -> Result<f64> {
    next_principal_phase(jd_from, PrincipalPhase::LastQuarter).map(|e| e.jd)
}

/// Find the next occurrence of any [`PrincipalPhase`] at or after `jd_from`.
pub fn next_principal_phase(jd_from: f64, phase: PrincipalPhase) -> Result<PhaseEvent> {
    let target = phase.elongation_target();

    // Use the new-moon grid offset by 0, ¼, ½, ¾ synodic month
    let offset = match phase {
        PrincipalPhase::NewMoon => 0.0,
        PrincipalPhase::FirstQuarter => SYNODIC_MONTH / 4.0,
        PrincipalPhase::FullMoon => SYNODIC_MONTH / 2.0,
        PrincipalPhase::LastQuarter => 3.0 * SYNODIC_MONTH / 4.0,
    };
    let epoch = EPOCH_NEW_MOON + offset;
    let n = ((jd_from - epoch) / SYNODIC_MONTH).floor() as i64;

    for k in n..=(n + 3) {
        let jd_approx = epoch + k as f64 * SYNODIC_MONTH;
        if jd_approx + 2.0 < jd_from {
            continue;
        }
        if let Ok(jd) = bisect_phase(jd_approx - 2.0, jd_approx + 2.0, target) {
            if jd >= jd_from - 0.01 {
                let elong = moon_elongation(jd)?;
                return Ok(PhaseEvent {
                    phase,
                    jd,
                    elongation: elong,
                });
            }
        }
    }
    Err(Error::Calc(format!(
        "could not find {phase:?} after JD {jd_from:.1}"
    )))
}

/// All four principal phases for a Gregorian calendar month.
///
/// Returns between 4 and 5 events (occasionally a month has two of the same phase).
pub fn moon_phases_for_month(year: i32, month: u8) -> Result<Vec<PhaseEvent>> {
    use crate::julday;

    let month_start = julday(year, month as i32, 1, 0.0, Calendar::Gregorian);
    // Last day of the month
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let month_end = julday(next_year, next_month as i32, 1, 0.0, Calendar::Gregorian);

    let mut events: Vec<PhaseEvent> = Vec::new();
    let phases = [
        PrincipalPhase::NewMoon,
        PrincipalPhase::FirstQuarter,
        PrincipalPhase::FullMoon,
        PrincipalPhase::LastQuarter,
    ];

    for &phase in &phases {
        let mut jd = month_start;
        loop {
            match next_principal_phase(jd, phase) {
                Ok(event) if event.jd < month_end => {
                    events.push(event);
                    jd = events.last().expect("just pushed").jd + SYNODIC_MONTH * 0.9;
                }
                _ => break,
            }
        }
    }

    events.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    Ok(events)
}

/// Detailed Moon phase information at a given Julian day.
#[derive(Debug, Clone)]
pub struct MoonPhaseInfo {
    /// Named phase (one of eight).
    pub phase: MoonPhase,
    /// Phase name as a string.
    pub phase_name: &'static str,
    /// Moon–Sun elongation (0°–360°).
    pub elongation: f64,
    /// Fraction of disk illuminated (0.0–1.0).
    pub illumination: f64,
    /// Julian day of the previous principal phase.
    pub prev_phase_jd: f64,
    /// Name of the previous principal phase.
    pub prev_phase_name: &'static str,
    /// Julian day of the next principal phase.
    pub next_phase_jd: f64,
    /// Name of the next principal phase.
    pub next_phase_name: &'static str,
    /// Days since the previous principal phase.
    pub age_days: f64,
}

/// Compute full Moon phase information for a given Julian day.
///
/// Includes current phase, illumination, and timing of adjacent principal phases.
pub fn moon_phase_info(jd: f64) -> Result<MoonPhaseInfo> {
    let elong = moon_elongation(jd)?;
    let illum = (1.0 - elong.to_radians().cos()) / 2.0;
    let phase = moon_phase(jd)?;

    // Find the next principal phase
    let next_event = [
        PrincipalPhase::NewMoon,
        PrincipalPhase::FirstQuarter,
        PrincipalPhase::FullMoon,
        PrincipalPhase::LastQuarter,
    ]
    .iter()
    .filter_map(|&p| next_principal_phase(jd, p).ok())
    .min_by(|a, b| a.jd.total_cmp(&b.jd))
    .ok_or_else(|| Error::PhaseNotFound {
        phase: "next".into(),
        from_jd: jd,
    })?;

    // Previous principal phase = next of the preceding lunation
    let prev_event = [
        PrincipalPhase::NewMoon,
        PrincipalPhase::FirstQuarter,
        PrincipalPhase::FullMoon,
        PrincipalPhase::LastQuarter,
    ]
    .iter()
    .filter_map(|&p| {
        next_principal_phase(jd - SYNODIC_MONTH - 2.0, p)
            .ok()
            .filter(|e| e.jd <= jd)
    })
    .max_by(|a, b| a.jd.total_cmp(&b.jd))
    .ok_or_else(|| Error::PhaseNotFound {
        phase: "previous".into(),
        from_jd: jd,
    })?;

    let age_days = jd - prev_event.jd;
    let phase_name = phase.name();

    Ok(MoonPhaseInfo {
        phase,
        phase_name,
        elongation: elong,
        illumination: illum,
        prev_phase_jd: prev_event.jd,
        prev_phase_name: prev_event.phase.name(),
        next_phase_jd: next_event.jd,
        next_phase_name: next_event.phase.name(),
        age_days,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julday;

    fn jd(y: i32, m: i32, d: i32, h: f64) -> f64 {
        julday(y, m, d, h, Calendar::Gregorian)
    }

    /// `MoonPhase::target_elongation` returns the canonical Sun-Moon
    /// angular separation that defines each named phase. Crescents share
    /// the New Moon target (0°), gibbous share First/Last Quarter targets.
    #[test]
    fn target_elongations_are_canonical() {
        assert_eq!(MoonPhase::NewMoon.target_elongation(), 0.0);
        assert_eq!(MoonPhase::WaxingCrescent.target_elongation(), 0.0);
        assert_eq!(MoonPhase::WaningCrescent.target_elongation(), 0.0);
        assert_eq!(MoonPhase::FirstQuarter.target_elongation(), 90.0);
        assert_eq!(MoonPhase::WaxingGibbous.target_elongation(), 90.0);
        assert_eq!(MoonPhase::FullMoon.target_elongation(), 180.0);
        assert_eq!(MoonPhase::WaningGibbous.target_elongation(), 180.0);
        assert_eq!(MoonPhase::LastQuarter.target_elongation(), 270.0);
    }

    /// `MoonPhase::name()` returns a short, human-readable label for each
    /// of the eight phases. Surface tests guard against accidental
    /// reordering or typos that would break user-facing output.
    #[test]
    fn names_are_distinct_and_nonempty() {
        let phases = [
            MoonPhase::NewMoon, MoonPhase::WaxingCrescent,
            MoonPhase::FirstQuarter, MoonPhase::WaxingGibbous,
            MoonPhase::FullMoon, MoonPhase::WaningGibbous,
            MoonPhase::LastQuarter, MoonPhase::WaningCrescent,
        ];
        let names: Vec<&str> = phases.iter().map(super::MoonPhase::name).collect();
        // All non-empty
        for n in &names {
            assert!(!n.is_empty());
        }
        // All distinct
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len(), "phase names must be unique");
    }

    // Known new moon: April 27, 2025 ≈ 19:31 UT → JD 2460793.31
    // Known full moon: April 13, 2025 ≈ 00:22 UT → JD 2460779.52
    // First quarter: April 5, 2025 → JD ~2460771.5
    // Last quarter: April 21, 2025 → JD ~2460787.6

    #[test]
    fn next_new_moon_apr_2025() {
        let start = jd(2025, 4, 25, 0.0);
        let nm = next_new_moon(start).unwrap();
        // Expected: April 27, 2025 ~19:31 UT
        let d = crate::revjul(nm, Calendar::Gregorian);
        assert_eq!(d.year, 2025);
        assert_eq!(d.month, 4);
        assert_eq!(d.day, 27);
        // Elongation should be ≈ 0°
        let e = moon_elongation(nm).unwrap();
        assert!(!(0.1..=359.9).contains(&e), "elongation {e:.4}°");
    }

    #[test]
    fn next_full_moon_apr_2025() {
        let start = jd(2025, 4, 10, 0.0);
        let fm = next_full_moon_phase(start).unwrap();
        let d = crate::revjul(fm, Calendar::Gregorian);
        assert_eq!(d.year, 2025);
        assert_eq!(d.month, 4);
        assert_eq!(d.day, 13);
        let e = moon_elongation(fm).unwrap();
        assert!((e - 180.0).abs() < 0.1, "elongation {e:.4}°");
    }

    #[test]
    fn next_first_quarter_apr_2025() {
        let start = jd(2025, 4, 3, 0.0);
        let fq = next_first_quarter(start).unwrap();
        let d = crate::revjul(fq, Calendar::Gregorian);
        assert_eq!(d.month, 4);
        assert!(d.day >= 4 && d.day <= 7, "day={}", d.day);
        let e = moon_elongation(fq).unwrap();
        assert!((e - 90.0).abs() < 0.5, "elongation {e:.4}°");
    }

    #[test]
    fn next_last_quarter_apr_2025() {
        let start = jd(2025, 4, 19, 0.0);
        let lq = next_last_quarter(start).unwrap();
        let d = crate::revjul(lq, Calendar::Gregorian);
        assert_eq!(d.month, 4);
        assert!(d.day >= 20 && d.day <= 23, "day={}", d.day);
        let e = moon_elongation(lq).unwrap();
        assert!((e - 270.0).abs() < 0.5, "elongation {e:.4}°");
    }

    #[test]
    fn moon_illumination_at_full() {
        let fm = next_full_moon_phase(jd(2025, 4, 10, 0.0)).unwrap();
        let illum = moon_illumination(fm).unwrap();
        assert!(illum > 0.99, "illumination at full moon={illum:.4}");
    }

    #[test]
    fn moon_illumination_at_new() {
        let nm = next_new_moon(jd(2025, 4, 25, 0.0)).unwrap();
        let illum = moon_illumination(nm).unwrap();
        assert!(illum < 0.01, "illumination at new moon={illum:.4}");
    }

    #[test]
    fn moon_illumination_at_quarter() {
        let fq = next_first_quarter(jd(2025, 4, 3, 0.0)).unwrap();
        let illum = moon_illumination(fq).unwrap();
        assert!(
            (illum - 0.5).abs() < 0.02,
            "illumination at quarter={illum:.4}"
        );
    }

    #[test]
    fn moon_phase_near_full() {
        let fm = next_full_moon_phase(jd(2025, 4, 10, 0.0)).unwrap();
        let p = moon_phase(fm).unwrap();
        assert_eq!(p, MoonPhase::FullMoon);
    }

    #[test]
    fn moon_phase_near_new() {
        let nm = next_new_moon(jd(2025, 4, 25, 0.0)).unwrap();
        let p = moon_phase(nm).unwrap();
        assert_eq!(p, MoonPhase::NewMoon);
    }

    #[test]
    fn moon_phases_for_month_april_2025() {
        let events = moon_phases_for_month(2025, 4).unwrap();
        // April 2025 should have 4 events
        assert_eq!(
            events.len(),
            4,
            "events={:?}",
            events.iter().map(|e| e.phase.name()).collect::<Vec<_>>()
        );
        // They should all be in April 2025
        for e in &events {
            let d = crate::revjul(e.jd, Calendar::Gregorian);
            assert_eq!(d.month, 4, "event month={}", d.month);
        }
    }

    #[test]
    fn moon_phase_info_structure() {
        let jd = jd(2025, 4, 15, 12.0); // waning gibbous after Apr 13 full moon
        let info = moon_phase_info(jd).unwrap();
        assert!(info.illumination > 0.0 && info.illumination <= 1.0);
        assert!(info.age_days >= 0.0 && info.age_days < 30.0);
        assert!(!info.phase_name.is_empty());
    }

    #[test]
    fn synodic_month_constant() {
        // Two consecutive new moons should differ by ≈ SYNODIC_MONTH
        let nm1 = next_new_moon(jd(2025, 4, 25, 0.0)).unwrap();
        let nm2 = next_new_moon(nm1 + 1.0).unwrap();
        let diff = nm2 - nm1;
        assert!(
            (diff - SYNODIC_MONTH).abs() < 0.5,
            "synodic month diff={diff:.4}"
        );
    }
}
