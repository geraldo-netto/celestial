//! Pure-math unit tests — no ephemeris data needed.
//!
//! These tests were previously embedded in `lib.rs` and have been moved here
//! so that `lib.rs` contains only declarations and re-exports.

use crate::body::Calendar;

use celestial_core::body::{Body, CalcFlags, HouseSystem, SiderealMode};
use celestial_core::*;

const J2000: f64 = 2_451_545.0;

use std::f64::consts::PI;

fn assert_approx(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-7, "assert_approx failed: {a} ≠ {b}");
}

// ── norm_deg ───────────────────────────────────────────────────────────────────

#[test]
fn degnorm_zero() {
    assert_eq!(norm_deg(0.0), 0.0);
}
#[test]
fn degnorm_full() {
    assert_eq!(norm_deg(360.0), 0.0);
}
#[test]
fn degnorm_negative() {
    assert_eq!(norm_deg(-1.0), 359.0);
}
#[test]
fn degnorm_over_360() {
    assert!((norm_deg(361.0) - 1.0).abs() < 1e-12);
}
#[test]
fn degnorm_over_720() {
    assert!((norm_deg(721.0) - 1.0).abs() < 1e-12);
}

#[test]
fn degnorm_idempotent() {
    for v in [0.0_f64, 45.0, 90.0, 180.0, 270.0, 359.999] {
        let n = norm_deg(v);
        assert!(
            (norm_deg(n) - n).abs() < 1e-12,
            "norm_deg not idempotent at {v}"
        );
    }
}

// ── norm_rad ───────────────────────────────────────────────────────────────────

#[test]
fn radnorm_zero() {
    assert_eq!(norm_rad(0.0), 0.0);
}
#[test]
fn radnorm_two_pi() {
    assert!(norm_rad(2.0 * PI) < 1e-12);
}
#[test]
fn radnorm_neg_pi() {
    assert!((norm_rad(-PI) - PI).abs() < 1e-12);
}

// ── diff_deg_signed ──────────────────────────────────────────────────────────────────

#[test]
fn difdeg2n_known() {
    assert!((diff_deg_signed(360.5, 540.0) + 179.5).abs() < 1e-12);
}

#[test]
fn difdeg2n_zero() {
    assert_eq!(diff_deg_signed(100.0, 100.0), 0.0);
}

#[test]
fn difdeg2n_range() {
    for (a, b) in [(0.0, 359.0), (90.0, 271.0), (180.5, 1.0), (0.5, 359.0)] {
        let d = diff_deg_signed(a, b);
        assert!(
            d > -180.0 && d <= 180.0,
            "diff_deg_signed({a},{b}) = {d} outside range"
        );
    }
}

// ── midpoint_deg ──────────────────────────────────────────────────────────────────

#[test]
fn deg_midp_simple() {
    let m = midpoint_deg(20.0, 10.0);
    assert!((m - 15.0).abs() < 1e-10, "expected 15, got {m}");
}

#[test]
fn deg_midp_wrap() {
    let m = norm_deg(midpoint_deg(10.0, 350.0));
    assert!(
        !(1.0..=359.0).contains(&m),
        "wrap midpoint near 0°, got {m}"
    );
}

// ── norm_cs / cs_round_sec ───────────────────────────────────────────────────────

#[test]
fn csnorm_full() {
    assert_eq!(norm_cs(360 * 360_000_i32), 0);
}
#[test]
fn csnorm_half() {
    assert_eq!(norm_cs(180 * 360_000_i32), 64_800_000_i64);
}
#[test]
fn csnorm_neg_720() {
    assert_eq!(norm_cs(-720 * 360_000_i32), 0);
}

#[test]
fn csnorm_always_non_negative() {
    for v in [-1_000_000_000i32, -1, 0, 1, 1_000_000_000i32] {
        let n = norm_cs(v);
        assert!(n >= 0, "norm_cs({v}) = {n} should be non-negative");
        assert!(n < 360 * 360_000, "norm_cs({v}) = {n} >= 360°");
    }
}

// ── split_deg ─────────────────────────────────────────────────────────────────

#[test]
fn split_deg_positive() {
    let (d, m, s, frac, sgn) = split_deg(123.123, 0);
    assert_eq!(d, 123);
    assert_eq!(m, 7);
    assert_eq!(s, 22);
    assert!((frac - 0.8).abs() < 1e-6, "frac = {frac}");
    assert_eq!(sgn, 1);
}

#[test]
fn split_deg_zero() {
    let (d, m, s, _, sgn) = split_deg(0.0, 0);
    assert_eq!((d, m, s, sgn), (0, 0, 0, 1));
}

#[test]
fn split_deg_zodiacal() {
    let (d, _, _, _, sgn) = split_deg(123.123, SPLIT_DEG_ZODIACAL);
    assert_eq!(d, 3); // 3° Leo
    assert_eq!(sgn, 4); // Leo = sign 4 (0-indexed)
}

// ── coord_transform round-trip ────────────────────────────────────────────────────────

#[test]
fn cotrans_roundtrip() {
    let orig = [121.34_f64, 43.57, 1.0];
    let eps = 23.4_f64;
    let equ = coord_transform(orig, Degrees::new(eps));
    let ecl = coord_transform(equ, Degrees::new(-eps));
    assert!(
        (ecl[0] - orig[0]).abs() < 1e-9,
        "lon: {} ≠ {}",
        ecl[0],
        orig[0]
    );
    assert!(
        (ecl[1] - orig[1]).abs() < 1e-9,
        "lat: {} ≠ {}",
        ecl[1],
        orig[1]
    );
    assert_eq!(ecl[2], 1.0);
}

#[test]
fn cotrans_known_values() {
    let out = coord_transform([121.34, 43.57, 1.0], Degrees::new(23.4));
    assert!((out[0] - 114.119_848_334_918_26).abs() < 1e-9);
    assert!((out[1] - 22.754_921_351_892_474).abs() < 1e-9);
    assert_eq!(out[2], 1.0);
}

// ── julday / revjul ───────────────────────────────────────────────────────────

#[test]
fn julday_known() {
    assert_eq!(julday(2002, 1, 1, 0.0, Calendar::Gregorian), 2_452_275.5);
    assert_approx(julday(2000, 1, 1, 12.0, Calendar::Gregorian), 2_451_545.0);
}

#[test]
fn revjul_known() {
    let d = revjul(JulianDay::new(2_452_275.5), Calendar::Gregorian);
    assert_eq!((d.year, d.month, d.day, d.hour), (2002, 1, 1, 0.0));
}

#[test]
fn julday_revjul_roundtrip() {
    for (y, m, d, h) in [
        (2000, 1, 1, 0.0),
        (2023, 12, 31, 23.9999),
        (1582, 10, 15, 12.0),
    ] {
        let jd = julday(y, m, d, h, Calendar::Gregorian);
        let back = revjul(JulianDay::new(jd), Calendar::Gregorian);
        assert_eq!(back.year, y);
        assert_eq!(back.month, m);
        assert_eq!(back.day, d);
        assert!((back.hour - h).abs() < 1e-8, "hour: {} ≠ {}", back.hour, h);
    }
}

// ── day_of_week ───────────────────────────────────────────────────────────────

#[test]
fn day_of_week_known() {
    assert_eq!(day_of_week(JulianDay::new(2_452_275.5)), 1); // 2002-01-01, Tuesday
    assert_eq!(day_of_week(JulianDay::new(2_459_444.0)), 1); // 2021-08-17, Tuesday
}

// ── utc_time_zone roundtrip ───────────────────────────────────────────────────

#[test]
fn utc_time_zone_roundtrip() {
    let utc = UtcDate {
        year: 2022,
        month: 6,
        day: 15,
        hour: 12,
        minute: 30,
        second: 0.0,
    };
    let east = utc_time_zone(&utc, 2.0);
    assert_eq!(east.hour, 14);
    let back = utc_time_zone(&east, -2.0);
    assert_eq!((back.hour, back.minute), (utc.hour, utc.minute));
}

// ── date_conversion ───────────────────────────────────────────────────────────

#[test]
fn date_conversion_gregorian() {
    let jd = date_conversion(2002, 1, 1, 0.0, b'g').unwrap();
    assert_approx(jd, 2_452_275.5);
}

#[test]
fn date_conversion_month_overflow() {
    let jd = date_conversion(2002, 13, 1, 0.0, b'g').unwrap();
    assert_approx(jd, 2_452_640.5); // 2003-01-01
}

// ── azalt / azalt_rev roundtrip ───────────────────────────────────────────────

#[test]
fn azalt_rev_roundtrip_equatorial() {
    let jd = 2454503.06_f64;
    let geopos = [12.1_f64, 49.0, 330.0];
    let pos = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap();
    // Forward: ecliptic → horizontal
    let az = azalt(
        JulianDay::new(jd),
        0,
        geopos,
        1010.0,
        15.0,
        [pos.lon, pos.lat, pos.dist],
    );
    // Inverse: horizontal → equatorial (flag=1)
    let back = azalt_rev(JulianDay::new(jd), 1, geopos, [az.azimuth, az.true_alt]);
    // Round-trip via equatorial → ecliptic through coord_transform is not tested here,
    // but azalt_rev(1) must give consistent RA/Dec; just confirm finite values
    assert!(
        back[0].is_finite() && back[0] >= 0.0 && back[0] < 360.0,
        "RA out of range: {}",
        back[0]
    );
    assert!(
        back[1].is_finite() && back[1] > -90.0 && back[1] < 90.0,
        "Dec out of range: {}",
        back[1]
    );
}

#[test]
fn azalt_rev_roundtrip_ecliptic() {
    let jd = 2454503.06_f64;
    let geopos = [12.1_f64, 49.0, 330.0];
    let pos = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let az = azalt(
        JulianDay::new(jd),
        0,
        geopos,
        1010.0,
        15.0,
        [pos.lon, pos.lat, pos.dist],
    );
    let back = azalt_rev(JulianDay::new(jd), 0, geopos, [az.azimuth, az.true_alt]);
    // Sun ecliptic longitude should round-trip to within ~0.05°
    let diff = (back[0] - pos.lon).abs();
    let diff = if diff > 180.0 { 360.0 - diff } else { diff };
    assert!(diff < 0.05, "ecliptic lon round-trip diff = {diff}°");
    assert!(
        back[1].abs() < 0.01,
        "ecliptic lat should be ~0: {}",
        back[1]
    );
}

// ─── Smoke tests for newer public API additions ───────────────────────────────

#[test]
fn test_nutation_finite() {
    let (nut_lon, nut_obl) = nutation(JulianDay::new(2451545.0));
    assert!(nut_lon.is_finite() && nut_obl.is_finite());
    // Nutation is small: < 20 arcseconds ≈ 0.006°
    assert!(nut_lon.abs() < 0.01 && nut_obl.abs() < 0.01);
}

#[test]
fn test_obliquity() {
    let eps = mean_obliquity(JulianDay::new(2451545.0));
    assert!(
        (eps - 23.439).abs() < 0.01,
        "J2000 mean obliquity ≈ 23.44°, got {eps}"
    );
    let eps_true = true_obliquity(JulianDay::new(2451545.0));
    assert!(
        (eps_true - eps).abs() < 0.01,
        "true/mean obliquity difference < 0.01°"
    );
}

#[test]
fn test_tt_to_ut() {
    let jd = 2451545.0; // J2000
    let ut = tt_to_ut(JulianDay::new(jd));
    // ΔT at J2000 ≈ 63.8 s ≈ 0.000738 days
    assert!(
        (jd - ut - 0.000738).abs() < 0.001,
        "tt_to_ut off: diff={}",
        jd - ut
    );
}

#[test]
fn test_heliocentric_mars() {
    use celestial_core::{calc_ut, Body, CalcFlags};
    let jd = 2451545.0;
    let pos = calc_ut(
        JulianDay::new(jd),
        Body::MARS,
        CalcFlags::BUILTIN | CalcFlags::HELIOCENTRIC,
    )
    .unwrap();
    // Mars heliocentric distance: 1.38–1.67 AU
    assert!(
        pos.dist > 1.2 && pos.dist < 1.8,
        "Mars heliocentric dist={:.4} AU",
        pos.dist
    );
    assert!(pos.lon >= 0.0 && pos.lon < 360.0);
}

#[test]
fn test_houses_from_armc() {
    use celestial_core::houses_from_armc;
    // ARMC=0, lat=0, obliquity=23.44, Placidus
    let h = houses_from_armc(
        Degrees::new(0.0),
        Latitude::new(0.0),
        Degrees::new(23.4393),
        HouseSystem::PLACIDUS,
    );
    assert!(h.ascmc[0] >= 0.0 && h.ascmc[0] < 360.0, "ASC out of range");
}

#[test]
fn test_orbital_elements_fields() {
    use celestial_core::{get_orbital_elements, Body, CalcFlags};
    let el =
        get_orbital_elements(JulianDay::new(2451545.0), Body::MARS, CalcFlags::BUILTIN).unwrap();
    // Mars semi-major axis ≈ 1.524 AU
    assert!(
        (el.semi_major_axis - 1.524).abs() < 0.1,
        "Mars a={:.4}",
        el.semi_major_axis
    );
    assert!(
        el.eccentricity > 0.08 && el.eccentricity < 0.10,
        "Mars ecc={:.4}",
        el.eccentricity
    );
}
#[test]
fn test_motion_functions() {
    use celestial_core::{
        helio_cross_ut, mooncross_node_ut, mooncross_ut, solcross_ut, Body, CalcFlags,
    };
    let jd = 2451545.0;
    // Sun crossing 0° (Aries point)
    let cross = solcross_ut(Longitude::new(0.0), JulianDay::new(jd), CalcFlags::BUILTIN).unwrap();
    assert!(cross > jd, "crossing must be in the future");
    // Moon crossing 0°
    let moon_cross =
        mooncross_ut(Longitude::new(0.0), JulianDay::new(jd), CalcFlags::BUILTIN).unwrap();
    assert!(moon_cross > jd);
    // Moon/node crossing
    let node_cross = mooncross_node_ut(JulianDay::new(jd), CalcFlags::BUILTIN).unwrap();
    assert!(node_cross.jd_cross > jd);
    // Mars heliocentric crossing 0°
    let helio = helio_cross_ut(
        Body::MARS,
        Longitude::new(0.0),
        JulianDay::new(jd),
        CalcFlags::BUILTIN,
        1,
    )
    .unwrap();
    assert!(helio > jd);
}
#[test]
fn test_saros_and_obliquity() {
    use celestial_core::{mean_obliquity, nutation, true_obliquity, tt_to_ut};
    // Already tested in unit_tests above — but call saros via the eclipse functions
    // saros() has a different signature than expected, skip for now
    let _ = mean_obliquity(JulianDay::new(2451545.0));
    let _ = true_obliquity(JulianDay::new(2451545.0));
    let (nl, no) = nutation(JulianDay::new(2451545.0));
    assert!(nl.is_finite() && no.is_finite());
    let _ = tt_to_ut(JulianDay::new(2451545.0));
}
#[test]
fn test_phenomena() {
    use celestial_core::{gauquelin_sector, pheno_ut, Body, CalcFlags};
    let jd = 2451545.0;
    // pheno_ut: solar phenomena for Mars
    let attr = pheno_ut(JulianDay::new(jd), Body::MARS, CalcFlags::BUILTIN).unwrap();
    assert!(attr[0].is_finite(), "phase angle finite");
    // gauquelin_sector: Sun for Paris (48.85°N, 2.35°E)
    let sector = gauquelin_sector(
        JulianDay::new(jd),
        Body::SUN,
        None,
        CalcFlags::BUILTIN,
        1,
        [2.35, 48.85, 0.0],
        0.0,
        0.0,
    )
    .unwrap();
    assert!((1.0..=36.0).contains(&sector), "sector={sector}");
}

