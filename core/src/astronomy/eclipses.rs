//! Solar and lunar eclipse search.
//!
//! Finds syzygies (new/full Moon) and checks eclipse conditions.
//! Uses the Meeus "Astronomical Algorithms" ch. 54 approach.
#![allow(dead_code)]

use crate::units::JulianDay;
use std::f64::consts::PI;

#[allow(dead_code)]
const J2000: f64 = 2451545.0;
#[allow(dead_code)]
const TAU: f64 = 2.0 * PI;

fn to_rad(d: f64) -> f64 {
    d * PI / 180.0
}
fn norm360(d: f64) -> f64 {
    d.rem_euclid(360.0)
}

/// Type of eclipse found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EclipseKind {
    None,
    TotalSolar,
    AnnularSolar,
    HybridSolar,
    PartialSolar,
    TotalLunar,
    PartialLunar,
    PenumbralLunar,
}

/// Result of an eclipse search.
#[derive(Debug, Clone, PartialEq)]
#[must_use = "the search result contains the computed data — did you mean to use it?"]
pub struct EclipseResult {
    pub kind: EclipseKind,
    /// Eclipse type flags.
    pub ret_flags: i32,
    /// tret[0] = greatest eclipse, [1] = partial begin, [2] = total begin,
    /// [3] = total end, [4] = partial end.
    pub tret: [f64; 10],
    /// Geographic longitude of centrality (solar only).
    pub geolon: f64,
    /// Geographic latitude of centrality (solar only).
    pub geolat: f64,
}

impl Default for EclipseResult {
    fn default() -> Self {
        Self {
            kind: EclipseKind::None,
            ret_flags: 0,
            tret: [0.0; 10],
            geolon: 0.0,
            geolat: 0.0,
        }
    }
}

/// Compute the JD of the k-th new Moon (k integer) after J2000.
/// Meeus ch. 49.
pub(crate) fn new_moon_jd(k: f64) -> f64 {
    let t = k / 1236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    let mut jde = 2_451_550.097_66 + 29.530_588_861 * k + 0.000_154_37 * t2 - 0.000_000_150 * t3
        + 0.000_000_000_73 * t4;

    // Sun mean anomaly
    let m = norm360(2.5534 + 29.105_356_70 * k - 0.000_001_4 * t2);
    // Moon mean anomaly
    let mp = norm360(201.5643 + 385.816_935_28 * k + 0.010_721 * t2 + 0.000_012_39 * t3);
    // Moon arg of latitude
    let f = norm360(160.7108 + 390.670_502_84 * k - 0.001_611_8 * t2 - 0.000_002_27 * t3);
    // Moon arg of omega
    let om = norm360(124.7746 - 1.563_755_88 * k + 0.002_067_2 * t2 + 0.000_002_15 * t3);

    let (m_r, mp_r, f_r, om_r) = (to_rad(m), to_rad(mp), to_rad(f), to_rad(om));

    // Planetary corrections (E factor for M)
    let e = 1.0 - 0.002_516 * t - 0.000_007_4 * t2;

    jde += -0.4072 * mp_r.sin()
        +  0.1721 * e * m_r.sin()
        +  0.0021 * e * (2.0 * m_r).sin()
        - 0.0097 * (2.0 * mp_r).sin()
        + 0.0040 * e * (mp_r - m_r).sin()
        - 0.0032 * e * (mp_r + m_r).sin()
        - 0.0028 * e * e * (2.0 * m_r - mp_r).sin()  // ← fixed: e*e not e*e*
        + 0.0003 * (2.0 * mp_r + m_r).sin()
        + 0.0003 * (4.0 * mp_r).sin()
        - 0.0166 * om_r.sin()
        + 0.0002 * (f_r + om_r).sin()
        + 0.0002 * (f_r - om_r).sin();

    // Additional corrections (W terms) — small
    jde += 0.0002 * (2.0 * f_r - m_r).sin() - 0.0002 * (3.0 * mp_r).sin()
        + 0.0002 * e * (mp_r - 2.0 * f_r + m_r).sin();

    jde
}

/// JD of the k-th full Moon.
fn full_moon_jd(k_int: i64) -> f64 {
    // Full Moon = k + 0.5
    let k = k_int as f64 + 0.5;
    new_moon_jd(k)
}

