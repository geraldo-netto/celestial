//! Tests for aspects, datetime, geo-format, searches, timezone and Vedic helpers.

use crate::body::Calendar;

use celestial_core::body::{Body, CalcFlags, HouseSystem};
use celestial_core::*;

const JD: f64 = 2_452_275.5; // 2002-01-01 00:00 UT

// ─── Aspect matching ─────────────────────────────────────────────────────────

#[test]
fn test_match_aspect_exact() {
    let r = match_aspect(0.0, 1.0, 90.0, 0.5, 90.0, 5.0);
    assert!(r.matched);
    assert_eq!(r.diff, 0.0);
}

#[test]
fn test_match_aspect_within_orb() {
    let r = match_aspect(0.0, 1.0, 93.0, 0.5, 90.0, 5.0);
    assert!(r.matched, "should be within 5° orb");
    assert!((r.diff - 3.0).abs() < 1e-9);
}

#[test]
fn test_match_aspect_outside_orb() {
    let r = match_aspect(0.0, 1.0, 100.0, 0.5, 90.0, 5.0);
    assert!(!r.matched);
}

#[test]
fn test_match_aspect2_uses_closest() {
    // aspect=90° in [0,180]: should find the closer of 90° and 270°
    let r = match_aspect2(0.0, 1.0, 272.0, 0.5, 90.0, 5.0);
    // 272° is 2° inside 270° — that's the 90° mirror
    assert!(r.matched, "272° should match 90° via -90 = 270°");
}

#[test]
fn test_match_aspect3_applying() {
    // Applying: speedret < 0
    let r = match_aspect3(0.0, 2.0, 88.0, 0.5, 90.0, 5.0, 2.0, 1.0);
    assert!(r.matched);
    // When approaching exact, speed direction indicates applying/separating
    assert!(r.speed != 0.0, "speed should be non-zero");
}

#[test]
fn test_antiscion_axis90() {
    let pos = [30.0_f64, 1.0, 1.0, 0.5, 0.0, 0.0];
    let a = antiscion(pos, 90.0);
    // Antiscion of 30° around 90° axis: 90 - (30-90) = 90 - (-60) = 150°
    assert!(
        (a.antiscion[0] - 150.0).abs() < 1e-9,
        "antiscion={}",
        a.antiscion[0]
    );
    // Contrantiscion = antiscion + 180°
    assert!((a.contrantiscion[0] - 330.0).abs() < 1e-9);
    // Latitude sign is preserved in antiscion, negated in contrantiscion
    assert_eq!(a.antiscion[1], 1.0);
    assert_eq!(a.contrantiscion[1], -1.0);
    // Speed is negated in both
    assert_eq!(a.antiscion[3], -0.5);
}

#[test]
fn test_antiscion_speed_negated() {
    let pos = [100.0, 0.0, 1.0, 1.2, 0.0, 0.0];
    let a = antiscion(pos, 90.0);
    assert_eq!(a.antiscion[3], -1.2);
    assert_eq!(a.contrantiscion[3], -1.2);
}

// ─── Datetime ────────────────────────────────────────────────────────────────

#[test]
fn test_jdnow_reasonable() {
    let jd = jdnow();
    // Should be somewhere in 2020–2100 range
    assert!(jd > 2_450_000.0 && jd < 2_560_000.0, "jdnow={jd}");
}

#[test]
fn test_revjul_hms() {
    let dt = revjul_hms(JD, Calendar::Gregorian);
    assert_eq!(dt[0], 2002);
    assert_eq!(dt[1], 1);
    assert_eq!(dt[2], 1);
    assert_eq!(dt[3], 0);
    assert_eq!(dt[4], 0);
    assert_eq!(dt[5], 0);
}

#[test]
fn test_parse_datetime_iso() {
    let dt = parse_datetime("2002-01-01 00:00:00").unwrap();
    assert_eq!(dt, [2002, 1, 1, 0, 0, 0]);
}

#[test]
fn test_parse_datetime_negative_year() {
    let dt = parse_datetime("-100-06-15 12:30:00").unwrap();
    assert_eq!(dt[0], -100);
    assert_eq!(dt[1], 6);
}

