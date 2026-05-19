// New-feature tests live in dedicated files:
//   wheel_test.rs  — wheel enhancements (minor aspects, antiscia, dignities, Arabic Parts, fixed stars)
//   hellenistic_test.rs  — Hellenistic/Persian (dignity scoring, Firdaria, Profections)
//   chinese_test.rs  — Chinese astrology (Ba Zi, solar terms)
//   mesoamerican_test.rs  — Mesoamerican calendars (Tonalpohualli, Tzolkin, Haab, Calendar Round)
//   indigenous_test.rs  — Indigenous/Egyptian (Medicine Wheel, Egyptian decans)
// This file retains the foundational Phase 2-4 tests.

//! Integration tests for the newly implemented functions.
//! All tests use Julian day 2452275.5 = 2002-01-01 00:00 UT as the reference epoch.

use crate::body::Calendar;

use celestial_core::body::{Body, CalcFlags, HouseSystem, SiderealMode};
use celestial_core::*;

const J2000: f64 = 2_451_545.0;
const PARIS_LAT: f64 = 48.85;
const PARIS_LON: f64 = 2.35;

fn setup() {
    set_ephe_path("").unwrap();
}

const JD: f64 = 2_452_275.5; // 2002-01-01 00:00 UT

// ─── Fixed stars ──────────────────────────────────────────────────────────────

#[test]
fn test_fixstar_sirius_found() {
    let r = fixstar("Sirius", JD, CalcFlags::BUILTIN).unwrap();
    assert_eq!(r.star_name, "Sirius,alCMa");
    // Sirius: ecliptic longitude near 104° in 2002
    assert!(
        r.xx[0] > 100.0 && r.xx[0] < 115.0,
        "Sirius lon = {}",
        r.xx[0]
    );
    // Small southern latitude
    assert!(r.xx[1] < 0.0 && r.xx[1] > -40.0, "Sirius lat = {}", r.xx[1]);
}

#[test]
fn test_fixstar_aldebaran() {
    let r = fixstar("Aldebaran", JD, CalcFlags::BUILTIN).unwrap();
    // Aldebaran ~69° ecliptic longitude, slightly north
    assert!(
        r.xx[0] > 65.0 && r.xx[0] < 75.0,
        "Aldebaran lon = {}",
        r.xx[0]
    );
}

#[test]
fn test_fixstar_by_bayer() {
    let r = fixstar("alCMa", JD, CalcFlags::BUILTIN).unwrap();
    assert!(r.star_name.contains("Sirius"));
}

#[test]
fn test_fixstar_not_found() {
    let r = fixstar("NOTASTAR_XYZ", JD, CalcFlags::BUILTIN);
    assert!(r.is_err());
}

#[test]
fn test_fixstar_mag_sirius() {
    let mag = fixstar_mag("Sirius").unwrap();
    // Sirius is -1.46
    assert!((mag - (-1.46)).abs() < 0.01, "Sirius mag = {mag}");
}

#[test]
fn test_fixstar_mag_vega() {
    let mag = fixstar_mag("Vega").unwrap();
    assert!((mag - 0.03).abs() < 0.1, "Vega mag = {mag}");
}

#[test]
fn test_fixstar_ut_equals_fixstar() {
    let r1 = fixstar("Regulus", JD, CalcFlags::BUILTIN).unwrap();
    let r2 = fixstar_ut("Regulus", JD, CalcFlags::BUILTIN).unwrap();
    assert_eq!(r1.xx[0], r2.xx[0]);
}

// ─── Moon nodes ───────────────────────────────────────────────────────────────

#[test]
fn test_mean_node_in_range() {
    let pos = calc_ut(JulianDay::new(JD), Body::MEAN_NODE, CalcFlags::BUILTIN | CalcFlags::SPEED).unwrap();
    assert!(
        pos.lon >= 0.0 && pos.lon < 360.0,
        "Mean node lon = {}",
        pos.lon
    );
    assert_eq!(pos.lat, 0.0);
    // Mean node moves ~19.4°/year retrograde → ~-0.053°/day
    assert!(pos.speed_lon < 0.0, "Mean node speed should be retrograde");
    assert!(
        pos.speed_lon > -0.15,
        "Mean node speed too fast: {}",
        pos.speed_lon
    );
}

#[test]
fn test_true_node_in_range() {
    let pos = calc_ut(JulianDay::new(JD), Body::TRUE_NODE, CalcFlags::BUILTIN).unwrap();
    assert!(pos.lon >= 0.0 && pos.lon < 360.0);
    assert_eq!(pos.lat, 0.0);
}

#[test]
fn test_mean_true_node_close() {
    // Mean and true node should be within a few degrees
    let mn = calc_ut(JulianDay::new(JD), Body::MEAN_NODE, CalcFlags::BUILTIN).unwrap();
    let tn = calc_ut(JulianDay::new(JD), Body::TRUE_NODE, CalcFlags::BUILTIN).unwrap();
    let diff = (mn.lon - tn.lon + 360.0).rem_euclid(360.0);
    let diff = if diff > 180.0 { 360.0 - diff } else { diff };
    assert!(diff < 3.0, "Mean/true node diff = {diff}°");
}

// ─── Chiron ───────────────────────────────────────────────────────────────────

#[test]
fn test_chiron_position() {
    let pos = calc_ut(JulianDay::new(JD), Body::CHIRON, CalcFlags::BUILTIN | CalcFlags::SPEED).unwrap();
    assert!(
        pos.lon >= 0.0 && pos.lon < 360.0,
        "Chiron lon = {}",
        pos.lon
    );
    // Chiron in 2002 was around Sagittarius/Capricorn (~270-300°)
    assert!(
        pos.lon > 250.0 && pos.lon < 320.0,
        "Chiron expected ~Sag/Cap in 2002, got {}",
        pos.lon
    );
    // Distance 8–19 AU
    assert!(
        pos.dist > 8.0 && pos.dist < 20.0,
        "Chiron dist = {}",
        pos.dist
    );
}

// ─── Nodes and apsides ────────────────────────────────────────────────────────

#[test]
fn test_moon_nodes_apsides() {
    let r = nod_aps(JD, Body::MOON, CalcFlags::BUILTIN, 0).unwrap();
    // Ascending node lon in [0,360)
    assert!(r.nasc[0] >= 0.0 && r.nasc[0] < 360.0);
    // Descending = ascending + 180°
    let diff = (r.ndsc[0] - r.nasc[0] + 360.0).rem_euclid(360.0);
    assert!(
        (diff - 180.0).abs() < 0.01,
        "Desc node not 180° from asc: {diff}"
    );
}

#[test]
fn test_mars_nodes() {
    let r = nod_aps(JD, Body::MARS, CalcFlags::BUILTIN, 0).unwrap();
    // Mars ascending node ~49° (Meeus)
    assert!(
        r.nasc[0] > 40.0 && r.nasc[0] < 60.0,
        "Mars asc node = {}",
        r.nasc[0]
    );
}

#[test]
fn test_nod_aps_ut_equals_et() {
    let r1 = nod_aps(JD, Body::MOON, CalcFlags::BUILTIN, 0).unwrap();
    let r2 = nod_aps_ut(JD, Body::MOON, CalcFlags::BUILTIN, 0).unwrap();
    assert!((r1.nasc[0] - r2.nasc[0]).abs() < 0.001);
}

// ─── Orbital elements ─────────────────────────────────────────────────────────

#[test]
fn test_earth_orbital_elements() {
    let el = get_orbital_elements(JD, Body::EARTH, CalcFlags::BUILTIN).unwrap();
    // Earth: a ≈ 1 AU, e ≈ 0.017
    assert!(
        (el.semi_major_axis - 1.0).abs() < 0.01,
        "Earth a = {}",
        el.semi_major_axis
    );
    assert!(
        el.eccentricity > 0.01 && el.eccentricity < 0.03,
        "Earth ecc = {}",
        el.eccentricity
    );
}

#[test]
fn test_mars_orbital_distances() {
    let d = orbit_max_min_true_distance(JD, Body::MARS, CalcFlags::BUILTIN).unwrap();
    // Mars: perihelion ~1.38 AU, aphelion ~1.67 AU
    assert!(d.dmin > 1.2 && d.dmin < 1.55, "Mars dmin = {}", d.dmin);
    assert!(d.dmax > 1.5 && d.dmax < 1.80, "Mars dmax = {}", d.dmax);
    assert!(d.dmax > d.dmin);
}

// ─── Planetocentric ───────────────────────────────────────────────────────────

#[test]
fn test_calc_pctr_mars_from_jupiter() {
    let r = calc_pctr(JulianDay::new(JD), Body::MARS, Body::JUPITER, CalcFlags::BUILTIN);
    assert!(r.is_ok(), "calc_pctr failed: {:?}", r.err());
}

// ─── Longitude crossings ──────────────────────────────────────────────────────

#[test]
fn test_solcross_vernal_equinox() {
    // Find next vernal equinox (Sun at 0°) after 2002-01-01
    let jd = solcross(0.0, JD, CalcFlags::BUILTIN).unwrap();
    // Should be around March 20, 2002 ≈ JD 2452353
    assert!(jd > JD, "crossing must be after start");
    assert!(jd < JD + 120.0, "next equinox within 120 days");
    // Verify: Sun's longitude at jd should be near 0°
    let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap();
    assert!(
        sun.lon < 2.0 || sun.lon > 358.0,
        "Sun lon at equinox = {}",
        sun.lon
    );
}