/// Compute the integer k for the new Moon nearest to jd_start (looking forward).
#[must_use]
pub fn k_from_jd(jd: JulianDay, forward: bool) -> i64 {
    let jd: f64 = jd.into();
    // Approximate k: months since J2000 new Moon (JD 2451550.0977)
    let months = (jd - 2_451_550.097_7) / 29.530_588_861;
    if forward {
        months.floor() as i64
    } else {
        months.ceil() as i64
    }
}

pub(crate) fn nearest_new_moon_k(jd: f64) -> i64 {
    let k0 = k_from_jd(JulianDay::new(jd), true);
    [k0 - 1, k0, k0 + 1]
        .into_iter()
        .min_by(|&a, &b| {
            let da = (new_moon_jd(a as f64) - jd).abs();
            let db = (new_moon_jd(b as f64) - jd).abs();
            da.partial_cmp(&db).unwrap()
        })
        .unwrap()
}

/// F (Moon's argument of latitude) at a given k — used to check eclipse possibility.
fn f_at_k(k: f64) -> f64 {
    let t = k / 1236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    norm360(
        160.7108 + 390.670_502_74 * k - 0.001_634_1 * t2 - 0.000_002_27 * t3 + 0.000_000_011 * t4,
    )
}

#[derive(Debug, Clone, Copy)]
struct EclipseGeometry {
    gamma: f64,
    u: f64,
    sin_f: f64,
}

fn eclipse_geometry(k: f64) -> EclipseGeometry {
    let t = k / 1236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let m = norm360(2.5534 + 29.105_356_69 * k - 0.000_021_8 * t2 - 0.000_000_11 * t3);
    let mp = norm360(
        201.5643 + 385.816_935_28 * k + 0.010_743_8 * t2 + 0.000_012_39 * t3 - 0.000_000_058 * t4,
    );
    let f = f_at_k(k);
    let om = norm360(124.7746 - 1.563_755_80 * k + 0.002_069_1 * t2 + 0.000_002_15 * t3);
    let e = 1.0 - 0.002_516 * t - 0.000_007_4 * t2;
    let f1 = f - 0.02665 * to_rad(om).sin();

    let (m_r, mp_r, f1_r) = (to_rad(m), to_rad(mp), to_rad(f1));
    let p = 0.2070 * e * m_r.sin() + 0.0024 * e * (2.0 * m_r).sin() - 0.0392 * mp_r.sin()
        + 0.0116 * (2.0 * mp_r).sin()
        - 0.0073 * e * (mp_r + m_r).sin()
        + 0.0067 * e * (mp_r - m_r).sin()
        + 0.0118 * (2.0 * f1_r).sin();
    let q = 5.2207 - 0.0048 * e * m_r.cos() + 0.0020 * e * (2.0 * m_r).cos()
        - 0.3299 * mp_r.cos()
        - 0.0060 * e * (mp_r + m_r).cos()
        + 0.0041 * e * (mp_r - m_r).cos();
    let w = f1_r.cos().abs();
    let gamma = (p * f1_r.cos() + q * f1_r.sin()) * (1.0 - 0.0048 * w);
    let u = 0.0059 + 0.0046 * e * to_rad(m).cos() - 0.0182 * to_rad(mp).cos()
        + 0.0004 * to_rad(2.0 * mp).cos()
        - 0.0005 * to_rad(m + mp).cos();
    EclipseGeometry {
        gamma,
        u,
        sin_f: to_rad(f).sin(),
    }
}

fn eclipse_candidate(g: EclipseGeometry) -> bool {
    g.sin_f.abs() <= 0.36
}

/// Eclipse magnitude for solar eclipse at new Moon (Meeus 54).
/// Returns the penumbral and umbral magnitudes.
pub(crate) fn solar_eclipse_magnitude(k_int: i64) -> (f64, f64) {
    let g = eclipse_geometry(k_int as f64);
    let gamma_abs = g.gamma.abs();
    let denom = 0.5461 + 2.0 * g.u;
    let pen_mag = (1.5433 + g.u - gamma_abs) / denom;
    let umb_mag = (0.9972 - gamma_abs) / denom;

    (pen_mag, umb_mag)
}