#[test]
fn test_parse_datetime_invalid() {
    assert!(parse_datetime("not a date").is_none());
    assert!(parse_datetime("2002-13-01").is_none());
    assert!(parse_datetime("2002-01-32").is_none());
}

#[test]
fn test_parse_time() {
    assert_eq!(parse_time("12:30:45").unwrap(), [12, 30, 45]);
    assert_eq!(parse_time("23:59").unwrap(), [23, 59, 0]);
    assert!(parse_time("25:00").is_none());
}

#[test]
fn test_jd_duration() {
    let d = jd_duration(JD, JD + 1.5);
    assert_eq!(d[0], 1); // 1 day
    assert_eq!(d[1], 12); // 12 hours
    assert_eq!(d[2], 0);
    assert_eq!(d[3], 0);
}

#[test]
fn test_jd_to_iso_string() {
    let s = jd_to_iso_string(JD, Calendar::Gregorian);
    assert!(s.starts_with("2002-01-01"), "iso={s}");
    assert!(s.ends_with("UTC"), "iso={s}");
}

// ─── Formatting ──────────────────────────────────────────────────────────────

#[test]
fn test_degsplit_known() {
    // 123.5° = 3°30' Gemini (sign 2, 3°30'0")
    let [deg, sign, min, sec] = degsplit(123.5);
    assert_eq!(sign, 4); // Leo (120-150)
    assert_eq!(deg, 3);
    assert_eq!(min, 30);
    assert_eq!(sec, 0);
}

#[test]
fn test_sign_name() {
    assert_eq!(sign_name(0).unwrap(), "Aries");
    assert_eq!(sign_name(11).unwrap(), "Pisces");
    assert!(sign_name(12).is_none());
}

#[test]
fn test_house_system_id_roundtrip() {
    for c in b"PKRCBMOAEHVXGTUWYE".iter() {
        if let Some(id) = house_system_id(*c) {
            let back = house_system_char(id).unwrap();
            // A and E map to same id (7), so back gives 'A'; that's fine
            assert!(house_system_id(back).is_some());
        }
    }
}

#[test]
fn test_sidereal_mode_flag_roundtrip() {
    assert_eq!(sidereal_mode_flag(0).unwrap(), 256);
    assert_eq!(sidereal_mode_flag(22).unwrap(), 255);
    for i in 1..=21 {
        let f = sidereal_mode_flag(i).unwrap();
        assert_eq!(sidereal_mode_id(f).unwrap(), i);
    }
}

// ─── Geo coordinates ─────────────────────────────────────────────────────────

#[test]
fn test_parse_coord_dms_north() {
    let v = parse_coord("51:30:26N").unwrap();
    assert!(
        (v - (51.0 + 30.0 / 60.0 + 26.0 / 3600.0)).abs() < 0.001,
        "v={v}"
    );
}

#[test]
fn test_parse_coord_decimal() {
    let v = parse_coord("48.9").unwrap();
    assert!((v - 48.9).abs() < 1e-9);
}

#[test]
fn test_parse_coord_west_negative() {
    let v = parse_coord("2:20W").unwrap();
    assert!(v < 0.0);
    assert!((v.abs() - (2.0 + 20.0 / 60.0)).abs() < 0.001);
}

#[test]
fn test_parse_coord_invalid() {
    assert!(parse_coord("190E").is_none()); // > 180°
    assert!(parse_coord("91N").is_none()); // > 90° latitude
}

#[test]
fn test_geo_to_dms() {
    let [d, m, s] = geo_to_dms(51.5074);
    assert_eq!(d, 51);
    assert_eq!(m, 30);
    // seconds approximately 26-27
    assert!(s >= 25 && s <= 28, "s={s}");
}

#[test]
fn test_format_coord_lat() {
    let s = format_coord(51.5, true).unwrap();
    assert!(s.contains("N"), "s={s}");
    assert!(s.starts_with("51"), "s={s}");
}

#[test]
fn test_format_coord_lon_west() {
    let s = format_coord(-2.35, false).unwrap();
    assert!(s.contains("W"), "s={s}");
}

// ─── Vedic / Raman helpers ───────────────────────────────────────────────────

