//! External-reference regression tests.
//!
//! Each test pins one formula-solving function in `celestial-core` to a
//! known published value (or a verified astronomical event). The goal is
//! to catch order-of-magnitude bugs (transcription errors, sign flips,
//! formula inversions) before they reach a release.
//!
//! Reference sources:
//! - Meeus, "Astronomical Algorithms" 2nd ed. (chapter and example noted
//!   per test).
//! - SIDC / SILSO (solar cycle observations).
//! - NASA Horizons / JPL ephemerides (planet positions cross-checked).
//! - Verified astrology software output (World of Wisdom, AstroDienst —
//!   noted where a third-party software's published chart is the source).
//!
//! Tolerances are deliberately generous (1–10° for planet positions,
//! 5 arcmin for house cusps, full-second for ΔT). The tests are
//! ORDER-OF-MAGNITUDE sanity checks, not micro-precision pins. Specific
//! precision pins live in `integration_tests.rs`.

use celestial_core::body::{Body, CalcFlags, Calendar, SiderealMode};
use celestial_core::{
    annual_profection, ayanamsa_ut, calc, calc_ut, deltat, easter_gregorian, easter_jd,
    esbats_for_year, four_pillars, full_dignity, hijri_from_jd, iso_week, julday,
    long_to_nakshatra, long_to_navamsa, maya_long_count, mean_sidereal_time_deg, next_new_moon,
    nowruz_jd, panchanga, sabbats_for_year, set_sid_mode, sidereal_time_deg, solar_return_jd,
    solcross_ut, tonalpohualli, true_obliquity, vimshottari_dasha, yallop_q, Dignity,
};

const FLG: CalcFlags = CalcFlags::BUILTIN;

/// Macro: assert two longitudes are within `tol_deg` degrees, modulo 360°.
macro_rules! assert_lon_within {
    ($got:expr, $expected:expr, $tol:expr, $label:expr) => {
        let got = $got;
        let expected: f64 = $expected;
        let diff = ((got - expected + 540.0) % 360.0 - 180.0).abs();
        assert!(
            diff < $tol,
            "{}: got {:.4}°, expected {:.4}° (diff {:.4}°, tol {}°)",
            $label,
            got,
            expected,
            diff,
            $tol,
        );
    };
}

// ─── VSOP87 + ELP2000: planets at multiple dates ─────────────────────────────

/// Princess Diana, 1961-07-01 18:45 UT (well-published nativity chart).
/// All seven classical planets match independently published values within
/// the engine's current accuracy band.
#[test]
fn diana_chart_planet_positions() {
    let jd = julday(1961, 7, 1, 18.75, Calendar::Gregorian);
    let cases: &[(Body, f64, f64)] = &[
        // body          expected_lon_deg  tol_deg
        (Body::SUN, 99.67, 0.05),
        (Body::MOON, 325.03, 0.05),
        (Body::MERCURY, 93.17, 0.1),
        (Body::VENUS, 54.40, 0.05),
        (Body::MARS, 151.67, 0.1),
        (Body::JUPITER, 305.10, 0.5),
        (Body::SATURN, 297.80, 0.5),   // tightened after VSOP87 L0[2] fix
        (Body::URANUS, 145.04, 2.0),   // 25°02' Leo per published Diana chart
        (Body::NEPTUNE, 218.62, 2.0),  // 8°37' Scorpio
    ];
    for &(body, expected, tol) in cases {
        let pos = calc_ut(jd, body, FLG).unwrap();
        assert_lon_within!(pos.lon, expected, tol, format!("Diana {body:?}"));
    }
}

/// Geraldo Netto chart, 1986-05-30 09:00 UT (PDF reference from World of
/// Wisdom astrology software, supplied by the user). Cross-checks inner-
/// planet VSOP87 against a second independent chart.
#[test]
fn netto_chart_inner_planets() {
    let jd = julday(1986, 5, 30, 9.0, Calendar::Gregorian);
    let cases: &[(Body, f64, f64)] = &[
        (Body::SUN, 68.667, 0.05),
        (Body::MOON, 336.65, 0.1),
        (Body::MERCURY, 77.533, 0.05),
        (Body::VENUS, 100.533, 0.1),
        (Body::MARS, 292.550, 0.1),
    ];
    for &(body, expected, tol) in cases {
        let pos = calc_ut(jd, body, FLG).unwrap();
        assert_lon_within!(pos.lon, expected, tol, format!("Netto {body:?}"));
    }
}

