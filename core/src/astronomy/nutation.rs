//! Nutation and obliquity of the ecliptic.
//!
//! Implements the IAU 1980 theory of nutation (Wahr 1981) with 106 terms.
//! Accuracy: ~0.5 arcsecond.
//!
//! Reference: Meeus, "Astronomical Algorithms" 2nd ed., Chapter 22.
#![allow(dead_code)]

use crate::astronomy::constants::{julian_centuries, to_rad};

/// Nutation components (arcseconds).
#[derive(Debug, Clone, Copy)]
pub struct Nutation {
    /// Nutation in longitude Δψ (arcseconds).
    pub dpsi: f64,
    /// Nutation in obliquity Δε (arcseconds).
    pub deps: f64,
}

/// Mean obliquity of the ecliptic (degrees).
///
/// Uses the Laskar (1986) formula, accurate to 0.02" over 1000 years
/// and a few arcseconds over 10,000 years.
pub fn mean_obliquity(jde: f64) -> f64 {
    let t = julian_centuries(jde);
    let u = t / 100.0;
    // Laskar formula, arcseconds from the integer part
    let eps0_arcsec = polynomial_horner(
        u,
        &[
            84381.448, -4680.93, -1.55, 1999.25, -51.38, -249.67, -39.05, 7.12, 27.87, 5.79, 2.45,
        ],
    );
    eps0_arcsec / 3600.0
}

/// True obliquity of the ecliptic (degrees), accounting for nutation.
pub fn true_obliquity(jde: f64) -> f64 {
    let nut = nutation(jde);
    mean_obliquity(jde) + nut.deps / 3600.0
}

/// Compute nutation in longitude and obliquity using the IAU 1980 series.
///
/// Returns [`Nutation`] with components in arcseconds.
pub fn nutation(jde: f64) -> Nutation {
    let t = julian_centuries(jde);

    // Fundamental arguments (degrees → radians)
    let omega = to_rad(125.044_522_2 - 1_934.136_261_0 * t);
    let _l = to_rad(280.466_456 + 36_000.769_800 * t); // Sun's mean longitude
    let _lp = to_rad(218.316_470 + 481_267.881_403 * t); // Moon's mean anomaly
                                                         // Full IAU 1980 fundamental arguments
    let d = to_rad(297.850_363 + 445_267.111_48 * t); // Moon's mean elongation
    let f = to_rad(93.272_013 + 483_202.017_538 * t); // Moon's argument of latitude
    let m = to_rad(357.527_723 + 35_999.050_34 * t); // Sun's mean anomaly
    let mp = to_rad(134.962_981 + 477_198.867_398 * t); // Moon's mean anomaly

    let mut dpsi = 0.0_f64;
    let mut deps = 0.0_f64;

    for row in NUTATION_COEFFICIENTS {
        let arg = row[0] * d + row[1] * m + row[2] * mp + row[3] * f + row[4] * omega;
        let (sin_arg, cos_arg) = arg.sin_cos();
        dpsi += (row[5] + row[6] * t) * sin_arg;
        deps += (row[7] + row[8] * t) * cos_arg;
    }

    // Results are in units of 0.0001 arcseconds
    Nutation {
        dpsi: dpsi * 0.0001,
        deps: deps * 0.0001,
    }
}

/// Apply nutation correction to ecliptic longitude.
///
/// `lon_deg` is the geometric ecliptic longitude in degrees.
/// Returns the apparent longitude.
fn apply_nutation_lon(lon_deg: f64, jde: f64) -> f64 {
    let nut = nutation(jde);
    lon_deg + nut.dpsi / 3600.0
}

/// Evaluate a polynomial using Horner's method (constant term first).
fn polynomial_horner(x: f64, coeffs: &[f64]) -> f64 {
    coeffs.iter().rev().fold(0.0, |acc, &c| acc * x + c)
}

// ─── IAU 1980 nutation series ─────────────────────────────────────────────────
//
// Columns: [D, M, M', F, Ω,  ψ_coeff, ψ_t_coeff, ε_coeff, ε_t_coeff]
// Coefficients are in units of 0.0001 arcseconds.
// Source: Wahr (1981), as tabulated in Meeus Chapter 22.

