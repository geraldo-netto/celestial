//! Tests extracted from new_features_test.rs — do not edit manually.

#[cfg(test)]
mod phase7_mesoamerican {
    use celestial_core::*;

    #[test]
    fn tonalpohualli_trecena_in_range() {
        for i in 0..260u32 {
            let jd = 2_451_545.0 + i as f64;
            let (trecena, sign_idx, _, _) = tonalpohualli(jd);
            assert!(
                trecena >= 1 && trecena <= 13,
                "trecena {trecena} out of [1,13] at day {i}"
            );
            assert!(sign_idx < 20, "sign {sign_idx} >= 20 at day {i}");
        }
    }

    #[test]
    fn tonalpohualli_260_day_cycle() {
        let jd = 2_451_545.0;
        let (t1, s1, _, _) = tonalpohualli(jd);
        let (t2, s2, _, _) = tonalpohualli(jd + 260.0);
        assert_eq!(t1, t2, "trecena should repeat after 260 days");
        assert_eq!(s1, s2, "sign should repeat after 260 days");
    }

    #[test]
    fn xiuhpohualli_365_day_cycle() {
        let jd = 2_451_545.0;
        let (m1, d1, _, _) = xiuhpohualli(jd);
        let (m2, d2, _, _) = xiuhpohualli(jd + 365.0);
        assert_eq!(m1, m2, "month should repeat after 365 days");
        assert_eq!(d1, d2, "day should repeat after 365 days");
    }

    #[test]
    fn tzolkin_same_structure_as_tonalpohualli() {
        // Tzolkin and Tonalpohualli share the 260-day base
        let jd = 2_451_545.0;
        let (tt, ts, _, _) = tzolkin(jd);
        let (at, as_, _, _) = tonalpohualli(jd);
        // Same trecena and same sign index (both use day_num % 260)
        assert_eq!(tt, at, "tzolkin and tonalpohualli trecena should match");
        assert_eq!(ts, as_, "tzolkin and tonalpohualli sign index should match");
    }

    #[test]
    fn haab_365_day_cycle() {
        let jd = 2_451_545.0;
        let (m1, d1, _) = haab(jd);
        let (m2, d2, _) = haab(jd + 365.0);
        assert_eq!(m1, m2);
        assert_eq!(d1, d2);
    }

    #[test]
    fn calendar_round_52_year_cycle() {
        // 18_980 days = LCM(260, 365) = 52 Haab years
        let jd = 2_451_545.0;
        let (t1, s1, hd1, hm1) = calendar_round(jd);
        let (t2, s2, hd2, hm2) = calendar_round(jd + 18_980.0);
        assert_eq!(t1, t2, "Calendar Round trecena");
        assert_eq!(s1, s2, "Calendar Round tzolkin sign");
        assert_eq!(hd1, hd2, "Calendar Round haab day");
        assert_eq!(hm1, hm2, "Calendar Round haab month");
    }

    #[test]
    fn tonalpohualli_signs_table_has_20() {
        assert_eq!(TONALPOHUALLI_SIGNS.len(), 20);
    }
}