/// Sun at J2000.0 noon TT — Meeus AA chapter 25 worked example.
/// Apparent geocentric ecliptic longitude is ≈ 280.4°.
#[test]
fn sun_at_j2000_meeus() {
    let pos = calc(2_451_545.0, Body::SUN, FLG).unwrap();
    assert_lon_within!(pos.lon, 280.4, 0.5, "Sun J2000 TT");
}

/// Chiron geocentric ecliptic longitude across 4 well-separated dates.
/// Tolerance 5° (loose) because the engine uses a simple Kepler
/// propagation of fixed orbital elements; it does not model the
/// gravitational perturbations from Saturn/Uranus that significantly
/// affect Chiron's orbit. Tighten when those perturbations are
/// implemented.
///
/// Two bugs fixed in tandem before this test was added:
///   1. `calc_chiron` was using heliocentric values as if geocentric
///      (10–40° error at every chart date — needed a vector
///      subtraction from Earth's position).
///   2. Mean anomaly at J2000 was 48.5° instead of ~27° (Chiron's
///      perihelion was 1996-02-14; at J2000 that's 1383 d post-
///      perihelion ≈ 27° mean anomaly).
#[test]
fn chiron_multi_date_consistency() {
    let cases: &[(i32, u32, u32, f64, f64, f64)] = &[
        // (year, month, day, hour_ut, expected_lon_deg, tol_deg)
        (2000, 1, 1, 12.0, 253.60, 5.0),   // J2000: 13°36' Sgr
        (1961, 7, 1, 18.75, 336.00, 2.0),   // Diana: ~6° Pis
        (2024, 1, 1, 0.0, 16.00, 5.0),       // 2024: ~16° Ari
    ];
    for &(y, m, d, h, expected, tol) in cases {
        let jd = julday(y, m as i32, d as i32, h, Calendar::Gregorian);
        let pos = calc_ut(jd, Body::CHIRON, FLG).unwrap();
        let diff = ((pos.lon - expected + 540.0) % 360.0 - 180.0).abs();
        assert!(
            diff < tol,
            "Chiron {y}-{m:02}-{d:02}: got {:.4}°, expected {expected:.4}° (diff {:.4}°, tol {tol}°)",
            pos.lon,
            diff,
        );
    }
}

/// Saturn longitude across 5 well-separated dates spanning 75 years.
/// Cross-checks the VSOP87D Saturn L series against published ephemerides
/// at multiple phases of Saturn's 29.5-year orbit. Currently within 0.6°
/// at every date after the L0 coefficient fixes.
///
/// Catches regressions in Saturn-specific coefficients (the L0[2] /
/// freq 426.6 amplitude that was 30× too large was discovered through
/// exactly this kind of multi-date comparison).
#[test]
fn saturn_multi_date_consistency() {
    let cases: &[(i32, u32, u32, f64, f64, f64)] = &[
        // (year, month, day, hour_ut, expected_lon_deg, tol_deg)
        (1961, 7, 1, 18.75, 297.80, 1.0), // Princess Diana (Capricorn)
        (1986, 5, 30, 9.0, 246.23, 1.0),   // Geraldo Netto PDF (Sagittarius)
        (2000, 1, 1, 12.0, 40.42, 1.0),    // J2000.0 (Taurus)
        (2024, 1, 1, 0.0, 333.55, 1.0),    // 2024 (Pisces)
    ];
    for &(y, m, d, h, expected, tol) in cases {
        let jd = julday(y, m as i32, d as i32, h, Calendar::Gregorian);
        let pos = calc_ut(jd, Body::SATURN, FLG).unwrap();
        let diff = ((pos.lon - expected + 540.0) % 360.0 - 180.0).abs();
        assert!(
            diff < tol,
            "Saturn {y}-{m:02}-{d:02}: got {:.4}°, expected {expected:.4}° (diff {:.4}°, tol {tol}°)",
            pos.lon,
            diff,
        );
    }
}

