//! Tests extracted from new_features_test.rs — do not edit manually.

#[cfg(test)]
mod hellenistic_dignities {
    use celestial_core::body::Body;
    use celestial_core::*;

    #[test]
    fn egyptian_terms_cover_every_degree() {
        // Every longitude 0-360 must return one of the 5 traditional planets
        let trad = [
            Body::SATURN,
            Body::JUPITER,
            Body::MARS,
            Body::VENUS,
            Body::MERCURY,
        ];
        for i in 0..360u32 {
            let lon = i as f64 + 0.5;
            let ruler = egyptian_terms_ruler(lon);
            assert!(
                trad.contains(&ruler),
                "lon {lon}: terms ruler {ruler:?} not a traditional planet"
            );
        }
    }

    #[test]
    fn decan_rulers_cycle_correctly() {
        // 36 decans of 10° each; ruler at 0° should be Mars (Aries decan 1)
        assert_eq!(decan_ruler(0.0), Body::MARS, "Aries 1st decan = Mars");
        assert_eq!(decan_ruler(10.0), Body::SUN, "Aries 2nd decan = Sun");
        assert_eq!(decan_ruler(20.0), Body::VENUS, "Aries 3rd decan = Venus");
        assert_eq!(
            decan_ruler(30.0),
            Body::MERCURY,
            "Taurus 1st decan = Mercury"
        );
    }

    #[test]
    fn triplicity_rulers_fire_signs() {
        // Aries (0°), Leo (120°), Sagittarius (240°) are fire signs
        for lon in [15.0_f64, 135.0, 255.0] {
            let (day, night, _part) = triplicity_rulers(lon);
            assert_eq!(day, Body::SUN, "fire day ruler = Sun at {lon}");
            assert_eq!(night, Body::JUPITER, "fire night ruler = Jupiter at {lon}");
        }
    }

    #[test]
    fn triplicity_rulers_water_signs() {
        // Cancer (90°), Scorpio (210°), Pisces (330°) are water
        for lon in [105.0_f64, 225.0, 345.0] {
            let (day, night, _part) = triplicity_rulers(lon);
            assert_eq!(day, Body::VENUS, "water day ruler = Venus at {lon}");
            assert_eq!(night, Body::MARS, "water night ruler = Mars at {lon}");
        }
    }

    #[test]
    fn full_dignity_sun_in_aries_is_exaltation() {
        // Sun is exalted in Aries (15° Aries is the exact degree)
        let (dig, score) = full_dignity(Body::SUN, 15.0, true);
        assert_eq!(
            dig.to_string(),
            "exaltation",
            "Sun at 15° Aries should be exalted"
        );
        assert_eq!(score, 4);
    }

    #[test]
    fn full_dignity_sun_in_leo_is_domicile() {
        let (dig, score) = full_dignity(Body::SUN, 135.0, true); // 15° Leo
        assert_eq!(dig.to_string(), "domicile");
        assert_eq!(score, 5);
    }

    #[test]
    fn full_dignity_sun_in_libra_is_fall() {
        let (dig, score) = full_dignity(Body::SUN, 195.0, true); // 15° Libra
        assert_eq!(dig.to_string(), "fall");
        assert_eq!(score, -4);
    }

    #[test]
    fn full_dignity_sun_in_aquarius_is_detriment() {
        let (dig, score) = full_dignity(Body::SUN, 315.0, true); // 15° Aquarius
        assert_eq!(dig.to_string(), "detriment");
        assert_eq!(score, -5);
    }

    #[test]
    fn almuten_returns_one_of_seven_planets() {
        let trad = [
            Body::SUN,
            Body::MOON,
            Body::MERCURY,
            Body::VENUS,
            Body::MARS,
            Body::JUPITER,
            Body::SATURN,
        ];
        for i in 0..36u32 {
            let lon = i as f64 * 10.0 + 5.0;
            let (body, _score) = almuten(lon, true);
            assert!(
                trad.contains(&body),
                "almuten at {lon}° returned {body:?}, not a traditional planet"
            );
        }
    }

    #[test]
    fn same_sect_day_chart() {
        // Sun, Jupiter, Saturn are diurnal
        assert!(same_sect(Body::SUN, true));
        assert!(same_sect(Body::JUPITER, true));
        assert!(same_sect(Body::SATURN, true));
        // Moon, Venus, Mars are nocturnal — out of sect in a day chart
        assert!(!same_sect(Body::MOON, true));
        assert!(!same_sect(Body::VENUS, true));
        assert!(!same_sect(Body::MARS, true));
    }

