//! Moon nodes (mean and true) and planetary nodes/apsides.

use crate::astronomy::constants::{norm_deg as norm360, to_rad};

const J2000: f64 = 2451545.0;

// ─── Moon: mean node ──────────────────────────────────────────────────────────

/// Mean ascending node of the Moon (ecliptic longitude, degrees).
/// Meeus "Astronomical Algorithms" ch. 47.
#[must_use]
pub fn moon_mean_node(jd_et: f64) -> f64 {
    let t = (jd_et - J2000) / 36525.0;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let om = 125.044_555_501 - 1_934.136_261_972 * t + 0.002_075_633 * t2 + 0.000_002_139 * t3
        - 0.000_000_016_5 * t4;
    norm360(om)
}

/// Speed of the mean node (deg/day).
#[must_use]
pub fn moon_mean_node_speed(jd_et: f64) -> f64 {
    let t = (jd_et - J2000) / 36525.0;
    // d/dt of mean node (deg/century → deg/day)
    let dc = -1_934.136_261_972 + 2.0 * 0.002_075_633 * t + 3.0 * 0.000_002_139 * t * t;
    dc / 36525.0
}

// ─── Moon: true node ──────────────────────────────────────────────────────────

/// True (osculating) ascending node of the Moon.
/// Adds the principal periodic corrections to the mean node.
#[must_use]
pub fn moon_true_node(jd_et: f64) -> f64 {
    let t = (jd_et - J2000) / 36525.0;

    // Mean anomalies and arguments (degrees)
    let m = norm360(357.529_109_2 + 35_999.050_29 * t); // Sun  mean anomaly
    let mp = norm360(134.963_396_0 + 477_198.867_398 * t); // Moon mean anomaly
    let f = norm360(93.272_095_0 + 483_202.017_538 * t); // Moon arg of lat
    let om = moon_mean_node(jd_et);

    // Periodic terms (Meeus table 47.b, truncated to significant terms)
    let delta = -1.4979 * (2.0 * to_rad(f - om)).sin()
        + 0.1500 * to_rad(m).sin()
        + 0.1226 * (2.0 * to_rad(f)).sin()
        + -0.1176 * (2.0 * to_rad(om)).sin()
        + -0.0801 * (2.0 * to_rad(mp - f)).sin();

    norm360(om + delta)
}

/// Speed of the true node (deg/day) — numerical derivative.
#[must_use]
pub fn moon_true_node_speed(jd_et: f64) -> f64 {
    let h = 0.5;
    (moon_true_node(jd_et + h) - moon_true_node(jd_et - h)) / (2.0 * h)
}

// ─── Lunar apsides ────────────────────────────────────────────────────────────

/// Longitude of lunar perigee (degrees) — mean value.
/// Meeus ch. 48.
#[must_use]
pub fn moon_mean_perigee(jd_et: f64) -> f64 {
    let t = (jd_et - J2000) / 36525.0;
    let pi = 83.353_243_0 + 4_069.013_564 * t - 0.010_325_2 * t * t - 0.000_012_468 * t * t * t;
    norm360(pi)
}

/// Speed of mean perigee (deg/day).
#[must_use]
pub fn moon_mean_perigee_speed(_jd_et: f64) -> f64 {
    // ~4069.013564 deg/century → deg/day
    4_069.013_564 / 36525.0
}

// ─── Planetary mean nodes / apsides ──────────────────────────────────────────
//
// From Meeus table 31.a — mean elements referred to the mean ecliptic of date.
// Valid roughly 1000 BCE – 3000 CE.

#[derive(Debug, Clone, Copy)]
pub struct PlanetElements {
    /// Longitude of ascending node (deg)
    pub node_lon: f64,
    /// Longitude of perihelion (deg)
    pub peri_lon: f64,
    /// Semi-major axis (AU)
    pub semi_major: f64,
    /// Eccentricity
    pub ecc: f64,
    /// Inclination (deg)
    pub inc: f64,
    /// Mean longitude (deg)
    pub mean_lon: f64,
}

