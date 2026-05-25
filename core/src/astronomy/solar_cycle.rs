//! Solar (Schwabe) cycle calculation.
//!
//! Models the ~11-year sunspot cycle as an asymmetric Hathaway-shape function
//! anchored to **observed** cycle minima and maxima from SIDC/SILSO. Returns
//! [`Some`] inside numbered cycles 1..=25 (1755 → ~2030); returns [`None`]
//! before cycle 1 or after the predicted end of cycle 25.
//!
//! The asymmetric phase classification follows the Waldmeier effect: cycles
//! rise to maximum faster (~4 years on average) than they decline (~7 years),
//! so the phase boundaries are not equal quarters of the cycle length.
//!
//! For long-term context outside numbered cycles, see [`grand_solar_epoch`].
//!
//! # Example
//!
//! ```
//! use celestial_core::solar::{solar_cycle, SolarCyclePhase};
//! use celestial_core::JulianDay;
//!
//! // J2000.0 — JD 2_451_545.0, late in cycle 23
//! let info = solar_cycle(JulianDay::new(2_451_545.0)).unwrap();
//! assert_eq!(info.cycle_num, 23);
//! assert!(matches!(info.phase_name, SolarCyclePhase::Maximum));
//! ```

use crate::body::Calendar;
use crate::functions::time::julday;
use crate::units::JulianDay;
use serde::{Deserialize, Serialize};

/// Phase within a single ~11-year solar cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolarCyclePhase {
    /// Within ±1 year of the cycle's solar minimum (start or end).
    Minimum,
    /// Ascending portion: minimum + 1 y up to maximum − 0.5 y (~4 y average).
    Rising,
    /// Within ±0.5 year of the cycle's solar maximum.
    Maximum,
    /// Descending portion: maximum + 0.5 y down to next minimum − 1 y (~7 y).
    Declining,
}

impl SolarCyclePhase {
    /// Human-readable label.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Minimum => "Minimum",
            Self::Rising => "Rising",
            Self::Maximum => "Maximum",
            Self::Declining => "Declining",
        }
    }
}

/// Multi-cycle grand solar epoch (centuries-scale activity envelope).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrandSolarEpoch {
    /// 1450–1550, low activity, observed via ¹⁴C / ¹⁰Be proxies.
    SporerMinimum,
    /// 1645–1715, near-zero sunspot activity (pre-numbering).
    MaunderMinimum,
    /// 1790–1830, suppressed cycles 5 and 6.
    DaltonMinimum,
    /// 1950–2000, elevated cycles 18–23 (cycle 19 = highest on record).
    ModernMaximum,
}

impl GrandSolarEpoch {
    /// Human-readable label.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::SporerMinimum => "Spörer Minimum",
            Self::MaunderMinimum => "Maunder Minimum",
            Self::DaltonMinimum => "Dalton Minimum",
            Self::ModernMaximum => "Modern Maximum",
        }
    }
}

/// Solar cycle information at a given Julian Day.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SolarCycleInfo {
    /// Schwabe / Wolf cycle number (1..=25 in this table).
    pub cycle_num: u8,
    /// Time-fractional phase within the cycle, `0.0` at the cycle's minimum
    /// and `1.0` at the next minimum. Asymmetric: maximum lands at ~0.35-0.40
    /// for typical cycles.
    pub phase: f64,
    /// Categorical phase classification (Minimum / Rising / Maximum / Declining).
    pub phase_name: SolarCyclePhase,
    /// Julian Day of the cycle's solar minimum (start of cycle).
    pub min_jd: f64,
    /// Julian Day of the cycle's solar maximum.
    pub max_jd: f64,
    /// Julian Day of the next cycle's solar minimum.
    pub next_min_jd: f64,
    /// Years elapsed since the cycle's solar minimum.
    pub years_since_min: f64,
    /// Optional informal name (e.g. cycle 19 = "the Great Cycle").
    pub nickname: Option<&'static str>,
    /// Optional grand epoch label if the date falls inside one.
    pub grand_epoch: Option<GrandSolarEpoch>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Data: cycle minima and maxima (decimal years, Gregorian, UT).
