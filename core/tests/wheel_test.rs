//! Tests extracted from new_features_test.rs — do not edit manually.

#[cfg(test)]
mod minor_aspects {
    // Body available via celestial_core::* glob above
    use celestial_core::*;

    #[test]
    fn quintile_72_detected() {
        // Quintile = 72°; test with two positions exactly 72° apart
        let diff = diff_deg_signed(0.0, 72.0).abs();
        assert!((diff - 72.0).abs() < 1e-9, "diff = {diff}");
        assert!(diff <= 72.0 + 1.5, "within quintile orb");
    }

    #[test]
    fn septile_51_4_approximation() {
        // Septile ≈ 360/7 ≈ 51.4286°
        let septile: f64 = 360.0 / 7.0;
        assert!((septile - 51.4286).abs() < 0.001, "septile = {septile}");
    }

    #[test]
    fn semi_square_45() {
        let diff = diff_deg_signed(10.0, 55.0).abs();
        assert!((diff - 45.0).abs() < 1e-9);
    }

    #[test]
    fn sesquiquadrate_135() {
        let diff = diff_deg_signed(10.0, 145.0).abs();
        assert!((diff - 135.0).abs() < 1e-9);
    }

    #[test]
    fn biquintile_144() {
        let diff = diff_deg_signed(0.0, 144.0).abs();
        assert!((diff - 144.0).abs() < 1e-9);
    }

    #[test]
    fn novile_40() {
        // Novile = 360/9 = 40°
        let novile: f64 = 360.0 / 9.0;
        assert!((novile - 40.0).abs() < 1e-9);
    }
}

#[cfg(test)]
mod antiscia {
    /// Antiscion = reflection over the Cancer-Capricorn solstice axis.
    /// Formula: (180° − lon) mod 360°
    fn antiscion(lon: f64) -> f64 {
        (180.0 - lon).rem_euclid(360.0)
    }

    /// Contra-antiscion = reflection over the Aries-Libra equinox axis.
    /// Formula: (360° − lon) mod 360°
    fn contra_antiscion(lon: f64) -> f64 {
        (360.0 - lon).rem_euclid(360.0)
    }

    #[test]
    fn aries_15_antiscion_is_virgo_15() {
        // 15° Aries = 15° → antiscion = 165° = 15° Virgo
        let a = antiscion(15.0);
        assert!((a - 165.0).abs() < 1e-9, "a = {a}");
    }

    #[test]
    fn cancer_0_antiscion_is_self() {
        // 0° Cancer = 90° → antiscion = 90° (on the axis)
        let a = antiscion(90.0);
        assert!((a - 90.0).abs() < 1e-9, "a = {a}");
    }

    #[test]
    fn capricorn_0_antiscion_is_self() {
        // 0° Capricorn = 270° → antiscion = 270° (on the axis)
        let a = antiscion(270.0);
        assert!((a - 270.0).abs() < 1e-9, "a = {a}");
    }

    #[test]
    fn taurus_15_antiscion_is_leo_15() {
        // 45° → antiscion = 135° = 15° Leo
        let a = antiscion(45.0);
        assert!((a - 135.0).abs() < 1e-9, "a = {a}");
    }

    #[test]
    fn contra_aries_15_is_pisces_15() {
        // 15° → contra = 345° = 15° Pisces
        let c = contra_antiscion(15.0);
        assert!((c - 345.0).abs() < 1e-9, "c = {c}");
    }

    #[test]
    fn antiscion_double_reflection_is_identity() {
        // Applying antiscion twice returns the original
        for lon in [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0] {
            let a = antiscion(antiscion(lon));
            assert!((a - lon).abs() < 1e-9, "double antiscion of {lon} = {a}");
        }
    }
}

#[cfg(test)]
mod dignities {
    use celestial_core::body::Body;
    use celestial_core::*;

    // sign_ruler and sign_exaltation are public — test expected values

    #[test]
    fn sun_rules_leo() {
        assert_eq!(sign_ruler(4), Body::SUN); // Leo = sign 4
    }

    #[test]
    fn moon_rules_cancer() {
        assert_eq!(sign_ruler(3), Body::MOON); // Cancer = sign 3
    }

    #[test]
    fn mars_rules_aries() {
        assert_eq!(sign_ruler(0), Body::MARS); // Aries = sign 0
    }

    #[test]
    fn sun_exalted_in_aries() {
        assert_eq!(sign_exaltation(Body::SUN), 0); // Aries = 0
    }

    #[test]
    fn moon_exalted_in_taurus() {
        assert_eq!(sign_exaltation(Body::MOON), 1); // Taurus = 1
    }