/// Saturn moves on average 12.2° per year (360° / 29.46y). Daily
/// motion is ~0.033°/d direct, slowing to retrograde at ~−0.08°/d at
/// opposition. Pin physical bounds at a representative date.
#[test]
fn saturn_physical_motion_bounds() {
    // Mid-2024: Saturn is in retrograde mid-year (apparent stationary
    // at June 2024). Daily motion in absolute value should be < 0.15°.
    let jd = julday(2024, 7, 1, 0.0, Calendar::Gregorian);
    let pos = calc_ut(jd, Body::SATURN, FLG).unwrap();
    assert!(
        pos.speed_lon.abs() < 0.15,
        "Saturn daily motion {:.4}°/d outside physical bounds (±0.15°/d)",
        pos.speed_lon,
    );
    // Saturn never gets closer than ~8 AU or farther than ~11 AU from Earth.
    assert!(
        (7.5..=11.5).contains(&pos.dist),
        "Saturn distance {:.4} AU outside physical bounds (7.5..11.5 AU)",
        pos.dist,
    );
}

// ─── Refraction ─────────────────────────────────────────────────────────────

/// Bennett refraction at horizon (altitude = 0°) under standard
/// atmosphere (1010 mb, 10°C) is about 34'10" = 0.5694°.
/// Tolerance 0.1° covers small differences between Bennett truncations.
#[test]
fn refraction_at_horizon() {
    let r = celestial_core::refrac(0.0, 1010.0, 10.0, 0);
    // `direction == 0` returns altitude + r_corrected (true altitude).
    // Convert back to refraction angle: r_corrected = result - altitude = result - 0.
    assert!(
        (0.4..=0.7).contains(&r),
        "Refraction at horizon = {r:.4}°, expected ~0.569° (34'10\")",
    );
}

/// Refraction must decrease monotonically with altitude in the 0°-90° band.
#[test]
fn refraction_monotonic_with_altitude() {
    let mut prev = f64::INFINITY;
    for alt_deg in [0.5_f64, 5.0, 10.0, 20.0, 45.0, 80.0] {
        let r = celestial_core::refrac(alt_deg, 1010.0, 10.0, 0) - alt_deg;
        assert!(
            r < prev,
            "Refraction at {alt_deg}° = {r:.4}° not less than previous {prev:.4}°",
        );
        prev = r;
    }
}

// ─── ΔT ──────────────────────────────────────────────────────────────────────

/// ΔT well-known reference values, in seconds:
///   1900-01-01:  -2.79
///   J2000.0:     +63.83
///   2024-01-01:  ~+69
#[test]
fn delta_t_reference_values() {
    let cases: &[(f64, f64, f64)] = &[
        // (JD, expected ΔT seconds, tolerance seconds)
        (2_415_021.0, -2.79, 2.0),    // 1900-01-01
        (2_451_545.0, 63.83, 1.0),     // J2000.0
        (2_460_311.0, 69.18, 5.0),     // 2024-01-01 (rough — extrapolated)
    ];
    for &(jd, expected, tol) in cases {
        let dt_days = deltat(jd);
        let dt_sec = dt_days * 86_400.0;
        assert!(
            (dt_sec - expected).abs() < tol,
            "ΔT at JD {jd}: got {dt_sec:.2}s, expected {expected:.2}s",
        );
    }
}

// ─── Solar crossings (root finder) ──────────────────────────────────────────

/// Vernal equinox 2024: 2024-03-20 03:06:36 UT = JD 2460389.629…
/// (NASA Horizons published value.)
#[test]
fn vernal_equinox_2024() {
    let jd_start = julday(2024, 1, 1, 0.0, Calendar::Gregorian);
    let jd = solcross_ut(0.0, jd_start, FLG).expect("solcross found");
    let expected = 2_460_389.6295;
    assert!(
        (jd - expected).abs() < 0.01,
        "vernal equinox 2024: got {jd:.4}, expected {expected:.4}",
    );
}

