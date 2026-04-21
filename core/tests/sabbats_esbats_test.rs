//! Tests for Celtic sabbat and esbat calculations.

use crate::body::Calendar;

use celestial_core::body::{Body, CalcFlags};
use celestial_core::*;

fn setup() {
    set_ephe_path("").unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Sabbat tests
// ══════════════════════════════════════════════════════════════════════════════

// ── Solar longitude at each sabbat ───────────────────────────────────────────

/// Helper: Sun's ecliptic longitude at a given JD.
fn sun_lon(jd: f64) -> f64 {
    calc_ut(jd, Body::SUN, CalcFlags::BUILTIN).unwrap().lon
}

/// Helper: Sun longitude difference (shortest arc, signed).
fn lon_diff(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 {
        d - 360.0
    } else {
        d
    }
}

#[test]
fn test_sabbat_yule_sun_at_270() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Yule).unwrap();
    let diff = lon_diff(sun_lon(jd), 270.0).abs();
    assert!(
        diff < 0.01,
        "Yule: Sun should be at 270°, diff = {diff:.6}°"
    );
}

#[test]
fn test_sabbat_imbolc_sun_at_315() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Imbolc).unwrap();
    let diff = lon_diff(sun_lon(jd), 315.0).abs();
    assert!(
        diff < 0.01,
        "Imbolc: Sun should be at 315°, diff = {diff:.6}°"
    );
}

#[test]
fn test_sabbat_ostara_sun_at_0() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Ostara).unwrap();
    let diff = lon_diff(sun_lon(jd), 0.0).abs();
    assert!(
        diff < 0.01,
        "Ostara: Sun should be at 0°, diff = {diff:.6}°"
    );
}

#[test]
fn test_sabbat_beltane_sun_at_45() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Beltane).unwrap();
    let diff = lon_diff(sun_lon(jd), 45.0).abs();
    assert!(
        diff < 0.01,
        "Beltane: Sun should be at 45°, diff = {diff:.6}°"
    );
}

#[test]
fn test_sabbat_litha_sun_at_90() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Litha).unwrap();
    let diff = lon_diff(sun_lon(jd), 90.0).abs();
    assert!(
        diff < 0.01,
        "Litha: Sun should be at 90°, diff = {diff:.6}°"
    );
}

#[test]
fn test_sabbat_lughnasadh_sun_at_135() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Lughnasadh).unwrap();
    let diff = lon_diff(sun_lon(jd), 135.0).abs();
    assert!(
        diff < 0.01,
        "Lughnasadh: Sun should be at 135°, diff = {diff:.6}°"
    );
}

#[test]
fn test_sabbat_mabon_sun_at_180() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Mabon).unwrap();
    let diff = lon_diff(sun_lon(jd), 180.0).abs();
    assert!(
        diff < 0.01,
        "Mabon: Sun should be at 180°, diff = {diff:.6}°"
    );
}

#[test]
fn test_sabbat_samhain_sun_at_225() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Samhain).unwrap();
    let diff = lon_diff(sun_lon(jd), 225.0).abs();
    assert!(
        diff < 0.01,
        "Samhain: Sun should be at 225°, diff = {diff:.6}°"
    );
}

// ── Calendar sanity ───────────────────────────────────────────────────────────

#[test]
fn test_sabbat_yule_in_december() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Yule).unwrap();
    let d = revjul(jd, Calendar::Gregorian);
    assert_eq!(
        d.month, 12,
        "Yule should be in December, got month {}",
        d.month
    );
    assert!(
        d.day >= 19 && d.day <= 23,
        "Yule day should be 19-23, got {}",
        d.day
    );
}

#[test]
fn test_sabbat_ostara_in_march() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Ostara).unwrap();
    let d = revjul(jd, Calendar::Gregorian);
    assert_eq!(
        d.month, 3,
        "Ostara should be in March, got month {}",
        d.month
    );
    assert!(
        d.day >= 19 && d.day <= 22,
        "Ostara day should be 19-22, got {}",
        d.day
    );
}