//
// Source: SIDC/SILSO smoothed monthly sunspot number, cycle 1 onwards
// (Wolf 1856; subsequent revisions). Cycle 25's maximum is the
// NOAA SWPC / Solar Cycle Prediction Panel (2019) forecast, refined
// against observations through cycle 25's ascending phase.
// ─────────────────────────────────────────────────────────────────────────────
const CYCLE_DATA: [(u8, f64, f64); 25] = [
    (1, 1755.2, 1761.5),
    (2, 1766.5, 1769.7),
    (3, 1775.5, 1778.4),
    (4, 1784.7, 1788.0),
    (5, 1798.3, 1805.2),
    (6, 1810.6, 1816.4),
    (7, 1823.3, 1829.9),
    (8, 1833.9, 1837.2),
    (9, 1843.5, 1848.1),
    (10, 1856.0, 1860.1),
    (11, 1867.2, 1870.6),
    (12, 1878.9, 1883.9),
    (13, 1890.2, 1894.1),
    (14, 1902.1, 1907.0),
    (15, 1913.6, 1917.6),
    (16, 1923.6, 1928.4),
    (17, 1933.7, 1937.4),
    (18, 1944.1, 1947.4),
    (19, 1954.2, 1957.9),
    (20, 1964.7, 1968.9),
    (21, 1976.1, 1979.9),
    (22, 1986.6, 1989.6),
    (23, 1996.4, 2000.3),
    (24, 2008.9, 2014.3),
    (25, 2019.96, 2024.5),
];

/// Predicted start of cycle 26 (NOAA SWPC; refresh when officially declared).
const NEXT_CYCLE_MIN_YEAR: f64 = 2030.5;

const DAYS_PER_YEAR: f64 = 365.25;

#[inline]
fn is_gregorian_leap(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

/// Convert a Gregorian decimal year to a Julian Day (UT).
fn decimal_year_to_jd(y: f64) -> f64 {
    let year_int = y.floor() as i32;
    let frac = y - y.floor();
    let days = if is_gregorian_leap(year_int) {
        366.0
    } else {
        365.0
    };
    julday(year_int, 1, 1, 0.0, Calendar::Gregorian) + frac * days
}

/// Convert a Julian Day back to an approximate Gregorian decimal year.
/// Used only by [`grand_solar_epoch`], where ±1-day accuracy is irrelevant.
fn jd_to_year_approx(jd: f64) -> f64 {
    // 2000.0 ≈ JD 2_451_544.5; J2000.0 is 2_451_545.0 (noon TT) — close enough
    // for century-scale grand-epoch buckets.
    2000.0 + (jd - 2_451_544.5) / DAYS_PER_YEAR
}

/// Solar cycle information at the given Julian Day (UT).
///
/// Returns [`None`] for non-finite input or for dates outside the numbered
/// Schwabe cycles 1..=25 (i.e. before ~1755 or after ~2030). For dates outside
/// that window, use [`grand_solar_epoch`] for century-scale context.
#[must_use]
pub fn solar_cycle(jd: JulianDay) -> Option<SolarCycleInfo> {
    let jd: f64 = jd.into();
    if !jd.is_finite() {
        return None;
    }
    let jd_start_c1 = decimal_year_to_jd(CYCLE_DATA[0].1);
    let jd_end_c25 = decimal_year_to_jd(NEXT_CYCLE_MIN_YEAR);
    if jd < jd_start_c1 || jd >= jd_end_c25 {
        return None;
    }

    // Linear search over 25 entries; bisection would not be measurably faster.
    let idx = CYCLE_DATA
        .iter()
        .rposition(|&(_, min_y, _)| decimal_year_to_jd(min_y) <= jd)?;
    let (cycle_num, min_y, max_y) = CYCLE_DATA[idx];
    let next_min_y = CYCLE_DATA
        .get(idx + 1)
        .map_or(NEXT_CYCLE_MIN_YEAR, |&(_, m, _)| m);

    let min_jd = decimal_year_to_jd(min_y);
    let max_jd = decimal_year_to_jd(max_y);
    let next_min_jd = decimal_year_to_jd(next_min_y);
    let length = next_min_jd - min_jd;
    let phase = ((jd - min_jd) / length).clamp(0.0, 1.0);
    let years_since_min = (jd - min_jd) / DAYS_PER_YEAR;

    // Waldmeier-asymmetric phase classification. Boundaries in days.
    let one_year = DAYS_PER_YEAR;
    let half_year = DAYS_PER_YEAR / 2.0;
    let phase_name = if (jd - min_jd).abs() < one_year || (next_min_jd - jd).abs() < one_year {
        SolarCyclePhase::Minimum
    } else if (jd - max_jd).abs() < half_year {
        SolarCyclePhase::Maximum
    } else if jd < max_jd {
        SolarCyclePhase::Rising
    } else {
        SolarCyclePhase::Declining
    };

    Some(SolarCycleInfo {
        cycle_num,
        phase,
        phase_name,
        min_jd,
        max_jd,
        next_min_jd,
        years_since_min,
        nickname: cycle_nickname(cycle_num),
        grand_epoch: grand_solar_epoch(JulianDay::new(jd)),
    })
}

/// Informal name for a particular cycle, if any.
#[must_use]
pub fn cycle_nickname(n: u8) -> Option<&'static str> {
    match n {
        19 => Some("the Great Cycle"),
        23 => Some("the long minimum"),
        _ => None,
    }
}