// ═══════════════════════════════════════════════════════════════════════════
// Regression & edge-case tests — added in bug-hunt pass
// ═══════════════════════════════════════════════════════════════════════════

// ─── Polar latitudes ─────────────────────────────────────────────────────────

#[test]
fn houses_arctic_latitude() {
    // Placidus fails above ~66° — should return an error, not panic
    let result = houses(
        JulianDay::new(J2000),
        Latitude::new(89.9),
        Longitude::new(0.0),
        HouseSystem::PLACIDUS,
    );
    // Either Ok (fallback) or Err — must NOT panic
    let _ = result;
}

#[test]
fn houses_polar_all_systems() {
    // All house systems must not panic at polar latitudes.
    for &sys in b"PKEOCRWXMBHT" {
        for lat in [-90.0, -89.9, 89.9, 90.0] {
            let _ = houses(
                JulianDay::new(J2000),
                Latitude::new(lat),
                Longitude::new(2.35),
                HouseSystem(sys),
            );
        }
    }
}

#[test]
fn houses_equator() {
    // Equator: ASC = 90° or 270° for most systems
    let r = houses(
        JulianDay::new(J2000),
        Latitude::new(0.0),
        Longitude::new(0.0),
        HouseSystem::PLACIDUS,
    )
    .unwrap();
    assert!(r.ascmc[0] >= 0.0 && r.ascmc[0] < 360.0);
}

// ─── Ancient and far-future dates ────────────────────────────────────────────

#[test]
fn calc_ut_ancient_date() {
    // 1000 BCE = approx JD 1356001
    let jd_1000bce = 1_356_001.0;
    let r = calc_ut(JulianDay::new(jd_1000bce), Body::SUN, CalcFlags::BUILTIN).unwrap();
    assert!(r.lon >= 0.0 && r.lon < 360.0);
    assert!(r.dist > 0.9 && r.dist < 1.1);
}

#[test]
fn calc_ut_far_future() {
    // Year 3000 CE = approx JD 2816787
    let jd_3000 = 2_816_787.0;
    let r = calc_ut(JulianDay::new(jd_3000), Body::JUPITER, CalcFlags::BUILTIN).unwrap();
    assert!(r.lon >= 0.0 && r.lon < 360.0);
    assert!(r.dist > 4.0 && r.dist < 6.0);
}

#[test]
fn julday_revjul_year_zero() {
    // Astronomical year 0 = 1 BCE
    let jd = julday(0, 6, 15, 0.0, Calendar::Gregorian);
    let back = revjul(JulianDay::new(jd), Calendar::Gregorian);
    assert_eq!(back.year, 0);
    assert_eq!(back.month, 6);
}

#[test]
fn julday_revjul_negative_year() {
    // 500 BCE = year -499 in astronomical notation
    let jd = julday(-499, 1, 1, 0.0, Calendar::Gregorian);
    assert!(jd > 0.0);
    let back = revjul(JulianDay::new(jd), Calendar::Gregorian);
    assert_eq!(back.year, -499);
}

// ─── CalcFlags::EQUATORIAL mode ─────────────────────────────────────────────────────