#[test]
fn test_sabbat_litha_in_june() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Litha).unwrap();
    let d = revjul(jd, Calendar::Gregorian);
    assert_eq!(d.month, 6, "Litha should be in June, got month {}", d.month);
    assert!(
        d.day >= 19 && d.day <= 23,
        "Litha day should be 19-23, got {}",
        d.day
    );
}

#[test]
fn test_sabbat_mabon_in_september() {
    setup();
    let jd = sabbat_jd(2024, SabbatKind::Mabon).unwrap();
    let d = revjul(jd, Calendar::Gregorian);
    assert_eq!(
        d.month, 9,
        "Mabon should be in September, got month {}",
        d.month
    );
    assert!(
        d.day >= 21 && d.day <= 24,
        "Mabon day should be 21-24, got {}",
        d.day
    );
}

// ── sabbats_for_year ──────────────────────────────────────────────────────────

#[test]
fn test_sabbats_for_year_count() {
    setup();
    let sabbats = sabbats_for_year(2024).unwrap();
    assert_eq!(sabbats.len(), 8, "Should return exactly 8 sabbats");
}

#[test]
fn test_sabbats_for_year_sorted() {
    setup();
    let sabbats = sabbats_for_year(2024).unwrap();
    for w in sabbats.windows(2) {
        assert!(
            w[0].jd < w[1].jd,
            "Sabbats should be in chronological order: {} ({:.1}) ≥ {} ({:.1})",
            w[0].name,
            w[0].jd,
            w[1].name,
            w[1].jd
        );
    }
}

#[test]
fn test_sabbats_for_year_correct_sun_longitudes() {
    setup();
    let sabbats = sabbats_for_year(2024).unwrap();
    for s in &sabbats {
        let expected = s.kind.solar_longitude();
        let actual = sun_lon(s.jd);
        let diff = lon_diff(actual, expected).abs();
        assert!(
            diff < 0.01,
            "{}: expected Sun at {expected}°, got {actual:.6}° (diff {diff:.6}°)",
            s.name
        );
    }
}

#[test]
fn test_sabbats_for_year_span() {
    setup();
    let sabbats = sabbats_for_year(2024).unwrap();
    let year_start = julday(2024, 1, 1, 0.0, Calendar::Gregorian);
    let year_end = julday(2025, 1, 1, 0.0, Calendar::Gregorian);
    for s in &sabbats {
        assert!(
            s.jd >= year_start && s.jd < year_end,
            "{} JD {:.1} is outside 2024",
            s.name,
            s.jd
        );
    }
}

#[test]
fn test_sabbats_spacing() {
    // Consecutive sabbats should be ~45–50 days apart
    setup();
    let sabbats = sabbats_for_year(2024).unwrap();
    for w in sabbats.windows(2) {
        let gap = w[1].jd - w[0].jd;
        assert!(
            gap > 40.0 && gap < 55.0,
            "Gap between {} and {} = {gap:.1} days (expected 40–55)",
            w[0].name,
            w[1].name
        );
    }
}

// ── next_sabbat ───────────────────────────────────────────────────────────────

#[test]
fn test_next_sabbat_returns_future() {
    setup();
    let jd_now = julday(2024, 6, 15, 0.0, Calendar::Gregorian); // mid-June 2024
    let next = next_sabbat(jd_now).unwrap();
    assert!(next.jd >= jd_now, "next_sabbat should return a future JD");
    // Mid-June → next sabbat should be Litha (late June)
    assert_eq!(
        next.kind,
        SabbatKind::Litha,
        "Next sabbat after Jun 15 should be Litha, got {}",
        next.name
    );
}

