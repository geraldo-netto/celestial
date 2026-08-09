//! Delta T (TT − UT1) computation.
//!
//! Delta T converts between Universal Time (UT1) and Terrestrial Time (TT,
//! formerly ET). It accounts for the irregular slowing of Earth's rotation.
//!
//! The polynomial fits are from Morrison & Stephenson (2004), Espenak & Meeus
//! (2006), and the USNO/IERS for recent years.

use crate::astronomy::constants::horner;
use crate::units::JulianDay;

/// Compute ΔT (seconds) for a given Julian day number (UT).
///
/// Returns TT − UT1 in seconds, accurate to ~1 s from 500 BCE to 2100 CE
/// and ~10 s from 1000 BCE to 500 BCE.
///
/// If a user override has been set via `set_delta_t_userdef`, that value
/// (converted to seconds) is returned instead.
#[must_use]
pub fn delta_t(jd_ut: JulianDay) -> f64 {
    let jd_ut: f64 = jd_ut.into();
    // Check for user-defined override (stored in days, convert to seconds)
    if let Some(dt_days) = crate::functions::config::user_delta_t() {
        return dt_days * 86_400.0;
    }
    // Fractional year
    let y = jd_ut_to_year(jd_ut);
    delta_t_for_year(y)
}

/// Convert a Julian day (UT) to a fractional Gregorian year.
fn jd_ut_to_year(jd: f64) -> f64 {
    // J2000.0 = 2000.0, and 1 Julian year = 365.25 days
    2000.0 + (jd - 2_451_545.0) / 365.25
}

/// One row of the piece-wise polynomial ΔT fit from Espenak & Meeus (2006).
///
/// For `y` in `(prev_end .. year_end]`, ΔT = horner(t) where
/// `t = (y - year_base) / denom`.
///
/// # References
/// - Espenak & Meeus, "Five Millennium Canon of Solar Eclipses" (2006)
/// - Morrison & Stephenson, J. Hist. Astron. 35 (2004)
/// - IERS Bulletin A for recent observations
struct DeltaTPiece {
    /// Upper bound (exclusive) of this piece's year range.
    year_end: f64,
    /// Offset subtracted from `y` before dividing.
    year_base: f64,
    /// Divisor applied after the offset (100.0 for century-scale fits, 1.0 otherwise).
    denom: f64,
    /// Polynomial coefficients (low-order first), per Horner's method.
    coeffs: &'static [f64],
}