    #[test]
    fn same_sect_night_chart() {
        assert!(same_sect(Body::MOON, false));
        assert!(same_sect(Body::VENUS, false));
        assert!(same_sect(Body::MARS, false));
        assert!(!same_sect(Body::SUN, false));
        assert!(!same_sect(Body::JUPITER, false));
        assert!(!same_sect(Body::SATURN, false));
    }
}

#[cfg(test)]
mod firdaria {
    use celestial_core::body::Body;
    use celestial_core::*;

    #[test]
    fn firdaria_day_starts_with_sun() {
        let jd = 2_451_545.0;
        let periods = firdaria(jd, true, 10.0);
        assert!(!periods.is_empty());
        assert_eq!(
            periods[0].major_lord,
            Body::SUN,
            "day chart Firdaria should start with Sun"
        );
    }

    #[test]
    fn firdaria_night_starts_with_moon() {
        let jd = 2_451_545.0;
        let periods = firdaria(jd, false, 10.0);
        assert!(!periods.is_empty());
        assert_eq!(
            periods[0].major_lord,
            Body::MOON,
            "night chart Firdaria should start with Moon"
        );
    }

    #[test]
    fn firdaria_periods_are_chronological() {
        let jd = 2_451_545.0;
        let periods = firdaria(jd, true, 75.0);
        for w in periods.windows(2) {
            assert!(
                w[0].end <= w[1].start + 1e-6,
                "Firdaria period end {:.2} > next start {:.2}",
                w[0].end,
                w[1].start
            );
        }
    }

    #[test]
    fn firdaria_start_is_birth_jd() {
        let jd = 2_451_545.0;
        let periods = firdaria(jd, true, 5.0);
        assert!(
            (periods[0].start - jd).abs() < 1e-6,
            "first Firdaria period should start at birth JD"
        );
    }

    #[test]
    fn firdaria_minor_duration_divides_major() {
        let jd = 2_451_545.0;
        let periods = firdaria(jd, true, 12.0);
        // Find the first complete major period (Sun = 10 years)
        let sun_minor: Vec<_> = periods
            .iter()
            .filter(|p| p.major_lord == Body::SUN)
            .collect();
        if sun_minor.len() == 7 {
            let total: f64 = sun_minor.iter().map(|p| p.years).sum();
            assert!(
                (total - 10.0).abs() < 0.1,
                "Sun major period sub-periods should sum to ~10 years, got {total:.2}"
            );
        }
    }

    #[test]
    fn firdaria_span_respected() {
        let jd = 2_451_545.0;
        for span in [10.0_f64, 30.0, 75.0] {
            let periods = firdaria(jd, true, span);
            if let Some(last) = periods.last() {
                // Allow up to 2 years of overshoot (sub-period rounding)
                assert!(
                    last.end <= jd + (span + 2.0) * 365.26,
                    "last period end exceeds span {span}y by more than 2 years"
                );
            }
        }
    }
}

#[cfg(test)]
mod profections {
    use celestial_core::body::HouseSystem;
    use celestial_core::*;

    #[test]
    fn annual_profection_age_0_is_house_1() {
        let jd = 2_451_545.0;
        let h = houses(JulianDay::new(jd), Latitude::new(48.85), Longitude::new(2.35), HouseSystem::PLACIDUS).unwrap();
        let cusps: [f64; 13] = {
            let mut a = [0.0f64; 13];
            a.copy_from_slice(&h.cusps);
            a
        };
        let (house, lon) = annual_profection(&cusps, 0);
        assert_eq!(house, 1, "age 0 profection = house 1 (ASC)");
        assert!(
            (lon - h.ascmc[0]).abs() < 1e-6,
            "age 0 profection lon should equal ASC"
        );
    }

    #[test]
    fn annual_profection_age_12_returns_to_house_1() {
        let jd = 2_451_545.0;
        let h = houses(JulianDay::new(jd), Latitude::new(48.85), Longitude::new(2.35), HouseSystem::PLACIDUS).unwrap();
        let cusps: [f64; 13] = {
            let mut a = [0.0f64; 13];
            a.copy_from_slice(&h.cusps);
            a
        };
        let (h0, _) = annual_profection(&cusps, 0);
        let (h12, _) = annual_profection(&cusps, 12);
        assert_eq!(h0, h12, "age 0 and age 12 should be the same house");
    }