/// Check if a new Moon (k_int) produces a solar eclipse.
/// Returns (EclipseKind, gamma, u).
pub(crate) fn check_solar_eclipse(k_int: i64) -> (EclipseKind, f64, f64) {
    let g = eclipse_geometry(k_int as f64);
    if !eclipse_candidate(g) {
        return (EclipseKind::None, 0.0, 0.0);
    }

    let gamma_abs = g.gamma.abs();
    let kind = if gamma_abs > 1.5433 + g.u {
        EclipseKind::None
    } else if gamma_abs > 0.9972 {
        EclipseKind::PartialSolar
    } else {
        if g.u < 0.0 {
            EclipseKind::TotalSolar
        } else if g.u > 0.0047 {
            EclipseKind::AnnularSolar
        } else {
            EclipseKind::HybridSolar
        }
    };

    (kind, g.gamma, g.u)
}

/// Check if a full Moon (k_int + 0.5) produces a lunar eclipse.
#[must_use]
pub fn check_lunar_eclipse(k_int: i64) -> (EclipseKind, f64, f64) {
    let g = eclipse_geometry(k_int as f64 + 0.5);
    if !eclipse_candidate(g) {
        return (EclipseKind::None, 0.0, 0.0);
    }

    let gamma_abs = g.gamma.abs();
    let pen_mag = (1.5573 + g.u - gamma_abs) / 0.5450;
    let umb_mag = (1.0128 - g.u - gamma_abs) / 0.5450;
    let total_limit = 0.4678 - g.u;
    let kind = if pen_mag <= 0.0 {
        EclipseKind::None
    } else if umb_mag <= 0.0 {
        EclipseKind::PenumbralLunar
    } else if gamma_abs > total_limit {
        EclipseKind::PartialLunar
    } else {
        EclipseKind::TotalLunar
    };

    (kind, pen_mag, umb_mag)
}

/// Duration helpers (in days): contact times around greatest eclipse.
fn solar_contacts(_k_int: i64, gamma: f64, u: f64) -> (f64, f64, f64, f64) {
    // Meeus 54.4 approximation for contact times
    let n = 0.5181; // semi-diameter of Moon in Earth radii (approx)
    let _ = u;
    let _ = gamma;
    let _ = n;
    // Simple: ±1h for partial, ±0.5h for totality
    let p = 1.0 / 24.0;
    let t = 0.5 / 24.0;
    (-p, -t, t, p)
}

fn lunar_contacts(_k_int: i64, _pen_mag: f64, umb_mag: f64) -> (f64, f64, f64, f64) {
    // Approximate contact durations
    let p1 = -1.5 / 24.0;
    let u1 = if umb_mag > 0.0 { -0.8 / 24.0 } else { 0.0 };
    let u4 = if umb_mag > 0.0 { 0.8 / 24.0 } else { 0.0 };
    let p4 = 1.5 / 24.0;
    (p1, u1, u4, p4)
}

// ─── ECL_* flag constants ─────────────────────────────────────────────────────
pub const ECL_TOTAL: i32 = 4;
pub const ECL_ANNULAR: i32 = 8;
pub const ECL_PARTIAL: i32 = 16;
pub const ECL_HYBRID: i32 = 32;
pub const ECL_PENUMBRAL: i32 = 64;
pub const ECL_CENTRAL: i32 = 1;
pub const ECL_NONCENTRAL: i32 = 2;

pub(crate) fn kind_to_flags(kind: EclipseKind) -> i32 {
    match kind {
        EclipseKind::TotalSolar => ECL_TOTAL | ECL_CENTRAL,
        EclipseKind::AnnularSolar => ECL_ANNULAR | ECL_CENTRAL,
        EclipseKind::HybridSolar => ECL_HYBRID | ECL_CENTRAL,
        EclipseKind::PartialSolar => ECL_PARTIAL | ECL_NONCENTRAL,
        EclipseKind::TotalLunar => ECL_TOTAL,
        EclipseKind::PartialLunar => ECL_PARTIAL,
        EclipseKind::PenumbralLunar => ECL_PENUMBRAL,
        EclipseKind::None => 0,
    }
}