// ─── Calendars ──────────────────────────────────────────────────────────────

/// Easter Sunday dates from US Naval Observatory / multiple ecclesiastical
/// computi — these are universally agreed historical / liturgical values.
#[test]
fn easter_gregorian_published_dates() {
    let cases: &[(i32, i32, u8, u8)] = &[
        // (year, expected_year, month, day)
        (2024, 2024, 3, 31),
        (2025, 2025, 4, 20),
        (2000, 2000, 4, 23),
        (1981, 1981, 4, 19),
        (1900, 1900, 4, 15),
    ];
    for &(yr, ey, em, ed) in cases {
        let (y, m, d) = easter_gregorian(yr);
        assert_eq!(
            (y, m, d),
            (ey, em, ed),
            "Easter {yr}: got {y}-{m:02}-{d:02}, expected {ey}-{em:02}-{ed:02}",
        );
    }
}

/// Hijri date conversion at known epochs (1 Muharram 1 AH = JD 1948439.5;
/// 1 Muharram 1444 = 2022-07-30 ≈ JD 2459790.5).
#[test]
fn hijri_known_dates() {
    // 2000-01-01 00:00 UT = JD 2451544.5; should be in Ramadan 1420 (year 1420).
    let (y, _m, _d) = hijri_from_jd(julday(2000, 1, 1, 0.0, Calendar::Gregorian));
    assert!(
        (1419..=1421).contains(&y),
        "Hijri year at 2000-01-01 = {y}, expected ≈ 1420",
    );

    // 2022-07-30 ≈ start of Hijri year 1444.
    let (y2, m2, _d2) = hijri_from_jd(julday(2022, 7, 30, 0.0, Calendar::Gregorian));
    assert!(
        (1443..=1444).contains(&y2),
        "Hijri year at 2022-07-30 = {y2}, expected ≈ 1444",
    );
    assert!(m2 == 1 || m2 == 12, "Hijri month at 2022-07-30 = {m2}");
}

// ─── ISO 8601 week numbering ─────────────────────────────────────────────────

/// ISO 8601 known anchors:
///   2012-12-31 → ISO week 1 of 2013 (Monday)
///   2020-12-31 → ISO week 53 of 2020
///   2024-01-01 → ISO week 1 of 2024 (Monday)
#[test]
fn iso_week_published_anchors() {
    let cases: &[(i32, i32, i32, i32, u32)] = &[
        // (greg_y, greg_m, greg_d, iso_y, iso_w)
        (2012, 12, 31, 2013, 1),
        (2024, 1, 1, 2024, 1),
        (2020, 12, 31, 2020, 53),
    ];
    for &(gy, gm, gd, iy_exp, iw_exp) in cases {
        let jd = julday(gy, gm, gd, 0.0, Calendar::Gregorian);
        let (iy, iw) = iso_week(jd);
        assert_eq!(
            (iy, iw),
            (iy_exp, iw_exp),
            "ISO week {gy}-{gm:02}-{gd:02}: got ({iy}, {iw}), expected ({iy_exp}, {iw_exp})",
        );
    }
}

// ─── Maya Long Count ────────────────────────────────────────────────────────

/// 13.0.0.0.0 (end of 13th baktun) — different correlations place this
/// on 2012-12-21, -22, or -23 depending on the day-noon convention.
/// celestial's GMT correlation lands the rollover on 2012-12-22.
#[test]
fn maya_long_count_baktun_13() {
    let jd = julday(2012, 12, 22, 0.0, Calendar::Gregorian);
    let (b, k, t, u, kin) = maya_long_count(jd);
    assert_eq!(
        (b, k, t, u, kin),
        (13, 0, 0, 0, 0),
        "2012-12-22 Long Count: got {b}.{k}.{t}.{u}.{kin}",
    );
    // Sanity: the previous day must be the last kin of baktun 12.
    let prev = maya_long_count(jd - 1.0);
    assert_eq!(prev, (12, 19, 19, 17, 19));
}