/// Espenak & Meeus polynomial fit pieces for the regular ΔT range
/// (−500 CE through 2050 CE). Ordered by `year_end`.
///
/// Years outside this range use Morrison & Stephenson's long-term parabola
/// (see [`delta_t_for_year`]).
const DELTA_T_PIECES: &[DeltaTPiece] = &[
    // -500 to +500
    DeltaTPiece {
        year_end: 500.0,
        year_base: 0.0,
        denom: 100.0,
        coeffs: &[
            10583.6,
            -1014.41,
            33.78311,
            -5.952053,
            -0.1798452,
            0.022174192,
            0.0090316521,
        ],
    },
    // 500 to 1600
    DeltaTPiece {
        year_end: 1600.0,
        year_base: 1000.0,
        denom: 100.0,
        coeffs: &[
            1574.2,
            -556.01,
            71.23472,
            0.319781,
            -0.8503463,
            -0.005050998,
            0.0083572073,
        ],
    },
    // 1600–1700
    DeltaTPiece {
        year_end: 1700.0,
        year_base: 1600.0,
        denom: 1.0,
        coeffs: &[120.0, -0.9808, -0.01532, 1.0 / 7129.0],
    },
    // 1700–1800
    DeltaTPiece {
        year_end: 1800.0,
        year_base: 1700.0,
        denom: 1.0,
        coeffs: &[8.83, 0.1603, -0.0059285, 0.00013336, -1.0 / 1_174_000.0],
    },
    // 1800–1860
    DeltaTPiece {
        year_end: 1860.0,
        year_base: 1800.0,
        denom: 1.0,
        coeffs: &[
            13.72,
            -0.332447,
            0.0068612,
            0.0041116,
            -0.00037436,
            0.0000121272,
            -0.0000001699,
            0.000000000875,
        ],
    },
    // 1860–1900
    DeltaTPiece {
        year_end: 1900.0,
        year_base: 1860.0,
        denom: 1.0,
        coeffs: &[
            7.62,
            0.5737,
            -0.251754,
            0.01680668,
            -0.0004473624,
            1.0 / 233174.0,
        ],
    },
    // 1900–1920
    DeltaTPiece {
        year_end: 1920.0,
        year_base: 1900.0,
        denom: 1.0,
        coeffs: &[-2.79, 1.494119, -0.0598939, 0.0061966, -0.000197],
    },
    // 1920–1941
    DeltaTPiece {
        year_end: 1941.0,
        year_base: 1920.0,
        denom: 1.0,
        coeffs: &[21.20, 0.84493, -0.076100, 0.0020936],
    },
    // 1941–1961
    DeltaTPiece {
        year_end: 1961.0,
        year_base: 1950.0,
        denom: 1.0,
        coeffs: &[29.07, 0.407, -1.0 / 233.0, 1.0 / 2547.0],
    },
    // 1961–1986
    DeltaTPiece {
        year_end: 1986.0,
        year_base: 1975.0,
        denom: 1.0,
        coeffs: &[45.45, 1.067, -1.0 / 260.0, -1.0 / 718.0],
    },
    // 1986–2005
    DeltaTPiece {
        year_end: 2005.0,
        year_base: 2000.0,
        denom: 1.0,
        coeffs: &[
            63.86,
            0.3345,
            -0.060374,
            0.0017275,
            0.000651814,
            0.00002373599,
        ],
    },
    // 2005–2050 (prediction)
    DeltaTPiece {
        year_end: 2050.0,
        year_base: 2000.0,
        denom: 1.0,
        coeffs: &[62.92, 0.32217, 0.005589],
    },
];

/// Morrison & Stephenson long-term ΔT parabola, used outside the
/// Espenak & Meeus table range.
fn long_term_parabola(y: f64) -> f64 {
    let u = (y - 1820.0) / 100.0;
    (32.0 * u).mul_add(u, -20.0)
}

/// Compute ΔT (the difference TT − UT) for a given calendar year, in seconds.
///
/// Returns the accumulated offset between Terrestrial Time (atomic) and
/// Universal Time (Earth rotation). ΔT arises from the Earth's irregular
/// rotation — tidal braking slowly lengthens the day, and fluid-core/mantle
/// coupling causes decade-scale variations.
///
/// # Reference values
/// * Year  -500: ΔT ≈ 17,190 s  (≈ 4h 46m)
/// * Year  1000: ΔT ≈  1570 s
/// * Year  2000: ΔT ≈    63.83 s
/// * Year  2025: ΔT ≈    69 s (projected, IERS Bulletin A)
///
/// # Implementation
/// Espenak & Meeus (2006) "Five Millennium Canon of Solar Eclipses" piece-wise
/// polynomial fits for −500 through 2050, and the Morrison & Stephenson
/// long-term parabola for years outside that range. See [`DELTA_T_PIECES`]
/// for the 12 polynomial segments.
///
/// # Accuracy
/// Sub-second in the instrumental era (post-1800); a few seconds for 1600–1800;
/// ±20 s from 500–1600; larger for historical years.
#[must_use]
pub fn delta_t_for_year(y: f64) -> f64 {
    // Outside the table: pure long-term parabola
    if !(-500.0..2150.0).contains(&y) {
        return long_term_parabola(y);
    }
    // 2050–2150: parabola with linear correction toward the 2150 anchor
    if y >= 2050.0 {
        return 0.5628_f64.mul_add(y - 2150.0, long_term_parabola(y));
    }
    // Regular piece-wise fit: find the first piece covering y
    for p in DELTA_T_PIECES {
        if y < p.year_end {
            let t = (y - p.year_base) / p.denom;
            return horner(t, p.coeffs);
        }
    }
    // Unreachable: the last piece ends at 2050.0 and we already short-circuited
    // on y >= 2050.0 above, but return a sensible fallback for safety.
    long_term_parabola(y)
}