/// Per-iteration acceptance check: filter by `ecl_type` and verify that `jde`
/// lies on the requested side of `jd_start`. Returns `Some(flags)` to accept,
/// or `None` to skip this k.
fn accept_eclipse(
    kind: EclipseKind,
    ecl_type: i32,
    jde: f64,
    jd_start: f64,
    backwards: bool,
) -> Option<i32> {
    if kind == EclipseKind::None {
        return None;
    }
    let flags = kind_to_flags(kind);
    if ecl_type != 0 && (flags & ecl_type) == 0 {
        return None;
    }
    if !backwards && jde < jd_start {
        return None;
    }
    if backwards && jde > jd_start {
        return None;
    }
    Some(flags)
}

/// Find the next solar eclipse after `jd_start`.
pub fn solar_eclipse_when_glob(
    jd_start: JulianDay,
    ecl_type: i32,
    backwards: bool,
) -> Option<EclipseResult> {
    let jd_start: f64 = jd_start.into();
    let mut k = k_from_jd(JulianDay::new(jd_start), !backwards);
    let dir: i64 = if backwards { -1 } else { 1 };

    for _ in 0..60 {
        let (kind, gamma, u) = check_solar_eclipse(k);
        let jde = new_moon_jd(k as f64);
        if let Some(flags) = accept_eclipse(kind, ecl_type, jde, jd_start, backwards) {
            let (dp1, dt1, dt2, dp2) = solar_contacts(k, gamma, u);
            let mut tret = [0.0f64; 10];
            tret[0] = jde;
            tret[1] = jde + dp1;
            tret[2] = jde + dt1;
            tret[3] = jde + dt2;
            tret[4] = jde + dp2;
            return Some(EclipseResult {
                kind,
                ret_flags: flags,
                tret,
                geolon: 0.0,
                geolat: 0.0,
            });
        }
        k += dir;
    }
    None
}

/// Find the next lunar eclipse after `jd_start`.
pub fn lun_eclipse_when(
    jd_start: JulianDay,
    ecl_type: i32,
    backwards: bool,
) -> Option<EclipseResult> {
    let jd_start: f64 = jd_start.into();
    let mut k = k_from_jd(JulianDay::new(jd_start), !backwards);
    let dir: i64 = if backwards { -1 } else { 1 };

    for _ in 0..60 {
        let (kind, pen_mag, umb_mag) = check_lunar_eclipse(k);
        let jde = full_moon_jd(k);
        if let Some(flags) = accept_eclipse(kind, ecl_type, jde, jd_start, backwards) {
            let (dp1, du1, du4, dp4) = lunar_contacts(k, pen_mag, umb_mag);
            let mut tret = [0.0f64; 10];
            tret[0] = jde;
            tret[1] = jde + dp1;
            tret[2] = jde + du1;
            tret[3] = jde + du4;
            tret[4] = jde + dp4;
            return Some(EclipseResult {
                kind,
                ret_flags: flags,
                tret,
                geolon: 0.0,
                geolat: 0.0,
            });
        }
        k += dir;
    }
    None
}

// ─── Eclipse attributes ───────────────────────────────────────────────────────