#[test]
fn test_next_sabbat_at_exact_sabbat_time() {
    setup();
    let litha_jd = sabbat_jd(2024, SabbatKind::Litha).unwrap();
    let next = next_sabbat(litha_jd).unwrap();
    // Should return Litha itself (jd >= litha_jd)
    assert!(next.jd >= litha_jd);
    assert_eq!(next.kind, SabbatKind::Litha);
}

// ── SabbatKind metadata ───────────────────────────────────────────────────────

#[test]
fn test_sabbat_kind_quarter_days() {
    assert!(SabbatKind::Yule.is_quarter_day());
    assert!(SabbatKind::Ostara.is_quarter_day());
    assert!(SabbatKind::Litha.is_quarter_day());
    assert!(SabbatKind::Mabon.is_quarter_day());
    assert!(!SabbatKind::Imbolc.is_quarter_day());
    assert!(!SabbatKind::Beltane.is_quarter_day());
    assert!(!SabbatKind::Lughnasadh.is_quarter_day());
    assert!(!SabbatKind::Samhain.is_quarter_day());
}

#[test]
fn test_sabbat_kind_cross_quarters() {
    assert!(SabbatKind::Imbolc.is_cross_quarter());
    assert!(SabbatKind::Beltane.is_cross_quarter());
    assert!(SabbatKind::Lughnasadh.is_cross_quarter());
    assert!(SabbatKind::Samhain.is_cross_quarter());
}

#[test]
fn test_sabbat_alt_names_nonempty() {
    for kind in SabbatKind::all_by_longitude() {
        let alts = kind.alt_names();
        assert!(!alts.is_empty(), "{} should have alt names", kind.name());
    }
}

#[test]
fn test_sabbat_solar_longitudes_unique() {
    let lons: Vec<f64> = SabbatKind::all_by_longitude()
        .iter()
        .map(|k| k.solar_longitude())
        .collect();
    for i in 0..lons.len() {
        for j in (i + 1)..lons.len() {
            assert!(
                (lons[i] - lons[j]).abs() > 1.0,
                "Duplicate solar longitude at index {i} and {j}"
            );
        }
    }
}

// ── Historical cross-check ────────────────────────────────────────────────────

#[test]
fn test_ostara_2000_known_date() {
    setup();
    // Spring equinox 2000: March 20, 07:35 UT
    let jd = sabbat_jd(2000, SabbatKind::Ostara).unwrap();
    let d = revjul(jd, Calendar::Gregorian);
    assert_eq!(d.year, 2000);
    assert_eq!(d.month, 3);
    assert_eq!(d.day, 20);
}

#[test]
fn test_litha_1992_known_date() {
    setup();
    // Summer solstice 1992: June 21, 03:14 UT
    let jd = sabbat_jd(1992, SabbatKind::Litha).unwrap();
    let d = revjul(jd, Calendar::Gregorian);
    assert_eq!(d.year, 1992);
    assert_eq!(d.month, 6);
    assert_eq!(d.day, 21);
}

// ══════════════════════════════════════════════════════════════════════════════
// Esbat tests
// ══════════════════════════════════════════════════════════════════════════════

/// Moon–Sun elongation at a given JD.
fn moon_elongation(jd: f64) -> f64 {
    let sun = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN).unwrap();
    let moon = calc_ut(jd, Body::MOON, CalcFlags::BUILTIN).unwrap();
    (moon.lon - sun.lon).rem_euclid(360.0)
}

// ── next_full_moon ────────────────────────────────────────────────────────────

#[test]
fn test_next_full_moon_elongation_is_180() {
    setup();
    let jd_start = julday(2024, 1, 1, 0.0, Calendar::Gregorian);
    let jd_fm = next_full_moon(jd_start).unwrap();
    let elong = moon_elongation(jd_fm);
    assert!(
        (elong - 180.0).abs() < 0.05,
        "Full moon elongation should be 180°, got {elong:.4}°"
    );
}