    #[test]
    fn annual_profection_house_number_in_range() {
        let jd = 2_451_545.0;
        let h = houses(JulianDay::new(jd), Latitude::new(0.0), Longitude::new(0.0), HouseSystem::PLACIDUS).unwrap();
        let cusps: [f64; 13] = {
            let mut a = [0.0f64; 13];
            a.copy_from_slice(&h.cusps);
            a
        };
        for age in 0..36u32 {
            let (house, _) = annual_profection(&cusps, age);
            assert!(
                (1..=12).contains(&house),
                "profection house {house} out of [1,12] for age {age}"
            );
        }
    }
}

mod is_day_chart {
    use celestial_core::*;

    #[test]
    fn is_day_chart_sun_above_horizon() {
        // Sun at 180° (Libra 0°) with ASC at 0° — Sun is in house 7, above horizon
        let cusps = [
            0.0_f64, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
        ];
        assert!(is_day_chart(180.0, &cusps));
        assert!(!is_day_chart(0.0, &cusps)); // Sun at ASC — below horizon (night)
    }

    #[test]
    fn egyptian_terms_boundaries() {
        // Aries: Jupiter 0-6°, Venus 6-12°, Mercury 12-20°
        assert_eq!(egyptian_terms_ruler(3.0), Body::JUPITER);
        assert_eq!(egyptian_terms_ruler(9.0), Body::VENUS);
        assert_eq!(egyptian_terms_ruler(15.0), Body::MERCURY);
        // Wraps at 360°
        assert_eq!(egyptian_terms_ruler(360.0), egyptian_terms_ruler(0.0));
    }

    #[test]
    fn decan_ruler_chaldean_sequence() {
        // Aries decans: Mars (0-10°), Sun (10-20°), Venus (20-30°)
        assert_eq!(decan_ruler(5.0), Body::MARS);
        assert_eq!(decan_ruler(15.0), Body::SUN);
        assert_eq!(decan_ruler(25.0), Body::VENUS);
        // Taurus first decan: Mercury
        assert_eq!(decan_ruler(35.0), Body::MERCURY);
    }

    #[test]
    fn triplicity_rulers_fire_signs() {
        // Aries is a fire sign: day=Sun, night=Jupiter, participating=Saturn
        let (day, night, part) = triplicity_rulers(5.0); // Aries 5°
        assert_eq!(day, Body::SUN);
        assert_eq!(night, Body::JUPITER);
        assert_eq!(part, Body::SATURN);
    }

    #[test]
    fn full_dignity_domicile() {
        // Sun in Leo (120-150°) = domicile
        let (dig, score) = full_dignity(Body::SUN, 125.0, true);
        assert_eq!(dig, Dignity::Domicile);
        assert!(score >= 5);
    }

    #[test]
    fn full_dignity_detriment() {
        // Sun in Aquarius (300-330°) = detriment
        let (dig, score) = full_dignity(Body::SUN, 315.0, true);
        assert_eq!(dig, Dignity::Detriment);
        assert!(score < 0);
    }

    #[test]
    fn almuten_returns_valid_body() {
        let (body, score) = almuten(15.0, true); // Aries 15°
        assert!(score >= 0);
        let name = planet_name(body);
        assert!(!name.is_empty());
    }

    #[test]
    fn firdaria_day_chart_starts_with_sun() {
        let jd = 2_451_545.0;
        let periods = firdaria(jd, true, 75.0);
        assert!(!periods.is_empty());
        assert_eq!(periods[0].major_lord, Body::SUN);
        // Day chart: Sun 10y, Venus 8y, Mercury 13y...
        let total: f64 = periods.iter().map(|p| p.years).sum();
        assert!((total - 75.0).abs() < 0.5, "total span should be ~75y");
    }

    #[test]
    fn firdaria_night_chart_starts_with_moon() {
        let jd = 2_451_545.0;
        let periods = firdaria(jd, false, 75.0);
        assert_eq!(periods[0].major_lord, Body::MOON);
    }

    #[test]
    fn annual_profection_house_rotation() {
        let cusps = [
            0.0_f64, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
        ];
        let (h0, _) = annual_profection(&cusps, 0);
        let (h1, _) = annual_profection(&cusps, 1);
        let (h12, _) = annual_profection(&cusps, 12);
        assert_eq!(h0, 1, "age 0 → house 1");
        assert_eq!(h1, 2, "age 1 → house 2");
        assert_eq!(h12, 1, "age 12 → house 1 (wraps)");
        assert_eq!(annual_profection(&cusps, 35).0, 12, "age 35 → house 12");
    }

