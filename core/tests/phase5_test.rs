//! Tests extracted from new_features_test.rs — do not edit manually.

#[cfg(test)]
mod phase5_dignities {
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
mod phase5_firdaria {
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
mod phase5_profections {
    use celestial_core::body::HouseSystem;
    use celestial_core::*;

    #[test]
    fn annual_profection_age_0_is_house_1() {
        let jd = 2_451_545.0;
        let h = houses(jd, 48.85, 2.35, HouseSystem::PLACIDUS).unwrap();
        let cusps: [f64; 13] = {
            let mut a = [0.0f64; 13];
            for i in 0..13 {
                a[i] = h.cusps[i];
            }
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
        let h = houses(jd, 48.85, 2.35, HouseSystem::PLACIDUS).unwrap();
        let cusps: [f64; 13] = {
            let mut a = [0.0f64; 13];
            for i in 0..13 {
                a[i] = h.cusps[i];
            }
            a
        };
        let (h0, _) = annual_profection(&cusps, 0);
        let (h12, _) = annual_profection(&cusps, 12);
        assert_eq!(h0, h12, "age 0 and age 12 should be the same house");
    }

    #[test]
    fn annual_profection_house_number_in_range() {
        let jd = 2_451_545.0;
        let h = houses(jd, 0.0, 0.0, HouseSystem::PLACIDUS).unwrap();
        let cusps: [f64; 13] = {
            let mut a = [0.0f64; 13];
            for i in 0..13 {
                a[i] = h.cusps[i];
            }
            a
        };
        for age in 0..36u32 {
            let (house, _) = annual_profection(&cusps, age);
            assert!(
                house >= 1 && house <= 12,
                "profection house {house} out of [1,12] for age {age}"
            );
        }
    }
}