#[test]
fn test_next_full_moon_is_in_future() {
    setup();
    let jd_start = julday(2024, 6, 1, 0.0, Calendar::Gregorian);
    let jd_fm = next_full_moon(jd_start).unwrap();
    assert!(
        jd_fm >= jd_start,
        "Full moon should be at or after search start"
    );
}

#[test]
fn test_next_full_moon_multiple_consecutive() {
    setup();
    let mut jd = julday(2024, 1, 1, 0.0, Calendar::Gregorian);
    for i in 0..12 {
        let jd_fm = next_full_moon(jd).unwrap();
        let elong = moon_elongation(jd_fm);
        assert!(
            (elong - 180.0).abs() < 0.05,
            "Full moon {i}: elongation {elong:.4}° should be ~180°"
        );
        let gap = jd_fm - jd;
        if i > 0 {
            assert!(
                gap > 28.0 && gap < 30.5,
                "Full moon {i}: gap {gap:.2} days should be 28-30.5"
            );
        }
        jd = jd_fm + 1.0; // advance past this moon
    }
}

#[test]
fn test_next_full_moon_synodic_period() {
    setup();
    let jd0 = julday(2024, 3, 1, 0.0, Calendar::Gregorian);
    let fm1 = next_full_moon(jd0).unwrap();
    let fm2 = next_full_moon(fm1 + 1.0).unwrap();
    let period = fm2 - fm1;
    assert!(
        period > 29.0 && period < 30.0,
        "Synodic period should be 29-30 days, got {period:.4}"
    );
}

// ── esbats_for_year ───────────────────────────────────────────────────────────

#[test]
fn test_esbats_for_year_count() {
    setup();
    for year in [2022, 2023, 2024, 2025] {
        let esbats = esbats_for_year(year).unwrap();
        assert!(
            esbats.len() == 12 || esbats.len() == 13,
            "Year {year} should have 12 or 13 full moons, got {}",
            esbats.len()
        );
    }
}

#[test]
fn test_esbats_for_year_all_at_180_degrees() {
    setup();
    let esbats = esbats_for_year(2024).unwrap();
    for e in &esbats {
        let elong = moon_elongation(e.jd);
        assert!(
            (elong - 180.0).abs() < 0.05,
            "{}: elongation {elong:.4}° should be ~180°",
            e.display_name
        );
    }
}

#[test]
fn test_esbats_for_year_sorted() {
    setup();
    let esbats = esbats_for_year(2024).unwrap();
    for w in esbats.windows(2) {
        assert!(
            w[0].jd < w[1].jd,
            "{} ({:.1}) should precede {} ({:.1})",
            w[0].display_name,
            w[0].jd,
            w[1].display_name,
            w[1].jd
        );
    }
}

#[test]
fn test_esbats_for_year_within_calendar_year() {
    setup();
    let year = 2024;
    let year_start = julday(year, 1, 1, 0.0, Calendar::Gregorian);
    let year_end = julday(year + 1, 1, 1, 0.0, Calendar::Gregorian);
    let esbats = esbats_for_year(year).unwrap();
    for e in &esbats {
        assert!(
            e.jd >= year_start && e.jd < year_end,
            "{} JD {:.1} is outside {year}",
            e.display_name,
            e.jd
        );
    }
}

#[test]
fn test_esbats_has_harvest_moon() {
    setup();
    let esbats = esbats_for_year(2024).unwrap();
    assert!(
        esbats.iter().any(|e| e.name == EsbatName::Harvest),
        "Year 2024 should have a Harvest Moon"
    );
}

#[test]
fn test_esbats_harvest_moon_near_equinox() {
    setup();
    let year = 2024;
    let equinox_jd = sabbat_jd(year, SabbatKind::Mabon).unwrap();
    let esbats = esbats_for_year(year).unwrap();
    let harvest = esbats
        .iter()
        .find(|e| e.name == EsbatName::Harvest)
        .unwrap();
    // Harvest Moon should be within ~15 days of the autumn equinox
    let diff = (harvest.jd - equinox_jd).abs();
    assert!(
        diff < 16.0,
        "Harvest Moon should be within 15 days of Mabon, diff = {diff:.1} days"
    );
}