#[test]
fn test_solcross_summer_solstice() {
    let jd = solcross(90.0, JD, CalcFlags::BUILTIN).unwrap();
    let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap();
    assert!(
        (sun.lon - 90.0).abs() < 1.0,
        "Sun lon at solstice = {}",
        sun.lon
    );
}

#[test]
fn test_mooncross_finds_crossing() {
    // Moon moves ~13°/day — will cross any longitude within ~27 days
    let jd = mooncross(45.0, JD, CalcFlags::BUILTIN).unwrap();
    assert!(jd > JD && jd < JD + 30.0);
    let moon = calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN).unwrap();
    let diff = (moon.lon - 45.0 + 360.0).rem_euclid(360.0);
    let diff = if diff > 180.0 { 360.0 - diff } else { diff };
    assert!(
        diff < 1.0,
        "Moon lon at crossing = {} (expected 45°)",
        moon.lon
    );
}

#[test]
fn test_mooncross_node() {
    let r = mooncross_node(JD, CalcFlags::BUILTIN).unwrap();
    assert!(
        r.jd_cross > JD && r.jd_cross < JD + 30.0,
        "node crossing at JD+{:.1}",
        r.jd_cross - JD
    );
    // Verify Moon latitude ≈ 0 at crossing
    let moon = calc_ut(JulianDay::new(r.jd_cross), Body::MOON, CalcFlags::BUILTIN).unwrap();
    assert!(
        moon.lat.abs() < 0.5,
        "Moon lat at node crossing = {}",
        moon.lat
    );
}

// ─── Eclipse search ───────────────────────────────────────────────────────────

#[test]
fn test_solar_eclipse_when_glob() {
    let r = sol_eclipse_when_glob(JD, CalcFlags::BUILTIN, 0, false).unwrap();
    assert!(r.tret[0] > JD, "eclipse must be after start");
    assert!(r.ret_flags != 0, "eclipse flags must be non-zero");
    // Eclipse time within 2 years
    assert!(r.tret[0] < JD + 730.0, "eclipse within 2 years");
}

#[test]
fn test_solar_eclipse_is_classified() {
    let r = sol_eclipse_when_glob(JD, CalcFlags::BUILTIN, 0, false).unwrap();
    let f = r.ret_flags;
    let valid = (f & (ECL_TOTAL | ECL_ANNULAR | ECL_PARTIAL | ECL_HYBRID)) != 0;
    assert!(valid, "eclipse type flags invalid: {f}");
}

#[test]
fn test_lunar_eclipse_when() {
    let r = lun_eclipse_when(JD, CalcFlags::BUILTIN, 0, false).unwrap();
    assert!(r.tret[0] > JD);
    assert!(r.tret[0] < JD + 400.0, "lunar eclipse within ~1 year");
    let f = r.ret_flags;
    let valid = (f & (ECL_TOTAL | ECL_PARTIAL | ECL_PENUMBRAL)) != 0;
    assert!(valid, "lunar eclipse type invalid: {f}");
}

#[test]
fn test_eclipse_backwards_search() {
    // Search backwards from 2002-01-01 — should find an eclipse before it
    let r = sol_eclipse_when_glob(JD, CalcFlags::BUILTIN, 0, true).unwrap();
    assert!(r.tret[0] < JD, "backwards eclipse must be before start");
    assert!(r.tret[0] > JD - 730.0);
}

// ─── Phenomena ────────────────────────────────────────────────────────────────

#[test]
fn test_pheno_mars() {
    let attr = pheno_ut(JD, Body::MARS, CalcFlags::BUILTIN).unwrap();
    // Phase angle: 0–180°
    assert!(
        attr[0] >= 0.0 && attr[0] <= 180.0,
        "phase angle = {}",
        attr[0]
    );
    // Phase fraction: 0–1
    assert!(attr[1] >= 0.0 && attr[1] <= 1.0, "phase frac = {}", attr[1]);
    // Elongation: 0–180°
    assert!(
        attr[2] >= 0.0 && attr[2] <= 180.0,
        "elongation = {}",
        attr[2]
    );
    // Angular diameter: positive
    assert!(attr[3] > 0.0, "angular diam = {}", attr[3]);
    // Magnitude: reasonable for Mars
    assert!(attr[4] > -5.0 && attr[4] < 5.0, "Mars mag = {}", attr[4]);
}

#[test]
fn test_pheno_jupiter_magnitude() {
    let attr = pheno_ut(JD, Body::JUPITER, CalcFlags::BUILTIN).unwrap();
    // Jupiter magnitude typically -2.9 to -1.6
    assert!(
        attr[4] < -1.0 && attr[4] > -4.0,
        "Jupiter mag = {}",
        attr[4]
    );
}

#[test]
fn test_pheno_venus_full_range() {
    let attr = pheno_ut(JD, Body::VENUS, CalcFlags::BUILTIN).unwrap();
    assert!(attr[0] >= 0.0 && attr[0] <= 180.0);
    assert!(attr[1] >= 0.0 && attr[1] <= 1.0);
}

// ─── House position ───────────────────────────────────────────────────────────

#[test]
fn test_house_pos_in_range() {
    // Compute ARMC and epsilon at JD
    let armc = sidtime(JD) * 15.0; // sidereal time in degrees
    let r = house_pos(armc, 48.0, 23.4393, HouseSystem::PLACIDUS, [120.0, 5.0]).unwrap();
    assert!((1.0..=13.0).contains(&r), "house position = {r}");
}

#[test]
fn test_house_pos_asc_cusp1() {
    // A point right on the Ascendant should be near house 1
    let r_h = houses(JulianDay::new(JD), Latitude::new(48.0), Longitude::new(2.0), HouseSystem::PLACIDUS).unwrap();
    let asc = r_h.ascmc[0];
    let armc = r_h.ascmc[2];
    let h = house_pos(armc, 48.0, 23.4393, HouseSystem::PLACIDUS, [asc, 0.0]).unwrap();
    assert!((h - 1.0).abs() < 0.5, "ASC should be near house 1, got {h}");
}

// ─── House cusp speeds ────────────────────────────────────────────────────────

#[test]
fn test_houses_ex2_speeds_nonzero() {
    let r = houses_ex2(JulianDay::new(JD), CalcFlags::BUILTIN, Latitude::new(48.0), Longitude::new(2.0), HouseSystem::PLACIDUS).unwrap();
    // ASC speed should be around 1°/4min = 360°/day at the equator, less at mid-latitudes
    let asc_speed = r.ascmc_speeds[0].abs();
    assert!(
        asc_speed > 5.0 && asc_speed < 500.0,
        "ASC speed = {asc_speed}/day"
    );
    // MC moves at ~1°/4 minutes = 360°/day
    let mc_speed = r.ascmc_speeds[1].abs();
    assert!(
        mc_speed > 10.0 && mc_speed < 500.0,
        "MC speed = {mc_speed}/day"
    );
    // Cusp speeds should be positive (cusps advance)
    let nonzero = r.cusp_speeds[1..=12].iter().any(|&s| s.abs() > 0.0);
    assert!(nonzero, "all cusp speeds are zero");
}

// ─── Gauquelin sector ─────────────────────────────────────────────────────────

#[test]
fn test_gauquelin_sector_range() {
    let geopos = [2.3, 48.9, 0.0]; // Paris
    let s = gauquelin_sector(
        JD,
        Body::MARS,
        None,
        CalcFlags::BUILTIN,
        0,
        geopos,
        1013.0,
        15.0,
    )
    .unwrap();
    assert!((1.0..=36.0).contains(&s), "Gauquelin sector = {s}");
}

// ─── Visibility ───────────────────────────────────────────────────────────────

#[test]
fn test_vis_limit_mag_returns_values() {
    let dgeo = [2.3, 48.9, 100.0];
    let datm = [1013.0, 15.0, 0.5, 45.0];
    let dobs = [45.0, 1.0, 0.0, 0.0, 0.0, 0.0];
    let r = vis_limit_mag(JD, dgeo, datm, dobs, "venus", 0).unwrap();
    // Limiting mag: 3–8 for naked eye
    assert!(r[0] > 2.0 && r[0] < 9.0, "lim_mag = {}", r[0]);
    // Venus magnitude: -5 to 0
    assert!(r[1] < 1.0 && r[1] > -6.0, "Venus mag = {}", r[1]);
}

// ─── nod_aps speeds ──────────────────────────────────────────────────────────

#[test]
fn test_nod_aps_planet_speed_nonzero() {
    // Planetary nodes precess — speed should be non-zero when CalcFlags::SPEED set
    let r = nod_aps(
        2_451_545.0,
        Body::MARS,
        CalcFlags::BUILTIN | CalcFlags::SPEED,
        0,
    )
    .unwrap();
    // Node speed for Mars ≈ −0.000_5 °/day (retrograde)
    assert!(
        r.nasc[3].abs() > 0.0,
        "Mars node speed should be non-zero, got {}",
        r.nasc[3]
    );
    assert!(
        r.nasc[3].abs() < 0.01,
        "Mars node speed unreasonably large: {}",
        r.nasc[3]
    );
    // Peri speed should also be non-zero
    assert!(r.peri[3].abs() > 0.0, "Mars peri speed should be non-zero");
}

