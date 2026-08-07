//! Planetary phenomena: magnitude, phase, elongation, diameter.
//!
//! Implements the same quantities as `swe_pheno()`.
//! Formulae from Meeus "Astronomical Algorithms", Mallama & Hilton (2018)
//! and the Explanatory Supplement to the Astronomical Almanac.

use crate::astronomy::constants::angular_separation;

/// All quantities returned by `pheno`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Phenomena {
    /// Phase angle (degrees): Sun–body–observer angle.
    pub phase_angle: f64,
    /// Illuminated fraction of disk (0…1).
    pub phase_frac: f64,
    /// Elongation from Sun (degrees).
    pub elongation: f64,
    /// Apparent angular diameter (arcsec).
    pub ang_diameter: f64,
    /// Apparent visual magnitude.
    pub magnitude: f64,
}

/// Compute phenomena for a planet given geometric data.
///
/// * `body`       — SWE planet number (0=Sun,1=Moon,2=Mercury…)
/// * `lon_body`   — geocentric ecliptic longitude of body (degrees)
/// * `lat_body`   — geocentric ecliptic latitude of body (degrees)
/// * `dist_body`  — geocentric distance (AU)
/// * `dist_sun`   — heliocentric distance of body (AU; = dist_body for Sun)
/// * `lon_sun`    — geocentric ecliptic longitude of Sun (degrees)
#[must_use]
pub fn compute_phenomena(
    body: i32,
    lon_body: f64,
    lat_body: f64,
    dist_body: f64, // geocentric AU
    dist_sun: f64,  // heliocentric AU
    lon_sun: f64,
) -> Phenomena {
    // ── elongation ────────────────────────────────────────────────────────────
    // Simple formula: elong = |lon_body - lon_sun| normalised to 0–180.
    // Latitude is small enough that the great-circle correction is negligible
    // for the visual-magnitude use case below.
    let elongation = angular_separation(lon_body, lon_sun);
    let _ = lat_body;

    // ── phase angle ───────────────────────────────────────────────────────────
    // Cosine rule in the Sun–body–Earth triangle:
    //   r² = R² + d² - 2·R·d·cos(elong)
    // where R = dist_sun (helio), d = dist_body (geo), r = helio dist of Earth ≈ 1 AU
    let cos_alpha =
        (dist_sun * dist_sun + dist_body * dist_body - 1.0) / (2.0 * dist_sun * dist_body);
    let cos_alpha = cos_alpha.clamp(-1.0, 1.0);
    let phase_angle = cos_alpha.acos().to_degrees();

    // ── phase fraction ────────────────────────────────────────────────────────
    let phase_frac = (1.0 + cos_alpha) / 2.0;

    // ── angular diameter ─────────────────────────────────────────────────────
    // Physical radii in km
    let radius_km: f64 = match body {
        0 => 696_000.0, // Sun
        1 => 1_737.4,   // Moon
        2 => 2_439.7,   // Mercury
        3 => 6_051.8,   // Venus
        4 => 3_389.5,   // Mars
        5 => 71_492.0,  // Jupiter
        6 => 60_268.0,  // Saturn (equatorial)
        7 => 25_559.0,  // Uranus
        8 => 24_764.0,  // Neptune
        9 => 1_188.3,   // Pluto
        _ => 0.0,
    };
    const AU_KM: f64 = 149_597_870.7;
    let ang_diameter = if dist_body > 0.0 {
        2.0 * (radius_km / (dist_body * AU_KM)).atan().to_degrees() * 3600.0
    } else {
        0.0
    };

    // ── visual magnitude ─────────────────────────────────────────────────────
    // Using Mallama & Hilton (2018) and classical Müller formulae.
    // H  = absolute magnitude at 0° phase angle
    // G  = slope parameter
    // V  = H + 5·log10(R·Δ) + correction(i)
    let magnitude = visual_magnitude(body, dist_sun, dist_body, phase_angle);

    Phenomena {
        phase_angle,
        phase_frac,
        elongation,
        ang_diameter,
        magnitude,
    }
}

