//! Tests extracted from new_features_test.rs — do not edit manually.

#[cfg(test)]
mod bazi {
    use celestial_core::body::{Body, CalcFlags};
    use celestial_core::*;

    #[test]
    fn four_pillars_returns_four() {
        let jd = 2_451_545.0;
        let sun = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN).unwrap();
        let pillars = four_pillars(jd, 12.0, sun.lon);
        assert_eq!(pillars.len(), 4, "must return 4 pillars");
    }

    #[test]
    fn pillars_stems_in_range() {
        let jd = 2_451_545.0;
        let sun = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN).unwrap();
        let pillars = four_pillars(jd, 12.0, sun.lon);
        for p in &pillars {
            assert!(p.stem < 10, "stem {} >= 10", p.stem);
            assert!(p.branch < 12, "branch {} >= 12", p.branch);
        }
    }

    #[test]
    fn stem_element_is_one_of_five() {
        let jd = 2_451_545.0;
        let sun = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN).unwrap();
        let pillars = four_pillars(jd, 12.0, sun.lon);
        let elements = ["Wood", "Fire", "Earth", "Metal", "Water"];
        for p in &pillars {
            assert!(
                elements.contains(&p.stem_element),
                "stem_element '{}' not valid",
                p.stem_element
            );
            assert!(
                elements.contains(&p.branch_element),
                "branch_element '{}' not valid",
                p.branch_element
            );
        }
    }

    #[test]
    fn animal_is_one_of_twelve() {
        let jd = 2_451_545.0;
        let sun = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN).unwrap();
        let pillars = four_pillars(jd, 12.0, sun.lon);
        let animals = [
            "Rat", "Ox", "Tiger", "Rabbit", "Dragon", "Snake", "Horse", "Goat", "Monkey",
            "Rooster", "Dog", "Pig",
        ];
        for p in &pillars {
            assert!(
                animals.contains(&p.animal),
                "animal '{}' not in zodiac",
                p.animal
            );
        }
    }

    #[test]
    fn stem_polarity_matches_yang_flag() {
        // Even stems (index 0,2,4,6,8) are Yang; odd are Yin
        for (i, &(_, _, yang)) in HEAVENLY_STEMS.iter().enumerate() {
            let expected_yang = i % 2 == 0;
            assert_eq!(
                yang, expected_yang,
                "stem {i}: yang={yang} but expected {expected_yang}"
            );
        }
    }

    #[test]
    fn make_pillar_wraps_correctly() {
        let p = make_pillar(0, 0);
        assert_eq!(p.stem_name, "Jiǎ");
        assert_eq!(p.animal, "Rat");
        // Overflow wraps
        let p2 = make_pillar(10, 12);
        assert_eq!(p2.stem, 10); // stored as-is
        assert_eq!(p2.stem_name, "Jiǎ"); // 10 % 10 = 0
        assert_eq!(p2.animal, "Rat"); // 12 % 12 = 0
    }

    #[test]
    fn sexagenary_name_first_is_jiazi() {
        let (stem, animal) = sexagenary_name(0);
        assert_eq!(stem, "Jiǎ");
        assert_eq!(animal, "Rat");
    }

    #[test]
    fn sexagenary_name_wraps_at_60() {
        // Cycle 60 == cycle 0
        let (s0, a0) = sexagenary_name(0);
        let (s60, a60) = sexagenary_name(60);
        assert_eq!(s0, s60);
        assert_eq!(a0, a60);
    }
}

#[cfg(test)]
mod solar_terms {
    // Body available via celestial_core::* glob above
    use celestial_core::*;

    #[test]
    fn solar_terms_has_24_entries() {
        assert_eq!(SOLAR_TERMS.len(), 24);
    }

    #[test]
    fn solar_terms_longitudes_ascending() {
        for w in SOLAR_TERMS.windows(2) {
            assert!(
                w[0].0 < w[1].0,
                "solar term longitudes must ascend: {:.0} >= {:.0}",
                w[0].0,
                w[1].0
            );
        }
    }

    #[test]
    fn solar_terms_span_360() {
        assert!((SOLAR_TERMS.first().unwrap().0 - 0.0).abs() < 1e-9);
        assert!((SOLAR_TERMS.last().unwrap().0 - 345.0).abs() < 1e-9);
    }

    #[test]
    fn solar_term_position_at_spring_equinox() {
        // Sun at 0° = Chūnfēn (Spring Equinox) = term index 0
        let (idx, deg_into, next_idx, _) = solar_term_position(0.0);
        assert_eq!(idx, 0, "0° should be Chūnfēn (index 0)");
        assert!(deg_into.abs() < 1e-9);
        assert_eq!(next_idx, 1);
    }