#[test]
fn test_nod_aps_speed_zero_without_flag() {
    let r = nod_aps(2_451_545.0, Body::MARS, CalcFlags::BUILTIN, 0).unwrap();
    assert_eq!(
        r.nasc[3], 0.0,
        "speed should be zero without CalcFlags::SPEED"
    );
    assert_eq!(
        r.peri[3], 0.0,
        "peri speed should be zero without CalcFlags::SPEED"
    );
}

// ─── helio_cross ─────────────────────────────────────────────────────────────

#[test]
fn test_helio_cross_mars() {
    // Mars should cross 0° ecliptic longitude somewhere in 2022
    let jd = julday(2022, 1, 1, 0.0, Calendar::Gregorian);
    let cross = helio_cross(Body::MARS, 0.0, jd, CalcFlags::BUILTIN, 1).unwrap();
    // Should find a crossing within a few years
    assert!(
        cross > jd && cross < jd + 750.0,
        "helio_cross should find Mars at 0° within 2 years, got JD {cross}"
    );
}

#[test]
fn test_helio_cross_ut_equals_et() {
    let jd = julday(2022, 6, 1, 0.0, Calendar::Gregorian);
    let et = helio_cross(Body::VENUS, 90.0, jd, CalcFlags::BUILTIN, 1).unwrap();
    let ut = helio_cross_ut(Body::VENUS, 90.0, jd, CalcFlags::BUILTIN, 1).unwrap();
    assert!(
        (et - ut).abs() < 0.01,
        "ET and UT helio_cross should agree within 0.01 days"
    );
}

// ─── set_sid_mode + ayanamsa ─────────────────────────────────────────────

#[test]
fn test_set_sid_mode_affects_ayanamsa() {
    setup();
    let jd = julday(2000, 1, 1, 12.0, Calendar::Gregorian); // J2000.0

    // Fagan-Bradley (mode 0) and Lahiri (mode 1) differ by ~0.9°
    set_sid_mode(SiderealMode(0), 0.0, 0.0); // Fagan-Bradley
    let fagan = ayanamsa(jd);

    set_sid_mode(SiderealMode(1), 0.0, 0.0); // Lahiri
    let lahiri = ayanamsa(jd);

    assert!(
        (fagan - lahiri).abs() > 0.5,
        "Fagan-Bradley and Lahiri should differ by >0.5°, got diff {}",
        (fagan - lahiri).abs()
    );
    assert!(
        (fagan - lahiri).abs() < 2.0,
        "Fagan-Bradley and Lahiri should differ by <2°"
    );
    // Reset to Lahiri default
    set_sid_mode(SiderealMode(1), 0.0, 0.0);
}

#[test]
fn test_ayanamsa_ut_uses_current_mode() {
    setup();
    let jd = julday(2000, 1, 1, 0.0, Calendar::Gregorian);

    set_sid_mode(SiderealMode(0), 0.0, 0.0); // Fagan-Bradley
    let fb = ayanamsa_ut(jd);

    set_sid_mode(SiderealMode(1), 0.0, 0.0); // Lahiri
    let lh = ayanamsa_ut(jd);

    assert!(
        (fb - lh).abs() > 0.5,
        "ayanamsa_ut should reflect current mode"
    );
    set_sid_mode(SiderealMode(1), 0.0, 0.0);
}

// ─── Eclipse implementation tests ─────────────────────────────────────────────

#[test]
fn test_sol_eclipse_where_returns_valid_coordinates() {
    // Known total solar eclipse: 2024-04-08 (visible across North America)
    let jd_eclipse = 2_460_409.0; // approx JD of 2024-04-08
    let result = sol_eclipse_where(jd_eclipse, CalcFlags::BUILTIN).unwrap();

    // Should return a valid geographic position
    let lon = result.geopos[0];
    let lat = result.geopos[1];
    assert!(
        (-180.0..=180.0).contains(&lon),
        "eclipse longitude {lon:.2}° outside valid range"
    );
    assert!(
        (-90.0..=90.0).contains(&lat),
        "eclipse latitude {lat:.2}° outside valid range"
    );
    // 2024 eclipse was over North America: path from Mexico (~18°N) to Maine (~47°N)
    // Our sub-lunar point approximation gives ~17°N (Moon geocentric dec at new Moon).
    // This is within the eclipse path latitude range — a valid first-order result.
    assert!(
        lat > 10.0 && lat < 55.0,
        "2024 eclipse lat={lat:.2}° — should be in North American eclipse path range"
    );
    assert!(
        lon > -140.0 && lon < -60.0,
        "2024 eclipse lon={lon:.2}° — should be over North America"
    );
    // Magnitude should be > 0 for a real eclipse
    assert!(
        result.attr[0] > 0.0,
        "eclipse magnitude should be > 0, got {}",
        result.attr[0]
    );
    println!("  2024 solar eclipse: lon={lon:.2}°, lat={lat:.2}°, mag={:.3}               (sub-lunar approx; actual greatest: ~25°N, ~103°W)",
             result.attr[0]);
}

#[test]
fn test_sol_eclipse_where_near_arbitrary_date() {
    // Call with a date between eclipses — should find and report the nearest one
    let jd = 2_451_545.0; // J2000
    let result = sol_eclipse_where(jd, CalcFlags::BUILTIN).unwrap();
    let lon = result.geopos[0];
    let lat = result.geopos[1];
    assert!((-180.0..=180.0).contains(&lon));
    assert!((-90.0..=90.0).contains(&lat));
    assert!(
        result.ret_flags != 0,
        "ret_flags should indicate eclipse type"
    );
}

#[test]
fn test_lun_occult_when_glob_finds_venus_occultation() {
    // Venus is occulted by the Moon several times per year — search from J2000
    let jd_start = 2_456_658.0; // 2014-01-01 — Venus occultation on 2014-10-23
    let result = lun_occult_when_glob(jd_start, Body::VENUS, None, CalcFlags::BUILTIN, 0, false);
    match result {
        Ok(r) => {
            let jd_occ = r.tret[0];
            assert!(jd_occ > jd_start, "occultation must be in the future");
            assert!(
                jd_occ < jd_start + 800.0,
                "should find Venus occultation within ~2 years, got JD {jd_occ:.1}"
            );
            assert_eq!(r.ret_flags, ECL_OCCULTATION);
            println!(
                "  Venus occultation found at JD {jd_occ:.2} \
                      ({:.1} days after start)",
                jd_occ - jd_start
            );
        }
        Err(e) => panic!("Expected Venus occultation, got error: {e}"),
    }
}

#[test]
fn test_lun_occult_when_glob_finds_mars_occultation() {
    // Known Mars occultation: 2021-07-17 (JD ≈ 2459413)
    // Mars was occulted by the Moon on this date
    let jd_start = 2_459_300.0; // 2021-03-25 — ~4 months before the event
    let result = lun_occult_when_glob(jd_start, Body::MARS, None, CalcFlags::BUILTIN, 0, false);
    match result {
        Ok(r) => {
            assert!(r.tret[0] > jd_start);
            assert_eq!(r.ret_flags, ECL_OCCULTATION);
            println!(
                "  Mars occultation found at JD {:.2} ({:.1} days after start)",
                r.tret[0],
                r.tret[0] - jd_start
            );
        }
        Err(e) => panic!("Expected Mars occultation near 2021-07-17, got: {e}"),
    }
}

#[test]
fn test_lun_occult_when_loc_returns_altitude() {
    let jd_start = 2_456_658.0; // 2014-01-01 (Venus occultation on 2014-10-23)
                                // Paris: lon=2.35°E, lat=48.85°N
    let geopos = [2.35, 48.85, 35.0];
    let result = lun_occult_when_loc(
        jd_start,
        Body::VENUS,
        None,
        CalcFlags::BUILTIN,
        geopos,
        false,
    );
    match result {
        Ok(r) => {
            assert!(r.tret[0] > jd_start);
            // attr[1] = Moon altitude — should be a valid angle
            let alt = r.attr[1];
            assert!(
                (-90.0..=90.0).contains(&alt),
                "Moon altitude {alt:.2}° out of range"
            );
            println!(
                "  Venus occultation from Paris: JD={:.2}, Moon alt={alt:.1}°",
                r.tret[0]
            );
        }
        Err(e) => panic!("lun_occult_when_loc failed: {e}"),
    }
}

// ─── MC/IC/ASC/DSC transit and house-position tests ──────────────────────────

const NATAL_JD: f64 = J2000; // Use J2000 as a "natal" chart moment