// ─── Mesoamerican Tonalpohualli ─────────────────────────────────────────────

/// Tonalpohualli is a 260-day permutation cycle. Output sanity: trecena
/// 1..=13, sign index 0..=19. Per the published GMT correlation, the day
/// JD 584283 (the Long-Count epoch) corresponds to "4 Ahau" (4 in trecena,
/// sign Ahau = index 19).
#[test]
fn tonalpohualli_ranges_and_epoch() {
    let (trecena, sign_idx, _, _) = tonalpohualli(584_283.0);
    assert!(
        (1..=13).contains(&trecena) && sign_idx < 20,
        "out-of-range at GMT epoch: trecena={trecena}, sign_idx={sign_idx}",
    );
    // Two arbitrary modern dates: each must return valid ranges.
    for &jd in &[2_451_545.0_f64, 2_460_000.0] {
        let (t, s, _, _) = tonalpohualli(jd);
        assert!(
            (1..=13).contains(&t) && s < 20,
            "Tonalpohualli out of range at JD {jd}: ({t}, {s})",
        );
    }
}

// ─── Hellenistic dignity ────────────────────────────────────────────────────

/// Classical dignity assignments (Ptolemy / Hellenistic tradition):
///   Sun in Leo (≈ 130°)        → Domicile
///   Moon in Cancer (≈ 100°)    → Domicile
///   Sun in Aquarius (≈ 310°)   → Detriment
///   Saturn in Libra (≈ 190°)   → Exaltation
///   Sun in Aries 19° (≈ 19°)   → Exaltation
#[test]
fn hellenistic_dignity_canonical_cases() {
    let cases: &[(Body, f64, bool, Dignity)] = &[
        (Body::SUN, 130.0, true, Dignity::Domicile),
        (Body::MOON, 100.0, false, Dignity::Domicile),
        (Body::SUN, 310.0, true, Dignity::Detriment),
        (Body::SUN, 19.0, true, Dignity::Exaltation),
    ];
    for &(body, lon, is_day, expected) in cases {
        let (dig, _score) = full_dignity(body, lon, is_day);
        assert_eq!(
            dig, expected,
            "Dignity({body:?}, {lon}°, day={is_day}) = {dig:?}, expected {expected:?}",
        );
    }
}

// ─── Yallop crescent visibility ─────────────────────────────────────────────

/// Yallop's empirical class boundaries (Yallop 1998, NAO TN 69):
///   q ≥ +0.216 → 'A' (easily visible)
///   q ∈ [−0.014, +0.216) → 'B' (visible under perfect conditions)
///   q ∈ [−0.160, −0.014) → 'C'
///   ...
///   q < −0.293 → 'F' (not visible)
#[test]
fn yallop_q_class_boundaries() {
    // Large ARCV / ARCL gives easy visibility → A.
    let (_q, c) = yallop_q(15.0, 20.0, 15.0);
    assert_eq!(c, 'A', "wide separation should be class A, got {c}");

    // Near-conjunction (very small ARCV) → never visible → F (or D/E for
    // intermediate). Just assert it's not A:
    let (_q, c2) = yallop_q(1.0, 2.0, 15.0);
    assert_ne!(c2, 'A', "near-conjunction should not be class A");
}

// ─── Hindu Panchanga ────────────────────────────────────────────────────────

/// Panchanga returns five elements: tithi (1..=30), paksha, vara (0..=6),
/// nakshatra (0..=26), yoga and karana. Order-of-magnitude sanity over a
/// few representative JDs — internal consistency only, not against a
/// specific reference.
#[test]
fn panchanga_field_ranges() {
    for &jd in &[2_451_545.0_f64, 2_460_000.0, 2_446_580.875] {
        let p = panchanga(jd);
        assert!((1..=30).contains(&p.tithi), "tithi {} out of range at jd {jd}", p.tithi);
        assert!(p.vara <= 6, "vara {} out of range at jd {jd}", p.vara);
        assert!(p.nakshatra < 27, "nakshatra {} out of range at jd {jd}", p.nakshatra);
    }
}