    #[test]
    fn mars_exalted_in_capricorn() {
        assert_eq!(sign_exaltation(Body::MARS), 9); // Capricorn = 9
    }

    #[test]
    fn outer_planets_no_exaltation() {
        // Uranus, Neptune, Pluto — no traditional exaltation
        assert_eq!(sign_exaltation(Body::URANUS), -1);
        assert_eq!(sign_exaltation(Body::NEPTUNE), -1);
        assert_eq!(sign_exaltation(Body::PLUTO), -1);
    }

    #[test]
    fn all_signs_have_a_ruler() {
        for s in 0u8..12 {
            let ruler = sign_ruler(s);
            // Should be one of the seven traditional planets
            assert!(
                [
                    Body::SUN,
                    Body::MOON,
                    Body::MERCURY,
                    Body::VENUS,
                    Body::MARS,
                    Body::JUPITER,
                    Body::SATURN
                ]
                .contains(&ruler),
                "sign {s} has unexpected ruler {:?}",
                ruler
            );
        }
    }
}

#[cfg(test)]
mod arabic_parts {
    use celestial_core::*;

    #[test]
    fn lot_of_fortune_day_chart() {
        // Day chart: Lot of Fortune = ASC + Moon - Sun
        // ASC=0, Moon=120, Sun=30 → Fortune = 0 + 120 - 30 = 90
        let parts = arabic_parts_seven(0.0, 30.0, 120.0, 200.0, 40.0, 80.0, 50.0, 60.0, true);
        let fortune = &parts[0];
        assert_eq!(fortune.name, "Lot of Fortune");
        assert!(
            (fortune.degree - 90.0).abs() < 1e-9,
            "Fortune = {}",
            fortune.degree
        );
    }

    #[test]
    fn lot_of_fortune_night_chart_reversed() {
        // Night chart: Lot of Fortune = ASC + Sun - Moon (reversed)
        // ASC=0, Sun=30, Moon=120 → Fortune = 0 + 30 - 120 = 270 (mod 360)
        let parts = arabic_parts_seven(0.0, 30.0, 120.0, 200.0, 40.0, 80.0, 50.0, 60.0, false);
        let fortune = &parts[0];
        assert!(
            (fortune.degree - 270.0).abs() < 1e-9,
            "Night Fortune = {}",
            fortune.degree
        );
    }

    #[test]
    fn seven_parts_always_returned() {
        let parts = arabic_parts_seven(10.0, 20.0, 100.0, 200.0, 50.0, 80.0, 40.0, 30.0, true);
        assert_eq!(parts.len(), 7);
    }

    #[test]
    fn all_parts_in_0_360() {
        let parts = arabic_parts_seven(350.0, 10.0, 200.0, 100.0, 50.0, 75.0, 120.0, 300.0, true);
        for p in &parts {
            assert!(
                p.degree >= 0.0 && p.degree < 360.0,
                "{} = {} out of [0,360)",
                p.name,
                p.degree
            );
        }
    }
}

#[cfg(test)]
mod fixed_stars {
    use celestial_core::body::CalcFlags;
    use celestial_core::{fixstar_mag, fixstar_ut};

    #[test]
    fn regulus_near_leo() {
        // Regulus is at ~5° Virgo (due to precession from Leo)
        let pos = fixstar_ut("Regulus", 2_451_545.0, CalcFlags::BUILTIN).unwrap();
        let lon = pos.xx[0];
        assert!(
            lon >= 148.0 && lon <= 162.0,
            "Regulus lon = {lon:.2}° (expected near 150°-160°)"
        );
    }

    #[test]
    fn algol_near_taurus() {
        // Algol (Beta Persei) at ~26° Taurus
        let pos = fixstar_ut("Algol", 2_451_545.0, CalcFlags::BUILTIN).unwrap();
        let lon = pos.xx[0];
        assert!(
            lon >= 50.0 && lon <= 65.0,
            "Algol lon = {lon:.2}° (expected 50°-65° Gemini region at J2000)"
        );
    }

    #[test]
    fn star_magnitude_reasonable() {
        let mag = fixstar_mag("Sirius").unwrap();
        assert!(mag < 0.0, "Sirius should be brighter than mag 0, got {mag}");
        let mag_r = fixstar_mag("Regulus").unwrap();
        assert!(mag_r > 0.0 && mag_r < 3.0, "Regulus mag = {mag_r}");
    }

    #[test]
    fn unknown_star_returns_error() {
        let res = fixstar_ut("XyzNotAReal", 2_451_545.0, CalcFlags::BUILTIN);
        assert!(res.is_err(), "non-existent star should error");
    }
}
