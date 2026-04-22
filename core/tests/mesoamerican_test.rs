//! Tests extracted from new_features_test.rs — do not edit manually.

#[cfg(test)]
mod mesoamerican {
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

mod extra_mesoamerican {
    use celestial_core::*;

    const GMT: f64 = 584_283.0;

    #[test]
    fn tonalpohualli_cycle_length_260() {
        // Exactly 260 days later should give the same position
        let jd = GMT as f64;
        let (t0, s0, _, _) = tonalpohualli(jd);
        let (t260, s260, _, _) = tonalpohualli(jd + 260.0);
        assert_eq!(t0, t260, "trecena repeats at 260 days");
        assert_eq!(s0, s260, "sign repeats at 260 days");
    }

    #[test]
    fn tonalpohualli_trecena_range() {
        // trecena is always 1-13, sign_idx always 0-19
        for offset in 0..260i64 {
            let jd = GMT + offset as f64;
            let (t, s, _, _) = tonalpohualli(jd);
            assert!(t >= 1 && t <= 13, "trecena {t} out of range [1,13]");
            assert!(s < 20, "sign_idx {s} out of range [0,19]");
        }
    }

    #[test]
    fn tonalpohualli_day_one_is_cipactli() {
        let (t, s, name, _) = tonalpohualli(GMT as f64);
        assert_eq!(t, 1, "GMT day: trecena 1");
        assert_eq!(s, 0, "GMT day: sign 0 (Cipactli)");
        assert_eq!(name, "Cipactli", "GMT day: name Cipactli");
    }

    #[test]
    fn xiuhpohualli_365_day_cycle() {
        let jd = GMT as f64;
        let (m0, d0, _, _) = xiuhpohualli(jd);
        let (m365, d365, _, _) = xiuhpohualli(jd + 365.0);
        assert_eq!(m0, m365, "xiuhpohualli month repeats at 365 days");
        assert_eq!(d0, d365, "xiuhpohualli day repeats at 365 days");
    }

    #[test]
    fn tzolkin_matches_tonalpohualli_cycle() {
        // Tzolkin is the Maya equivalent of Tonalpohualli — same 260-day period
        let jd = GMT as f64 + 17.0;
        let (tt, _, _, _) = tonalpohualli(jd);
        let (tz, _, _, _) = tzolkin(jd);
        assert_eq!(
            tt, tz,
            "trecena should match between Tzolkin and Tonalpohualli"
        );
    }

    #[test]
    fn haab_365_day_cycle() {
        let jd = GMT as f64;
        let (m0, d0, _) = haab(jd);
        let (m365, d365, _) = haab(jd + 365.0);
        assert_eq!(m0, m365);
        assert_eq!(d0, d365);
    }

    #[test]
    fn calendar_round_18980_day_cycle() {
        // Calendar Round = LCM(260, 365) = 18980 days
        let jd = GMT as f64;
        let cr0 = calendar_round(jd);
        let cr18980 = calendar_round(jd + 18980.0);
        assert_eq!(cr0, cr18980, "Calendar Round repeats at 18980 days");
    }

    #[test]
    fn gmt_correlation_constant_value() {
        assert_eq!(
            GMT_CORRELATION, GMT as i64,
            "GMT correlation should be 584283"
        );
    }
}