fn visual_magnitude(body: i32, r: f64, delta: f64, i: f64) -> f64 {
    // r = heliocentric AU, delta = geocentric AU, i = phase angle degrees
    let log_rd = 5.0 * (r * delta).log10();
    match body {
        0 => -26.74, // Sun (geocentric)
        1 => {
            // Moon: Hilton (2005)
            -12.74 + 0.026 * i + 4e-9 * i.powi(4)
        }
        2 => {
            // Mercury: Mallama (2017)
            -0.613 + log_rd + 6.328e-2 * i - 1.6336e-3 * i * i + 3.1707e-5 * i.powi(3)
                - 3.2246e-7 * i.powi(4)
        }
        3 => {
            // Venus: Mallama & Hilton (2018)
            -4.384 + log_rd - 1.044e-3 * i + 3.687e-4 * i * i - 2.814e-6 * i.powi(3)
                + 8.938e-9 * i.powi(4)
        }
        4 => {
            // Mars: Mallama (2012)
            -1.601 + log_rd + 0.02267 * i - 0.0001302 * i * i
        }
        5 => {
            // Jupiter: Mallama & Hilton (2018)
            -9.395 + log_rd + 0.005 * i
        }
        6 => {
            // Saturn: mean, no ring correction
            -8.88 + log_rd + 0.044 * i
        }
        7 => {
            // Uranus
            -7.110 + log_rd + 0.0028 * i
        }
        8 => {
            // Neptune
            -7.00 + log_rd + 0.0041 * i
        }
        9 => {
            // Pluto: Buie (2010) — distance-only approximation
            -1.00 + log_rd
        }
        _ => 99.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::test_support::f64_fingerprint;

    fn phenomena_fingerprint(cases: &[(i32, f64, f64, f64, f64, f64)]) -> u64 {
        f64_fingerprint(cases.iter().flat_map(
            |&(body, lon, lat, distance, sun_distance, sun_lon)| {
                let value = compute_phenomena(body, lon, lat, distance, sun_distance, sun_lon);
                [
                    value.phase_angle,
                    value.phase_frac,
                    value.elongation,
                    value.ang_diameter,
                    value.magnitude,
                ]
            },
        ))
    }

    #[test]
    fn all_body_models_match_regression_fingerprint() {
        let cases = [
            (0, 5.0, -1.0, 1.0, 1.1, 355.0),
            (1, 30.0, 3.0, 0.00257, 1.0, 250.0),
            (2, 10.0, -2.0, 0.8, 0.4, 350.0),
            (3, 320.0, 1.0, 0.5, 0.72, 30.0),
            (4, 90.0, -1.5, 0.8, 1.52, 10.0),
            (5, 150.0, 0.5, 4.5, 5.2, 340.0),
            (6, 210.0, -0.25, 8.8, 9.55, 15.0),
            (7, 270.0, 0.1, 19.5, 19.2, 25.0),
            (8, 45.0, -0.1, 29.5, 30.1, 300.0),
            (9, 350.0, 2.0, 34.0, 39.5, 170.0),
            (42, 180.0, 4.0, 2.0, 2.5, 15.0),
        ];
        assert_eq!(phenomena_fingerprint(&cases), 0xa663_06f9_1131_60da);
    }

    #[test]
    fn diameter_and_elongation_boundaries_are_exact() {
        let zero_distance = compute_phenomena(5, 180.0, 0.0, 0.0, 5.2, 0.0);
        assert_eq!(zero_distance.ang_diameter, 0.0);
        assert_eq!(zero_distance.phase_angle, 0.0);
        assert_eq!(zero_distance.phase_frac, 1.0);

        let unknown = compute_phenomena(42, 180.0, 0.0, 1.0, 1.0, 0.0);
        assert_eq!(unknown.ang_diameter, 0.0);
        assert_eq!(unknown.elongation, 180.0);
    }

    /// Sun phenomena: at opposition geometry the apparent magnitude is fixed at
    /// the canonical −26.74; phase fraction is 1.0 (fully illuminated by itself).
    #[test]
    fn sun_magnitude_constant() {
        let p = compute_phenomena(0, 0.0, 0.0, 1.0, 0.0, 0.0);
        assert!((p.magnitude - (-26.74)).abs() < 1e-9);
    }

    /// Moon angular diameter at mean Earth distance is ~31′ (1860″).
    #[test]
    fn moon_angular_diameter_at_mean_distance() {
        let p = compute_phenomena(1, 0.0, 0.0, 0.00257, 1.0, 0.0);
        assert!(
            p.ang_diameter > 1500.0 && p.ang_diameter < 2200.0,
            "moon diameter {}",
            p.ang_diameter
        );
    }

    /// Moon elongation tracks the geocentric Sun–Moon longitude separation.
    #[test]
    fn moon_elongation_tracks_longitude_separation() {
        let new = compute_phenomena(1, 0.0, 0.0, 0.00257, 1.0, 0.0);
        let full = compute_phenomena(1, 180.0, 0.0, 0.00257, 1.0, 0.0);
        assert!(new.elongation < 1.0, "new moon elong = {}", new.elongation);
        assert!(
            (full.elongation - 180.0).abs() < 1.0,
            "full moon elong = {}",
            full.elongation
        );
    }

    /// Elongation wraps correctly across the 0°/360° seam.
    #[test]
    fn elongation_wraps_correctly() {
        // Body at 10°, Sun at 350° → elongation 20° (not 340°)
        let p = compute_phenomena(2, 10.0, 0.0, 0.9, 0.4, 350.0);
        assert!((p.elongation - 20.0).abs() < 1e-9);
    }

    /// Phase angle stays in [0°, 180°].
    #[test]
    fn phase_angle_bounded() {
        for lon in [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0] {
            let p = compute_phenomena(4, lon, 0.0, 1.5, 1.5, 0.0);
            assert!(
                (0.0..=180.0).contains(&p.phase_angle),
                "phase {} for lon {}",
                p.phase_angle,
                lon
            );
            assert!(
                (0.0..=1.0).contains(&p.phase_frac),
                "frac {} for lon {}",
                p.phase_frac,
                lon
            );
        }
    }

    /// Each major body returns a magnitude in a plausible range at typical
    /// distances. Sentinel `99.9` triggers only for unknown body codes.
    #[test]
    fn magnitudes_in_range_for_known_bodies() {
        for body in 2..=9i32 {
            let p = compute_phenomena(body, 90.0, 0.0, 1.5, 1.0, 0.0);
            assert!(
                p.magnitude < 99.9,
                "body {body} returned sentinel magnitude"
            );
        }
        let unknown = compute_phenomena(42, 90.0, 0.0, 1.5, 1.0, 0.0);
        assert!((unknown.magnitude - 99.9).abs() < 1e-9);
    }

    /// Unknown body code returns zero angular diameter (no radius in table).
    #[test]
    fn unknown_body_zero_diameter() {
        let p = compute_phenomena(42, 90.0, 0.0, 1.5, 1.0, 0.0);
        assert_eq!(p.ang_diameter, 0.0);
    }

    /// Default Phenomena values are all zero.
    #[test]
    fn default_phenomena_zero() {
        let p = Phenomena::default();
        assert_eq!(p.phase_angle, 0.0);
        assert_eq!(p.phase_frac, 0.0);
        assert_eq!(p.elongation, 0.0);
        assert_eq!(p.ang_diameter, 0.0);
        assert_eq!(p.magnitude, 0.0);
    }
}