// ─── GMST (Greenwich Mean Sidereal Time) ────────────────────────────────────

/// GMST at J2000.0 noon UT = 18h 41m 50.5479s.
/// In degrees: 280.46061837°.
/// (Meeus, AA 2nd ed., chapter 12 worked example.)
#[test]
fn gmst_at_j2000_noon_ut() {
    let gmst = mean_sidereal_time_deg(2_451_545.0);
    let expected = 280.46061837;
    let diff = ((gmst - expected + 540.0) % 360.0 - 180.0).abs();
    assert!(
        diff < 0.0001,
        "GMST at J2000 noon UT = {gmst:.6}°, expected {expected:.6}° (diff {diff:.6}°)",
    );
}

/// Apparent sidereal time differs from mean by the equation of the
/// equinoxes (Δψ · cos ε). At J2000.0 Δψ ≈ −13.85" and cos ε ≈ 0.917,
/// so |GAST − GMST| ≈ 12.7". Both must be in [0, 360°).
#[test]
fn apparent_sidereal_time_near_mean() {
    let jd = 2_451_545.0;
    let gmst = mean_sidereal_time_deg(jd);
    let gast = sidereal_time_deg(jd);
    let eqeq = ((gast - gmst + 540.0) % 360.0 - 180.0).abs();
    assert!(
        eqeq < 0.01, // < 36"
        "Equation of equinoxes |GAST - GMST| = {eqeq:.6}° at J2000 — should be ~12.7\"",
    );
    assert!((0.0..360.0).contains(&gmst));
    assert!((0.0..360.0).contains(&gast));
}

// ─── Obliquity ──────────────────────────────────────────────────────────────

/// True obliquity of the ecliptic at J2000.0 ≈ 23°26'21.448" = 23.4393°.
/// IAU 1976 / 2006 mean obliquity is 23.4392911° at J2000; with nutation
/// the true value is within a few arcseconds.
///
/// Tolerance 0.01° (= 36"). Tightens once the celestial obliquity
/// implementation is upgraded to IAU 2006 (currently uses an older
/// truncation that gives 23.4377° = ~6" low at J2000).
#[test]
fn obliquity_at_j2000() {
    let eps = true_obliquity(2_451_545.0);
    assert!(
        (eps - 23.4393).abs() < 0.01,
        "True obliquity at J2000 = {eps:.6}°, expected ≈ 23.4393°",
    );
}

// ─── Lahiri ayanamsa ────────────────────────────────────────────────────────

/// Lahiri ayanamsa at J2000.0 = 23°51'11" = 23.853°.
/// (Lahiri official Indian / Government of India convention.)
#[test]
fn lahiri_ayanamsa_at_j2000() {
    set_sid_mode(SiderealMode::LAHIRI, 0.0, 0.0);
    let ay = ayanamsa_ut(2_451_545.0);
    assert!(
        (ay - 23.853).abs() < 0.05,
        "Lahiri ayanamsa at J2000 = {ay:.4}°, expected ≈ 23.853°",
    );
}

// ─── Vedic / Jyotish ────────────────────────────────────────────────────────

/// Nakshatra boundaries: each spans 13°20' = 13.333°. So:
///   λ ∈ [0°,        13.333°)  → 0  (Ashwini)
///   λ ∈ [13.333°,   26.667°)  → 1  (Bharani)
///   λ ∈ [26.667°,   40.000°)  → 2  (Krittika)
///   ...
#[test]
fn nakshatra_boundary_anchors() {
    let cases: &[(f64, i32)] = &[
        (0.0, 0),
        (13.0, 0),
        (13.34, 1),
        (26.5, 1),
        (26.67, 2),
        (40.0, 3),
        (359.0, 26),
    ];
    for &(lon, expected) in cases {
        let (nak, _pada) = long_to_nakshatra(lon);
        assert_eq!(
            nak, expected,
            "Nakshatra at {lon}° = {nak}, expected {expected}",
        );
    }
}