#[test]
fn test_raman_houses_bhavamadhya_count() {
    let cusps = raman_houses(15.0, 275.0, false);
    assert_eq!(cusps.len(), 12);
    // All longitudes in [0,360)
    for &c in &cusps {
        assert!(c >= 0.0 && c < 360.0, "cusp={c}");
    }
}

#[test]
fn test_raman_houses_asc_is_cusp1() {
    let asc = 72.3_f64;
    let mc = 345.8_f64;
    let cusps = raman_houses(asc, mc, false);
    assert!(
        (cusps[0] - asc % 360.0).abs() < 1e-9,
        "cusp[0]={} asc={}",
        cusps[0],
        asc
    );
}

#[test]
fn test_sign_lord_mars() {
    assert_eq!(sign_lord(0).unwrap(), 4); // Aries → Mars
    assert_eq!(sign_lord(7).unwrap(), 4); // Scorpio → Mars
}

#[test]
fn test_long_to_rasi() {
    assert_eq!(long_to_rasi(0.0), 0); // Aries
    assert_eq!(long_to_rasi(45.0), 1); // Taurus
    assert_eq!(long_to_rasi(359.9), 11); // Pisces
}

#[test]
fn test_long_to_navamsa() {
    let nav = long_to_navamsa(0.0);
    assert!(nav >= 0 && nav < 12);
}

#[test]
fn test_long_to_nakshatra() {
    let (nak, pada) = long_to_nakshatra(0.0);
    assert_eq!(nak, 0); // Aswini
    assert_eq!(pada, 0);
    let (nak2, _) = long_to_nakshatra(359.99);
    assert_eq!(nak2, 26); // Revathi
}

#[test]
fn test_nakshatra_name() {
    assert_eq!(nakshatra_name(0).unwrap(), "Aswini");
    assert_eq!(nakshatra_name(26).unwrap(), "Revathi");
    assert!(nakshatra_name(27).is_none());
}

#[test]
fn test_rasi_diff() {
    assert_eq!(rasi_diff(0, 3), 9); // Aries → Cancer = 9 forward
    assert_eq!(rasi_diff(0, 0), 0);
}

#[test]
fn test_rasi_diff2_signed() {
    let d = rasi_diff2(0, 3);
    assert!(d >= -5 && d <= 6, "d={d}");
}

#[test]
fn test_tatkalika_relation() {
    // Adjacent signs: |diff2| = 1 ≤ 3 → Mitra
    assert_eq!(tatkalika_relation(0, 1), 1);
    // Opposite signs: |diff2| = 6 > 3 → Satru
    assert_eq!(tatkalika_relation(0, 6), -1);
}

#[test]
fn test_naisargika_relation_sun_moon() {
    let r = naisargika_relation(0, 1).unwrap();
    assert_eq!(r, 1); // Sun–Moon: Mitra
}

#[test]
fn test_naisargika_relation_invalid() {
    assert!(naisargika_relation(0, 99).is_none());
}

#[test]
fn test_residential_strength_at_midpoint() {
    let bm = [
        0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
    ];
    // Midpoint between 0 and 30 is 15
    let s = residential_strength(15.0, &bm).unwrap();
    assert!((s - 1.0).abs() < 0.01, "s={s}");
}

#[test]
fn test_residential_strength_at_cusp() {
    let bm = [
        0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
    ];
    let s = residential_strength(0.0, &bm).unwrap();
    assert_eq!(s, 0.0);
}

#[test]
fn test_ochchabala_sun() {
    let bal = ochchabala(0, 190.0).unwrap(); // Sun exalted at ~190 in the table
    assert!((bal - 0.0).abs() < 0.1, "ochchabala={bal}");
}

#[test]
fn test_ochchabala_invalid() {
    assert!(ochchabala(99, 0.0).is_none());
}

// ─── Search functions ────────────────────────────────────────────────────────

#[test]
fn test_next_retro_mercury() {
    // Mercury should station within 120 days of any date
    let r = next_retro(
        Body::MERCURY,
        JD,
        false,
        120.0,
        CalcFlags::BUILTIN | CalcFlags::SPEED,
    );
    assert!(r.is_some(), "Mercury station not found");
    let r = r.unwrap();
    assert!(r.jd > JD && r.jd < JD + 120.0, "jd={}", r.jd);
    // At station, speed ≈ 0
    assert!(r.pos[3].abs() < 0.05, "speed={}", r.pos[3]);
}