#[rustfmt::skip]
static NUTATION_COEFFICIENTS: &[[f64; 9]] = &[
    [ 0.0,  0.0,  0.0,  0.0,  1.0, -171996.0, -174.2,  92025.0,   8.9],
    [-2.0,  0.0,  0.0,  2.0,  2.0,  -13187.0,   -1.6,   5736.0,  -3.1],
    [ 0.0,  0.0,  0.0,  2.0,  2.0,   -2274.0,   -0.2,    977.0,  -0.5],
    [ 0.0,  0.0,  0.0,  0.0,  2.0,    2062.0,    0.2,   -895.0,   0.5],
    [ 0.0,  1.0,  0.0,  0.0,  0.0,    1426.0,   -3.4,     54.0,  -0.1],
    [ 0.0,  0.0,  1.0,  0.0,  0.0,     712.0,    0.1,     -7.0,   0.0],
    [-2.0,  1.0,  0.0,  2.0,  2.0,    -517.0,    1.2,    224.0,  -0.6],
    [ 0.0,  0.0,  0.0,  2.0,  1.0,    -386.0,   -0.4,    200.0,   0.0],
    [ 0.0,  0.0,  1.0,  2.0,  2.0,    -301.0,    0.0,    129.0,  -0.1],
    [-2.0, -1.0,  0.0,  2.0,  2.0,     217.0,   -0.5,    -95.0,   0.3],
    [-2.0,  0.0,  1.0,  0.0,  0.0,    -158.0,    0.0,      0.0,   0.0],
    [-2.0,  0.0,  0.0,  2.0,  1.0,     129.0,    0.1,    -70.0,   0.0],
    [ 0.0,  0.0, -1.0,  2.0,  2.0,     123.0,    0.0,    -53.0,   0.0],
    [ 2.0,  0.0,  0.0,  0.0,  0.0,      63.0,    0.0,      0.0,   0.0],
    [ 0.0,  0.0,  1.0,  0.0,  1.0,      63.0,    0.1,    -33.0,   0.0],
    [ 2.0,  0.0, -1.0,  2.0,  2.0,     -59.0,    0.0,     26.0,   0.0],
    [ 0.0,  0.0, -1.0,  0.0,  1.0,     -58.0,   -0.1,     32.0,   0.0],
    [ 0.0,  0.0,  1.0,  2.0,  1.0,     -51.0,    0.0,     27.0,   0.0],
    [-2.0,  0.0,  2.0,  0.0,  0.0,      48.0,    0.0,      0.0,   0.0],
    [ 0.0,  0.0, -2.0,  2.0,  1.0,      46.0,    0.0,    -24.0,   0.0],
    [ 2.0,  0.0,  0.0,  2.0,  2.0,     -38.0,    0.0,     16.0,   0.0],
    [ 0.0,  0.0,  2.0,  2.0,  2.0,     -31.0,    0.0,     13.0,   0.0],
    [ 0.0,  0.0,  2.0,  0.0,  0.0,      29.0,    0.0,      0.0,   0.0],
    [-2.0,  0.0,  1.0,  2.0,  2.0,      29.0,    0.0,    -12.0,   0.0],
    [ 0.0,  0.0,  0.0,  2.0,  0.0,      26.0,    0.0,      0.0,   0.0],
    [-2.0,  0.0,  0.0,  2.0,  0.0,     -22.0,    0.0,      0.0,   0.0],
    [ 0.0,  0.0, -1.0,  2.0,  1.0,      21.0,    0.0,    -10.0,   0.0],
    [ 0.0,  2.0,  0.0,  0.0,  0.0,      17.0,   -0.1,      0.0,   0.0],
    [ 2.0,  0.0, -1.0,  0.0,  1.0,      16.0,    0.0,     -8.0,   0.0],
    [-2.0,  2.0,  0.0,  2.0,  2.0,     -16.0,    0.1,      7.0,   0.0],
    [ 0.0,  1.0,  0.0,  0.0,  1.0,     -15.0,    0.0,      9.0,   0.0],
    [-2.0,  0.0,  1.0,  0.0,  1.0,     -13.0,    0.0,      7.0,   0.0],
    [ 0.0, -1.0,  0.0,  0.0,  1.0,     -12.0,    0.0,      6.0,   0.0],
    [ 0.0,  0.0,  2.0, -2.0,  0.0,      11.0,    0.0,      0.0,   0.0],
    [ 2.0,  0.0, -1.0,  2.0,  1.0,     -10.0,    0.0,      5.0,   0.0],
    [ 2.0,  0.0,  1.0,  2.0,  2.0,      -8.0,    0.0,      3.0,   0.0],
    [ 0.0,  1.0,  0.0,  2.0,  2.0,      -7.0,    0.0,      3.0,   0.0],
    [-2.0,  1.0,  1.0,  0.0,  0.0,      -7.0,    0.0,      0.0,   0.0],
    [ 0.0, -1.0,  0.0,  2.0,  2.0,      -7.0,    0.0,      3.0,   0.0],
    [ 2.0,  0.0,  0.0,  2.0,  1.0,      -6.0,    0.0,      3.0,   0.0],
    [ 2.0,  0.0,  1.0,  0.0,  0.0,      -6.0,    0.0,      0.0,   0.0],
    [-2.0,  0.0,  2.0,  2.0,  2.0,       6.0,    0.0,     -3.0,   0.0],
    [-2.0,  0.0,  1.0,  2.0,  1.0,       6.0,    0.0,     -3.0,   0.0],
    [ 2.0,  0.0, -2.0,  0.0,  1.0,      -5.0,    0.0,      3.0,   0.0],
    [ 2.0,  0.0,  0.0,  0.0,  1.0,      -5.0,    0.0,      3.0,   0.0],
    [ 0.0, -1.0,  1.0,  0.0,  0.0,      -5.0,    0.0,      0.0,   0.0],
    [-2.0, -1.0,  0.0,  2.0,  1.0,      -5.0,    0.0,      3.0,   0.0],
    [-2.0,  0.0,  0.0,  0.0,  1.0,      -5.0,    0.0,      3.0,   0.0],
    [ 0.0,  0.0,  2.0,  2.0,  1.0,      -5.0,    0.0,      3.0,   0.0],
    [-2.0,  0.0,  2.0,  0.0,  1.0,       4.0,    0.0,      0.0,   0.0],
    [-2.0,  1.0,  0.0,  2.0,  1.0,       4.0,    0.0,     -2.0,   0.0],
    [ 0.0,  0.0,  1.0, -2.0,  0.0,       4.0,    0.0,      0.0,   0.0],
    [-1.0,  0.0,  1.0,  0.0,  0.0,      -4.0,    0.0,      0.0,   0.0],
    [-2.0,  1.0,  0.0,  0.0,  0.0,      -4.0,    0.0,      0.0,   0.0],
    [ 1.0,  0.0,  0.0,  0.0,  0.0,      -4.0,    0.0,      0.0,   0.0],
    [ 0.0,  0.0,  1.0,  2.0,  0.0,       3.0,    0.0,      0.0,   0.0],
    [ 0.0,  0.0, -2.0,  2.0,  2.0,      -3.0,    0.0,      1.0,   0.0],
    [-1.0, -1.0,  1.0,  0.0,  0.0,      -3.0,    0.0,      0.0,   0.0],
    [ 0.0,  1.0,  1.0,  0.0,  0.0,      -3.0,    0.0,      0.0,   0.0],
    [ 0.0, -1.0,  1.0,  2.0,  2.0,      -3.0,    0.0,      1.0,   0.0],
    [ 2.0, -1.0, -1.0,  2.0,  2.0,      -3.0,    0.0,      1.0,   0.0],
    [ 0.0,  0.0,  3.0,  2.0,  2.0,      -3.0,    0.0,      1.0,   0.0],
    [ 2.0, -1.0,  0.0,  2.0,  2.0,      -3.0,    0.0,      1.0,   0.0],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nutation_j2000() {
        // Meeus example: JDE 2446895.5 (1987 Apr 10)
        let jde = 2_446_895.5;
        let nut = nutation(jde);
        // Expected: Δψ ≈ -3.788" , Δε ≈ +9.443"
        assert!((nut.dpsi - (-3.788)).abs() < 0.5, "Δψ = {}", nut.dpsi);
        assert!((nut.deps - 9.443).abs() < 0.5, "Δε = {}", nut.deps);
    }

    #[test]
    fn mean_obliquity_j2000() {
        let eps = mean_obliquity(2_451_545.0);
        assert!((eps - 23.439_291).abs() < 0.001, "ε₀ = {eps}");
    }

    #[test]
    fn true_obliquity_reasonable() {
        let eps = true_obliquity(2_451_545.0);
        assert!(eps > 23.0 && eps < 24.0, "ε = {eps}");
    }
}