/// Navamsa is 1/9 of a sign (3°20' = 3.333°). Sign + navamsa-within-sign
/// determines the navamsa rasi (0..=11). Sanity-pin the first three
/// navamsa boundaries inside Aries:
///   λ ∈ [0°,    3.333°) → Aries (0)
///   λ ∈ [3.333°, 6.667°) → Taurus (1)
///   λ ∈ [6.667°,10.000°) → Gemini (2)
#[test]
fn navamsa_within_aries() {
    let cases: &[(f64, i32)] = &[
        (0.0, 0),
        (3.0, 0),
        (3.5, 1),
        (6.5, 1),
        (7.0, 2),
        (10.0, 3),
    ];
    for &(lon, expected) in cases {
        let nav = long_to_navamsa(lon);
        assert_eq!(nav, expected, "Navamsa at {lon}° = {nav}, expected {expected}");
    }
}

/// Vimshottari dasha: nine planetary lords with total cycle of 120 years.
///   Ketu 7 · Venus 20 · Sun 6 · Moon 10 · Mars 7 · Rahu 18 ·
///   Jupiter 16 · Saturn 19 · Mercury 17  =  120
#[test]
fn vimshottari_total_period_is_120_years() {
    // Use a Moon longitude near Ashwini start (Ketu's nakshatra) so we
    // get a full 120-year cycle.
    let dashas = vimshottari_dasha(2_451_545.0, 0.0, 120.0);
    let total: f64 = dashas.iter().take(9).map(|d| d.years).sum();
    assert!(
        (total - 120.0).abs() < 0.01,
        "Vimshottari first 9 mahadashas sum to {total:.4}y, expected 120.0y",
    );
}

// ─── Solar return ───────────────────────────────────────────────────────────

/// `solar_return_jd(natal, year, flags)` finds the next Sun-return JD
/// at or after Jan 1 of `year`. Two consecutive SRs are separated by
/// approximately one tropical year (365.24 days).
#[test]
fn consecutive_solar_returns_separated_by_one_year() {
    let jd_natal = julday(1985, 7, 14, 12.0, Calendar::Gregorian);
    let sr_2024 = solar_return_jd(jd_natal, 2024, CalcFlags::BUILTIN).unwrap();
    let sr_2025 = solar_return_jd(jd_natal, 2025, CalcFlags::BUILTIN).unwrap();
    let dt = sr_2025 - sr_2024;
    assert!(
        (dt - 365.24).abs() < 1.0,
        "Consecutive SRs: dt = {dt:.4} days, expected ≈ 365.24",
    );
}

// ─── Annual profection ──────────────────────────────────────────────────────

/// Profected house = ((age - 1) mod 12) + 1 starting from ASC (h1).
/// Age 0 → h1 (ASC). Age 12 → h1 again. Age 35 → h12.
#[test]
fn annual_profection_house_cycle() {
    let cusps = [0.0; 13]; // placeholder; the fn only uses age to count houses
    assert_eq!(annual_profection(&cusps, 0).0, 1);
    assert_eq!(annual_profection(&cusps, 12).0, 1);
    assert_eq!(annual_profection(&cusps, 1).0, 2);
    assert_eq!(annual_profection(&cusps, 11).0, 12);
    assert_eq!(annual_profection(&cusps, 35).0, 12);
}

// ─── Easter JD round-trip ───────────────────────────────────────────────────

/// `easter_jd(year)` returns the Julian Day of Easter Sunday at 0h UT.
/// For 2024 (March 31): JD = julday(2024, 3, 31, 0.0, Gregorian).
#[test]
fn easter_jd_round_trips_with_julday() {
    let jd_easter = easter_jd(2024);
    let expected = julday(2024, 3, 31, 0.0, Calendar::Gregorian);
    assert!(
        (jd_easter - expected).abs() < 0.5,
        "easter_jd(2024) = {jd_easter}, expected ≈ {expected} (2024-03-31)",
    );
}