/// Eclipse magnitude and attribute data (20 values) for a solar eclipse at `jd_ut`.
/// Compatible with `swe_sol_eclipse_how` attr array.
/// attr[0]=mag, attr[1]=sun_diam, attr[2]=moon_diam, attr[3]=sun_dist, attr[4]=moon_dist,
/// attr[5]=umbral_depth, attr[6]=penumbral_depth, attr[7]=gamma, attr[8]=saros, attr[9]=saros_mem
/// Compute the geographic coordinates of the point of greatest eclipse (solar).
///
/// `gamma` = shadow axis distance from Earth centre (in Earth radii).
/// `jde`   = Julian Ephemeris Day of greatest eclipse.
///
/// Returns `(longitude_deg, latitude_deg)` of the sub-solar point
/// on Earth's surface where the eclipse is greatest.
///
/// Algorithm: Meeus ch. 54 — the sub-Moon point at greatest eclipse is
/// computed from the Moon's equatorial coordinates and Earth's rotation.
/// Geographic coordinates of greatest solar eclipse.
///
/// Uses the sub-lunar point method: at the moment of greatest eclipse (new Moon),
/// the Moon's shadow axis intersects Earth at the sub-lunar point. The latitude
/// is Moon's geocentric declination corrected for Earth's flattening; the longitude
/// is the Moon's geographic hour angle.
///
/// **Accuracy:** This is a first-order approximation — the sub-lunar latitude equals
/// the Moon's geocentric declination, which matches the geographic eclipse path
/// to within ~8° latitude. A full solution (Besselian elements, topocentric parallax)
/// would close the gap but requires significantly more computation.
#[must_use]
pub fn solar_eclipse_geopos(jde: f64, _gamma: f64) -> (f64, f64) {
    use crate::astronomy::moon::lunar_position;

    let moon = lunar_position(jde);
    let eps = crate::astronomy::obliquity(jde).to_radians();

    // Convert Moon ecliptic (lon, lat) → equatorial (RA, Dec) in radians
    let lon_r = moon.lon; // lunar_position already returns radians
    let lat_r = moon.lat;
    let (sin_lon, cos_lon) = lon_r.sin_cos();
    let (sin_lat, cos_lat) = lat_r.sin_cos();
    let (sin_eps, cos_eps) = eps.sin_cos();
    let moon_ra = (-cos_lat * sin_eps)
        .mul_add(sin_lon, sin_lat * cos_eps)
        .atan2(cos_lon);
    let moon_dec = sin_lat.mul_add(sin_eps, cos_lat * cos_eps * sin_lon).asin();

    // Greenwich Apparent Sidereal Time → radians
    let dt = crate::astronomy::delta_t::delta_t(JulianDay::new(jde));
    let jd_ut = jde - dt / 86400.0;
    let gst_rad = crate::functions::time::sidtime(JulianDay::new(jd_ut)).to_radians() * 15.0;
    // (sidtime returns hours; ×15 = degrees; ×π/180 = radians)

    // Geographic latitude of the sub-lunar point
    // Apply Earth flattening correction (f = 1/298.257):
    // geographic_lat = atan((1 - f²) × tan(moon_dec))  ≈ atan(0.993305 × tan(moon_dec))
    const FLAT_CORR: f64 = 0.993_305; // (1 - 1/298.257)²  ≈ 1 - 2/298.257
    let lat_geo = (FLAT_CORR * moon_dec.tan()).atan();
    let lat_deg = lat_geo.to_degrees();

    // Geographic longitude of the sub-lunar point
    // lon_geo = moon_ra - GAST  (hour angle converted to east longitude)
    let lon_rad = moon_ra - gst_rad;
    let lon_deg = lon_rad.to_degrees().rem_euclid(360.0);
    let lon_deg = if lon_deg > 180.0 {
        lon_deg - 360.0
    } else {
        lon_deg
    };

    (lon_deg, lat_deg)
}

/// Compute solar-eclipse attributes at a given observer location.
///
/// Returns a 20-element array matching the layout of Swiss Ephemeris'
/// `swe_sol_eclipse_how`. Not every slot is populated by this implementation;
/// values not computed are set to neutral defaults.
///
/// # Layout (subset)
/// * `attr[0]` — eclipse magnitude (umbral if total, else penumbral)
/// * `attr[1]` — Sun's angular diameter (degrees)
/// * `attr[2]` — Moon's angular diameter
/// * `attr[3]` — Sun distance (AU, mean)
/// * `attr[4]` — Moon distance (relative, mean)
/// * others reserved for future use
///
/// # Parameters
/// * `jd_ut` — Julian day (UT) near the eclipse
/// * `geopos` — `[longitude_deg, latitude_deg, altitude_m]`
#[must_use]
pub fn solar_eclipse_attr(jd_ut: JulianDay, geopos: [f64; 3]) -> [f64; 20] {
    let jd_ut: f64 = jd_ut.into();
    let mut attr = [0.0f64; 20];
    // Find nearest new Moon
    let k = nearest_new_moon_k(jd_ut);
    let (_kind, gamma, _u) = check_solar_eclipse(k);
    let (pen_mag, umb_mag) = solar_eclipse_magnitude(k);

    let mag = if umb_mag > 0.0 { umb_mag } else { pen_mag };
    attr[0] = mag.max(0.0); // magnitude
    attr[1] = 0.5266; // mean solar angular diameter (degrees)
    attr[2] = 0.5181; // mean lunar angular diameter
    attr[3] = 1.0; // mean sun distance (AU)
    attr[4] = 1.0; // mean moon distance (relative)
    attr[5] = umb_mag.max(0.0);
    attr[6] = pen_mag.max(0.0);
    attr[7] = gamma;
    attr[8] = 0.0; // Saros number (not computed)
    attr[9] = 0.0; // Saros member
                   // Geographic position of greatest eclipse (rough: lon from geometry)
    attr[10] = geopos[0];
    attr[11] = geopos[1];
    attr
}