/// Mean orbital elements at J2000 with per-century linear rates.
/// Layout: `[L0,L1, a0,a1, e0,e1, i0,i1, Om0,Om1, w0,w1]`.
const MERCURY_ELEMENTS: [f64; 12] = [
    252.250_324_4,
    149_472.674_1,
    0.387_098_310,
    0.0,
    0.205_630_69,
    0.000_020_59,
    7.004_986,
    0.001_821,
    48.330_766,
    1.186_189,
    77.456_119,
    1.556_478,
];
const VENUS_ELEMENTS: [f64; 12] = [
    181.979_801_0,
    58_517.815_7,
    0.723_329_820,
    0.0,
    0.006_773_23,
    -0.000_047_02,
    3.394_662,
    0.001_098_7,
    76.679_920,
    0.901_190,
    131.563_703,
    1.402_427,
];
const EARTH_ELEMENTS: [f64; 12] = [
    100.464_457_0,
    35_999.372_9,
    1.000_001_018,
    0.0,
    0.016_708_63,
    -0.000_042_04,
    0.0,
    0.000_013_0,
    0.0,
    0.018_45,
    102.937_348,
    1.719_526,
];
const MARS_ELEMENTS: [f64; 12] = [
    355.433_000,
    19_140.299_1,
    1.523_679_342,
    0.0,
    0.093_405_29,
    0.000_092_04,
    1.849_726,
    -0.000_813_0,
    49.558_093,
    0.772_990,
    336.060_234,
    1.840_969,
];
const JUPITER_ELEMENTS: [f64; 12] = [
    34.351_519,
    3_034.905_7,
    5.202_603_191,
    0.000_001_913,
    0.048_494_85,
    0.000_016_32,
    1.303_270,
    -0.001_987_0,
    100.464_407,
    1.020_955,
    14.331_207,
    1.612_020,
];
const SATURN_ELEMENTS: [f64; 12] = [
    50.077_444,
    1_222.113_9,
    9.554_909_596,
    -0.000_002_139,
    0.055_508_62,
    -0.000_034_56,
    2.488_878,
    0.002_551_7,
    113.665_503,
    0.877_088,
    93.057_237,
    1.963_542,
];
const URANUS_ELEMENTS: [f64; 12] = [
    314.055_005,
    428.466_0,
    19.218_446_062,
    -0.000_000_372,
    0.046_380_90,
    -0.000_026_58,
    0.773_197,
    0.000_771_6,
    74.005_957,
    0.521_467,
    173.005_159,
    1.484_678,
];
const NEPTUNE_ELEMENTS: [f64; 12] = [
    304.348_665,
    218.459_1,
    30.110_386_869,
    -0.000_001_663,
    0.009_455_75,
    0.000_006_06,
    1.769_952,
    -0.000_299_4,
    131.784_057,
    1.102_182,
    48.123_691,
    1.426_296,
];

#[inline]
fn elements_for_body(body: i32) -> Option<&'static [f64; 12]> {
    match body {
        1 => Some(&MERCURY_ELEMENTS),
        2 => Some(&VENUS_ELEMENTS),
        0 | 3 | 14 => Some(&EARTH_ELEMENTS),
        4 => Some(&MARS_ELEMENTS),
        5 => Some(&JUPITER_ELEMENTS),
        6 => Some(&SATURN_ELEMENTS),
        7 => Some(&URANUS_ELEMENTS),
        8 => Some(&NEPTUNE_ELEMENTS),
        _ => None,
    }
}

/// Orbital elements for a planet at Julian ephemeris date.
/// body: 0=Sun/Earth, 1=Mercury, 2=Venus, 4=Mars,
///       5=Jupiter, 6=Saturn, 7=Uranus, 8=Neptune
pub fn planet_mean_elements(body: i32, jd_et: f64) -> Option<PlanetElements> {
    let t = (jd_et - J2000) / 36525.0;
    let [l0, l1, a0, a1, e0, e1, i0, i1, om0, om1, w0, w1] = *elements_for_body(body)?;
    Some(PlanetElements {
        mean_lon: norm360(l0 + l1 * t),
        semi_major: a0 + a1 * t,
        ecc: e0 + e1 * t,
        inc: i0 + i1 * t,
        node_lon: norm360(om0 + om1 * t),
        peri_lon: norm360(w0 + w1 * t),
    })
}