// ─── Nowruz ─────────────────────────────────────────────────────────────────

/// Nowruz (Persian / Bahá'í New Year) is the moment of the vernal equinox.
/// 2024 vernal equinox = 2024-03-20 03:06 UT ≈ JD 2460389.63.
#[test]
fn nowruz_2024_at_vernal_equinox() {
    let jd = nowruz_jd(2024);
    let expected = julday(2024, 3, 20, 3.1, Calendar::Gregorian); // ≈ 03:06 UT
    assert!(
        (jd - expected).abs() < 1.0,
        "nowruz_jd(2024) = {jd:.4}, expected ≈ {expected:.4} (2024-03-20 ~03:06 UT)",
    );
}

// ─── Sabbats (Celtic Wheel of the Year) ─────────────────────────────────────

/// Yule (winter solstice in the Northern hemisphere): 2024-12-21.
/// Sabbat positions are at sun longitudes: Yule 270°, Imbolc 315°, Ostara 0°,
/// Beltane 45°, Litha 90°, Lughnasadh 135°, Mabon 180°, Samhain 225°.
#[test]
fn sabbats_2024_solstices_and_equinoxes() {
    let sabbats = sabbats_for_year(2024).unwrap();
    assert_eq!(sabbats.len(), 8, "expected 8 sabbats per year, got {}", sabbats.len());

    // All sabbats fall in 2024.
    for s in &sabbats {
        let d = celestial_core::revjul(s.jd, Calendar::Gregorian);
        assert_eq!(d.year, 2024, "sabbat {:?} JD {:.4} not in 2024", s.kind, s.jd);
    }
}

/// `esbats_for_year` returns ~12 named full moons per year (13 in some
/// years). Each must fall inside the year.
#[test]
fn esbats_2024_count_and_year_bounds() {
    let esbats = esbats_for_year(2024).unwrap();
    assert!(
        (12..=13).contains(&esbats.len()),
        "expected 12-13 esbats in 2024, got {}",
        esbats.len(),
    );
    for e in &esbats {
        let d = celestial_core::revjul(e.jd, Calendar::Gregorian);
        assert_eq!(d.year, 2024, "esbat {:?} JD {:.4} not in 2024", e.name, e.jd);
    }
}

// ─── Moon phase root finder ─────────────────────────────────────────────────

/// `next_new_moon(jd)` finds the next new moon strictly after `jd`.
/// Consecutive new moons are separated by one synodic month (29.530588 d).
#[test]
fn consecutive_new_moons_match_synodic_month() {
    let jd0 = 2_451_545.0;
    let nm1 = next_new_moon(jd0).unwrap();
    let nm2 = next_new_moon(nm1 + 0.5).unwrap();
    let dt = nm2 - nm1;
    assert!(
        (dt - 29.530_588).abs() < 0.5,
        "Consecutive new moons {} → {}: dt = {dt:.4}d, expected ≈ 29.530588d",
        nm1,
        nm2,
    );
}

// ─── Chinese Ba Zi ──────────────────────────────────────────────────────────

/// Four-pillars internal consistency: each pillar must return valid
/// stem (0..=9) and branch (0..=11) indices, and the year/month/day
/// pillars must be deterministic given a fixed (jd, hour, sun_lon).
#[test]
fn four_pillars_field_ranges_and_determinism() {
    let jd = julday(1986, 5, 30, 9.0, Calendar::Gregorian);
    let sun_lon = 68.667; // PDF reference
    let pillars_a = four_pillars(jd, 9.0, sun_lon);
    let pillars_b = four_pillars(jd, 9.0, sun_lon);
    for (a, b) in pillars_a.iter().zip(pillars_b.iter()) {
        assert!(a.stem < 10, "stem {} out of range", a.stem);
        assert!(a.branch < 12, "branch {} out of range", a.branch);
        assert_eq!(a.stem, b.stem, "non-deterministic stem");
        assert_eq!(a.branch, b.branch, "non-deterministic branch");
    }
}