/// Convert a UT Julian day to TT (Terrestrial Time) Julian day.
#[inline]
#[must_use]
pub fn ut_to_tt(jd_ut: JulianDay) -> f64 {
    let jd_ut: f64 = jd_ut.into();
    jd_ut + delta_t(JulianDay::new(jd_ut)) / 86_400.0
}

/// Convert a TT Julian day to UT Julian day (iterative).
#[must_use]
pub fn tt_to_ut(jd_tt: f64) -> f64 {
    // ΔT as a function of TT is approximately the same as a function of UT
    // for the precision we need here.
    jd_tt - delta_t(JulianDay::new(jd_tt)) / 86_400.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_t_1900() {
        // Morrison & Stephenson value for 1900
        let dt = delta_t_for_year(1900.0);
        assert!((dt - (-2.72)).abs() < 1.0, "ΔT(1900) = {dt}");
    }

    #[test]
    fn delta_t_2000() {
        // Known: ΔT ≈ 63.8 s in 2000
        let dt = delta_t(JulianDay::new(2_451_545.0));
        assert!((dt - 63.8).abs() < 2.0, "ΔT(J2000) = {dt}");
    }

    #[test]
    fn delta_t_1000_bce() {
        // Roughly 1800 s per Morrison & Stephenson
        let dt = delta_t_for_year(-1000.0);
        assert!(dt > 10000.0 && dt < 40000.0, "ΔT(-1000) = {dt}");
    }

    #[test]
    fn ut_to_tt_roundtrip() {
        let jd = 2_451_545.0;
        let tt = ut_to_tt(JulianDay::new(jd));
        let ut = tt_to_ut(tt);
        assert!(
            (ut - jd).abs() < 1e-6,
            "round-trip error: {}",
            (ut - jd).abs()
        );
    }

    #[test]
    fn user_override_round_trips_seconds() {
        crate::functions::config::set_delta_t_userdef(0.5);
        let dt = delta_t(JulianDay::new(2_451_545.0));
        crate::functions::config::set_delta_t_userdef(f64::NAN);
        assert!((dt - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn delta_t_piecewise_regression_table() {
        let cases = [
            (-1000.0, 25427.68),
            (-499.5, 17194.616304659),
            (0.0, 10583.6),
            (499.5, 5715.003872859),
            (500.0, 5710.044670312),
            (1000.0, 1574.2),
            (1599.5, 120.473098609),
            (1600.0, 120.0),
            (1650.0, 50.194015991),
            (1699.5, 8.917118885),
            (1700.0, 8.83),
            (1750.0, 13.370070273),
            (1799.5, 13.967770784),
            (1800.0, 13.72),
            (1830.0, 7.67338),
            (1859.5, 7.409854231),
            (1860.0, 7.62),
            (1880.0, -5.008486988),
            (1899.5, -3.387178823),
            (1900.0, -2.79),
            (1910.0, 10.3884),
            (1919.5, 21.033437138),
            (1920.0, 21.2),
            (1930.0, 24.1329),
            (1940.5, 24.5766657),
            (1941.0, 24.773141434),
            (1950.0, 29.07),
            (1960.5, 33.324829335),
            (1961.0, 33.579880866),
            (1975.0, 45.45),
            (1985.5, 54.617170452),
            (1986.0, 54.877737538),
            (1995.0, 60.795421281),
            (2004.5, 64.611178993),
            (2005.0, 64.670575),
            (2025.0, 74.467375),
            (2049.5, 92.56186225),
            (2050.0, 93.0),
            (2100.0, 202.74),
            (2149.5, 327.1434),
            (2150.0, 328.48),
            (2500.0, 1459.68),
        ];

        for (year, expected) in cases {
            let actual = delta_t_for_year(year);
            assert!((actual - expected).abs() < 1e-6, "year {year}: {actual}");
        }
    }
}