    #[test]
    fn same_sect_sun_day_chart() {
        assert!(same_sect(Body::SUN, true), "Sun in sect by day");
        assert!(same_sect(Body::JUPITER, true), "Jupiter in sect by day");
        assert!(same_sect(Body::SATURN, true), "Saturn in sect by day");
        assert!(!same_sect(Body::MOON, true), "Moon out of sect by day");
        assert!(!same_sect(Body::VENUS, true), "Venus out of sect by day");
        assert!(!same_sect(Body::MARS, true), "Mars out of sect by day");
    }
}

mod profection_rotation {
    use celestial_core::*;

    // Fixed cusp array for all profection tests: signs start at 0°, 30°, 60°…
    fn sample_cusps() -> [f64; 13] {
        [
            0.0, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
        ]
    }

    #[test]
    fn annual_profection_first_twelve_ages() {
        let c = sample_cusps();
        for age in 0u32..12 {
            let (house, _) = annual_profection(&c, age);
            assert_eq!(house as u32, age + 1, "age {age} → house {}", age + 1);
        }
    }

    #[test]
    fn annual_profection_wraps_at_12() {
        let c = sample_cusps();
        for age in 0u32..60 {
            let (h, _) = annual_profection(&c, age);
            let expected = (age % 12) + 1;
            assert_eq!(h as u32, expected, "age {age}");
        }
    }

    #[test]
    fn monthly_profection_advances_within_house() {
        let c = sample_cusps();
        let (h0, lon0) = monthly_profection(&c, 0, 0);
        let (h1, lon1) = monthly_profection(&c, 0, 1);
        assert_eq!(h0, 1, "month 0 starts in house 1");
        assert_eq!(h1, 1, "month 1 still in house 1");
        assert!(lon1 > lon0, "profected lon increases each month");
    }

    #[test]
    fn monthly_profection_crosses_house_after_12_months() {
        let c = sample_cusps();
        let (h0, _) = monthly_profection(&c, 0, 0);
        let (h12, _) = monthly_profection(&c, 1, 0);
        assert_eq!(h0, 1);
        assert_eq!(h12, 2, "age 1 starts in house 2");
    }
}

mod extra_hellenistic {
    use celestial_core::*;

    #[test]
    fn full_dignity_exaltation() {
        // Sun exalted in Aries (0-30°)
        let (dig, score) = full_dignity(Body::SUN, 15.0, true);
        assert_eq!(dig, Dignity::Exaltation);
        assert!(score >= 4);
    }

    #[test]
    fn full_dignity_fall() {
        // Sun in fall in Libra (180-210°) — opposite exaltation
        let (dig, score) = full_dignity(Body::SUN, 195.0, true);
        assert_eq!(dig, Dignity::Fall);
        assert!(score < 0);
    }

    #[test]
    fn planet_on_midpoint_within_orb() {
        use celestial_core::planet_on_midpoint;
        // Planet at 15°, midpoint at 15° — exact hit
        let hit = planet_on_midpoint(15.0, 15.0, 1.5);
        assert!(hit.is_some());
        assert!(hit.unwrap().abs() < 0.001);
        // Planet at 20°, midpoint at 15°, orb 1.5° — miss
        assert!(planet_on_midpoint(20.0, 15.0, 1.5).is_none());
    }

    #[test]
    fn default_orb_luminaries_are_wider() {
        use celestial_core::default_orb;
        let sun_orb = default_orb(Body::SUN, Body::MOON, 0.0); // conjunction
        let sat_orb = default_orb(Body::SATURN, Body::MARS, 0.0);
        assert!(sun_orb > sat_orb, "luminaries get wider orbs");
    }

    #[test]
    fn triplicity_rulers_all_signs_non_empty() {
        for sign in 0..12 {
            let lon = sign as f64 * 30.0 + 1.0;
            let (d, n, p) = triplicity_rulers(lon);
            let name_d = planet_name(d);
            let name_n = planet_name(n);
            let name_p = planet_name(p);
            assert!(!name_d.is_empty(), "sign {sign} day ruler empty");
            assert!(!name_n.is_empty(), "sign {sign} night ruler empty");
            assert!(!name_p.is_empty(), "sign {sign} part ruler empty");
        }
    }

    #[test]
    fn firdaria_periods_non_overlapping() {
        let jd = 2_451_545.0;
        let periods = firdaria(jd, true, 75.0);
        for w in periods.windows(2) {
            assert!(
                (w[1].start - w[0].end).abs() < 0.01,
                "gap between firdaria periods"
            );
        }
    }
}