    #[test]
    fn solar_term_position_at_summer_solstice() {
        // Summer solstice = 90° = index 6 (Xiàzhì)
        let (idx, _, _, _) = solar_term_position(90.0);
        assert_eq!(idx, 6, "90° = Summer Solstice (index 6)");
        assert_eq!(SOLAR_TERMS[6].1, "Xiàzhì");
    }

    #[test]
    fn solar_term_position_in_range() {
        for lon in (0..360u32).map(|i| i as f64 + 0.5) {
            let (cur, deg_into, next, deg_to) = solar_term_position(lon);
            assert!(cur < 24, "current term index {cur} >= 24");
            assert!(next < 24, "next term index {next} >= 24");
            assert!(deg_into >= 0.0, "deg_into {deg_into} < 0");
            assert!(deg_to > 0.0, "deg_to {deg_to} <= 0 at lon {lon}");
        }
    }

    #[test]
    fn spring_begins_at_315() {
        // Lìchūn (Start of Spring) = 315° = index 21
        let (idx, _, _, _) = solar_term_position(315.5);
        assert_eq!(
            SOLAR_TERMS[idx].2, "Start of Spring",
            "315° should be Start of Spring, got {}",
            SOLAR_TERMS[idx].2
        );
    }
}

mod extra_chinese {
    use celestial_core::*;

    #[test]
    fn four_pillars_returns_four() {
        let jd = 2_451_545.0; // J2000.0
        let p = four_pillars(jd, 12.0, 280.0); // Sun ~280° at J2000
        assert_eq!(p.len(), 4);
        // Each pillar must have non-empty names
        for pillar in &p {
            assert!(!pillar.stem_name.is_empty(), "stem name empty");
            assert!(!pillar.branch_name.is_empty(), "branch name empty");
            assert!(!pillar.animal.is_empty(), "animal name empty");
        }
    }

    #[test]
    fn four_pillars_stems_in_range() {
        let jd = 2_451_545.0;
        let p = four_pillars(jd, 9.0, 280.0);
        for pillar in &p {
            assert!(
                HEAVENLY_STEMS.iter().any(|s| s.0 == pillar.stem_name),
                "stem '{}' not in HEAVENLY_STEMS",
                pillar.stem_name
            );
        }
    }

    #[test]
    fn four_pillars_branches_in_range() {
        let jd = 2_451_545.0;
        let p = four_pillars(jd, 9.0, 280.0);
        for pillar in &p {
            assert!(
                EARTHLY_BRANCHES.iter().any(|b| b.0 == pillar.branch_name),
                "branch '{}' not in EARTHLY_BRANCHES",
                pillar.branch_name
            );
        }
    }

    #[test]
    fn solar_term_all_24_positions() {
        for (i, &(lon, _, _)) in SOLAR_TERMS.iter().enumerate() {
            let (cur, into, _next, _to) = solar_term_position(lon + 1.0);
            assert_eq!(
                cur, i,
                "term {i}: position at lon={:.0}°+1 should be term {i}",
                lon
            );
            assert!(
                into >= 0.0 && into < 15.5,
                "deg_into={into:.2} out of range"
            );
        }
    }

    #[test]
    fn solar_term_boundary_wrap() {
        // At exactly 345° (last term boundary), should be in last term or wrap
        let (cur, _, _, _) = solar_term_position(345.0);
        assert!(cur < 24, "term index out of range");
    }

    #[test]
    fn sexagenary_name_cycle_length() {
        // The 60-cycle: index 60 should equal index 0
        let (stem0, animal0) = sexagenary_name(0);
        let (stem60, animal60) = sexagenary_name(60);
        assert_eq!(stem0, stem60, "stem wraps at 60");
        assert_eq!(animal0, animal60, "animal wraps at 60");
    }

    #[test]
    fn sexagenary_name_jiazi_is_first() {
        let (stem, animal) = sexagenary_name(0);
        assert_eq!(stem, "Jiǎ", "first stem should be Jiǎ");
        assert_eq!(animal, "Rat", "first branch should be Rat");
    }

    #[test]
    fn make_pillar_roundtrip() {
        for stem in 0u8..10 {
            for branch in 0u8..12 {
                let p = make_pillar(stem, branch);
                assert_eq!(p.stem_name, HEAVENLY_STEMS[stem as usize].0);
                assert_eq!(p.branch_name, EARTHLY_BRANCHES[branch as usize].0);
                assert_eq!(p.yang, HEAVENLY_STEMS[stem as usize].2);
            }
        }
    }
}