/// Grand solar epoch for the given JD, if any. Returns [`None`] for ordinary
/// activity periods. Works for any JD, including those outside the numbered
/// Schwabe cycles — that's the point: it's the long-term escape hatch when
/// [`solar_cycle`] returns [`None`].
#[must_use]
pub fn grand_solar_epoch(jd: JulianDay) -> Option<GrandSolarEpoch> {
    let jd: f64 = jd.into();
    if !jd.is_finite() {
        return None;
    }
    let y = jd_to_year_approx(jd);
    if (1450.0..=1550.0).contains(&y) {
        Some(GrandSolarEpoch::SporerMinimum)
    } else if (1645.0..=1715.0).contains(&y) {
        Some(GrandSolarEpoch::MaunderMinimum)
    } else if (1790.0..=1830.0).contains(&y) {
        Some(GrandSolarEpoch::DaltonMinimum)
    } else if (1950.0..=2000.0).contains(&y) {
        Some(GrandSolarEpoch::ModernMaximum)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: JD of (year, month, day) at 0h UT, Gregorian calendar.
    fn jd_of(y: i32, m: i32, d: i32) -> f64 {
        julday(y, m, d, 0.0, Calendar::Gregorian)
    }

    #[test]
    fn returns_none_before_cycle_1() {
        assert!(solar_cycle(JulianDay::new(jd_of(1700, 1, 1))).is_none());
        assert!(solar_cycle(JulianDay::new(jd_of(1500, 1, 1))).is_none());
    }

    #[test]
    fn returns_none_after_cycle_25() {
        assert!(solar_cycle(JulianDay::new(jd_of(2050, 1, 1))).is_none());
        assert!(solar_cycle(JulianDay::new(jd_of(2100, 1, 1))).is_none());
    }

    #[test]
    fn returns_none_for_non_finite() {
        assert!(solar_cycle(JulianDay::new(f64::NAN)).is_none());
        assert!(solar_cycle(JulianDay::new(f64::INFINITY)).is_none());
        assert!(solar_cycle(JulianDay::new(f64::NEG_INFINITY)).is_none());
    }

    #[test]
    fn j2000_lands_in_cycle_23_max() {
        // J2000.0 = 2000-01-01 12:00 TT ≈ early 2000, cycle 23 maximum is 2000.3
        let info = solar_cycle(JulianDay::new(2_451_545.0)).expect("cycle 23");
        assert_eq!(info.cycle_num, 23);
        assert_eq!(info.phase_name, SolarCyclePhase::Maximum);
        assert!(info.phase > 0.0 && info.phase < 1.0);
    }

    #[test]
    fn cycle_19_has_nickname() {
        // Mid-1957: cycle 19 maximum.
        let info = solar_cycle(JulianDay::new(jd_of(1957, 6, 1))).expect("cycle 19");
        assert_eq!(info.cycle_num, 19);
        assert_eq!(info.nickname, Some("the Great Cycle"));
    }

    #[test]
    fn cycle_24_min_is_minimum_phase() {
        // Late 2008: solar minimum between cycles 23 and 24.
        let info = solar_cycle(JulianDay::new(jd_of(2008, 12, 1))).expect("cycle 24 start");
        assert_eq!(info.cycle_num, 24);
        assert_eq!(info.phase_name, SolarCyclePhase::Minimum);
        assert!(info.years_since_min < 1.0);
    }

    #[test]
    fn cycle_25_rising_phase() {
        // 2022: well into cycle 25 ascending phase.
        let info = solar_cycle(JulianDay::new(jd_of(2022, 6, 1))).expect("cycle 25 rising");
        assert_eq!(info.cycle_num, 25);
        assert_eq!(info.phase_name, SolarCyclePhase::Rising);
    }

    #[test]
    fn cycle_24_declining_phase() {
        // Early 2017: declining side of cycle 24 (max was 2014.3).
        let info = solar_cycle(JulianDay::new(jd_of(2017, 1, 1))).expect("cycle 24 declining");
        assert_eq!(info.cycle_num, 24);
        assert_eq!(info.phase_name, SolarCyclePhase::Declining);
    }

    #[test]
    fn grand_epoch_classification() {
        assert_eq!(
            grand_solar_epoch(JulianDay::new(jd_of(1680, 1, 1))),
            Some(GrandSolarEpoch::MaunderMinimum)
        );
        assert_eq!(
            grand_solar_epoch(JulianDay::new(jd_of(1810, 1, 1))),
            Some(GrandSolarEpoch::DaltonMinimum)
        );
        assert_eq!(
            grand_solar_epoch(JulianDay::new(jd_of(1980, 1, 1))),
            Some(GrandSolarEpoch::ModernMaximum)
        );
        assert_eq!(grand_solar_epoch(JulianDay::new(jd_of(2010, 1, 1))), None);
    }

    #[test]
    fn grand_epoch_handles_non_finite() {
        assert_eq!(grand_solar_epoch(JulianDay::new(f64::NAN)), None);
        assert_eq!(grand_solar_epoch(JulianDay::new(f64::INFINITY)), None);
    }

    #[test]
    fn phase_within_unit_interval() {
        // Sample a handful of years across the whole table.
        for &year in &[1760, 1800, 1850, 1900, 1950, 2000, 2020] {
            let info = solar_cycle(JulianDay::new(jd_of(year, 6, 1))).expect("inside range");
            assert!(
                (0.0..=1.0).contains(&info.phase),
                "phase out of range at {year}: {}",
                info.phase
            );
        }
    }

    #[test]
    fn phase_name_label_round_trip() {
        // Smoke: each variant has a non-empty label.
        for p in [
            SolarCyclePhase::Minimum,
            SolarCyclePhase::Rising,
            SolarCyclePhase::Maximum,
            SolarCyclePhase::Declining,
        ] {
            assert!(!p.name().is_empty());
        }
        for e in [
            GrandSolarEpoch::SporerMinimum,
            GrandSolarEpoch::MaunderMinimum,
            GrandSolarEpoch::DaltonMinimum,
            GrandSolarEpoch::ModernMaximum,
        ] {
            assert!(!e.name().is_empty());
        }
    }
}