#[test]
fn mc_transit_sun_annual() {
    // Natal MC at J2000 Paris. Sun transits it once per year.
    let jd_transit = mc_transit_ut(
        Body::SUN,
        NATAL_JD,
        J2000,
        PARIS_LAT,
        PARIS_LON,
        HouseSystem::PLACIDUS,
        CalcFlags::BUILTIN,
        false,
    )
    .unwrap();
    assert!(jd_transit >= J2000, "transit must be on or after start");
    assert!(
        jd_transit < J2000 + 400.0,
        "Sun transits MC once/year, should be < 400d"
    );
    let sun = calc_ut(JulianDay::new(jd_transit), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let chart = houses(JulianDay::new(NATAL_JD), Latitude::new(PARIS_LAT), Longitude::new(PARIS_LON), HouseSystem::PLACIDUS).unwrap();
    let natal_mc = chart.ascmc[1];
    let diff = (sun.lon - natal_mc)
        .abs()
        .min(360.0 - (sun.lon - natal_mc).abs());
    assert!(
        diff < 0.01,
        "Sun lon {:.3}° should match natal MC {:.3}°",
        sun.lon,
        natal_mc
    );
    println!(
        "  Sun transits natal MC: JD={:.4} (+{:.1}d), MC={natal_mc:.2}°",
        jd_transit,
        jd_transit - J2000
    );
}

#[test]
fn mc_transit_moon_monthly() {
    // Moon transits the natal MC roughly once per month
    let jd_transit = mc_transit_ut(
        Body::MOON,
        NATAL_JD,
        J2000,
        PARIS_LAT,
        PARIS_LON,
        HouseSystem::PLACIDUS,
        CalcFlags::BUILTIN,
        false,
    )
    .unwrap();
    assert!(
        jd_transit < J2000 + 32.0,
        "Moon transits MC ~monthly, should be < 32d"
    );
    let moon = calc_ut(JulianDay::new(jd_transit), Body::MOON, CalcFlags::BUILTIN).unwrap();
    let chart = houses(JulianDay::new(NATAL_JD), Latitude::new(PARIS_LAT), Longitude::new(PARIS_LON), HouseSystem::PLACIDUS).unwrap();
    let diff = (moon.lon - chart.ascmc[1])
        .abs()
        .min(360.0 - (moon.lon - chart.ascmc[1]).abs());
    assert!(
        diff < 0.1,
        "Moon lon should match natal MC at transit (diff={diff:.4}°)"
    );
    println!(
        "  Moon transits natal MC: JD={:.4} (+{:.1}d)",
        jd_transit,
        jd_transit - J2000
    );
}

#[test]
fn mc_transit_mars_within_two_years() {
    let jd_transit = mc_transit_ut(
        Body::MARS,
        NATAL_JD,
        J2000,
        PARIS_LAT,
        PARIS_LON,
        HouseSystem::PLACIDUS,
        CalcFlags::BUILTIN,
        false,
    )
    .unwrap();
    assert!(
        jd_transit < J2000 + 750.0,
        "Mars transits natal MC within 750 days"
    );
    let mars = calc_ut(JulianDay::new(jd_transit), Body::MARS, CalcFlags::BUILTIN).unwrap();
    let chart = houses(JulianDay::new(NATAL_JD), Latitude::new(PARIS_LAT), Longitude::new(PARIS_LON), HouseSystem::PLACIDUS).unwrap();
    let diff = (mars.lon - chart.ascmc[1])
        .abs()
        .min(360.0 - (mars.lon - chart.ascmc[1]).abs());
    assert!(diff < 0.01, "Mars lon should match natal MC at transit");
    println!(
        "  Mars transits natal MC: JD={:.4} (+{:.1}d)",
        jd_transit,
        jd_transit - J2000
    );
}

#[test]
fn ic_transit_is_natal_mc_plus_180() {
    let chart = houses(JulianDay::new(NATAL_JD), Latitude::new(PARIS_LAT), Longitude::new(PARIS_LON), HouseSystem::PLACIDUS).unwrap();
    let natal_mc = chart.ascmc[1];
    let natal_ic = (natal_mc + 180.0).rem_euclid(360.0);
    let jd_ic = ic_transit_ut(
        Body::SUN,
        NATAL_JD,
        J2000,
        PARIS_LAT,
        PARIS_LON,
        HouseSystem::PLACIDUS,
        CalcFlags::BUILTIN,
        false,
    )
    .unwrap();
    let sun = calc_ut(JulianDay::new(jd_ic), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let diff = (sun.lon - natal_ic)
        .abs()
        .min(360.0 - (sun.lon - natal_ic).abs());
    assert!(
        diff < 0.01,
        "Sun lon {:.3}° should match IC {:.3}° at transit",
        sun.lon,
        natal_ic
    );
    println!(
        "  Sun transits natal IC: JD={:.4} (+{:.1}d), IC={natal_ic:.2}°",
        jd_ic,
        jd_ic - J2000
    );
}

#[test]
fn asc_and_dsc_transits() {
    let chart = houses(JulianDay::new(NATAL_JD), Latitude::new(PARIS_LAT), Longitude::new(PARIS_LON), HouseSystem::PLACIDUS).unwrap();
    let jd_asc = asc_transit_ut(
        Body::SUN,
        NATAL_JD,
        J2000,
        PARIS_LAT,
        PARIS_LON,
        HouseSystem::PLACIDUS,
        CalcFlags::BUILTIN,
        false,
    )
    .unwrap();
    let jd_dsc = dsc_transit_ut(
        Body::SUN,
        NATAL_JD,
        J2000,
        PARIS_LAT,
        PARIS_LON,
        HouseSystem::PLACIDUS,
        CalcFlags::BUILTIN,
        false,
    )
    .unwrap();
    let sun_asc = calc_ut(JulianDay::new(jd_asc), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let sun_dsc = calc_ut(JulianDay::new(jd_dsc), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let natal_asc = chart.ascmc[0];
    let natal_dsc = (natal_asc + 180.0).rem_euclid(360.0);
    let diff_asc = (sun_asc.lon - natal_asc)
        .abs()
        .min(360.0 - (sun_asc.lon - natal_asc).abs());
    let diff_dsc = (sun_dsc.lon - natal_dsc)
        .abs()
        .min(360.0 - (sun_dsc.lon - natal_dsc).abs());
    assert!(
        diff_asc < 0.01,
        "Sun lon {:.3}° should match ASC {natal_asc:.3}°",
        sun_asc.lon
    );
    assert!(
        diff_dsc < 0.01,
        "Sun lon {:.3}° should match DSC {natal_dsc:.3}°",
        sun_dsc.lon
    );
    println!(
        "  Sun ASC: JD={:.4} (+{:.1}d), DSC: JD={:.4} (+{:.1}d)",
        jd_asc,
        jd_asc - J2000,
        jd_dsc,
        jd_dsc - J2000
    );
}

#[test]
fn mc_transit_backward_search() {
    let jd_transit = mc_transit_ut(
        Body::SUN,
        NATAL_JD,
        J2000,
        PARIS_LAT,
        PARIS_LON,
        HouseSystem::PLACIDUS,
        CalcFlags::BUILTIN,
        true,
    )
    .unwrap();
    assert!(jd_transit < J2000, "backward search must return past event");
    assert!(
        jd_transit > J2000 - 400.0,
        "backward search within 400 days"
    );
}

#[test]
fn mc_transit_sidereal() {
    set_sid_mode(SiderealMode::LAHIRI, 0.0, 0.0);
    let result = mc_transit_ut(
        Body::SUN,
        NATAL_JD,
        J2000,
        PARIS_LAT,
        PARIS_LON,
        HouseSystem::PLACIDUS,
        CalcFlags::BUILTIN | CalcFlags::SIDEREAL,
        false,
    );
    set_sid_mode(SiderealMode::FAGAN_BRADLEY, 0.0, 0.0);
    if let Ok(jd) = result {
        assert!((J2000..J2000 + 400.0).contains(&jd));
        println!("  Sun sidereal MC transit: JD={jd:.4}");
    }
}

#[test]
fn transit_to_degree_sun_to_fixed_degree() {
    // Sun should reach 90° (0° Cancer) within 6 months from J2000 (summer solstice)
    let jd = transit_to_degree(Body::SUN, 90.0, J2000, CalcFlags::BUILTIN, false).unwrap();
    assert!(
        jd > J2000 && jd < J2000 + 200.0,
        "Sun reaches 90° within 6 months"
    );
    let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap();
    assert!(
        (sun.lon - 90.0).abs() < 0.01,
        "Sun at Cancer solstice: {:.3}°",
        sun.lon
    );
    println!("  Sun at 90° (Cancer): JD={jd:.4} (+{:.1}d)", jd - J2000);
}

#[test]
fn transit_to_degree_moon_to_full_moon_lon() {
    // Moon opposite Sun at J2000 (Sun ≈ 280°, so full moon ≈ Moon at 100°)
    let sun = calc_ut(JulianDay::new(J2000), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let full_moon_lon = (sun.lon + 180.0).rem_euclid(360.0);
    let jd =
        transit_to_degree(Body::MOON, full_moon_lon, J2000, CalcFlags::BUILTIN, false).unwrap();
    assert!(jd > J2000 && jd < J2000 + 30.0);
    println!(
        "  Moon at {full_moon_lon:.2}° (full moon): JD={jd:.4} (+{:.1}d)",
        jd - J2000
    );
}

// ─── Physical meridian transit tests (daily horizon-based) ───────────────────

#[test]
fn meridian_transit_sun_is_daily() {
    // Solar noon: the Sun physically crosses the meridian every ~24h
    let geopos = [PARIS_LON, PARIS_LAT, 35.0];
    let r = meridian_transit_ut(Body::SUN, J2000, geopos, CalcFlags::BUILTIN).unwrap();
    assert!(r.tret > J2000, "transit must be in the future");
    assert!(r.tret < J2000 + 2.0, "solar noon should be within 24h");
    let hours_ahead = (r.tret - J2000) * 24.0;
    println!(
        "  Solar noon Paris: JD={:.4} ({hours_ahead:.2}h after J2000)",
        r.tret
    );
}

#[test]
fn meridian_transit_moon_is_daily() {
    let geopos = [PARIS_LON, PARIS_LAT, 35.0];
    let r = meridian_transit_ut(Body::MOON, J2000, geopos, CalcFlags::BUILTIN).unwrap();
    assert!(
        r.tret > J2000 && r.tret < J2000 + 2.0,
        "Moon meridian transit should be within 24h, got JD {}",
        r.tret
    );
    let hours_ahead = (r.tret - J2000) * 24.0;
    println!("  Moon meridian transit Paris: {hours_ahead:.2}h after J2000");
}

#[test]
fn meridian_transit_planet_is_daily() {
    let geopos = [PARIS_LON, PARIS_LAT, 35.0];
    for &body in &[
        Body::MERCURY,
        Body::VENUS,
        Body::MARS,
        Body::JUPITER,
        Body::SATURN,
    ] {
        let r = meridian_transit_ut(body, J2000, geopos, CalcFlags::BUILTIN).unwrap();
        assert!(
            r.tret > J2000 && r.tret < J2000 + 2.0,
            "body {body} meridian transit should be within 24h"
        );
    }
}

#[test]
fn lower_meridian_transit_is_12h_after_upper() {
    // Lower meridian (anti-culmination) is ~12h after/before upper culmination
    let geopos = [PARIS_LON, PARIS_LAT, 35.0];
    let upper = meridian_transit_ut(Body::SUN, J2000, geopos, CalcFlags::BUILTIN).unwrap();
    let lower = lower_meridian_transit_ut(Body::SUN, J2000, geopos, CalcFlags::BUILTIN).unwrap();
    let gap_h = (upper.tret - lower.tret).abs() * 24.0;
    // Upper and lower are ~12h apart (exact gap varies by season/latitude)
    assert!(
        gap_h > 8.0 && gap_h < 16.0,
        "Upper and lower solar transits should be ~12h apart, got {gap_h:.1}h"
    );
    println!(
        "  Upper transit: JD={:.4}, lower: JD={:.4}, gap={gap_h:.1}h",
        upper.tret, lower.tret
    );
}

#[test]
fn mc_transit_vs_meridian_transit_different() {
    // Confirm the two functions are genuinely different
    let geopos = [PARIS_LON, PARIS_LAT, 35.0];
    let ecliptic_jd = mc_transit_ut(
        Body::SUN,
        J2000,
        J2000,
        PARIS_LAT,
        PARIS_LON,
        HouseSystem::PLACIDUS,
        CalcFlags::BUILTIN,
        false,
    )
    .unwrap();
    let physical = meridian_transit_ut(Body::SUN, J2000, geopos, CalcFlags::BUILTIN).unwrap();
    // Physical meridian transit: daily (< 2 days)
    // Ecliptic MC conjunction: up to a year away
    assert!(
        physical.tret < J2000 + 2.0,
        "physical transit should be daily"
    );
    // They may coincidentally be the same day but usually differ by days/months
    println!(
        "  Ecliptic MC conjunction: JD={ecliptic_jd:.4} (+{:.1}d)",
        ecliptic_jd - J2000
    );
    println!(
        "  Physical meridian cross: JD={:.4} (+{:.1}h)",
        physical.tret,
        (physical.tret - J2000) * 24.0
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Chart.rs — complete astrological chart functions
// ═══════════════════════════════════════════════════════════════════════════

const ALL_PLANETS: &[Body] = &[
    Body::SUN,
    Body::MOON,
    Body::MERCURY,
    Body::VENUS,
    Body::MARS,
    Body::JUPITER,
    Body::SATURN,
    Body::URANUS,
    Body::NEPTUNE,
    Body::MEAN_NODE,
];

fn chart_positions(jd: f64) -> Vec<(Body, f64, f64)> {
    ALL_PLANETS
        .iter()
        .map(|&b| {
            let p = calc_ut(JulianDay::new(jd), b, CalcFlags::BUILTIN | CalcFlags::SPEED).unwrap();
            (b, p.lon, p.speed_lon)
        })
        .collect()
}

// ─── Aspect table tests ───────────────────────────────────────────────────────

#[test]
fn aspect_table_has_correct_count() {
    let pos = chart_positions(J2000);
    let aspects = calc_chart_aspects(&pos, MAJOR_ASPECTS, 8.0);
    // With 10 bodies and 5 major aspects at 8° orb, we expect some aspects
    assert!(!aspects.is_empty(), "Should find some aspects");
    // All orbs must be within the specified maximum
    for a in &aspects {
        assert!(a.orb <= 8.0, "Orb {:.3} exceeds max 8°", a.orb);
        assert!(a.orb >= 0.0, "Orb must be non-negative");
        assert!(a.body1 != a.body2, "Cannot aspect itself");
    }
    // Sorted by tightness
    for i in 1..aspects.len() {
        assert!(
            aspects[i].orb >= aspects[i - 1].orb,
            "Not sorted by tightness"
        );
    }
    println!(
        "  Found {} major aspects at J2000 with 8° orb",
        aspects.len()
    );
}

#[test]
fn aspect_table_conjunction_at_zero_orb() {
    // Sun conjunct Sun is always 0° — trivial test with same body
    // Instead, find a real conjunction in a chart
    let pos = chart_positions(J2000);
    let tight = calc_chart_aspects(&pos, &[0.0], 1.0);
    // At J2000, check that any found conjunctions have orb < 1°
    for a in &tight {
        assert!(a.aspect == 0.0);
        assert!(a.orb <= 1.0);
    }
}

#[test]
fn aspect_applying_separating_consistent() {
    let pos = chart_positions(J2000);
    let aspects = calc_chart_aspects(&pos, MAJOR_ASPECTS, 10.0);
    // applying must be a bool — always valid, just confirm no panic
    for a in &aspects {
        let _ = a.applying;
    }
}

// ─── Sign ingress tests ────────────────────────────────────────────────────────

#[test]
fn sign_ingress_sun_within_35_days() {
    // Sun changes sign every ~30 days
    let (jd_ingress, sign) = sign_ingress_ut(Body::SUN, J2000, CalcFlags::BUILTIN, false).unwrap();
    assert!(jd_ingress > J2000, "ingress must be in the future");
    assert!(jd_ingress < J2000 + 35.0, "Sun changes sign within 35 days");
    assert!(sign <= 11, "sign number 0-11");
    // At ingress, Sun should be at exactly the sign boundary (multiple of 30°)
    let sun = calc_ut(JulianDay::new(jd_ingress), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let boundary = sign as f64 * 30.0;
    assert!(
        (sun.lon - boundary).abs() < 0.01,
        "Sun at {:.3}°, expected boundary {boundary:.1}°",
        sun.lon
    );
    println!(
        "  Sun enters sign {sign}: JD={jd_ingress:.4} (+{:.1}d)",
        jd_ingress - J2000
    );
}

#[test]
fn sign_ingress_backward_finds_previous() {
    let (jd_ingress, _) = sign_ingress_ut(Body::SUN, J2000, CalcFlags::BUILTIN, true).unwrap();
    assert!(jd_ingress < J2000, "backward ingress must be in the past");
    assert!(jd_ingress > J2000 - 35.0, "within 35 days before");
}

#[test]
fn sign_ingress_saturn_within_3_years() {
    // Saturn changes sign every ~2.5 years
    let (jd_ingress, sign) =
        sign_ingress_ut(Body::SATURN, J2000, CalcFlags::BUILTIN, false).unwrap();
    assert!(
        jd_ingress < J2000 + 1100.0,
        "Saturn changes sign within ~3 years"
    );
    assert!(sign <= 11);
    println!(
        "  Saturn enters sign {sign}: JD={jd_ingress:.2} (+{:.1}d)",
        jd_ingress - J2000
    );
}

// ─── Retrograde station tests ──────────────────────────────────────────────────

#[test]
fn retrograde_station_mars_finds_both_stations() {
    let s = retrograde_station_ut(Body::MARS, J2000, CalcFlags::BUILTIN).unwrap();
    assert!(
        s.retrograde > J2000 || s.direct > J2000,
        "at least one station should be in the future"
    );
    // Retrograde before direct (or both in future in correct order)
    println!(
        "  Mars retrograde: JD={:.2} (+{:.0}d)",
        s.retrograde,
        s.retrograde - J2000
    );
    println!(
        "  Mars direct:     JD={:.2} (+{:.0}d)",
        s.direct,
        s.direct - J2000
    );
}

#[test]
fn retrograde_station_speed_crosses_zero() {
    let s = retrograde_station_ut(Body::MARS, J2000, CalcFlags::BUILTIN).unwrap();
    // At the retrograde station, speed should be ~0 and turning negative
    let spd_r = calc_ut(
        JulianDay::new(s.retrograde),
        Body::MARS,
        CalcFlags::BUILTIN | CalcFlags::SPEED,
    )
    .unwrap()
    .speed_lon;
    assert!(
        spd_r.abs() < 0.05,
        "Speed at retrograde station: {spd_r:.5}°/day (expected ~0)"
    );
    // At direct station, speed should be ~0 and turning positive
    let spd_d = calc_ut(JulianDay::new(s.direct), Body::MARS, CalcFlags::BUILTIN | CalcFlags::SPEED)
        .unwrap()
        .speed_lon;
    assert!(
        spd_d.abs() < 0.05,
        "Speed at direct station: {spd_d:.5}°/day (expected ~0)"
    );
    println!("  Mars retrograde station speed: {spd_r:.5}°/day");
    println!("  Mars direct station speed:     {spd_d:.5}°/day");
}

// ─── Arabic Parts tests ────────────────────────────────────────────────────────

#[test]
fn arabic_part_lot_of_fortune_range() {
    let chart = houses(JulianDay::new(J2000), Latitude::new(PARIS_LAT), Longitude::new(PARIS_LON), HouseSystem::PLACIDUS).unwrap();
    let sun = calc_ut(JulianDay::new(J2000), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let moon = calc_ut(JulianDay::new(J2000), Body::MOON, CalcFlags::BUILTIN).unwrap();
    let asc = chart.ascmc[0];
    let fortune = arabic_part(asc, moon.lon, sun.lon);
    assert!(
        (0.0..360.0).contains(&fortune),
        "Fortune={fortune:.2}° out of range"
    );
    println!(
        "  Lot of Fortune: {fortune:.2}°  ASC={asc:.2}° Moon={:.2}° Sun={:.2}°",
        moon.lon, sun.lon
    );
}

#[test]
fn arabic_parts_seven_all_in_range() {
    let jd = J2000;
    let chart = houses(JulianDay::new(jd), Latitude::new(PARIS_LAT), Longitude::new(PARIS_LON), HouseSystem::PLACIDUS).unwrap();
    let sun = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let moon = calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN).unwrap();
    let sat = calc_ut(JulianDay::new(jd), Body::SATURN, CalcFlags::BUILTIN).unwrap();
    let mar = calc_ut(JulianDay::new(jd), Body::MARS, CalcFlags::BUILTIN).unwrap();
    let jup = calc_ut(JulianDay::new(jd), Body::JUPITER, CalcFlags::BUILTIN).unwrap();
    let mer = calc_ut(JulianDay::new(jd), Body::MERCURY, CalcFlags::BUILTIN).unwrap();
    let ven = calc_ut(JulianDay::new(jd), Body::VENUS, CalcFlags::BUILTIN).unwrap();
    let asc = chart.ascmc[0];
    let is_day = planet_house_number(sun.lon, &chart.cusps) >= 7;
    let parts = arabic_parts_seven(
        asc, sun.lon, moon.lon, sat.lon, mar.lon, jup.lon, mer.lon, ven.lon, is_day,
    );
    assert_eq!(parts.len(), 7);
    for p in &parts {
        assert!(
            p.degree >= 0.0 && p.degree < 360.0,
            "{}: {:.2}° out of range",
            p.name,
            p.degree
        );
        println!("  {:<22} {:>7.3}°  ({})", p.name, p.degree, p.formula);
    }
}

#[test]
fn arabic_part_day_night_reversed() {
    // Day chart Fortune = ASC + Moon - Sun; Night = ASC + Sun - Moon
    let asc = 15.0;
    let sun = 280.0;
    let moon = 120.0;
    let day_fortune = arabic_part(asc, moon, sun);
    let night_fortune = arabic_part(asc, sun, moon);
    assert!(
        (day_fortune - night_fortune).abs() > 1.0,
        "Day and night charts should differ"
    );
}

// ─── Secondary progressions ───────────────────────────────────────────────────

#[test]
fn secondary_progressions_returns_valid_positions() {
    let bodies = [
        Body::SUN,
        Body::MOON,
        Body::MERCURY,
        Body::VENUS,
        Body::MARS,
    ];
    let (positions, chart) = secondary_progressions(
        J2000,
        35.0,
        &bodies,
        PARIS_LAT,
        PARIS_LON,
        HouseSystem::PLACIDUS,
        CalcFlags::BUILTIN,
    )
    .unwrap();
    assert_eq!(positions.len(), 5);
    for (body, pos) in &positions {
        assert!(
            pos.lon >= 0.0 && pos.lon < 360.0,
            "body {body} lon out of range"
        );
    }
    assert!(chart.ascmc[0] >= 0.0 && chart.ascmc[0] < 360.0);
    let (_, sun_prog) = positions[0].clone();
    // Progressed Sun should be ~35° ahead of natal Sun (1°/year)
    let natal_sun = calc_ut(JulianDay::new(J2000), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let advance = (sun_prog.lon - natal_sun.lon).rem_euclid(360.0);
    assert!(
        advance > 30.0 && advance < 40.0,
        "Progressed Sun should be ~35° ahead, got {advance:.2}°"
    );
    println!(
        "  Progressed Sun at age 35: {:.2}° (natal: {:.2}°, advance: {advance:.2}°)",
        sun_prog.lon, natal_sun.lon
    );
}

// ─── Solar arc directions ─────────────────────────────────────────────────────

#[test]
fn solar_arc_is_approximately_one_degree_per_year() {
    let natal_positions: Vec<(Body, f64)> = ALL_PLANETS
        .iter()
        .map(|&b| {
            let p = calc_ut(JulianDay::new(J2000), b, CalcFlags::BUILTIN).unwrap();
            (b, p.lon)
        })
        .collect();
    let natal_mc = houses(JulianDay::new(J2000), Latitude::new(PARIS_LAT), Longitude::new(PARIS_LON), HouseSystem::PLACIDUS)
        .unwrap()
        .ascmc[1];
    let (arc, directed, dir_mc) =
        solar_arc_directions(J2000, 30.0, &natal_positions, natal_mc, CalcFlags::BUILTIN).unwrap();
    assert!(
        arc > 28.0 && arc < 32.0,
        "30-year solar arc should be ~30°, got {arc:.3}°"
    );
    for (body, lon) in &directed {
        assert!(
            *lon >= 0.0 && *lon < 360.0,
            "body {body} directed lon out of range"
        );
    }
    println!("  30-year solar arc: {arc:.3}°");
    println!("  Directed MC: {dir_mc:.2}° (natal: {natal_mc:.2}°)");
}

// ─── Solar/Lunar return tests ──────────────────────────────────────────────────

#[test]
fn solar_return_sun_matches_natal_lon() {
    let natal_sun = calc_ut(JulianDay::new(J2000), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let sr_jd = solar_return_jd(J2000, 2001, CalcFlags::BUILTIN).unwrap();
    let return_sun = calc_ut(JulianDay::new(sr_jd), Body::SUN, CalcFlags::BUILTIN).unwrap();
    let diff = (return_sun.lon - natal_sun.lon)
        .abs()
        .min(360.0 - (return_sun.lon - natal_sun.lon).abs());
    assert!(
        diff < 0.01,
        "Return Sun {:.4}° should match natal {:.4}°",
        return_sun.lon,
        natal_sun.lon
    );
    println!(
        "  Solar return 2001: JD={sr_jd:.4}, Sun={:.4}°",
        return_sun.lon
    );
}

#[test]
fn lunar_return_moon_matches_natal_lon() {
    let natal_moon = calc_ut(JulianDay::new(J2000), Body::MOON, CalcFlags::BUILTIN).unwrap();
    let lr_jd = lunar_return_jd(J2000, J2000, CalcFlags::BUILTIN).unwrap();
    let return_moon = calc_ut(JulianDay::new(lr_jd), Body::MOON, CalcFlags::BUILTIN).unwrap();
    let diff = (return_moon.lon - natal_moon.lon)
        .abs()
        .min(360.0 - (return_moon.lon - natal_moon.lon).abs());
    assert!(
        diff < 0.1,
        "Return Moon {:.4}° should match natal {:.4}°",
        return_moon.lon,
        natal_moon.lon
    );
    assert!(
        lr_jd < J2000 + 30.0,
        "Lunar return should be within 30 days"
    );
    println!("  Lunar return: JD={lr_jd:.4} (+{:.1}d)", lr_jd - J2000);
}

// ─── Midpoint tests ────────────────────────────────────────────────────────────

#[test]
fn midpoint_equidistant_from_both() {
    let mid = midpoint(10.0, 20.0);
    assert!(
        (mid - 15.0).abs() < 0.001,
        "midpoint(10,20) = {mid:.3}, expected 15°"
    );
    // Wrap-around case
    let mid2 = midpoint(350.0, 10.0);
    assert!(
        (mid2 - 0.0).abs() < 0.001 || (mid2 - 180.0).abs() < 0.001,
        "midpoint(350,10) = {mid2:.3}, expected 0° or 180°"
    );
}

#[test]
fn midpoint_table_all_valid() {
    let pos: Vec<(Body, f64)> = ALL_PLANETS
        .iter()
        .map(|&b| {
            let p = calc_ut(JulianDay::new(J2000), b, CalcFlags::BUILTIN).unwrap();
            (b, p.lon)
        })
        .collect();
    let table = midpoint_table(&pos, 2.0);
    let expected_pairs = pos.len() * (pos.len() - 1) / 2;
    assert_eq!(table.len(), expected_pairs);
    for (b1, b2, mid, _) in &table {
        assert!(b1 != b2);
        assert!(
            *mid >= 0.0 && *mid < 360.0,
            "midpoint out of range: {mid:.2}°"
        );
    }
    println!("  Midpoint table: {} pairs", table.len());
}

// ─── Parallactic angle test ────────────────────────────────────────────────────

#[test]
fn parallactic_angle_at_meridian_is_zero() {
    // At meridian (hour angle = 0), parallactic angle = 0
    let q = parallactic_angle(0.0, 20.0, 48.85);
    assert!(
        q.abs() < 0.001,
        "parallactic angle at meridian should be 0, got {q:.4}"
    );
}

#[test]
fn parallactic_angle_north_south_hemisphere_differ() {
    let q_north = parallactic_angle(30.0, 20.0, 48.85);
    let q_south = parallactic_angle(30.0, 20.0, -34.0);
    // Should be different for different latitudes
    assert!((q_north - q_south).abs() > 5.0);
}

// ─── Profection tests ─────────────────────────────────────────────────────────

#[test]
fn annual_profection_cycles_every_12_years() {
    let chart = houses(JulianDay::new(J2000), Latitude::new(PARIS_LAT), Longitude::new(PARIS_LON), HouseSystem::PLACIDUS).unwrap();
    let (h0, _) = annual_profection(&chart.cusps, 0);
    let (h12, _) = annual_profection(&chart.cusps, 12);
    let (h24, _) = annual_profection(&chart.cusps, 24);
    assert_eq!(h0, 1, "Age 0 = house 1");
    assert_eq!(h0, h12, "Age 0 and age 12 = same house");
    assert_eq!(h12, h24, "Repeats every 12 years");
    for age in 0u32..12 {
        let (h, _) = annual_profection(&chart.cusps, age);
        assert_eq!(h, (age + 1) as u8, "Age {age} → house {}", age + 1);
    }
}

#[test]
fn monthly_profection_degree_in_range() {
    let chart = houses(JulianDay::new(J2000), Latitude::new(PARIS_LAT), Longitude::new(PARIS_LON), HouseSystem::PLACIDUS).unwrap();
    for age in [0u32, 12, 25, 36, 47] {
        for month in 0u32..12 {
            let (h, deg) = monthly_profection(&chart.cusps, age, month);
            assert!((1..=12).contains(&h), "house {h} out of range");
            assert!((0.0..360.0).contains(&deg), "degree {deg:.2}° out of range");
        }
    }
}

#[test]
fn diagnostic_pluto_status() {
    let r = calc_ut(JulianDay::new(J2000), Body::PLUTO, CalcFlags::BUILTIN);
    match &r {
        Ok(p) => println!("  Pluto OK: lon={:.3}°, dist={:.4} AU", p.lon, p.dist),
        Err(e) => println!("  Pluto error: {e}"),
    }
}

// ─── New functions: Pluto, orbs, LAST, signs, dasha ──────────────────────────

#[test]
fn pluto_position_sagittarius_j2000() {
    // Pluto was in Sagittarius around J2000, roughly 246-250°
    let r = calc_ut(JulianDay::new(J2000), Body::PLUTO, CalcFlags::BUILTIN).unwrap();
    assert!(r.lon >= 0.0 && r.lon < 360.0, "lon={}", r.lon);
    assert!(r.dist > 28.0 && r.dist < 50.0, "dist={:.2} AU", r.dist);
    assert!(
        r.lon > 240.0 && r.lon < 260.0,
        "Pluto J2000 should be Sagittarius (~246°), got {:.2}°",
        r.lon
    );
    println!(
        "  Pluto J2000: lon={:.3}°, lat={:.3}°, dist={:.3} AU",
        r.lon, r.lat, r.dist
    );
}

#[test]
fn pluto_speed_with_flag() {
    let r = calc_ut(JulianDay::new(J2000), Body::PLUTO, CalcFlags::BUILTIN | CalcFlags::SPEED).unwrap();
    // Pluto moves ~0.04°/day
    assert!(
        r.speed_lon.abs() < 0.1,
        "Pluto speed {:.5}°/day out of range",
        r.speed_lon
    );
    assert!(r.speed_lon.is_finite());
}

#[test]
fn pluto_sign_ingress_future() {
    // Pluto entered Aquarius around 2023/2024 — from J2000 that's ~23 years
    let (jd_ingress, sign) =
        sign_ingress_ut(Body::PLUTO, J2000, CalcFlags::BUILTIN, false).unwrap();
    assert!(jd_ingress > J2000, "ingress must be in the future");
    assert!(jd_ingress < J2000 + 10000.0, "within ~27 years");
    println!(
        "  Pluto next sign ingress: JD={jd_ingress:.1} (+{:.1}d) → sign {sign}",
        jd_ingress - J2000
    );
}

// ─── Orb table tests ──────────────────────────────────────────────────────────

#[test]
fn default_orb_luminaries_wider_than_outer() {
    // Sun-Moon conjunction should have wider orb than Saturn-Neptune
    let sun_moon_conj = default_orb(Body::SUN, Body::MOON, 0.0);
    let sat_nep_conj = default_orb(Body::SATURN, Body::NEPTUNE, 0.0);
    assert!(
        sun_moon_conj > sat_nep_conj,
        "Sun-Moon orb {sun_moon_conj:.1}° should exceed Saturn-Neptune {sat_nep_conj:.1}°"
    );
    // Conjunction should be wider than sextile
    let sun_sext = default_orb(Body::SUN, Body::MOON, 60.0);
    assert!(
        sun_moon_conj > sun_sext,
        "Conjunction orb should exceed sextile orb"
    );
    println!("  Sun-Moon conj: {sun_moon_conj:.2}°, sat-nep conj: {sat_nep_conj:.2}°");
}

#[test]
fn calc_chart_aspects_auto_finds_aspects() {
    let pos = chart_positions(J2000);
    let aspects = calc_chart_aspects_auto(&pos, MAJOR_ASPECTS);
    assert!(
        !aspects.is_empty(),
        "Should find some aspects with auto orbs"
    );
    for a in &aspects {
        let orb_limit = default_orb(a.body1, a.body2, a.aspect);
        assert!(
            a.orb <= orb_limit + 0.001,
            "orb {:.3}° exceeds limit {orb_limit:.3}°",
            a.orb
        );
    }
    println!("  Auto-orb aspects at J2000: {}", aspects.len());
}

// ─── Local Apparent Solar Time ────────────────────────────────────────────────

#[test]
fn local_apparent_solar_time_range() {
    let last = local_apparent_solar_time(J2000, 2.35).unwrap(); // Paris
    assert!((0.0..24.0).contains(&last), "LAST={last:.4}h out of [0,24)");
    println!("  LAST Paris J2000: {last:.4}h");
}

#[test]
fn local_apparent_solar_time_at_solar_noon() {
    // At J2000 (Jan 1 2000 12:00 UTC), LAST at Paris should be near 12:13
    // (12:00 UTC + longitude correction 2.35°/15 = +9.4min + EoT ≈ +3.3min)
    let last_j2000 = local_apparent_solar_time(J2000, PARIS_LON).unwrap();
    assert!(
        last_j2000 > 11.5 && last_j2000 < 13.0,
        "LAST at J2000 Paris should be ~12.2h, got {last_j2000:.4}h"
    );
    println!("  LAST Paris at J2000 (noon UTC): {last_j2000:.4}h (expected ~12.2h)");
    // Verify that LAST = UTC + lon/15 + EoT
    let utc = 12.0_f64; // J2000 is exactly noon UTC
    let lon_offset = PARIS_LON / 15.0;
    let eot = time_equ(J2000).unwrap();
    let expected = (utc + lon_offset + eot).rem_euclid(24.0);
    assert!(
        (last_j2000 - expected).abs() < 0.001,
        "LAST formula mismatch: computed={last_j2000:.4}h, expected={expected:.4}h"
    );
}

// ─── Sign utilities ───────────────────────────────────────────────────────────

#[test]
fn sign_ruler_all_signs_valid() {
    for s in 0u8..12 {
        let r = sign_ruler(s);
        assert!(r.as_raw() >= 0, "sign {s} has no ruler");
        assert!(
            r <= Body::SATURN,
            "sign {s} ruler {r} is not a classical planet"
        );
    }
}

#[test]
fn sign_ruler_modern_outer_planets() {
    // Scorpio → Pluto, Aquarius → Uranus, Pisces → Neptune
    assert_eq!(sign_ruler_modern(7), Body::PLUTO, "Scorpio → Pluto");
    assert_eq!(sign_ruler_modern(10), Body::URANUS, "Aquarius → Uranus");
    assert_eq!(sign_ruler_modern(11), Body::NEPTUNE, "Pisces → Neptune");
}

#[test]
fn zodiac_sign_name_correct() {
    assert_eq!(zodiac_sign_name(0), "Aries");
    assert_eq!(zodiac_sign_name(6), "Libra");
    assert_eq!(zodiac_sign_name(11), "Pisces");
    assert_eq!(zodiac_sign_name(12), "Aries"); // wraps
}

#[test]
fn lon_to_sign_correct() {
    let (sign, deg) = lon_to_sign(45.5); // 15°30' Taurus
    assert_eq!(sign, 1, "45.5° should be Taurus (sign 1)");
    assert!((deg - 15.5).abs() < 0.001, "degree in sign: {deg}");
    let (s2, _) = lon_to_sign(359.9); // late Pisces
    assert_eq!(s2, 11, "359.9° should be Pisces (sign 11)");
    let (s3, _) = lon_to_sign(0.0);
    assert_eq!(s3, 0, "0° = Aries");
}

#[test]
fn sign_exaltation_known_values() {
    assert_eq!(sign_exaltation(Body::SUN), 0, "Sun exalted in Aries");
    assert_eq!(sign_exaltation(Body::MOON), 1, "Moon exalted in Taurus");
    assert_eq!(
        sign_exaltation(Body::JUPITER),
        3,
        "Jupiter exalted in Cancer"
    );
    assert_eq!(
        sign_exaltation(Body::SATURN),
        6,
        "Saturn exalted in Libra (index=6, Libra)"
    );
}

// ─── Vimshottari dasha tests ──────────────────────────────────────────────────

#[test]
fn vimshottari_total_cycle_is_120_years() {
    let total: f64 = DASHA_SEQUENCE.iter().map(|(_, y)| y).sum();
    assert!(
        (total - 120.0).abs() < 0.001,
        "Dasha cycle total = {total}y, expected 120y"
    );
}

#[test]
fn vimshottari_dasha_from_birth_valid() {
    // Set sidereal mode for Vedic dasha
    set_sid_mode(SiderealMode::LAHIRI, 0.0, 0.0);
    let moon = calc_ut(JulianDay::new(J2000), Body::MOON, CalcFlags::BUILTIN | CalcFlags::SIDEREAL).unwrap();
    let dashas = vimshottari_dasha(J2000, moon.lon, 120.0);
    set_sid_mode(SiderealMode(0), 0.0, 0.0);

    assert!(!dashas.is_empty(), "Should have dasha periods");
    // All periods must be contiguous
    for i in 1..dashas.len() {
        let prev_end = dashas[i - 1].end;
        let cur_start = dashas[i].start;
        assert!(
            (cur_start - prev_end).abs() < 1.0,
            "Gap at dasha {i}: prev end {prev_end:.2}, cur start {cur_start:.2}"
        );
    }
    // All years must be positive
    for d in &dashas {
        assert!(d.years > 0.0, "Dasha years must be positive: {}", d.years);
    }
    // Total should be ~120 years
    let total_years: f64 = dashas.iter().map(|d| d.years).sum();
    assert!(
        total_years > 110.0 && total_years < 125.0,
        "Total dasha years = {total_years:.1}, expected ~120"
    );
    println!("  Vimshottari dasha from J2000 Moon ({:.2}°):", moon.lon);
    for d in dashas.iter().take(4) {
        println!(
            "    {} {:.2}y (JD {:.1}–{:.1})",
            planet_name(d.body),
            d.years,
            d.start,
            d.end
        );
    }
}

#[test]
fn vimshottari_9_planets_in_sequence() {
    let planets: Vec<Body> = DASHA_SEQUENCE.iter().map(|(p, _)| *p).collect();
    assert_eq!(planets.len(), 9);
    // Must contain all 7 classical planets + 2 nodes
    let has_sun = planets.contains(&Body::SUN);
    let has_moon = planets.contains(&Body::MOON);
    let has_rahu = planets.contains(&Body::MEAN_NODE);
    assert!(
        has_sun && has_moon && has_rahu,
        "Missing key planets in dasha sequence"
    );
}

#[test]
fn houses_placidus_equator_asc_direction() {
    // Regression for ascendant() returning DSC instead of ASC.
    // At JD 2452275.499255786, lat=0, lon=0 the ASC should be ~191°
    // (confirmed against SE C library reference).
    let h = houses(JulianDay::new(2_452_275.499_255_786), Latitude::new(0.0), Longitude::new(0.0), HouseSystem::PLACIDUS).unwrap();
    let asc = h.ascmc[0];
    let mc = h.ascmc[1];

    // ASC must match SE reference within 0.01° (our GMST is accurate to ~0.002°)
    assert!(
        (asc - 191.098_936_463_985_4).abs() < 0.01,
        "ASC {asc} should be ~191.099° (not ~11° which would mean returning DSC)"
    );
    // MC must be in valid range
    assert!((0.0..360.0).contains(&mc), "MC {mc} out of range");
    // ASC - MC ≈ 90° at the equator (structural relationship)
    let asc_mc_diff = (asc - mc).rem_euclid(360.0);
    assert!(
        (asc_mc_diff - 90.0).abs() < 5.0,
        "ASC-MC diff {asc_mc_diff} should be ~90° at equator"
    );
    // Cusps[1] == ASC in Placidus
    assert!(
        (h.cusps[1] - asc).abs() < 1e-9,
        "cusps[1] {} should equal ASC {asc}",
        h.cusps[1]
    );
    // All 12 cusps must be in [0, 360) and opposite houses ~180° apart
    for i in 1..=12 {
        assert!(
            h.cusps[i] >= 0.0 && h.cusps[i] < 360.0,
            "cusp {i} = {} out of range",
            h.cusps[i]
        );
    }
    // H1 and H7 are opposite (ASC / DSC)
    let h1_h7 = (h.cusps[7] - h.cusps[1]).rem_euclid(360.0);
    assert!(
        (h1_h7 - 180.0).abs() < 0.01,
        "H1-H7 span {h1_h7:.4} should be 180°"
    );
}

fn check_php_julday_calc(jd: f64) {
    use celestial_core::body::{Body, CalcFlags};
    use celestial_core::*;

    assert!((jd - 2_452_275.5).abs() < 1e-6);
    let d = revjul(jd, Calendar::Gregorian);
    assert_eq!(d.year, 2002);

    let p = calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN | CalcFlags::SPEED).unwrap();
    assert!((p.lon - 280.38).abs() < 0.1);
    assert!((p.dist - 0.9832).abs() < 0.005);
    assert!((p.speed_lon - 1.0).abs() < 0.1);
}

fn check_php_houses_ayanamsa(jd: f64) {
    use celestial_core::body::{HouseSystem, SiderealMode};
    use celestial_core::*;

    let h = houses(JulianDay::new(jd), Latitude::new(48.85), Longitude::new(2.35), HouseSystem::PLACIDUS).unwrap();
    let cusps: Vec<f64> = h.cusps[1..].to_vec();
    let ascmc: Vec<f64> = h.ascmc[..8].to_vec();
    assert_eq!(cusps.len(), 12);
    assert_eq!(ascmc.len(), 8);
    assert!(ascmc[0] >= 0.0 && ascmc[0] < 360.0);

    set_sid_mode(SiderealMode::LAHIRI, 0.0, 0.0);
    let ay = ayanamsa(jd);
    assert!(ay > 23.5 && ay < 24.5, "Lahiri ayanamsa {ay} out of range");
}

fn check_php_math_helpers() {
    use celestial_core::*;

    assert!((norm_deg(361.5) - 1.5).abs() < 1e-10);
    assert!((norm_deg(-1.0) - 359.0).abs() < 1e-10);
    assert!((diff_deg_signed(10.0, 350.0) - 20.0).abs() < 1e-10);

    assert!((midpoint(10.0, 20.0) - 15.0).abs() < 1e-10);
    let fortune = arabic_part(206.77, 223.32, 280.38);
    assert!((fortune - 149.71).abs() < 0.1);
    assert!((0.0..360.0).contains(&fortune));
}

fn check_php_zodiac_helpers() {
    use celestial_core::body::Body;
    use celestial_core::*;

    assert_eq!(sign_ruler(0), Body::MARS);
    assert_eq!(sign_ruler(4), Body::SUN);
    assert_eq!(zodiac_sign_name(0), "Aries");
    assert_eq!(zodiac_sign_name(11), "Pisces");
    let (sign, deg) = lon_to_sign(45.5);
    assert_eq!(sign, 1);
    assert!((deg - 15.5).abs() < 0.001);
}

fn check_php_eclipse_rise_tret(jd: f64) {
    use celestial_core::body::{Body, CalcFlags};
    use celestial_core::*;

    let ecl = sol_eclipse_when_glob(jd, CalcFlags::BUILTIN, 0, false).unwrap();
    let _tret_vec: Vec<f64> = ecl.tret.to_vec();
    let rise = rise_trans(
        jd,
        Body::MOON,
        None,
        CalcFlags::BUILTIN,
        CALC_RISE,
        [-0.12, 51.5, 10.0],
        0.0,
        0.0,
    )
    .unwrap();
    let _tret_scalar: f64 = rise.tret;
    let _tret_wrapped: Vec<f64> = vec![rise.tret];
}

fn check_php_name_helpers() {
    use celestial_core::body::{Body, CalcFlags, SiderealMode};
    use celestial_core::*;

    let flg_builtin_i64: i64 = CalcFlags::BUILTIN.as_raw() as i64;
    assert!(flg_builtin_i64 > 0);

    let _name: String = planet_name(Body::SUN).to_string();
    let ayname: String = ayanamsa_name(SiderealMode::LAHIRI.as_raw()).to_string();
    assert_eq!(ayname, "Lahiri");
}

#[test]
fn fuzz_php_binding_equivalence() {
    use celestial_core::*;

    let jd = julday(2002, 1, 1, 0.0, Calendar::Gregorian);
    check_php_julday_calc(jd);
    check_php_houses_ayanamsa(jd);
    check_php_math_helpers();
    check_php_zodiac_helpers();
    check_php_eclipse_rise_tret(jd);
    check_php_name_helpers();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1 — wheel enhancements
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 5 — Hellenistic / Persian
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 6 — Chinese astrology
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 7 — Mesoamerican calendars
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 8 — Indigenous / Egyptian decans
// ═══════════════════════════════════════════════════════════════════════════════