/// Compute nodes & apsides for a planet.
/// Returns (asc_node_lon, desc_node_lon, perihelion_lon, aphelion_lon)
/// all in ecliptic degrees, along with the inclination.
/// Returns `(asc_lon, desc_lon, peri_lon, aphe_lon, inc)` for a planet.
pub fn planet_nodes_apsides(body: i32, jd_et: f64) -> Option<(f64, f64, f64, f64, f64)> {
    let el = planet_mean_elements(body, jd_et)?;
    let asc = el.node_lon;
    let desc = norm360(asc + 180.0);
    let peri = el.peri_lon;
    let aphe = norm360(peri + 180.0);
    Some((asc, desc, peri, aphe, el.inc))
}

/// Returns speeds (°/day) for node and perihelion by numerical differentiation.
pub fn planet_nodes_speeds(body: i32, jd_et: f64) -> Option<(f64, f64)> {
    let el_p = planet_mean_elements(body, jd_et + 0.5)?;
    let el_m = planet_mean_elements(body, jd_et - 0.5)?;

    let node_speed = crate::functions::utils::wrap_signed_180(el_p.node_lon - el_m.node_lon);
    let peri_speed = crate::functions::utils::wrap_signed_180(el_p.peri_lon - el_m.peri_lon);
    Some((node_speed, peri_speed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elements_lookup_known_bodies() {
        for body in [0i32, 1, 2, 3, 4, 5, 6, 7, 8, 14] {
            assert!(elements_for_body(body).is_some(), "body {body}");
        }
    }

    #[test]
    fn elements_lookup_unknown_returns_none() {
        for body in [-1i32, 9, 10, 11, 12, 13, 15, 100] {
            assert!(elements_for_body(body).is_none(), "body {body}");
        }
    }

    #[test]
    fn earth_aliases_share_table() {
        let a = elements_for_body(0).unwrap();
        let b = elements_for_body(3).unwrap();
        let c = elements_for_body(14).unwrap();
        assert!(std::ptr::eq(a, b));
        assert!(std::ptr::eq(b, c));
    }

    #[test]
    fn mean_elements_at_j2000_match_constants() {
        // At t=0, mean_lon = L0 normalized; semi_major = a0; ecc = e0; inc = i0
        let el = planet_mean_elements(1, J2000).unwrap();
        assert!((el.mean_lon - norm360(MERCURY_ELEMENTS[0])).abs() < 1e-9);
        assert!((el.semi_major - MERCURY_ELEMENTS[2]).abs() < 1e-12);
        assert!((el.ecc - MERCURY_ELEMENTS[4]).abs() < 1e-12);
        assert!((el.inc - MERCURY_ELEMENTS[6]).abs() < 1e-12);
    }

    #[test]
    fn nodes_apsides_inclination_consistent() {
        for body in [1, 2, 4, 5, 6, 7, 8] {
            let (_a, _d, _p, _ap, inc) = planet_nodes_apsides(body, J2000).unwrap();
            assert!((0.0..=10.0).contains(&inc), "body {body} inc {inc}");
        }
    }

    #[test]
    fn fuzz_mean_elements_no_nan_no_unbounded() {
        let mut s: u64 = 0xDEADBEEFCAFEu64;
        for _ in 0..512 {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let jd = J2000 + ((s % 2_000_000) as f64 - 1_000_000.0); // ± ~2700 yr
            for body in [1, 2, 3, 4, 5, 6, 7, 8] {
                let el = planet_mean_elements(body, jd).unwrap();
                assert!(el.mean_lon.is_finite() && (0.0..360.0).contains(&el.mean_lon));
                assert!(el.node_lon.is_finite() && (0.0..360.0).contains(&el.node_lon));
                assert!(el.peri_lon.is_finite() && (0.0..360.0).contains(&el.peri_lon));
                assert!(el.semi_major.is_finite() && el.semi_major > 0.0);
                assert!(el.ecc.is_finite());
                assert!(el.inc.is_finite());
            }
        }
    }
}