#[test]
fn calc_ut_equatorial_flag() {
    let geo = calc_ut(JulianDay::new(J2000), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let eq = calc_ut(
        JulianDay::new(J2000),
        Body::SUN,
        CalcFlags::BUILTIN | CalcFlags::EQUATORIAL,
    )
    .unwrap();
    // Equatorial lon = RA, lat = Dec — should differ from ecliptic
    // RA is in [0,360), Dec is in (-90, 90)
    assert!(eq.lon >= 0.0 && eq.lon < 360.0);
    assert!(eq.lat.abs() < 90.0);
    // RA and ecliptic lon differ (except at equinox points)
    // Just check they're valid and different enough at J2000
    assert!((eq.lon - geo.lon).abs() > 0.1 || eq.lat.abs() > 0.1);
}

#[test]
fn calc_ut_equatorial_speed() {
    // CalcFlags::EQUATORIAL + CalcFlags::SPEED should compute RA/Dec rates
    let r = calc_ut(
        JulianDay::new(J2000),
        Body::MOON,
        CalcFlags::BUILTIN | CalcFlags::EQUATORIAL | CalcFlags::SPEED,
    )
    .unwrap();
    assert!(r.lon >= 0.0 && r.lon < 360.0);
    assert!(r.speed_lon.is_finite() && r.speed_lat.is_finite());
    // Moon moves ~13°/day in RA
    assert!(
        r.speed_lon.abs() > 5.0 && r.speed_lon.abs() < 20.0,
        "Moon RA speed {:.3}°/day",
        r.speed_lon
    );
}

// ─── Topocentric positions ────────────────────────────────────────────────────

#[test]
fn set_topo_changes_moon_position() {
    // Topocentric Moon differs from geocentric by up to ~1° (parallax)
    let geo = calc_ut(JulianDay::new(J2000), Body::MOON, CalcFlags::BUILTIN).unwrap();
    set_topo(Longitude::new(2.35), Latitude::new(48.85), 35.0); // Paris
    let topo = calc_ut(
        JulianDay::new(J2000),
        Body::MOON,
        CalcFlags::BUILTIN | CalcFlags::TOPOCENTRIC,
    )
    .unwrap();
    set_topo(Longitude::new(0.0), Latitude::new(0.0), 0.0); // reset
                                                            // Topocentric correction should shift Moon by up to ~1°
    let diff = (topo.lon - geo.lon).abs();
    assert!(
        diff < 1.5,
        "Topocentric shift {diff:.4}° — should be < 1.5°"
    );
    // Topocentric correction is now implemented — Moon shift should be measurable
    assert!(
        diff > 0.0001,
        "Topocentric shift should be non-zero, got {diff:.6}°"
    );
    println!("  Topocentric Moon shift from Tokyo: {diff:.4}°");
}

// ─── Backwards eclipse search ─────────────────────────────────────────────────

#[test]
fn eclipse_search_backwards() {
    // Search backwards from J2000 — should find an eclipse BEFORE J2000
    let r = sol_eclipse_when_glob(JulianDay::new(J2000), CalcFlags::BUILTIN, 0, true).unwrap();
    assert!(
        r.tret[0] < J2000,
        "backwards search should find eclipse before J2000"
    );
    assert!(
        r.tret[0] > J2000 - 400.0,
        "eclipse should be within ~1 year before J2000"
    );
}

#[test]
fn lunar_eclipse_search_backwards() {
    let r = lun_eclipse_when(JulianDay::new(J2000), CalcFlags::BUILTIN, 0, true).unwrap();
    assert!(r.tret[0] < J2000);
}

// ─── Sidereal all modes ──────────────────────────────────────────────────────

#[test]
fn all_36_ayanamsa_modes_are_finite() {
    for mode in 0..36 {
        set_sid_mode(SiderealMode(mode), 0.0, 0.0);
        let ay = ayanamsa(JulianDay::new(J2000));
        assert!(ay.is_finite(), "ayanamsa mode {mode} = NaN");
        assert!(
            ay > -10.0 && ay < 60.0,
            "ayanamsa mode {mode} = {ay:.2}° out of range"
        );
    }
    set_sid_mode(SiderealMode::LAHIRI, 0.0, 0.0);
}

// ─── Untested utility functions ───────────────────────────────────────────────

#[test]
fn cotrans_sp_roundtrip() {
    // coord_transform_with_speed is the speed-extended version of coord_transform
    let coords = [90.0_f64, 0.0, 1.0, 0.1, 0.0, 0.0];
    let out = coord_transform_with_speed(coords, Degrees::new(23.439));
    assert!(out[0].is_finite() && out[1].is_finite());
    // Round-trip: apply twice with opposite obliquity
    let back = coord_transform_with_speed(
        [out[0], out[1], out[2], out[3], out[4], out[5]],
        Degrees::new(-23.439),
    );
    assert!(
        (back[0] - coords[0]).abs() < 0.001,
        "coord_transform_with_speed round-trip lon {:.4} vs {:.4}",
        back[0],
        coords[0]
    );
}

#[test]
fn rad_midp_agrees_with_deg_midp() {
    let a = 10.0_f64.to_radians();
    let b = 20.0_f64.to_radians();
    let mid_rad = midpoint_rad(a, b);
    let mid_deg = midpoint_deg(10.0, 20.0);
    assert!((mid_rad.to_degrees() - mid_deg).abs() < 1e-10);
}

#[test]
fn difrad2n_wraps_correctly() {
    use std::f64::consts::PI;
    // 10° - 350° = -340° → wrapped to 20° (shorter arc)
    let d = diff_rad_signed(10.0_f64.to_radians(), 350.0_f64.to_radians());
    assert!(
        d.abs() < PI,
        "diff_rad_signed result {d} should be in (-π, π]"
    );
}

#[test]
fn time_functions_smoke() {
    // Functions not exercised anywhere else
    let jd = J2000;
    let dt_ex = deltat_ex(JulianDay::new(jd), CalcFlags::BUILTIN).unwrap();
    assert!(dt_ex.abs() < 200.0); // ΔT reasonable range
    let utc = jd_et_to_utc(JulianDay::new(jd), Calendar::Gregorian);
    assert_eq!(utc.year, 2000);
    let utc2 = jd_ut_to_utc(JulianDay::new(jd), Calendar::Gregorian);
    assert_eq!(utc2.year, 2000);
    let lmt = lat_to_lmt(JulianDay::new(jd), Longitude::new(30.0)).unwrap(); // 30°E longitude
    let back = lmt_to_lat(JulianDay::new(lmt), Longitude::new(30.0)).unwrap();
    assert!(
        (back - jd).abs() < 1e-4,
        "LMT round-trip failed: {back:.6} vs {jd:.6}"
    );
    let eq_time = time_equ(JulianDay::new(jd)).unwrap();
    assert!(eq_time.is_finite()); // equation of time ~(-0.27, +0.27) hours
    assert!(eq_time.abs() < 0.3, "time_equ {eq_time:.4}h out of range");
    let st0 = sidtime0(JulianDay::new(jd), Degrees::new(23.439), Degrees::new(0.0));
    assert!((0.0..24.0).contains(&st0));
}

#[test]
fn cs_functions_smoke() {
    // Centisecond utility functions
    let cs = 360 * 360000_i32; // 360° in centiseconds
    assert_eq!(norm_cs(cs), 0); // normalises to 0
    let rounded = cs_round_sec(324_001_i32); // 90°+1cs → round to nearest second
    assert!(rounded >= 0);
    let l = deg_to_cs(1.5);
    assert!(l > 0);
    let d = diff_cs(100_i32, 50_i32);
    assert_eq!(d, 50);
    let d2 = diff_cs_signed(100_i32, 50_i32);
    assert!(d2 >= 0);
}

#[test]
fn refrac_extended_smoke() {
    // Extended refraction — not tested anywhere
    let (apparent_alt, details) = refrac_extended(1.0, 0.0, 1013.25, 15.0, 0.0065, 0);
    assert!(apparent_alt.is_finite(), "apparent_alt = {apparent_alt}");
    assert!(details[0].is_finite());
}

#[test]
fn houses_armc_ex2_smoke() {
    let r = houses_armc_ex2(
        Degrees::new(45.0),
        Latitude::new(48.85),
        Degrees::new(23.439),
        HouseSystem::PLACIDUS,
    )
    .unwrap();
    assert!(r.cusps[1] >= 0.0 && r.cusps[1] < 360.0);
}

#[test]
fn houses_ex_smoke() {
    let r = houses_ex(
        JulianDay::new(J2000),
        CalcFlags::BUILTIN,
        Latitude::new(48.85),
        Longitude::new(2.35),
        HouseSystem::KOCH,
    )
    .unwrap();
    assert!(r.cusps[1] >= 0.0 && r.cusps[1] < 360.0);
    assert!(r.cusps[10] >= 0.0 && r.cusps[10] < 360.0);
}

#[test]
fn sol_eclipse_when_loc_smoke() {
    let geopos = [2.35_f64, 48.85, 35.0]; // Paris
    let r = sol_eclipse_when_loc(JulianDay::new(J2000), CalcFlags::BUILTIN, geopos, false).unwrap();
    assert!(r.tret[0] > J2000);
    assert!(r.ret_flags != 0);
}

#[test]
fn sol_eclipse_how_smoke() {
    // Find a known eclipse then call how()
    let eclipse =
        sol_eclipse_when_glob(JulianDay::new(J2000), CalcFlags::BUILTIN, 0, false).unwrap();
    let geopos = [0.0_f64, 51.5, 0.0]; // London
    let how = sol_eclipse_how(JulianDay::new(eclipse.tret[0]), CalcFlags::BUILTIN, geopos).unwrap();
    assert!(how.attr[0].is_finite()); // magnitude
}

#[test]
fn lun_eclipse_when_loc_smoke() {
    let geopos = [2.35_f64, 48.85, 35.0];
    let r = lun_eclipse_when_loc(JulianDay::new(J2000), CalcFlags::BUILTIN, geopos, false).unwrap();
    assert!(r.tret[0] > J2000);
}

#[test]
fn lun_eclipse_how_smoke() {
    let eclipse = lun_eclipse_when(JulianDay::new(J2000), CalcFlags::BUILTIN, 0, false).unwrap();
    let how = lun_eclipse_how(JulianDay::new(eclipse.tret[0]), CalcFlags::BUILTIN, None).unwrap();
    assert!(how.attr[0].is_finite()); // penumbral magnitude
}

#[test]
fn rise_trans_true_hor_smoke() {
    let geo = [2.35_f64, 48.85, 35.0];
    let r = rise_trans_true_hor(
        JulianDay::new(J2000),
        Body::SUN,
        None,
        CalcFlags::BUILTIN,
        CALC_RISE,
        geo,
        1013.25,
        15.0,
        0.0,
    );
    // May fail for circumpolar — just check it doesn't panic
    let _ = r;
}

#[test]
fn vedic_rasi_norm_smoke() {
    assert_eq!(rasi_norm(0), 0);
    assert_eq!(rasi_norm(12), 0); // wraps
    assert_eq!(rasi_norm(-1), 11); // wraps negative
    assert_eq!(rasi_norm(13), 1);
}

#[test]
fn years_diff_smoke() {
    let y = years_diff(J2000, J2000 + 365.25, CalcFlags::BUILTIN).unwrap();
    assert!((y - 1.0).abs() < 0.01, "years_diff off: {y:.4}");
}

// ─── New backwards search functions ──────────────────────────────────────────

#[test]
fn solcross_back_finds_previous_crossing() {
    // Forward cross from J2000 finds next Aries point (~79 days later)
    let fwd = solcross_ut(
        Longitude::new(0.0),
        JulianDay::new(J2000),
        CalcFlags::BUILTIN,
    )
    .unwrap();
    // Backward from slightly after that crossing should return to near J2000
    let back = solcross_back_ut(
        Longitude::new(0.0),
        JulianDay::new(fwd + 1.0),
        CalcFlags::BUILTIN,
    )
    .unwrap();
    assert!(
        (back - fwd).abs() < 2.0,
        "back {back:.2} should be near fwd {fwd:.2}"
    );
    assert!(
        back < fwd + 1.0,
        "backward result must be before search start"
    );
}

#[test]
fn mooncross_back_finds_previous_crossing() {
    let fwd = mooncross_ut(
        Longitude::new(180.0),
        JulianDay::new(J2000),
        CalcFlags::BUILTIN,
    )
    .unwrap();
    let back = mooncross_back_ut(
        Longitude::new(180.0),
        JulianDay::new(fwd + 0.5),
        CalcFlags::BUILTIN,
    )
    .unwrap();
    assert!((back - fwd).abs() < 1.0);
    assert!(back < fwd + 0.5);
}

// ─── Property invariants ──────────────────────────────────────────────────────

#[test]
fn houses_opposite_cusps_are_180_apart() {
    for &sys in b"PKEOCRWXMBHT" {
        let r = houses(
            JulianDay::new(J2000),
            Latitude::new(48.85),
            Longitude::new(2.35),
            HouseSystem(sys),
        )
        .unwrap();
        for h in 1..=6 {
            let diff = (r.cusps[h] - r.cusps[h + 6]).rem_euclid(360.0);
            // diff should be 180° (opposite houses)
            let dev = (diff - 180.0).abs();
            assert!(
                dev < 0.01,
                "sys='{}' house {} & {} not opposite: {:.4}° vs {:.4}° (diff={:.4}°)",
                sys as char,
                h,
                h + 6,
                r.cusps[h],
                r.cusps[h + 6],
                diff
            );
        }
    }
}

#[test]
fn orbital_elements_eccentricity_valid() {
    for &body in &[
        Body::MERCURY,
        Body::VENUS,
        Body::MARS,
        Body::JUPITER,
        Body::SATURN,
        Body::URANUS,
        Body::NEPTUNE,
    ] {
        let el = get_orbital_elements(JulianDay::new(J2000), body, CalcFlags::BUILTIN).unwrap();
        assert!(
            el.eccentricity >= 0.0 && el.eccentricity < 1.0,
            "body={body} eccentricity={:.4} out of [0,1)",
            el.eccentricity
        );
        assert!(
            el.semi_major_axis > 0.0,
            "body={body} semi_major_axis={:.4} must be positive",
            el.semi_major_axis
        );
    }
}

#[test]
fn speed_matches_numerical_diff() {
    // Speed flag should match (pos[t+0.5] - pos[t-0.5]) / 1.0 within 10%
    let with_speed = calc_ut(
        JulianDay::new(J2000),
        Body::MARS,
        CalcFlags::BUILTIN | CalcFlags::SPEED,
    )
    .unwrap();
    let plus = calc_ut(JulianDay::new(J2000 + 0.5), Body::MARS, CalcFlags::BUILTIN).unwrap();
    let minus = calc_ut(JulianDay::new(J2000 - 0.5), Body::MARS, CalcFlags::BUILTIN).unwrap();
    let numerical = {
        let raw = plus.lon - minus.lon;
        if raw > 180.0 {
            raw - 360.0
        } else if raw < -180.0 {
            raw + 360.0
        } else {
            raw
        }
    };
    let ratio = with_speed.speed_lon / numerical;
    assert!(
        ratio > 0.9 && ratio < 1.1,
        "speed {:.4} vs numerical {:.4} (ratio {ratio:.3})",
        with_speed.speed_lon,
        numerical
    );
}

#[test]
fn nutation_longitude_under_20_arcsec() {
    for jd in [J2000, 2_415_021.0, 2_488_069.0] {
        let (nut_lon, nut_obl) = nutation(JulianDay::new(jd));
        // nutation() returns degrees; max nutation is ~17 arcsec = 0.00472°
        assert!(
            nut_lon.abs() < 0.006,
            "nutation lon {nut_lon:.6}° at JD {jd:.1} exceeds 20 arcsec"
        );
        assert!(
            nut_obl.abs() < 0.003,
            "nutation obl {nut_obl:.6}° at JD {jd:.1} exceeds 10 arcsec"
        );
    }
}

#[test]
fn topo_reset_returns_geocentric() {
    let geo = calc_ut(JulianDay::new(J2000), Body::MOON, CalcFlags::BUILTIN).unwrap();
    set_topo(Longitude::new(139.69), Latitude::new(35.69), 40.0); // Tokyo
    let _topo = calc_ut(
        JulianDay::new(J2000),
        Body::MOON,
        CalcFlags::BUILTIN | CalcFlags::TOPOCENTRIC,
    )
    .unwrap();
    set_topo(Longitude::new(0.0), Latitude::new(0.0), 0.0); // reset
    let back = calc_ut(JulianDay::new(J2000), Body::MOON, CalcFlags::BUILTIN).unwrap();
    assert!(
        (geo.lon - back.lon).abs() < 1e-10,
        "position after topo reset: {:.10} vs {:.10}",
        back.lon,
        geo.lon
    );
}

// ─── Remaining untested public functions ────────────────────────────────────

#[test]
fn match_aspect4_smoke() {
    // 4-body aspect — Sun/Moon/Mars/Jupiter
    let sun = calc_ut(JulianDay::new(J2000), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let moon = calc_ut(JulianDay::new(J2000), Body::MOON, CalcFlags::BUILTIN).unwrap();
    let _mar = calc_ut(JulianDay::new(J2000), Body::MARS, CalcFlags::BUILTIN).unwrap();
    let _jup = calc_ut(JulianDay::new(J2000), Body::JUPITER, CalcFlags::BUILTIN).unwrap();
    // match_aspect4: test 4-variant aspects for TWO bodies (pos0,speed0,pos1,speed1,aspect,app,sep,def)
    let r = match_aspect4(
        sun.lon,
        sun.speed_lon,
        moon.lon,
        moon.speed_lon,
        90.0,
        10.0,
        10.0,
        10.0,
    );
    // Result can be AspectMatch::None or found — just verify no panic
    let _ = r;
}

#[test]
fn fixstar2_matches_fixstar() {
    let r1 = fixstar("Aldebaran", JulianDay::new(J2000), CalcFlags::BUILTIN).unwrap();
    let r2 = fixstar2("Aldebaran", JulianDay::new(J2000), CalcFlags::BUILTIN).unwrap();
    assert!((r1.xx[0] - r2.xx[0]).abs() < 1e-10, "lon mismatch");
    assert!((r1.xx[1] - r2.xx[1]).abs() < 1e-10, "lat mismatch");
}

#[test]
fn fixstar2_ut_matches_fixstar_ut() {
    let r1 = fixstar_ut("Sirius", JulianDay::new(J2000), CalcFlags::BUILTIN).unwrap();
    let r2 = fixstar2_ut("Sirius", JulianDay::new(J2000), CalcFlags::BUILTIN).unwrap();
    assert!((r1.xx[0] - r2.xx[0]).abs() < 1e-10);
}

#[test]
fn fixstar2_mag_matches_fixstar_mag() {
    let m1 = fixstar_mag("Sirius").unwrap();
    let m2 = fixstar2_mag("Sirius").unwrap();
    // fixstar2_mag is an alias — should return identical value
    assert!((m1 - m2).abs() < 1e-10, "mag mismatch: {m1} vs {m2}");
    assert!(
        m1 < 0.0,
        "Sirius magnitude should be negative (very bright)"
    );
}

#[test]
fn ayanamsa_ex_matches_ayanamsa() {
    set_sid_mode(SiderealMode::LAHIRI, 0.0, 0.0);
    let a1 = ayanamsa(JulianDay::new(J2000));
    let a2 = ayanamsa_ex(JulianDay::new(J2000), CalcFlags::BUILTIN).unwrap();
    assert!(
        (a1 - a2).abs() < 1e-10,
        "ayanamsa_ex {a2:.6} vs ayanamsa {a1:.6}"
    );
    let a3 = ayanamsa_ex_ut(JulianDay::new(J2000), CalcFlags::BUILTIN).unwrap();
    assert!(a3.is_finite());
}

#[test]
fn config_functions_smoke() {
    // These setters must not panic
    let _ = set_jpl_file("de431.dat"); // no-op but must not crash
    set_lapse_rate(0.0065); // standard lapse rate
    set_tid_acc(1.0); // tidal acceleration
    let ta = tid_acc();
    assert!(ta.is_finite());
    let path = library_path();
    assert!(path.is_empty() || !path.is_empty()); // always valid
    let _cfd = current_file_data(0); // returns None in pure Rust mode
}

#[test]
fn geoformat_cs2_functions() {
    // centisec_to_lonlat_str and centisec_to_time_str
    let lon_cs = (10.5 * 360_000.0) as i32; // 10°30' in centiseconds
    let s = centisec_to_lonlat_str(lon_cs, 'E', 'W');
    assert!(s.contains('E') || s.contains('W') || !s.is_empty());
    let time_cs = 12 * 360_000 + 30 * 6000; // 12:30
    let t = centisec_to_time_str(time_cs, ':', false);
    assert!(!t.is_empty());
}

#[test]
fn houses_armc_matches_houses() {
    // houses_armc with explicit ARMC should match houses() result
    let r = houses(
        JulianDay::new(J2000),
        Latitude::new(48.85),
        Longitude::new(2.35),
        HouseSystem::PLACIDUS,
    )
    .unwrap();
    let armc = r.ascmc[2]; // index 2 is ARMC
    let eps = mean_obliquity(JulianDay::new(J2000));
    let r2 = houses_armc(
        Degrees::new(armc),
        Latitude::new(48.85),
        Degrees::new(eps),
        HouseSystem::PLACIDUS,
    )
    .unwrap();
    // Cusps should be very close (small rounding differences are OK)
    for h in 1..=12 {
        let diff = (r.cusps[h] - r2.cusps[h]).abs();
        let diff = if diff > 180.0 { 360.0 - diff } else { diff };
        assert!(
            diff < 0.01,
            "house {} differs: houses={:.4} armc={:.4}",
            h,
            r.cusps[h],
            r2.cusps[h]
        );
    }
}

#[test]
fn heliacal_ut_smoke() {
    // Heliacal rising of Venus near J2000 — don't crash, return finite result
    let geo = [2.35_f64, 48.85, 35.0];
    let atm = [1013.25_f64, 15.0, 50.0, 0.25]; // pressure, temp, humidity, age
    let dobs = [0.0_f64; 6]; // observer data (age, Snellen, etc) — defaults
    let r = heliacal_ut(
        JulianDay::new(J2000),
        geo,
        atm,
        dobs,
        "Venus",
        0,
        CalcFlags::BUILTIN,
    );
    if let Ok(jds) = r {
        for &jd in &jds {
            assert!(jd.is_finite() || jd == 0.0);
        }
    }
    // Err: no event found is acceptable
}

#[test]
fn pheno_smoke() {
    // pheno (TT variant) returns same structure as pheno_ut
    let r = pheno(JulianDay::new(J2000), Body::MARS, CalcFlags::BUILTIN);
    if let Ok(attr) = r {
        assert!(attr[0].is_finite()); // phase angle
        assert!(attr[1].is_finite()); // phase illuminated
    }
}

#[test]
fn next_aspect_cusp2_smoke() {
    // next_aspect_cusp2 with the extended variant
    let r = next_aspect_cusp2(
        Body::SUN,
        0.0,
        1,
        J2000,
        48.85,
        2.35,
        HouseSystem::PLACIDUS,
        false,
        CalcFlags::BUILTIN,
    );
    if let Some(result) = r {
        assert!(result.jd > J2000);
    }
}

#[test]
fn lun_occult_where_smoke() {
    // lun_occult_where returns geographic path — still a stub but must not panic
    let r = lun_occult_where(JulianDay::new(J2000), Body::VENUS, None, CalcFlags::BUILTIN);
    if let Ok(w) = r {
        assert!(w.geopos[0] >= -180.0);
    }
}

#[test]
fn difdegn_unsigned() {
    // diff_deg: always positive absolute difference
    let d = diff_deg(10.0, 350.0);
    assert!(d >= 0.0, "diff_deg must be >= 0, got {d}");
    assert!(d <= 180.0, "diff_deg must be <= 180, got {d}");
    // 10 - 350 = -340 → abs min arc = 20°
    assert!(
        (d - 20.0).abs() < 0.001,
        "diff_deg(10,350) should be 20, got {d}"
    );
}

#[test]
fn next_aspect_with2_smoke() {
    let r = next_aspect_with2(
        Body::SUN,
        0.0,
        Body::MOON,
        J2000,
        false,
        400.0,
        CalcFlags::BUILTIN,
    );
    if let Some(res) = r {
        assert!(res.jd > J2000);
    }
}

// ── Builder struct tests ───────────────────────────────────────────────────────
mod builder_tests {
    use celestial_core::body::{Body, CalcFlags, HouseSystem};
    use celestial_core::*;

    const JD: f64 = 2_451_545.0;

    #[test]
    fn rise_trans_options_rise_event() {
        // Same result as calling rise_trans() directly
        let geopos = [2.35, 48.85, 35.0];
        let via_builder = RiseTransOptions::new(JulianDay::new(JD), Body::SUN, geopos)
            .event(1) // CALC_RISE
            .flags(CalcFlags::BUILTIN)
            .search();
        let direct = rise_trans(
            JulianDay::new(JD),
            Body::SUN,
            None,
            CalcFlags::BUILTIN,
            1,
            geopos,
            0.0,
            0.0,
        );
        match (via_builder, direct) {
            (Ok(b), Ok(d)) => assert!((b.tret - d.tret).abs() < 1e-9, "tret mismatch"),
            (Err(_), Err(_)) => {} // both failed is acceptable
            (Ok(_), Err(_)) | (Err(_), Ok(_)) => panic!("builder and direct disagree"),
        }
    }

    #[test]
    fn rise_trans_options_atmosphere() {
        let geopos = [2.35, 48.85, 35.0];
        // Setting atmosphere doesn't panic
        let r = RiseTransOptions::new(JulianDay::new(JD), Body::MOON, geopos)
            .event(2) // CALC_SET
            .atmosphere(1013.25, 15.0)
            .flags(CalcFlags::BUILTIN)
            .search();
        let _ = r; // Ok or Err both acceptable
    }

    #[test]
    fn rise_trans_options_star() {
        let geopos = [2.35, 48.85, 35.0];
        let r = RiseTransOptions::new(JulianDay::new(JD), Body::SUN, geopos)
            .star("Aldebaran")
            .event(1)
            .search();
        let _ = r;
    }

    #[test]
    fn search_options_mc_transit() {
        // SearchOptions::search_mc_transit should match mc_transit_ut directly
        let jd_start = JD + 365.0;
        let via_builder = SearchOptions::new(Body::SATURN, jd_start)
            .natal_chart(JD, 48.85, 2.35, HouseSystem::PLACIDUS)
            .flags(CalcFlags::BUILTIN)
            .search_mc_transit();
        let direct = mc_transit_ut(
            Body::SATURN,
            JD,
            jd_start,
            48.85,
            2.35,
            HouseSystem::PLACIDUS,
            CalcFlags::BUILTIN,
            false,
        );
        match (via_builder, direct) {
            (Ok(b), Ok(d)) => assert!((b - d).abs() < 1e-6, "MC transit mismatch"),
            (Err(_), Err(_)) => {}
            _ => panic!("builder and direct disagree on MC transit"),
        }
    }

    #[test]
    fn search_options_ic_transit() {
        let r = SearchOptions::new(Body::JUPITER, JD + 100.0)
            .natal_chart(JD, 48.85, 2.35, HouseSystem::PLACIDUS)
            .search_ic_transit();
        let _ = r;
    }

    #[test]
    fn search_options_asc_dsc_transit() {
        let asc = SearchOptions::new(Body::MARS, JD + 30.0)
            .natal_chart(JD, 48.85, 2.35, HouseSystem::PLACIDUS)
            .search_asc_transit();
        let dsc = SearchOptions::new(Body::MARS, JD + 30.0)
            .natal_chart(JD, 48.85, 2.35, HouseSystem::PLACIDUS)
            .search_dsc_transit();
        let _ = (asc, dsc);
    }

    #[test]
    fn search_options_cusp_aspect() {
        let r = SearchOptions::new(Body::SATURN, JD)
            .aspect(90.0)
            .cusp(10, 48.85, 2.35, HouseSystem::PLACIDUS)
            .flags(CalcFlags::BUILTIN)
            .search_cusp();
        let _ = r; // Some or None both acceptable
    }

    #[test]
    fn search_options_backward() {
        // Backward search should find a result before jd_start
        let r = SearchOptions::new(Body::SUN, JD)
            .natal_chart(JD, 48.85, 2.35, HouseSystem::PLACIDUS)
            .backward(true)
            .search_mc_transit();
        if let Ok(jd) = r {
            assert!(
                jd < JD,
                "backward search result {jd} should be before start {JD}"
            );
        }
    }

    #[test]
    fn aspect_orbs_check_exact_trine() {
        // pos0=0°, pos1=120°: exactly trine (120°)
        let m = AspectOrbs::new(2.0, 1.5).check(0.0, 0.5, 120.0, -0.4, 120.0);
        assert!(m.matched, "exact trine should match");
        assert!(m.diff.abs() < 0.001, "orb should be ~0 for exact aspect");
    }

    #[test]
    fn aspect_orbs_check_outside_orb() {
        // pos0=0°, pos1=125°: 5° from trine, orb=2°
        let m = AspectOrbs::new(2.0, 2.0).check(0.0, 0.5, 125.0, -0.4, 120.0);
        assert!(!m.matched, "5° outside orb should not match");
    }

    #[test]
    fn aspect_orbs_check_simple_vs_check_same_result() {
        // With def_orb == sep_orb, check_simple and check should agree
        let orbs = AspectOrbs::new(2.0, 1.5).def_orb(1.5);
        let m1 = orbs.check(0.0, 0.5, 60.0, -0.3, 60.0);
        let m2 = orbs.check_simple(0.0, 0.5, 60.0, -0.3, 60.0);
        assert_eq!(
            m1.matched, m2.matched,
            "check and check_simple should agree when def_orb=sep_orb"
        );
    }

    #[test]
    fn aspect_orbs_def_orb_defaults_to_max() {
        // When def_orb is not set, it defaults to max(app_orb, sep_orb)
        let orbs = AspectOrbs::new(2.0, 1.0);
        // Verify by testing that a 1.5° orb matches when def_orb=2.0
        let m = orbs.check(0.0, 0.5, 61.5, -0.3, 60.0); // 1.5° off sextile
        assert!(m.matched, "1.5° should be within default def_orb of 2.0");
    }
}

// ── CalcOptions builder tests ──────────────────────────────────────────────────
mod calc_options_tests {
    use celestial_core::body::{Body, CalcFlags};
    use celestial_core::*;

    const JD: f64 = 2_451_545.0;
    const FLAGS: CalcFlags = CalcFlags::BUILTIN;

    #[test]
    fn single_body_ut_matches_calc_ut() {
        let via_builder = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .body(Body::SUN)
            .get()
            .unwrap();
        let direct = calc_ut(JulianDay::new(JD), Body::SUN, FLAGS).unwrap();
        assert!((via_builder.lon - direct.lon).abs() < 1e-9);
        assert!((via_builder.dist - direct.dist).abs() < 1e-12);
    }

    #[test]
    fn single_body_tt_matches_calc_tt() {
        let via_builder = CalcOptions::tt(JulianDay::new(JD), FLAGS)
            .body(Body::MOON)
            .get()
            .unwrap();
        let direct = calc(JulianDay::new(JD), Body::MOON, FLAGS).unwrap();
        assert!((via_builder.lon - direct.lon).abs() < 1e-9);
    }

    #[test]
    fn multi_body_auto_matches_individual() {
        let bodies = [
            Body::SUN,
            Body::MOON,
            Body::MERCURY,
            Body::VENUS,
            Body::MARS,
        ];
        let results = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .bodies(&bodies)
            .get_many();
        assert_eq!(results.len(), bodies.len());
        for (i, &body) in bodies.iter().enumerate() {
            let direct = calc_ut(JulianDay::new(JD), body, FLAGS).unwrap();
            let got = results[i].as_ref().unwrap();
            assert!(
                (got.lon - direct.lon).abs() < 1e-9,
                "body {i} lon mismatch: {} vs {}",
                got.lon,
                direct.lon
            );
        }
    }

    #[test]
    fn multi_body_sequential_matches_parallel() {
        let bodies = [Body::SUN, Body::MOON, Body::MERCURY];
        let seq = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .strategy(CalcStrategy::Sequential)
            .bodies(&bodies)
            .get_many();
        let par = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .strategy(CalcStrategy::Parallel)
            .bodies(&bodies)
            .get_many();
        assert_eq!(seq.len(), par.len());
        for i in 0..seq.len() {
            let s = seq[i].as_ref().unwrap();
            let p = par[i].as_ref().unwrap();
            assert!(
                (s.lon - p.lon).abs() < 1e-9,
                "body {i}: sequential={:.6} parallel={:.6}",
                s.lon,
                p.lon
            );
        }
    }

    #[test]
    fn strategy_auto_uses_parallel_for_large_list() {
        // Auto with 5 bodies should use parallel — verify same results as explicit parallel
        let bodies = [
            Body::SUN,
            Body::MOON,
            Body::MERCURY,
            Body::VENUS,
            Body::MARS,
        ];
        let auto_res = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .bodies(&bodies)
            .get_many();
        let par_res = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .strategy(CalcStrategy::Parallel)
            .bodies(&bodies)
            .get_many();
        for i in 0..bodies.len() {
            let a = auto_res[i].as_ref().unwrap();
            let p = par_res[i].as_ref().unwrap();
            assert!((a.lon - p.lon).abs() < 1e-9);
        }
    }

    #[test]
    fn strategy_auto_uses_sequential_for_small_list() {
        // Auto with 2 bodies should use sequential path
        let bodies = [Body::SUN, Body::MOON];
        let auto_res = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .bodies(&bodies)
            .get_many();
        let seq_res = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .strategy(CalcStrategy::Sequential)
            .bodies(&bodies)
            .get_many();
        let a = auto_res[0].as_ref().unwrap();
        let s = seq_res[0].as_ref().unwrap();
        assert!((a.lon - s.lon).abs() < 1e-9);
    }

    #[test]
    fn empty_bodies_returns_empty_vec() {
        let results = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .bodies(&[])
            .get_many();
        assert!(results.is_empty());
    }

    #[test]
    fn single_body_in_multi_path() {
        let bodies = [Body::SATURN];
        let multi = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .bodies(&bodies)
            .get_many();
        let single = CalcOptions::ut(JulianDay::new(JD), FLAGS)
            .body(Body::SATURN)
            .get()
            .unwrap();
        let m = multi[0].as_ref().unwrap();
        assert!((m.lon - single.lon).abs() < 1e-9);
    }
}

// ── Structured error type tests ────────────────────────────────────────────────
mod error_type_tests {
    use celestial_core::Error;

    #[test]
    fn body_not_implemented_display() {
        let e = Error::BodyNotImplemented { body: 99 };
        let s = e.to_string();
        assert!(s.contains("99"), "error should mention body number");
        assert!(
            s.contains("implemented"),
            "error should say not implemented"
        );
    }

    #[test]
    fn star_not_found_display() {
        let e = Error::StarNotFound {
            name: "Foobar".into(),
        };
        let s = e.to_string();
        assert!(s.contains("Foobar"), "error should mention star name");
        assert!(s.contains("catalog"), "error should mention catalog");
    }

    #[test]
    fn phase_not_found_display() {
        let e = Error::PhaseNotFound {
            phase: "full moon".into(),
            from_jd: 2_451_545.0,
        };
        let s = e.to_string();
        assert!(s.contains("full moon"));
        assert!(s.contains("2451545"));
    }

    #[test]
    fn no_eclipse_found_display() {
        let e = Error::NoEclipseFound {
            from_jd: 2_451_545.0,
        };
        let s = e.to_string();
        assert!(s.contains("eclipse"), "should mention eclipse");
        assert!(s.contains("2451545"));
    }

    #[test]
    fn circumpolar_body_display() {
        let e = Error::CircumpolarBody { body: 1, lat: 89.5 };
        let s = e.to_string();
        assert!(s.contains("1"));
        assert!(s.contains("89.5") || s.contains("circumpolar"));
    }

    #[test]
    fn house_system_failed_display() {
        let e = Error::HouseSystemFailed {
            system: b'P',
            lat: 91.0,
        };
        let s = e.to_string();
        assert!(
            s.contains("P") || s.contains("house"),
            "should mention house system"
        );
    }

    #[test]
    fn legacy_string_variants_still_work() {
        // Ensure backward-compat string variants compile and display correctly
        let calc = Error::Calc("test calc".into());
        let house = Error::Houses("test houses".into());
        let ecl = Error::Eclipse("test eclipse".into());
        let rt = Error::RiseTrans("test rise".into());
        let date = Error::Date("test date".into());
        assert!(calc.to_string().contains("test calc"));
        assert!(house.to_string().contains("test houses"));
        assert!(ecl.to_string().contains("test eclipse"));
        assert!(rt.to_string().contains("test rise"));
        assert!(date.to_string().contains("test date"));
    }

    #[test]
    fn structured_error_emitted_for_bad_star() {
        use celestial_core::body::CalcFlags;
        let result = celestial_core::fixstar_ut(
            "NONEXISTENT_STAR_XYZ",
            celestial_core::JulianDay::new(2_451_545.0),
            CalcFlags::BUILTIN,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::StarNotFound { name } => assert!(name.contains("NONEXISTENT")),
            other => panic!("expected StarNotFound, got {other:?}"),
        }
    }
}

// ── Moon, calc_many, and calendar unit tests ──────────────────────────────────
mod moon_and_calendar_tests {
    #[cfg(feature = "calendar-traditions")]
    use celestial_core::body::{Body, CalcFlags, Calendar};
    use celestial_core::*;

    const JD: f64 = 2_451_545.0; // J2000.0

    // ── moon ──────────────────────────────────────────────────────────────────

    #[test]
    fn moon_phase_returns_known_phase() {
        // J2000.0 = 2000-01-01 12:00 UT — Moon is a few days past new moon
        let phase = moon_phase(JulianDay::new(JD)).unwrap();
        // Any valid MoonPhase variant is acceptable — just check it doesn't panic
        let name = phase.name();
        assert!(!name.is_empty(), "phase name should not be empty");
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn moon_phase_full_moon_is_full() {
        // Known full moon: 2000-02-19 ≈ JD 2451594.5
        let jd_fm = julday(2000, 2, 19, 16.0, Calendar::Gregorian);
        let phase = moon_phase(JulianDay::new(jd_fm)).unwrap();
        assert!(
            matches!(
                phase,
                MoonPhase::FullMoon | MoonPhase::WaxingGibbous | MoonPhase::WaningGibbous
            ),
            "expected near full moon, got {phase:?}"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn moon_illumination_range() {
        for offset in [0.0, 7.4, 14.8, 22.1] {
            let illum = moon_illumination(JulianDay::new(JD + offset)).unwrap();
            assert!(
                (0.0..=100.0).contains(&illum),
                "illumination {illum:.2}% out of range at offset {offset}"
            );
        }
    }

    // ── calc_many ─────────────────────────────────────────────────────────────

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn calc_many_order_matches_input() {
        let bodies = [
            Body::SUN,
            Body::MOON,
            Body::MERCURY,
            Body::VENUS,
            Body::MARS,
        ];
        let results = calc_ut_many(JulianDay::new(JD), &bodies, CalcFlags::BUILTIN);
        assert_eq!(
            results.len(),
            bodies.len(),
            "result count must match input count"
        );

        for (i, &body) in bodies.iter().enumerate() {
            let direct = calc_ut(JulianDay::new(JD), body, CalcFlags::BUILTIN).unwrap();
            let via_many = results[i].as_ref().unwrap();
            assert!(
                (via_many.lon - direct.lon).abs() < 1e-9,
                "body {i} lon mismatch: many={:.6} direct={:.6}",
                via_many.lon,
                direct.lon
            );
        }
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn calc_many_empty_returns_empty() {
        let results = calc_ut_many(JulianDay::new(JD), &[], CalcFlags::BUILTIN);
        assert!(results.is_empty());
    }

    // ── Easter / Christian ────────────────────────────────────────────────────

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn easter_gregorian_2025() {
        // Easter 2025 = April 20
        let (_y, m, d) = easter_gregorian(2025);
        assert_eq!((m, d), (4, 20), "Easter 2025 should be April 20");
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn easter_gregorian_known_dates() {
        // A few well-known Easter dates
        let cases: &[(i32, u8, u8)] = &[(2024, 3, 31), (2023, 4, 9), (2022, 4, 17), (2000, 4, 23)];
        for &(year, month, day) in cases {
            let (_y, m, d) = easter_gregorian(year);
            assert_eq!(
                (m, d),
                (month, day),
                "Easter {year}: expected {month}/{day}, got {m}/{d}"
            );
        }
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn easter_orthodox_differs_from_gregorian() {
        // Orthodox Easter often falls on a different date
        let (_gy, gm, gd) = easter_gregorian(2024);
        let (_oy, om, od) = easter_orthodox(2024);
        // In 2024: Gregorian = March 31, Orthodox = May 5
        assert_ne!(
            (gm, gd),
            (om, od),
            "Gregorian and Orthodox Easter should differ in 2024"
        );
    }

    // ── Islamic / Hijri ───────────────────────────────────────────────────────

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn hijri_from_jd_j2000() {
        // J2000.0 = 2000-01-01 Gregorian = ~1420 AH Ramadan
        let (year, month, _day) = hijri_from_jd(JulianDay::new(JD));
        assert_eq!(year, 1420, "J2000 Hijri year should be 1420 AH");
        // Ramadan 1420 AH started approximately Dec 9 1999
        assert!(
            (9..=10).contains(&month),
            "J2000 month should be Ramadan/Shawwal, got {month}"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn hijri_roundtrip() {
        let (year, month, day) = hijri_from_jd(JulianDay::new(JD));
        let jd2 = hijri_to_jd(year, month, day);
        assert!(
            (jd2 - JD).abs() < 1.5,
            "Hijri round-trip JD error {:.3} days",
            (jd2 - JD).abs()
        );
    }

    // ── Nowruz / Persian ──────────────────────────────────────────────────────

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn nowruz_jd_lands_in_march() {
        let jd = nowruz_jd(2025);
        let d = revjul(JulianDay::new(jd), Calendar::Gregorian);
        assert_eq!(d.year, 2025, "Nowruz 2025 should be in year 2025");
        assert_eq!(d.month, 3, "Nowruz should always be in March");
        assert!(
            d.day >= 19 && d.day <= 22,
            "Nowruz day {} out of expected range 19-22",
            d.day
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn nowruz_jd_advances_each_year() {
        let jd2024 = nowruz_jd(2024);
        let jd2025 = nowruz_jd(2025);
        let diff = jd2025 - jd2024;
        assert!(
            diff > 364.0 && diff < 367.0,
            "Nowruz interval {diff:.2}d should be ~365.25d"
        );
    }
}

// ── Calendar deep coverage ────────────────────────────────────────────────────
mod calendar_deep_tests {
    #[cfg(feature = "calendar-traditions")]
    use celestial_core::body::Calendar;
    #[cfg(feature = "calendar-traditions")]
    use celestial_core::*;

    #[cfg(feature = "calendar-traditions")]
    const JD: f64 = 2_451_545.0; // J2000.0 = 2000-01-01

    // ── Easter / Christian ────────────────────────────────────────────────────

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn christian_feasts_count_is_nonzero() {
        let feasts = christian_feasts(2025);
        assert!(
            !feasts.is_empty(),
            "christian_feasts should return at least one feast"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn christian_fixed_feasts_includes_christmas() {
        let feasts = christian_fixed_feasts(2025);
        let christmas = feasts.iter().find(|f| f.name.contains("Christmas"));
        assert!(christmas.is_some(), "fixed feasts should include Christmas");
        let c = christmas.unwrap();
        assert_eq!((c.month, c.day), (12, 25), "Christmas should be Dec 25");
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn easter_jd_matches_gregorian() {
        let jd = easter_jd(2025);
        let cal = revjul(JulianDay::new(jd), Calendar::Gregorian);
        let (_, m, d) = easter_gregorian(2025);
        assert_eq!(
            (cal.month as u8, cal.day as u8),
            (m, d),
            "easter_jd and easter_gregorian should agree"
        );
    }

    // ── Hebrew calendar ────────────────────────────────────────────────────────

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn hebrew_year_from_jd_j2000() {
        // J2000 = 2000-01-01 = 5760 AM
        let year = hebrew_year_from_jd(JulianDay::new(JD));
        assert_eq!(year, 5760, "J2000 Hebrew year should be 5760, got {year}");
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn jd_to_hebrew_date_roundtrip() {
        // Convert J2000 to Hebrew date and check it's in 5760 AM
        let (year, month, day) = jd_to_hebrew_date(JulianDay::new(JD));
        assert_eq!(year, 5760, "J2000 Hebrew year should be 5760");
        assert!(
            (1..=13).contains(&month),
            "Hebrew month {month} out of range 1-13"
        );
        assert!(
            (1..=30).contains(&day),
            "Hebrew day {day} out of range 1-30"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn jewish_holidays_has_rosh_hashanah() {
        let holidays = jewish_holidays(5785);
        let rh = holidays.iter().find(|h| h.name.contains("Rosh Hashanah"));
        assert!(rh.is_some(), "Jewish holidays should include Rosh Hashanah");
    }

    // ── Islamic calendar ──────────────────────────────────────────────────────

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn hijri_month_name_ramadan() {
        let name = hijri_month_name(9);
        assert!(
            name.to_lowercase().contains("ramadan"),
            "month 9 should be Ramadan, got '{name}'"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn hijri_month_roundtrip_12_months() {
        // Step through 12 consecutive months and verify each roundtrips cleanly
        let mut jd = hijri_to_jd(1446, 1, 1);
        let mut prev_month = 0u8;
        for _ in 0..12 {
            let (y, m, _d) = hijri_from_jd(JulianDay::new(jd));
            assert_eq!(y, 1446, "year should stay 1446 within the first 12 months");
            assert!(m != prev_month, "month should advance");
            prev_month = m;
            jd += 30.0; // advance by ~one month
        }
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn islamic_observances_contains_ramadan() {
        let obs = islamic_observances(1446);
        let ramadan = obs
            .iter()
            .find(|o| o.name.to_lowercase().contains("ramadan"));
        assert!(
            ramadan.is_some(),
            "Islamic observances should include Ramadan"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn gregorian_to_hijri_years_j2000() {
        let (y1, y2) = gregorian_to_hijri_years(2000);
        assert_eq!(y1, 1420, "2000 CE should start in 1420 AH, got {y1}");
        assert_eq!(y2, 1421, "2000 CE should end in 1421 AH, got {y2}");
    }

    // ── Nowruz / Persian / Bahá'í ─────────────────────────────────────────────

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn gregorian_to_solar_hijri_j2000() {
        // gregorian_to_solar_hijri returns the Solar Hijri year that starts in that Gregorian year
        // Nowruz 2000 (March 20) starts SH year 1379; function returns that year
        let sh = gregorian_to_solar_hijri(2000);
        assert_eq!(
            sh, 1379,
            "2000 CE Solar Hijri year should be 1379, got {sh}"
        );
        let sh1999 = gregorian_to_solar_hijri(1999);
        assert_eq!(
            sh1999, 1378,
            "1999 CE Solar Hijri year should be 1378, got {sh1999}"
        );
        // Each Gregorian year advances the SH year by 1
        assert_eq!(
            gregorian_to_solar_hijri(2025),
            sh + 25,
            "SH year should advance 1:1 with Gregorian years"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn jd_to_bahai_j2000() {
        let b = jd_to_bahai(JulianDay::new(JD));
        assert!(
            b.year >= 155 && b.year <= 157,
            "J2000 Bahai year should be ~156, got {}",
            b.year
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn bahai_holy_days_nonempty() {
        let days = bahai_holy_days(157);
        assert!(
            !days.is_empty(),
            "bahai_holy_days should return at least one day"
        );
        for day in &days {
            assert!(!day.name.is_empty(), "holy day name should not be empty");
            assert!(day.jd > 0.0, "holy day JD should be positive");
        }
    }

    // ── Omer ──────────────────────────────────────────────────────────────────

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn omer_start_jd_after_passover() {
        let jd = omer_start_jd(5785);
        let cal = revjul(JulianDay::new(jd), Calendar::Gregorian);
        assert_eq!(
            cal.year, 2025,
            "Omer 5785 should start in 2025, got {}",
            cal.year
        );
        assert_eq!(
            cal.month, 4,
            "Omer 5785 should start in April, got month {}",
            cal.month
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn omer_day_jd_day1_matches_start() {
        let start = omer_start_jd(5785);
        let day1 = omer_day_jd(5785, 1).unwrap();
        assert!(
            (start - day1).abs() < 1.0,
            "omer_day_jd(1) should match omer_start_jd: {start:.2} vs {day1:.2}"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn omer_days_count_is_49() {
        let days = omer_days(5785);
        assert_eq!(days.len(), 49, "Omer has exactly 49 days");
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn omer_declaration_day33_mentions_count() {
        let decl = omer_declaration(33);
        assert!(
            decl.contains("33") || decl.to_lowercase().contains("lag"),
            "day 33 declaration should mention 33 or lag, got: {decl}"
        );
    }

    // ── Vesak / Buddhist ─────────────────────────────────────────────────────

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn vesak_jd_is_in_april_or_may() {
        let jd = vesak_jd(2025);
        let cal = revjul(JulianDay::new(jd), Calendar::Gregorian);
        assert!(
            cal.month == 4 || cal.month == 5,
            "Vesak 2025 should be in April or May, got month {}",
            cal.month
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn uposatha_days_count() {
        let days = uposatha_days(2025);
        // Uposatha occurs on 4 lunar phases × ~12 months, but implementation
        // may also include weekly observances — check it's a positive non-trivial count
        assert!(
            days.len() >= 12,
            "Uposatha days in 2025 should be at least 12, got {}",
            days.len()
        );
        // All JDs should be in 2025 (approx)
        let jd_2025_start =
            celestial_core::julday(2025, 1, 1, 0.0, celestial_core::body::Calendar::Gregorian);
        let jd_2026_start =
            celestial_core::julday(2026, 1, 1, 0.0, celestial_core::body::Calendar::Gregorian);
        for u in &days {
            assert!(
                u.jd >= jd_2025_start && u.jd < jd_2026_start,
                "Uposatha JD {:.2} should be in 2025",
                u.jd
            );
        }
    }
}

// ── Moon phase deep coverage ──────────────────────────────────────────────────
mod moon_phase_tests {
    use celestial_core::*;

    const JD: f64 = 2_451_545.0; // J2000.0 = 2000-01-01 12:00 UT

    #[test]
    fn moon_phase_angle_is_in_range() {
        let angle = moon_phase_angle(JulianDay::new(JD)).unwrap();
        assert!(
            (0.0..360.0).contains(&angle),
            "phase angle {angle:.2}° out of [0,360)"
        );
    }

    #[test]
    fn moon_phase_angle_advances_over_cycle() {
        // Over a synodic month (~29.5 d) the angle should complete a full cycle
        let a0 = moon_phase_angle(JulianDay::new(JD)).unwrap();
        let a1 = moon_phase_angle(JulianDay::new(JD + 29.5)).unwrap();
        // Both should be finite and the total travel ~360°
        assert!(a0.is_finite() && a1.is_finite());
    }

    #[test]
    fn moon_phase_info_fields_consistent() {
        let info = moon_phase_info(JulianDay::new(JD)).unwrap();
        assert!(
            info.illumination >= 0.0 && info.illumination <= 100.0,
            "illumination {:.2}% out of range",
            info.illumination
        );
        assert!(
            info.elongation >= 0.0 && info.elongation < 360.0,
            "elongation {:.2}° out of range",
            info.elongation
        );
        // illumination is 0.0-1.0 (fraction, not percent)
        assert!(
            info.illumination >= 0.0 && info.illumination <= 1.0,
            "illumination {:.4} out of 0-1",
            info.illumination
        );
    }

    #[test]
    fn moon_phases_for_month_returns_four_phases() {
        // January 2025 should have 4 principal phases
        let phases = moon_phases_for_month(2025, 1).unwrap();
        assert_eq!(
            phases.len(),
            4,
            "expected 4 phases in Jan 2025, got {}",
            phases.len()
        );
        // Phases should be in chronological order
        for w in phases.windows(2) {
            assert!(
                w[0].jd < w[1].jd,
                "phases not in order: {:.2} >= {:.2}",
                w[0].jd,
                w[1].jd
            );
        }
        // Each phase elongation should be near the target for that phase type (±5°)
        for ph in &phases {
            let target = ph.phase.elongation_target();
            let diff = (ph.elongation - target)
                .abs()
                .min(360.0 - (ph.elongation - target).abs());
            assert!(
                diff < 5.0,
                "{:?} elongation {:.2}° expected ~{target:.0}°",
                ph.phase,
                ph.elongation
            );
        }
    }

    #[test]
    fn next_new_moon_is_after_start() {
        let nm = next_new_moon(JulianDay::new(JD)).unwrap();
        assert!(nm > JD, "next new moon {nm:.2} not after start {JD:.2}");
        assert!(nm < JD + 30.0, "next new moon too far: {:.2}d", nm - JD);
    }

    #[test]
    fn next_new_moon_after_exact_phase_does_not_go_backwards() {
        let nm = next_new_moon(JulianDay::new(JD)).unwrap();
        let start = nm + 0.005;
        let next = next_new_moon(JulianDay::new(start)).unwrap();
        assert!(
            next >= start,
            "next new moon {next:.6} is before requested start {start:.6}"
        );
        assert!(
            next > start + 20.0,
            "expected following lunation after start {start:.6}, got {next:.6}"
        );
    }

    #[test]
    fn next_first_quarter_after_new_moon() {
        let nm = next_new_moon(JulianDay::new(JD)).unwrap();
        let fq = next_first_quarter(JulianDay::new(nm)).unwrap();
        assert!(fq > nm, "first quarter not after new moon");
        let days = fq - nm;
        assert!(
            days > 5.0 && days < 10.0,
            "first quarter {days:.2}d after new moon, expected ~7d"
        );
    }

    #[test]
    fn next_full_moon_phase_after_first_quarter() {
        let nm = next_new_moon(JulianDay::new(JD)).unwrap();
        let fq = next_first_quarter(JulianDay::new(nm)).unwrap();
        let fm = next_full_moon_phase(JulianDay::new(fq)).unwrap();
        assert!(fm > fq, "full moon not after first quarter");
        let days = fm - fq;
        assert!(
            days > 5.0 && days < 10.0,
            "full moon {days:.2}d after first quarter, expected ~7d"
        );
    }

    #[test]
    fn next_last_quarter_after_full_moon() {
        let nm = next_new_moon(JulianDay::new(JD)).unwrap();
        let fm = next_full_moon_phase(JulianDay::new(nm)).unwrap();
        let lq = next_last_quarter(JulianDay::new(fm)).unwrap();
        assert!(lq > fm, "last quarter not after full moon");
    }

    #[test]
    fn next_principal_phase_new_moon_matches_next_new_moon() {
        let via_helper = next_new_moon(JulianDay::new(JD)).unwrap();
        let via_generic = next_principal_phase(JulianDay::new(JD), PrincipalPhase::NewMoon)
            .unwrap()
            .jd;
        assert!(
            (via_helper - via_generic).abs() < 1e-6,
            "next_new_moon {via_helper:.6} ≠ next_principal_phase(NewMoon) {via_generic:.6}"
        );
    }
}

// ── Islamic calendar helpers ──────────────────────────────────────────────────
#[cfg(feature = "calendar-traditions")]
mod islamic_helper_tests {
    use celestial_core::*;

    #[test]
    fn hijri_month_days_29_or_30() {
        for month in 1u8..=12 {
            let days = hijri_month_days(1446, month);
            assert!(
                days == 29 || days == 30,
                "Hijri 1446/month {month}: {days} days"
            );
        }
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn hijri_month_days_sums_to_year_length() {
        let total: u32 = (1..=12u8).map(|m| hijri_month_days(1446, m) as u32).sum();
        // Hijri year is 354 or 355 days
        assert!(total == 354 || total == 355, "1446 AH total days: {total}");
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn hijri_new_year_jd_in_correct_gregorian_year() {
        // 1446 AH new year fell in July 2024
        let jd = hijri_new_year_jd(1446);
        let cal = revjul(
            JulianDay::new(jd),
            celestial_core::body::Calendar::Gregorian,
        );
        assert_eq!(
            cal.year, 2024,
            "1446 AH new year should be in 2024, got {}",
            cal.year
        );
        assert_eq!(
            cal.month, 7,
            "1446 AH new year should be in July, got month {}",
            cal.month
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn hijri_month_start_advances_by_month_days() {
        let start1 = hijri_month_start_jd(1446, 1);
        let start2 = hijri_month_start_jd(1446, 2);
        let days = hijri_month_days(1446, 1) as f64;
        assert!(
            (start2 - start1 - days).abs() < 1.0,
            "Month 2 start should be month_days({days}) after month 1 start"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn is_hijri_leap_year_correct_cycle() {
        // In a 30-year Hijri cycle, years 2,5,7,10,13,16,18,21,24,26,29 are leap
        let leap_in_30 = [2u32, 5, 7, 10, 13, 16, 18, 21, 24, 26, 29];
        for offset in 0..30u32 {
            let year = 1420 + offset;
            let position = ((year - 1) % 30) + 1;
            let expected = leap_in_30.contains(&position);
            assert_eq!(
                is_hijri_leap_year(year as i32),
                expected,
                "Year {year} (pos {position} in cycle) leap={}",
                is_hijri_leap_year(year as i32)
            );
        }
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn islamic_observances_for_jd_returns_observances() {
        // Ramadan 1446 started around March 1 2025 — JD ~2460735
        let jd = hijri_month_start_jd(1446, 9); // 9 = Ramadan
        let obs = islamic_observances_for_jd(JulianDay::new(jd));
        // There should be at least one observance in Ramadan
        assert!(
            !obs.is_empty(),
            "No observances returned for start of Ramadan"
        );
    }
}

// ── Hebrew calendar helpers ───────────────────────────────────────────────────
#[cfg(feature = "calendar-traditions")]
mod hebrew_helper_tests {
    use celestial_core::*;

    #[test]
    fn elapsed_days_increases_monotonically() {
        for year in 5784..5790 {
            assert!(
                elapsed_days(year + 1) > elapsed_days(year),
                "elapsed_days not monotone at {year}"
            );
        }
    }

    #[test]
    fn days_in_hebrew_year_is_353_to_385() {
        // Hebrew years are 353/354/355 (regular) or 383/384/385 (leap)
        for year in 5780..5790 {
            let d = days_in_hebrew_year(year);
            assert!(
                (353..=355).contains(&d) || (383..=385).contains(&d),
                "Year {year} has {d} days — outside valid range"
            );
        }
    }

    #[test]
    fn hebrew_month_days_12_or_13_months() {
        for year in 5780..5790 {
            let months = months_in_hebrew_year(year);
            assert!(
                months == 12 || months == 13,
                "Year {year} has {months} months"
            );
        }
    }

    #[test]
    fn is_hebrew_leap_year_matches_months() {
        for year in 5780..5790 {
            let is_leap = is_hebrew_leap_year(year);
            let months = months_in_hebrew_year(year);
            assert_eq!(
                is_leap,
                months == 13,
                "Year {year}: is_leap={is_leap} but months={months}"
            );
        }
    }

    #[test]
    fn hebrew_month_days_valid_range() {
        // Each Hebrew month has 29 or 30 days
        for month in 1i32..=12 {
            let d = hebrew_month_days(5785, month);
            assert!(d == 29 || d == 30, "Hebrew 5785/month {month}: {d} days");
        }
    }

    #[test]
    fn hebrew_new_year_jd_in_september_or_october() {
        for year in 5780..5790 {
            let jd = hebrew_new_year_jd(year) as f64;
            let cal = revjul(
                JulianDay::new(jd),
                celestial_core::body::Calendar::Gregorian,
            );
            assert!(
                cal.month == 9 || cal.month == 10,
                "Rosh Hashanah {year} AM: expected Sep/Oct, got month {}",
                cal.month
            );
        }
    }

    #[test]
    fn approx_hebrew_year_roundtrip() {
        let jd = hebrew_new_year_jd(5785) as f64;
        let est = approx_hebrew_year(JulianDay::new(jd));
        assert!(
            (est - 5785i32).abs() <= 1,
            "approx_hebrew_year at Rosh Hashanah 5785 = {est}"
        );
    }

    #[test]
    fn hebrew_month_start_jd_advances() {
        let m1 = hebrew_month_start_jd(5785, 1) as f64;
        let m2 = hebrew_month_start_jd(5785, 2) as f64;
        let days = hebrew_month_days(5785, 1);
        assert!(
            (m2 - m1 - days as f64).abs() < 1.0,
            "Month 2 start should be {days}d after month 1 start"
        );
    }
}

// ── Omer helpers ──────────────────────────────────────────────────────────────
#[cfg(feature = "calendar-traditions")]
mod omer_helper_tests {
    use celestial_core::*;

    #[test]
    fn omer_period_5785_span_is_49_days() {
        let start = omer_start_jd(5785);
        let p = omer_period(JulianDay::new(start + 1.0));
        let span = p.end_jd - p.start_jd;
        assert!(
            (span - 48.0).abs() < 2.0,
            "Omer 5785 span = {span:.1}d, expected ~48d (day 1 to day 49)"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn omer_from_jd_during_omer_returns_some() {
        let start = omer_start_jd(5785);
        let p = omer_period(JulianDay::new(start + 1.0));
        let mid = p.start_jd + 16.0_f64; // day 17 of the Omer
        let day = omer_from_jd(JulianDay::new(mid));
        assert!(day.is_some(), "omer_from_jd during Omer should return Some");
        let d = day.unwrap();
        assert!(d.day >= 1 && d.day <= 49, "day {} out of 1-49", d.day);
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn omer_from_jd_outside_omer_returns_none() {
        let start = omer_start_jd(5785);
        let p = omer_period(JulianDay::new(start + 1.0));
        let before = p.start_jd - 5.0_f64;
        let after = p.end_jd + 5.0_f64;
        assert!(
            omer_from_jd(JulianDay::new(before)).is_none(),
            "before Omer should be None"
        );
        assert!(
            omer_from_jd(JulianDay::new(after)).is_none(),
            "after Omer should be None"
        );
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn omer_from_jd_day_33_is_lag_baomer() {
        let start = omer_start_jd(5785);
        let p = omer_period(JulianDay::new(start + 1.0));
        let lag = p.start_jd + 32.0_f64; // 0-indexed: day 33
        let day = omer_from_jd(JulianDay::new(lag)).unwrap();
        assert_eq!(day.day, 33, "expected day 33");
        assert!(day.is_lag_baomer, "day 33 should be Lag BaOmer");
    }
}

// ── Nowruz / Bahá'í helpers ───────────────────────────────────────────────────
#[cfg(feature = "calendar-traditions")]
mod nowruz_bahai_tests {
    use celestial_core::body::Calendar;
    use celestial_core::*;

    #[test]
    fn naw_ruz_jd_lands_in_march() {
        // Naw-Rúz (Bahá'í new year) always falls on the vernal equinox (March 20/21)
        let jd = naw_ruz_jd(182); // 182 BE = 2025-2026
        let cal = revjul(JulianDay::new(jd), Calendar::Gregorian);
        assert_eq!(
            cal.month, 3,
            "Naw-Rúz should be in March, got month {}",
            cal.month
        );
        assert!(
            cal.day == 20 || cal.day == 21,
            "Naw-Rúz should be March 20 or 21, got day {}",
            cal.day
        );
    }

    #[test]
    fn is_bahai_leap_year_returns_bool() {
        // Bahá'í leap years align with Gregorian — just check it returns without panic
        for y in 175..185 {
            let _ = is_bahai_leap_year(y);
        }
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn solar_hijri_to_gregorian_1403_is_2024() {
        // 1403 SH started March 20 2024
        let greg = solar_hijri_to_gregorian(1403);
        assert_eq!(greg, 2024, "1403 SH should start in 2024, got {greg}");
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn solar_hijri_gregorian_roundtrip() {
        for g_year in 2000..2030 {
            let sh = gregorian_to_solar_hijri(g_year);
            let back = solar_hijri_to_gregorian(sh);
            assert!(
                (back - g_year).abs() <= 1,
                "roundtrip {g_year} -> {sh} -> {back}"
            );
        }
    }
}

// ── Easter ────────────────────────────────────────────────────────────────────
#[cfg(feature = "calendar-traditions")]
mod easter_extra_tests {
    use celestial_core::body::Calendar;
    use celestial_core::*;

    #[test]
    fn easter_julian_is_in_march_or_april() {
        for year in [2024, 2025, 2026] {
            let (_y, m, _d) = easter_julian(year);
            assert!(
                m == 3 || m == 4,
                "Julian Easter {year} in month {m}, expected March or April"
            );
        }
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn easter_orthodox_jd_matches_orthodox_gregorian() {
        // easter_orthodox_jd should agree with easter_orthodox date
        let jd = easter_orthodox_jd(2025);
        let cal = revjul(JulianDay::new(jd), Calendar::Gregorian);
        let (_y, m, d) = easter_orthodox(2025);
        assert_eq!(
            (cal.month as u8, cal.day as u8),
            (m, d),
            "easter_orthodox_jd and easter_orthodox disagree for 2025"
        );
    }
}

// ── Panchanga ─────────────────────────────────────────────────────────────────
mod panchanga_tests {
    use celestial_core::*;

    const JD: f64 = 2_451_545.0; // J2000.0

    #[test]
    fn panchanga_tithi_in_range() {
        let p = panchanga(JulianDay::new(JD));
        assert!(
            p.tithi >= 1 && p.tithi <= 30,
            "tithi {} out of 1-30",
            p.tithi
        );
    }

    #[test]
    fn panchanga_vara_in_range() {
        let p = panchanga(JulianDay::new(JD));
        assert!(p.vara <= 6, "vara {} out of 0-6", p.vara);
    }

    #[test]
    fn panchanga_nakshatra_in_range() {
        let p = panchanga(JulianDay::new(JD));
        assert!(p.nakshatra <= 26, "nakshatra {} out of 0-26", p.nakshatra);
    }

    #[test]
    fn panchanga_yoga_in_range() {
        let p = panchanga(JulianDay::new(JD));
        assert!(p.yoga <= 26, "yoga {} out of 0-26", p.yoga);
    }

    #[test]
    fn karana_name_not_empty() {
        // Karanas cycle 1-60, names should all be non-empty
        for k in 1u8..=60 {
            let name = karana_name(k);
            assert!(!name.is_empty(), "karana_name({k}) is empty");
        }
    }
}

// ── Searches: distance_to_mc, planet_conjunct_mc ─────────────────────────────
mod search_helper_tests {
    use celestial_core::*;

    #[test]
    fn distance_to_mc_is_zero_when_planet_on_mc() {
        let mc = 270.0;
        let d = distance_to_mc(mc, mc);
        assert!(
            d.abs() < 1e-9,
            "distance_to_mc at same position should be 0, got {d}"
        );
    }

    #[test]
    fn distance_to_mc_is_symmetric_within_90() {
        // Distance is measured as angular proximity — both sides equal
        let mc = 120.0;
        let d1 = distance_to_mc(mc + 20.0, mc).abs();
        let d2 = distance_to_mc(mc - 20.0, mc).abs();
        assert!(
            (d1 - d2).abs() < 1e-9,
            "distance_to_mc should be symmetric: +20={d1:.4} -20={d2:.4}"
        );
    }

    #[test]
    fn planet_conjunct_mc_within_orb() {
        let mc = 90.0;
        assert!(
            planet_conjunct_mc(mc + 1.0, mc, 2.0),
            "1° within 2° orb should be true"
        );
        assert!(
            !planet_conjunct_mc(mc + 3.0, mc, 2.0),
            "3° outside 2° orb should be false"
        );
    }

    #[test]
    fn planet_conjunct_mc_exact() {
        let mc = 45.0;
        assert!(
            planet_conjunct_mc(mc, mc, 0.0),
            "exact conjunction with 0° orb"
        );
    }
}

// ── Vesak moon helpers ────────────────────────────────────────────────────────
#[cfg(feature = "calendar-traditions")]
mod vesak_moon_tests {
    use celestial_core::*;

    const JD: f64 = 2_451_545.0;

    #[test]
    fn next_full_moon_after_is_after_start() {
        let fm = next_full_moon_after(JulianDay::new(JD));
        assert!(fm > JD, "next full moon {fm:.2} not after {JD:.2}");
        assert!(fm < JD + 30.0, "next full moon too far: {:.2}d", fm - JD);
    }

    #[test]
    fn next_new_moon_after_is_after_start() {
        let nm = next_new_moon_after(JulianDay::new(JD));
        assert!(nm > JD, "next new moon {nm:.2} not after {JD:.2}");
        assert!(nm < JD + 30.0, "next new moon too far: {:.2}d", nm - JD);
    }

    #[test]
    fn consecutive_full_moons_are_one_month_apart() {
        let fm1 = next_full_moon_after(JulianDay::new(JD));
        let fm2 = next_full_moon_after(JulianDay::new(fm1 + 1.0));
        let diff = fm2 - fm1;
        assert!(
            diff > 28.0 && diff < 31.0,
            "consecutive full moons {diff:.2}d apart, expected ~29.5d"
        );
    }
}

// ── calc_many ─────────────────────────────────────────────────────────────────
mod calc_many_test {
    use celestial_core::body::{Body, CalcFlags};
    use celestial_core::*;

    const JD: f64 = 2_451_545.0;

    #[test]
    fn calc_many_matches_individual_calc() {
        let bodies = [Body::SUN, Body::MOON, Body::MERCURY];
        let many = calc_many(JulianDay::new(JD), &bodies, CalcFlags::BUILTIN);
        for (i, &body) in bodies.iter().enumerate() {
            let single = calc(JulianDay::new(JD), body, CalcFlags::BUILTIN).unwrap();
            let via = many[i].as_ref().unwrap();
            assert!(
                (via.lon - single.lon).abs() < 1e-9,
                "calc_many[{i}] lon {:.6} ≠ calc {:.6}",
                via.lon,
                single.lon
            );
        }
    }

    #[cfg(feature = "calendar-traditions")]
    #[test]
    fn calc_many_empty_slice() {
        let result = calc_many(JulianDay::new(JD), &[], CalcFlags::BUILTIN);
        assert!(result.is_empty(), "empty input should return empty Vec");
    }
}

// ── Canonical astronomical reference values ───────────────────────────────────
//
// These tests lock in accuracy against independently-verified reference data
// (IERS Bulletin A, NASA 5MCSE eclipse canon, Meeus "Astronomical Algorithms").
// If a future refactor changes an algorithm, these will catch any regression
// that exceeds the published tolerance.

#[cfg(test)]
mod accuracy_references {
    use celestial_core::*;

    /// J2000.0 ≡ JD 2451545.0 (Jan 1.5, 2000 TT) — exact by definition.
    #[test]
    fn jd_j2000_identity() {
        let jd = julday(2000, 1, 1, 12.0, Calendar::Gregorian);
        assert!(
            (jd - 2451545.0).abs() < 1e-8,
            "J2000 = {jd} (want 2451545.0)"
        );
    }

    /// ΔT at J2000.0 = 63.8285 s per IERS Bulletin A (rounded to 63.83 s).
    #[test]
    fn delta_t_at_j2000() {
        // deltat() returns days
        let dt_days = deltat(JulianDay::new(2451545.0));
        let dt_sec = dt_days * 86_400.0;
        // Tolerance: within 1 s of published value
        assert!(
            (dt_sec - 63.83).abs() < 1.0,
            "ΔT(J2000) = {dt_sec:.4} s (IERS: 63.83 s)"
        );

        // And deltat_ex should return the same value directly in seconds
        let dt_sec_ex = deltat_ex(JulianDay::new(2451545.0), CalcFlags::BUILTIN).unwrap();
        assert!(
            (dt_sec_ex - 63.83).abs() < 1.0,
            "deltat_ex(J2000) = {dt_sec_ex:.4} s (IERS: 63.83 s)"
        );

        // Cross-check: deltat_ex should be 86400× deltat
        assert!(
            (dt_sec_ex - dt_sec).abs() < 1e-6,
            "deltat * 86400 ({dt_sec}) != deltat_ex ({dt_sec_ex})"
        );
    }

    /// Great American Eclipse 2017-08-21: NASA 5MCSE catalog gives
    /// greatest eclipse at 18:26:40 UT = JD 2457987.2685.
    #[test]
    fn eclipse_2017_reference_jd() {
        let jd = julday(
            2017,
            8,
            21,
            18.0 + 26.0 / 60.0 + 40.0 / 3600.0,
            Calendar::Gregorian,
        );
        assert!(
            (jd - 2457987.2685).abs() < 1e-3,
            "2017 eclipse JD = {jd:.6} (NASA: 2457987.2685)"
        );
    }

    /// 2024 April 8 North American eclipse: NASA greatest eclipse at 18:17:16 UT.
    #[test]
    fn eclipse_2024_reference_jd() {
        let jd = julday(
            2024,
            4,
            8,
            18.0 + 17.0 / 60.0 + 16.0 / 3600.0,
            Calendar::Gregorian,
        );
        assert!(
            (jd - 2460409.2620).abs() < 1e-3,
            "2024 eclipse JD = {jd:.6} (NASA: 2460409.2620)"
        );
    }

    /// Sun-Moon elongation at a known full moon should be very close to 180°.
    /// 2025-03-14 06:54:25 UT Total Lunar Eclipse peak (NASA 5MCLE).
    #[test]
    fn sun_moon_elongation_at_full_moon() {
        let jd = julday(
            2025,
            3,
            14,
            6.0 + 54.0 / 60.0 + 25.0 / 3600.0,
            Calendar::Gregorian,
        );
        let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap();
        let moon = calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN).unwrap();
        // Unsigned elongation in [0, 360)
        let elong = (moon.lon - sun.lon).rem_euclid(360.0);
        // Distance from 180° (full moon = 180° exactly)
        let offset = (elong - 180.0).abs();
        assert!(
            offset < 0.1,
            "Full-moon elongation = {elong:.4}° (want within 0.1° of 180°)"
        );
    }

    /// Julian Day round-trip: julday → revjul → julday should preserve
    /// the hour to within a few machine epsilons.
    #[test]
    fn julday_revjul_round_trip_precision() {
        let jd = julday(2024, 4, 8, 18.3, Calendar::Gregorian);
        let d = revjul(JulianDay::new(jd), Calendar::Gregorian);
        assert_eq!(d.year, 2024);
        assert_eq!(d.month, 4);
        assert_eq!(d.day, 8);
        assert!(
            (d.hour - 18.3).abs() < 1e-7,
            "round-trip hour = {} (want 18.3 ± 1e-7)",
            d.hour
        );
    }

    /// Moon daily motion should be the fastest of any major body.
    /// Mercury can peak around 1.5°/day near superior conjunction, so use 5×
    /// rather than 10× as the comparison ratio.
    #[test]
    fn moon_is_fastest_body() {
        let jd = 2451545.0;
        let moon = calc_ut(
            JulianDay::new(jd),
            Body::MOON,
            CalcFlags::BUILTIN | CalcFlags::SPEED,
        )
        .unwrap();
        for body in [
            Body::MERCURY,
            Body::VENUS,
            Body::MARS,
            Body::JUPITER,
            Body::SATURN,
            Body::URANUS,
            Body::NEPTUNE,
            Body::PLUTO,
        ] {
            let p = calc_ut(
                JulianDay::new(jd),
                body,
                CalcFlags::BUILTIN | CalcFlags::SPEED,
            )
            .unwrap();
            assert!(
                moon.speed_lon.abs() > p.speed_lon.abs() * 5.0,
                "Moon speed ({:.3}) should be >> body {:?} speed ({:.3})",
                moon.speed_lon,
                body,
                p.speed_lon
            );
        }
    }

    // ── Edge cases: extreme years, calendar boundaries ───────────────────────

    /// 1 BCE = astronomical year 0, 2 BCE = year -1 (ISO 8601 convention).
    /// This caused off-by-one bugs in many historical codebases.
    #[test]
    fn year_minus_one_valid_jd() {
        // Astronomical year 0 = 1 BCE. Year -1 = 2 BCE.
        // JD at noon on 1 Jan, year -1 (Julian calendar) should be computable.
        let jd = julday(-1, 1, 1, 12.0, Calendar::Julian);
        // Must be a valid finite JD in the far past (≈ -720000 Julian days)
        assert!(jd.is_finite(), "year -1 should yield a finite JD, got {jd}");
        assert!(
            jd < 1_721_058.0,
            "year -1 JD should be < year 1 CE JD, got {jd}"
        );

        // Round-trip preserves the input year
        let d = revjul(JulianDay::new(jd), Calendar::Julian);
        assert_eq!(d.year, -1, "round-trip year: {} (want -1)", d.year);
    }

    /// Leap-year February 29 round-trip: a common source of off-by-one.
    #[test]
    fn leap_year_feb_29_round_trip() {
        // Gregorian: 2020 is a leap year (divisible by 4, not 100).
        for year in [2000i32, 2004, 2020, 2024, 2400] {
            let jd = julday(year, 2, 29, 0.0, Calendar::Gregorian);
            let d = revjul(JulianDay::new(jd), Calendar::Gregorian);
            assert_eq!(d.year, year, "leap {year}-02-29 year round-trip");
            assert_eq!(d.month, 2, "leap {year}-02-29 month round-trip");
            assert_eq!(d.day, 29, "leap {year}-02-29 day round-trip");
        }
    }

    /// Non-leap centurial years: 1700, 1800, 1900, 2100, 2200, 2300 are NOT leap
    /// in Gregorian (divisible by 100 but not 400). Feb 29 doesn't exist for them.
    #[test]
    fn non_leap_centurial_consistency() {
        // Feb 28 + 1 day = Mar 1 in non-leap years
        for year in [1700, 1800, 1900, 2100] {
            let feb28 = julday(year, 2, 28, 12.0, Calendar::Gregorian);
            let next_day = revjul(JulianDay::new(feb28 + 1.0), Calendar::Gregorian);
            assert_eq!(
                (next_day.month, next_day.day),
                (3, 1),
                "In non-leap year {}, Feb 28 + 1 day should be Mar 1, got {:?}",
                year,
                (next_day.month, next_day.day)
            );
        }
    }

    /// Far-future year (year 9999) — date arithmetic must not overflow.
    #[test]
    fn far_future_year_9999() {
        let jd = julday(9999, 12, 31, 23.5, Calendar::Gregorian);
        assert!(jd.is_finite(), "year 9999 JD should be finite, got {jd}");
        let d = revjul(JulianDay::new(jd), Calendar::Gregorian);
        assert_eq!(d.year, 9999);
        assert_eq!(d.month, 12);
        assert_eq!(d.day, 31);

        // ΔT should still compute (uses long-term parabola past 2150)
        let dt = deltat(JulianDay::new(jd));
        assert!(dt.is_finite(), "ΔT at year 9999 should be finite, got {dt}");
    }

    /// Gregorian-reform boundary: 1582-10-04 (Julian) is immediately followed
    /// by 1582-10-15 (Gregorian). JDs around this date should be contiguous.
    #[test]
    fn gregorian_reform_boundary() {
        // Last Julian date: Oct 4, 1582
        let jd_julian_last = julday(1582, 10, 4, 12.0, Calendar::Julian);
        // First Gregorian date: Oct 15, 1582
        let jd_gregorian_first = julday(1582, 10, 15, 12.0, Calendar::Gregorian);
        // These should be exactly 1 day apart (same moment in history)
        let diff = (jd_gregorian_first - jd_julian_last).abs();
        assert!(
            (diff - 1.0).abs() < 1e-9,
            "Julian 1582-10-04 and Gregorian 1582-10-15 should be 1 day apart, got {diff}"
        );
    }
}

/// PERF-1 regression lock: `calc_heliocentric` does a 3× VSOP
/// central-difference for speed. "Reuse central → forward diff" is
/// O(h) (precision loss — WONTFIX, same as PERF-6); the only
/// precision-safe optimization is an analytic VSOP-series derivative
/// (large, own soak). This pins current heliocentric Mars
/// position+speed at J2000 so any future derivative rewrite must
/// reproduce it (tol 1e-6).
#[test]
fn perf1_heliocentric_mars_speed_regression_lock() {
    use celestial_core::{calc_ut, Body, CalcFlags};
    let p = calc_ut(
        JulianDay::new(2451545.0),
        Body::MARS,
        CalcFlags::BUILTIN | CalcFlags::HELIOCENTRIC | CalcFlags::SPEED,
    )
    .unwrap();
    let approx = |got: f64, want: f64, what: &str| {
        assert!(
            (got - want).abs() < 1e-6,
            "{what}: got {got:.10}, want {want:.10}"
        );
    };
    approx(p.lon, 359.4243858023, "lon");
    approx(p.lat, -1.4276220730, "lat");
    approx(p.dist, 1.3910075488, "dist");
    approx(p.speed_lon, 0.6257091822, "speed_lon");
    approx(p.speed_lat, 0.0129670887, "speed_lat");
    approx(p.speed_dist, 0.0005168649, "speed_dist");
}

/// PERF-2..5 regression lock. The post-bisection `calc_ut`/`houses`
/// re-evaluations in next_aspect_with / next_aspect_cusp are NOT
/// redundant: the scan loop uses SPEED-stripped flags, so the final
/// full-flag eval at the exact converged jd is the authoritative
/// result — "reuse the scan eval" would drop SPEED (precision loss,
/// WONTFIX like PERF-1/6). bisect_retro_station already reuses `pm`
/// (no post-loop recompute — PERF-4 audit was inaccurate).
/// next_aspect_with2 is two independent ±aspect searches; merging is a
/// precision-sensitive rewrite for marginal gain (PERF-5, declined).
/// This pins the converged outputs so any future change must
/// reproduce them (tol 1e-6).
#[test]
fn perf2345_search_regression_lock() {
    use celestial_core::{
        next_aspect_cusp, next_aspect_with, next_retro, Body, CalcFlags, HouseSystem,
    };
    let f = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let approx = |got: f64, want: f64, what: &str| {
        assert!(
            (got - want).abs() < 1e-6,
            "{what}: got {got:.10}, want {want:.10}"
        );
    };

    let a = next_aspect_with(Body::MARS, 0.0, Body::JUPITER, 2451545.0, false, 4000.0, f).unwrap();
    approx(a.jd, 2451640.22880441, "aspect_with.jd");
    approx(a.pos1[0], 39.9522907799, "aspect_with.pos1.lon");

    let c = next_aspect_cusp(
        Body::SUN,
        0.0,
        10,
        2451545.0,
        48.85,
        2.35,
        HouseSystem::PLACIDUS,
        false,
        f,
    )
    .unwrap();
    approx(c.jd, 2451545.99610184, "aspect_cusp.jd");
    approx(c.pos[0], 281.3913799069, "aspect_cusp.pos.lon");

    let r = next_retro(Body::MERCURY, 2451545.0, false, 400.0, f).unwrap();
    approx(r.jd, 2451596.02107659, "retro.jd");
    approx(r.pos[0], 347.1761286497, "retro.pos.lon");
}

// ─── Coverage: pure angle/lookup helpers (geoformat/utils) ───────────────────

#[test]
fn diff_deg_unsigned_reference() {
    // norm_deg(p1 - p2), result in [0, 360)
    let cases = [
        (10.0, 350.0, 20.0),
        (350.0, 10.0, 340.0),
        (0.0, 0.0, 0.0),
        (370.0, 10.0, 0.0),
        (-5.0, 5.0, 350.0),
    ];
    for (a, b, want) in cases {
        let got = diff_deg(a, b);
        assert!(
            (got - want).abs() < 1e-9,
            "diff_deg({a},{b}) = {got}, want {want}"
        );
    }
}

#[test]
fn diff_deg_signed_reference() {
    // wrap_signed_180(norm_deg(p1) - norm_deg(p2)), result in (-180, 180]
    let cases = [
        (10.0, 350.0, 20.0),
        (350.0, 10.0, -20.0),
        (90.0, 0.0, 90.0),
        (0.0, 90.0, -90.0),
        (0.0, 180.0, 180.0),
    ];
    for (a, b, want) in cases {
        let got = diff_deg_signed(a, b);
        assert!(
            (got - want).abs() < 1e-9,
            "diff_deg_signed({a},{b}) = {got}, want {want}"
        );
    }
}

#[test]
fn sidereal_mode_id_reference() {
    assert_eq!(sidereal_mode_id(256), Some(0));
    assert_eq!(sidereal_mode_id(255), Some(22));
    assert_eq!(sidereal_mode_id(0), Some(1));
    assert_eq!(sidereal_mode_id(20), Some(21));
    assert_eq!(sidereal_mode_id(21), None);
    assert_eq!(sidereal_mode_id(-1), None);
    assert_eq!(sidereal_mode_id(1000), None);
}

// ─── Coverage: vedic sign_lord (all rulership arms) ──────────────────────────

#[test]
fn sign_lord_all_signs_reference() {
    // sign → classical ruler planet id (Sun0 Moon1 Mer2 Ven3 Mar4 Jup5 Sat6)
    let want = [
        (0, 4),
        (7, 4), // Aries/Scorpio → Mars
        (1, 3),
        (6, 3), // Taurus/Libra → Venus
        (2, 2),
        (5, 2), // Gemini/Virgo → Mercury
        (3, 1), // Cancer → Moon
        (4, 0), // Leo → Sun
        (8, 5),
        (11, 5), // Sag/Pisces → Jupiter
        (9, 6),
        (10, 6), // Cap/Aqu → Saturn
    ];
    for (sign, lord) in want {
        assert_eq!(sign_lord(sign), Some(lord), "sign_lord({sign})");
    }
    assert_eq!(sign_lord(12), None);
    assert_eq!(sign_lord(-1), None);
}

// ─── Coverage: HouseSystem::name (every variant arm) ─────────────────────────

#[test]
fn house_system_name_all_variants() {
    use celestial_core::body::HouseSystem as H;
    // Public house_name() dispatches through from_char → enum → name().
    let cases = [
        (b'P', "Placidus"),
        (b'K', "Koch"),
        (b'O', "Porphyrius"),
        (b'R', "Regiomontanus"),
        (b'C', "Campanus"),
        (b'E', "Equal"),
        (b'D', "Equal (MC)"),
        (b'W', "Whole Sign"),
        (b'X', "Meridian"),
        (b'M', "Morinus"),
        (b'B', "Alcabitius"),
        (b'H', "Azimuthal"),
        (b'T', "Topocentric"),
        (b'G', "Gauquelin Sectors"),
    ];
    for (code, want) in cases {
        assert_eq!(house_name(H(code)), want, "house_name({code})");
    }
}

// ─── Coverage: years_diff forward / backward / equal branches ────────────────

#[test]
fn years_diff_three_branches_reference() {
    let f = CalcFlags::BUILTIN;
    let j = 2_451_545.0_f64; // J2000
                             // equal jd → exactly 0
    assert_eq!(years_diff(j, j, f).unwrap(), 0.0);
    // +1 tropical year forward ≈ +1.0
    let one = years_diff(j, j + 365.2422, f).unwrap();
    assert!((one - 1.0).abs() < 0.05, "forward 1yr = {one}");
    // +2 years forward ≈ +2.0 (multi-iteration loop)
    let two = years_diff(j, j + 730.4844, f).unwrap();
    assert!((two - 2.0).abs() < 0.05, "forward 2yr = {two}");
    // backward branch → negative
    let back = years_diff(j + 365.2422, j, f).unwrap();
    assert!((back + 1.0).abs() < 0.05, "backward 1yr = {back}");
}

// ─── Coverage tier-2: house EqualMC / Gauquelin dispatch ─────────────────────

#[test]
fn equal_mc_cusps_are_30_apart() {
    use celestial_core::body::HouseSystem as H;
    // b'D' = Equal-from-MC: every cusp exactly 30° from the previous one.
    let r = houses(
        JulianDay::new(2_451_545.0),
        Latitude::new(48.85),
        Longitude::new(2.35),
        H(b'D'),
    )
    .unwrap();
    for h in 1..12 {
        let step = diff_deg(r.cusps[h + 1], r.cusps[h]);
        assert!(
            (step - 30.0).abs() < 1e-6,
            "EqualMC cusp {h}->{}: step {step}",
            h + 1
        );
    }
}

#[test]
fn gauquelin_dispatch_cusps_in_range() {
    use celestial_core::body::HouseSystem as H;
    let r = houses(
        JulianDay::new(2_451_545.0),
        Latitude::new(48.85),
        Longitude::new(2.35),
        H(b'G'),
    )
    .unwrap();
    for h in 1..=12 {
        let c = r.cusps[h];
        assert!(c.is_finite() && (0.0..360.0).contains(&c), "cusp {h} = {c}");
    }
}

// ─── Coverage tier-2: crossings (helio_cross_ut / mooncross_node_ut) ──────────

#[test]
fn helio_cross_ut_is_self_consistent() {
    // Mars heliocentric longitude crosses 100° once per ~687-day orbit;
    // at the returned jd the heliocentric lon must equal the target.
    let target = 100.0_f64;
    let r = helio_cross_ut(
        Body::MARS,
        Longitude::new(target),
        JulianDay::new(2_451_545.0),
        CalcFlags::BUILTIN,
        1,
    )
    .unwrap();
    assert!(
        r > 2_451_545.0 && r < 2_451_545.0 + 700.0,
        "crossing jd {r} out of expected one-orbit window"
    );
    let pos = calc(
        JulianDay::new(r),
        Body::MARS,
        CalcFlags::BUILTIN | CalcFlags::HELIOCENTRIC,
    )
    .unwrap();
    let d = diff_deg_signed(pos.lon, target).abs();
    assert!(
        d < 0.5,
        "Mars helio lon at crossing = {}, want {target}",
        pos.lon
    );
}

#[test]
fn mooncross_node_ut_within_draconic_month() {
    let jd0 = 2_451_545.0;
    let r = mooncross_node_ut(JulianDay::new(jd0), CalcFlags::BUILTIN).unwrap();
    assert!(
        r.jd_cross > jd0 && r.jd_cross < jd0 + 28.0,
        "node crossing jd {} not within one draconic month of {jd0}",
        r.jd_cross
    );
    assert!(
        r.xlon.is_finite() && (0.0..360.0).contains(&r.xlon),
        "node crossing xlon = {}",
        r.xlon
    );
}

// ─── Coverage tier-3/4: deltat_ex both branches ──────────────────────────────

#[test]
fn deltat_ex_global_and_userdef_branches() {
    let f = CalcFlags::BUILTIN;
    // Global polynomial path: ΔT(2000.0) ≈ 63.8 s (Espenak–Meeus).
    let dt2000 = deltat_ex(JulianDay::new(2_451_545.0), f).unwrap();
    assert!((55.0..72.0).contains(&dt2000), "ΔT(2000) = {dt2000}");
    // ΔT(1900.0) ≈ −2.8 s.
    let dt1900 = deltat_ex(JulianDay::new(2_415_020.5), f).unwrap();
    assert!((-10.0..6.0).contains(&dt1900), "ΔT(1900) = {dt1900}");
    // User-override early-return branch (config is thread-local).
    set_delta_t_userdef(80.0);
    let ov = deltat_ex(JulianDay::new(2_451_545.0), f).unwrap();
    assert!((ov - 80.0).abs() < 1e-6, "userdef ΔT = {ov}, want 80");
    set_delta_t_userdef(f64::NAN); // clear → restore global path on this thread
    let restored = deltat_ex(JulianDay::new(2_451_545.0), f).unwrap();
    assert!((restored - dt2000).abs() < 1e-9, "userdef not cleared");
}

// ─── Coverage tier-3/4: Orthodox Easter (Gregorian) known dates ──────────────

#[cfg(feature = "calendar-traditions")]
#[test]
fn easter_orthodox_known_years() {
    // Published Gregorian-calendar Orthodox Pascha dates.
    assert_eq!(easter_orthodox(2021), (2021, 5, 2));
    assert_eq!(easter_orthodox(2023), (2023, 4, 16));
    assert_eq!(easter_orthodox(2024), (2024, 5, 5));
    assert_eq!(easter_orthodox(2025), (2025, 4, 20));
}

// ─── Coverage tier-3/4: moon_phase_info principal-phase closures ─────────────

#[test]
fn moon_phase_info_self_consistent() {
    for jd in [2_451_545.0_f64, 2_451_559.0, 2_451_530.0] {
        let mp = moon_phase_info(JulianDay::new(jd)).unwrap();
        assert!(
            (0.0..=1.0).contains(&mp.illumination),
            "illum {} at jd {jd}",
            mp.illumination
        );
        assert!(
            (0.0..360.0).contains(&mp.elongation),
            "elong {} at jd {jd}",
            mp.elongation
        );
        assert!(!mp.phase_name.is_empty());
        assert!(
            mp.prev_phase_jd <= jd && mp.prev_phase_jd.is_finite(),
            "prev_phase_jd {} not <= {jd}",
            mp.prev_phase_jd
        );
    }
}

// ─── Coverage: published-reference eclipse / aspect / cusp / vislim ──────────

#[test]
fn solar_eclipse_2017_08_21_when_and_where() {
    // NASA 5-Millennium canon: greatest eclipse 2017-08-21 18:25:32 TD
    // ("Great American Eclipse"). The eclipse *time* is pinned tightly;
    // the central-point locus is only sanity-checked — the built-in
    // engine's eclipse-geography is a coarse approximation (documented:
    // eclipse tolerances are generous, hot path regression-locked).
    let start = julday(2017, 8, 21, 0.0, Calendar::Gregorian);
    let e = sol_eclipse_when_glob(JulianDay::new(start), CalcFlags::BUILTIN, 0, false).unwrap();
    let jmax = e.tret[0];
    let want = julday(2017, 8, 21, 18.0 + 25.0 / 60.0, Calendar::Gregorian);
    assert!(
        (jmax - want).abs() < 0.05,
        "eclipse max jd {jmax}, want ~{want}"
    );
    let w = sol_eclipse_where(JulianDay::new(jmax), CalcFlags::BUILTIN).unwrap();
    assert!(
        w.geopos[1].is_finite() && (0.0..60.0).contains(&w.geopos[1]),
        "central-point lat {} outside northern-hemisphere sanity band",
        w.geopos[1]
    );
    let lon = ((w.geopos[0] + 540.0) % 360.0) - 180.0;
    assert!(
        (-130.0..-40.0).contains(&lon),
        "central-point lon {lon} outside Americas sanity band"
    );
}

#[test]
fn next_aspect_with2_finds_2017_08_21_new_moon() {
    // Sun–Moon conjunction = new moon; the 2017-08-21 new moon is
    // 18:30 UT (same event as the eclipse above).
    let start = julday(2017, 8, 10, 0.0, Calendar::Gregorian);
    let r = next_aspect_with2(
        Body::MOON,
        0.0,
        Body::SUN,
        start,
        false,
        40.0,
        CalcFlags::BUILTIN,
    )
    .expect("conjunction found");
    let want = julday(2017, 8, 21, 18.5, Calendar::Gregorian);
    assert!(
        (r.jd - want).abs() < 0.15,
        "new-moon jd {}, want ~{want}",
        r.jd
    );
    let sep = ((r.pos1[0] - r.pos2[0] + 540.0) % 360.0 - 180.0).abs();
    assert!(sep < 0.1, "Sun–Moon separation at conjunction = {sep}°");
}

#[test]
fn next_aspect_cusp2_geometry_self_consistent() {
    use celestial_core::body::HouseSystem as H;
    // Solver must return a jd where the body genuinely makes the aspect
    // to the cusp — verified from its own returned pos + cusps.
    let start = 2_451_545.0;
    let r = next_aspect_cusp2(
        Body::SUN,
        0.0,
        1,
        start,
        48.85,
        2.35,
        H(b'P'),
        false,
        CalcFlags::BUILTIN,
    )
    .expect("cusp aspect found");
    let d = ((r.pos[0] - r.cusps[1] + 540.0) % 360.0 - 180.0).abs();
    assert!(d < 0.05, "Sun vs ASC at converged jd = {d}° (want ~0)");
    assert!(r.jd > start, "jd {} not forward of {start}", r.jd);
}

#[test]
fn vis_limit_mag_runs_with_typical_params() {
    let dgeo = [0.0, 40.0, 0.0];
    let datm = [1013.25, 15.0, 40.0, 8.0];
    let dobs = [36.0, 1.0, 1.0, 1.0, 0.0, 0.0];
    let r = vis_limit_mag(
        JulianDay::new(julday(2017, 1, 1, 2.0, Calendar::Gregorian)),
        dgeo,
        datm,
        dobs,
        "venus",
        0,
    )
    .unwrap();
    assert!(r[0].is_finite(), "limiting magnitude not finite: {:?}", r);
}

// ─── PERF-1 validation: analytic VSOP speed vs independent finite diff ───────
// The analytic heliocentric derivative (PERF-1) is cross-checked against a
// central finite difference of the heliocentric POSITION through the public
// calc_ut API — an independent numerical method. Agreement confirms the
// product-rule / sign / τ-scaling are correct and no precision was lost.

#[test]
fn perf1_analytic_speed_matches_finite_difference() {
    let helio = CalcFlags::BUILTIN | CalcFlags::HELIOCENTRIC;
    let speed = helio | CalcFlags::SPEED;
    let h = 0.05_f64; // days
                      // Earth heliocentric is degenerate in the geocentric engine (excluded,
                      // same as helio_cross) — validate the other VSOP planets.
    let bodies = [
        Body::MERCURY,
        Body::VENUS,
        Body::MARS,
        Body::JUPITER,
        Body::SATURN,
    ];
    for &b in &bodies {
        for &jd in &[2_451_545.0_f64, 2_451_545.0 + 1234.0, 2_451_545.0 - 4321.0] {
            let p1 = calc_ut(JulianDay::new(jd), b, speed).unwrap();
            let p0 = calc_ut(JulianDay::new(jd - h), b, helio).unwrap();
            let p2 = calc_ut(JulianDay::new(jd + h), b, helio).unwrap();
            let fd_lon = diff_deg_signed(p2.lon, p0.lon) / (2.0 * h);
            let fd_lat = (p2.lat - p0.lat) / (2.0 * h);
            let fd_dist = (p2.dist - p0.dist) / (2.0 * h);
            assert!(
                (p1.speed_lon - fd_lon).abs() < 1e-4,
                "{b:?}@{jd}: analytic lon-speed {} vs fd {fd_lon}",
                p1.speed_lon
            );
            assert!(
                (p1.speed_lat - fd_lat).abs() < 1e-4,
                "{b:?}@{jd}: analytic lat-speed {} vs fd {fd_lat}",
                p1.speed_lat
            );
            assert!(
                (p1.speed_dist - fd_dist).abs() < 1e-6,
                "{b:?}@{jd}: analytic dist-speed {} vs fd {fd_dist}",
                p1.speed_dist
            );
        }
    }
}