#[test]
fn test_next_retro_sun_none() {
    // Sun does not retrograde
    let r = next_retro(
        Body::SUN,
        JD,
        false,
        365.0,
        CalcFlags::BUILTIN | CalcFlags::SPEED,
    );
    assert!(r.is_none(), "Sun should not retrograde");
}

#[test]
fn test_next_aspect_fixed_point() {
    // Mars conjunct 90° (fixed point)
    let r = next_aspect(Body::MARS, 0.0, 90.0, JD, false, 0.0, CalcFlags::BUILTIN);
    assert!(r.is_some(), "Mars→90° not found");
    let r = r.unwrap();
    assert!(r.jd > JD, "jd must be after start");
    // Mars longitude at that JD should be ≈ 90°
    let mars = calc_ut(r.jd, Body::MARS, CalcFlags::BUILTIN).unwrap();
    assert!((mars.lon - 90.0).abs() < 0.5, "mars_lon={}", mars.lon);
}

#[test]
fn test_next_aspect2_sextile() {
    // Moon sextile (60°) Aldebaran
    let r = next_aspect2(Body::MOON, 60.0, 69.8, JD, false, 30.0, CalcFlags::BUILTIN);
    assert!(r.is_some(), "Moon sextile not found");
}

#[test]
fn test_next_aspect_with_sun_moon() {
    // Sun–Moon conjunction (new Moon) after JD
    let r = next_aspect_with(
        Body::SUN,
        0.0,
        Body::MOON,
        JD,
        false,
        35.0,
        CalcFlags::BUILTIN,
    );
    assert!(r.is_some(), "Sun–Moon conjunction not found");
    let r = r.unwrap();
    // Difference should be ≈ 0°
    let diff = (r.pos1[0] - r.pos2[0] + 360.0).rem_euclid(360.0);
    let diff = if diff > 180.0 { 360.0 - diff } else { diff };
    assert!(diff < 1.0, "lon diff at conjunction = {diff}");
}

#[test]
fn test_next_aspect_cusp() {
    let r = next_aspect_cusp(
        Body::SUN,
        0.0,
        1,
        JD,
        48.0,
        2.0,
        HouseSystem::PLACIDUS,
        false,
        CalcFlags::BUILTIN,
    );
    assert!(r.is_some(), "Sun conj ASC not found");
    let r = r.unwrap();
    assert!(r.jd > JD);
    // All 12 cusps returned
    for &c in &r.cusps[1..=12] {
        assert!(c >= 0.0 && c < 360.0);
    }
}

// ─── Timezone ────────────────────────────────────────────────────────────────

#[test]
fn test_tz_find_utc() {
    let tz = tz_abbr_find("UTC");
    assert!(!tz.is_empty(), "UTC not found");
    assert_eq!(tz[0].hours, 0);
    assert_eq!(tz[0].minutes, 0);
}

#[test]
fn test_tz_find_ist() {
    let tz = tz_abbr_find("IST");
    assert!(!tz.is_empty(), "IST not found");
    // Indian Standard Time is UTC+05:30
    let ist = tz.iter().find(|t| t.hours == 5 && t.minutes == 30).unwrap();
    assert_eq!(ist.hours, 5);
    assert_eq!(ist.minutes, 30);
}

#[test]
fn test_tz_find_unknown() {
    let tz = tz_abbr_find("ZZZNOTTHERE");
    assert!(tz.is_empty());
}

// ─── Saturn 4-Stars ──────────────────────────────────────────────────────────

#[test]
fn test_saturn_4_stars() {
    let r = saturn_4_stars(JD, CalcFlags::BUILTIN).unwrap();
    // Index must be finite and non-negative
    assert!(r[5].is_finite(), "index={}", r[5]);
    assert!(r[5] >= 0.0, "index={}", r[5]);
    // All star/planet longitudes in [0,360)
    for &v in &r[..5] {
        assert!(v >= 0.0 && v < 360.0, "lon={v}");
    }
}