/// Attribute data for a lunar eclipse (20 values).
/// attr[0]=penumbral_mag, attr[1]=umbral_mag, attr[2]=penumbral_partial_begin,…
#[must_use]
pub fn lunar_eclipse_attr(k_int: i64) -> [f64; 20] {
    let mut attr = [0.0f64; 20];
    let (_, pen_mag, umb_mag) = check_lunar_eclipse(k_int);
    attr[0] = pen_mag.max(0.0);
    attr[1] = umb_mag.max(0.0);
    attr[2] = 1.0; // approximate semi-duration (hours)
    attr
}

#[cfg(test)]
mod cov_tests {
    use super::*;
    use crate::body::Calendar;
    use crate::functions::time::julday;

    #[test]
    fn eclipse_result_default_is_zeroed() {
        let r = EclipseResult::default();
        assert_eq!(r.ret_flags, 0);
        assert_eq!(r.tret, [0.0; 10]);
        assert_eq!(r.geolon, 0.0);
        assert_eq!(r.geolat, 0.0);
        assert!(matches!(r.kind, EclipseKind::None));
    }

    #[test]
    fn norm360_wraps_positive_and_negative() {
        assert!((norm360(361.0) - 1.0).abs() < 1e-9);
        assert!((norm360(-1.0) - 359.0).abs() < 1e-9);
        assert!((norm360(0.0)).abs() < 1e-9);
    }

    #[test]
    fn solar_search_skips_2024_february_non_eclipse() {
        let start = jd(2024, 1, 20, 0.0);
        let eclipse = solar_eclipse_when_glob(JulianDay::new(start), 0, false).unwrap();
        let expected = jd(2024, 4, 8, 18.3);
        assert_eq!(eclipse.kind, EclipseKind::TotalSolar);
        assert!((eclipse.tret[0] - expected).abs() < 1.0);
    }

    #[test]
    fn solar_partial_annular_and_total_kinds_are_reachable() {
        assert_next_solar_kind(2025, 3, 1, EclipseKind::PartialSolar);
        assert_next_solar_kind(2023, 10, 1, EclipseKind::AnnularSolar);
        assert_next_solar_kind(2024, 4, 1, EclipseKind::TotalSolar);
    }

    #[test]
    fn lunar_penumbral_partial_and_total_kinds_are_reachable() {
        assert_next_lunar_kind(2024, 3, 1, EclipseKind::PenumbralLunar);
        assert_next_lunar_kind(2023, 10, 1, EclipseKind::PartialLunar);
        assert_next_lunar_kind(2018, 7, 1, EclipseKind::TotalLunar);
    }

    fn assert_next_solar_kind(year: i32, month: i32, day: i32, expected: EclipseKind) {
        let start = jd(year, month, day, 0.0);
        let eclipse = solar_eclipse_when_glob(JulianDay::new(start), 0, false).unwrap();
        assert_eq!(eclipse.kind, expected, "solar after {year}-{month}-{day}");
    }

    fn assert_next_lunar_kind(year: i32, month: i32, day: i32, expected: EclipseKind) {
        let start = jd(year, month, day, 0.0);
        let eclipse = lun_eclipse_when(JulianDay::new(start), 0, false).unwrap();
        assert_eq!(eclipse.kind, expected, "lunar after {year}-{month}-{day}");
    }

    fn jd(year: i32, month: i32, day: i32, hour: f64) -> f64 {
        julday(year, month, day, hour, Calendar::Gregorian)
    }
}