#[test]
fn test_esbats_unique_names_except_blue() {
    setup();
    let esbats = esbats_for_year(2024).unwrap();
    let mut names: Vec<EsbatName> = esbats.iter().map(|e| e.name).collect();
    // Blue Moon may appear once; all others should be unique
    names.retain(|&n| n != EsbatName::Blue);
    let deduped: std::collections::HashSet<_> = names.iter().collect();
    assert_eq!(
        names.len(),
        deduped.len(),
        "Non-Blue esbat names should be unique"
    );
}

#[test]
fn test_esbats_spacing() {
    // Consecutive full moons should be 29–30 days apart
    setup();
    let esbats = esbats_for_year(2024).unwrap();
    for w in esbats.windows(2) {
        let gap = w[1].jd - w[0].jd;
        assert!(
            gap > 28.5 && gap < 30.5,
            "Gap between {} and {} = {gap:.2} days (expected 28.5–30.5)",
            w[0].display_name,
            w[1].display_name
        );
    }
}

// ── next_esbat ────────────────────────────────────────────────────────────────

#[test]
fn test_next_esbat_returns_future() {
    setup();
    let jd = julday(2024, 6, 1, 0.0, Calendar::Gregorian);
    let e = next_esbat(jd).unwrap();
    assert!(
        e.jd >= jd,
        "next_esbat should return at or after search start"
    );
}

#[test]
fn test_next_esbat_elongation_is_180() {
    setup();
    let jd = julday(2024, 3, 1, 0.0, Calendar::Gregorian);
    let e = next_esbat(jd).unwrap();
    let elong = moon_elongation(e.jd);
    assert!(
        (elong - 180.0).abs() < 0.05,
        "next_esbat elongation = {elong:.4}°, should be ~180°"
    );
}

#[test]
fn test_next_esbat_has_nonempty_name() {
    setup();
    let jd = julday(2024, 1, 1, 0.0, Calendar::Gregorian);
    let e = next_esbat(jd).unwrap();
    assert!(!e.display_name.is_empty());
}

// ── EsbatName metadata ────────────────────────────────────────────────────────

#[test]
fn test_esbat_name_display_nonempty() {
    let names = [
        EsbatName::Wolf,
        EsbatName::Snow,
        EsbatName::Worm,
        EsbatName::Pink,
        EsbatName::Flower,
        EsbatName::Strawberry,
        EsbatName::Buck,
        EsbatName::Sturgeon,
        EsbatName::Harvest,
        EsbatName::Hunter,
        EsbatName::Beaver,
        EsbatName::Cold,
        EsbatName::Blue,
    ];
    for name in names {
        assert!(!name.display_name().is_empty());
        assert!(!name.alt_names().is_empty());
    }
}

// ── Historical cross-check ────────────────────────────────────────────────────

#[test]
fn test_full_moon_jan_2000_known_date() {
    setup();
    // Full moon January 21, 2000 at ~04:40 UT
    let jd_start = julday(2000, 1, 20, 0.0, Calendar::Gregorian);
    let jd_fm = next_full_moon(jd_start).unwrap();
    let d = revjul(jd_fm, Calendar::Gregorian);
    assert_eq!(d.year, 2000);
    assert_eq!(d.month, 1);
    assert_eq!(d.day, 21);
}

#[test]
fn test_full_moon_elongation_exactly_180_at_result() {
    setup();
    // The returned JD must have elongation within 0.05° of 180°
    let jd_start = julday(2024, 9, 1, 0.0, Calendar::Gregorian);
    let jd_fm = next_full_moon(jd_start).unwrap();
    let elong = moon_elongation(jd_fm);
    assert!(
        (elong - 180.0).abs() < 0.05,
        "elongation at returned JD = {elong:.6}°"
    );
}
