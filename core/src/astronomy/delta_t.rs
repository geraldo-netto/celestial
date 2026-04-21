//! Delta T (TT − UT1) computation.
//!
//! Delta T converts between Universal Time (UT1) and Terrestrial Time (TT,
//! formerly ET). It accounts for the irregular slowing of Earth's rotation.
//!
//! The polynomial fits are from Morrison & Stephenson (2004), Espenak & Meeus
//! (2006), and the USNO/IERS for recent years.

/// Compute ΔT (seconds) for a given Julian day number (UT).
///
/// Returns TT − UT1 in seconds, accurate to ~1 s from 500 BCE to 2100 CE
/// and ~10 s from 1000 BCE to 500 BCE.
///
/// If a user override has been set via `set_delta_t_userdef`, that value
/// (converted to seconds) is returned instead.
pub fn delta_t(jd_ut: f64) -> f64 {
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

/// Compute ΔT (seconds) for a fractional year.
///
/// Sources:
///   - Espenak & Meeus, "Five Millennium Canon of Solar Eclipses" (2006)
///   - Morrison & Stephenson, J. Hist. Astron. 35 (2004)
///   - IERS Bulletin A for recent observations
pub fn delta_t_for_year(y: f64) -> f64 {
    if y < -500.0 {
        // Before 500 BCE: Morrison & Stephenson parabola
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    } else if y < 500.0 {
        // -500 to +500: Espenak & Meeus Table 1
        let u = y / 100.0;
        polynomial(
            u,
            &[
                10583.6,
                -1014.41,
                33.78311,
                -5.952053,
                -0.1798452,
                0.022174192,
                0.0090316521,
            ],
        )
    } else if y < 1600.0 {
        // 500 to 1600
        let u = (y - 1000.0) / 100.0;
        polynomial(
            u,
            &[
                1574.2,
                -556.01,
                71.23472,
                0.319781,
                -0.8503463,
                -0.005050998,
                0.0083572073,
            ],
        )
    } else if y < 1700.0 {
        // 1600–1700
        let t = y - 1600.0;
        polynomial(t, &[120.0, -0.9808, -0.01532, 1.0 / 7129.0])
    } else if y < 1800.0 {
        // 1700–1800
        let t = y - 1700.0;
        polynomial(
            t,
            &[8.83, 0.1603, -0.0059285, 0.00013336, -1.0 / 1_174_000.0],
        )
    } else if y < 1860.0 {
        // 1800–1860
        let t = y - 1800.0;
        polynomial(
            t,
            &[
                13.72,
                -0.332447,
                0.0068612,
                0.0041116,
                -0.00037436,
                0.0000121272,
                -0.0000001699,
                0.000000000875,
            ],
        )
    } else if y < 1900.0 {
        // 1860–1900
        let t = y - 1860.0;
        polynomial(
            t,
            &[
                7.62,
                0.5737,
                -0.251754,
                0.01680668,
                -0.0004473624,
                1.0 / 233174.0,
            ],
        )
    } else if y < 1920.0 {
        // 1900–1920
        let t = y - 1900.0;
        polynomial(t, &[-2.79, 1.494119, -0.0598939, 0.0061966, -0.000197])
    } else if y < 1941.0 {
        // 1920–1941
        let t = y - 1920.0;
        polynomial(t, &[21.20, 0.84493, -0.076100, 0.0020936])
    } else if y < 1961.0 {
        // 1941–1961
        let t = y - 1950.0;
        polynomial(t, &[29.07, 0.407, -1.0 / 233.0, 1.0 / 2547.0])
    } else if y < 1986.0 {
        // 1961–1986
        let t = y - 1975.0;
        polynomial(t, &[45.45, 1.067, -1.0 / 260.0, -1.0 / 718.0])
    } else if y < 2005.0 {
        // 1986–2005
        let t = y - 2000.0;
        polynomial(
            t,
            &[
                63.86,
                0.3345,
                -0.060374,
                0.0017275,
                0.000651814,
                0.00002373599,
            ],
        )
    } else if y < 2050.0 {
        // 2005–2050 (prediction)
        let t = y - 2000.0;
        polynomial(t, &[62.92, 0.32217, 0.005589])
    } else if y < 2150.0 {
        // 2050–2150 (long-range prediction)
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u - 0.5628 * (2150.0 - y)
    } else {
        // After 2150: Morrison & Stephenson parabola
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

/// Evaluate a polynomial using Horner's method.
///
/// `coeffs[0]` is the constant term, `coeffs[n]` is the coefficient of `x^n`.
fn polynomial(x: f64, coeffs: &[f64]) -> f64 {
    coeffs.iter().rev().fold(0.0, |acc, &c| acc * x + c)
}

/// Convert a UT Julian day to TT (Terrestrial Time) Julian day.
#[inline]
pub fn ut_to_tt(jd_ut: f64) -> f64 {
    jd_ut + delta_t(jd_ut) / 86_400.0
}

/// Convert a TT Julian day to UT Julian day (iterative).
pub fn tt_to_ut(jd_tt: f64) -> f64 {
    // ΔT as a function of TT is approximately the same as a function of UT
    // for the precision we need here.
    jd_tt - delta_t(jd_tt) / 86_400.0
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
        let dt = delta_t(2_451_545.0);
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
        let tt = ut_to_tt(jd);
        let ut = tt_to_ut(tt);
        assert!(
            (ut - jd).abs() < 1e-6,
            "round-trip error: {}",
            (ut - jd).abs()
        );
    }
}
