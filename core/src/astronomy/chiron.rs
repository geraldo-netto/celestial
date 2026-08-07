//! Chiron (2060 Chiron / 95P/Chiron) position.
//!
//! Chiron orbits between Saturn and Uranus with a ~50-year period.
//! We use the mean orbital elements from the JPL small-body database
//! and propagate them with Kepler's equation for a good approximation.
//!
//! Accuracy: ~0.5° over 1800–2200 CE; degrades outside that range.

use crate::units::JulianDay;
use std::f64::consts::PI;

const J2000: f64 = 2451545.0;
const TWO_PI: f64 = 2.0 * PI;

#[allow(dead_code)]
fn norm360(d: f64) -> f64 {
    d.rem_euclid(360.0)
}
fn to_rad(d: f64) -> f64 {
    d * PI / 180.0
}
fn to_deg(r: f64) -> f64 {
    r * 180.0 / PI
}

/// Solve Kepler's equation M = E - e·sin(E) for the eccentric anomaly E.
/// Newton iteration with paired `sin_cos` (one transcendental call per iter).
fn kepler(m: f64, ecc: f64) -> f64 {
    let m = m.rem_euclid(TWO_PI);
    let mut e = m;
    for _ in 0..8 {
        let (sin_e, cos_e) = e.sin_cos();
        let de = (m - e + ecc * sin_e) / (1.0 - ecc * cos_e);
        e += de;
    }
    e
}

/// Compute Chiron's heliocentric ecliptic longitude and latitude (degrees)
/// and distance (AU) for a given Julian day (UT ≈ ET for this purpose).
///
/// Elements: epoch J2000.0, from MPC / AstDys.
#[must_use]
pub fn chiron_pos(jd: JulianDay) -> (f64, f64, f64) {
    let jd: f64 = jd.into();
    // Mean elements at epoch J2000.0
    let a = 13.648_16_f64; // AU
    let ecc = 0.382_95_f64;
    let inc = to_rad(6.930_2_f64);
    let node = to_rad(209.386_7_f64); // ascending node
    let peri = to_rad(339.534_3_f64); // argument of perihelion (from node)
                                      // Mean anomaly at J2000.0 in degrees.
                                      // Chiron perihelion was JD 2450162.0 (1996-02-14). At J2000.0 the
                                      // time since perihelion is 1383 d → M = n·1383 ≈ 27.0° where the
                                      // mean motion n = 360° / (50.45 y · 365.25 d/y) = 0.01956°/d.
                                      // The previous value 48.5° was ~21° ahead of orbit (caused ~26° too
                                      // far advanced in ecliptic longitude at all dates).
    let m0 = to_rad(27.0_f64);
    // Orbital period: P = sqrt(a³) years
    let period = a.powf(1.5) * 365.25; // days
    let n = TWO_PI / period; // mean motion rad/day

    // Mean anomaly at JD
    let dt = jd - J2000;
    let m = (m0 + n * dt).rem_euclid(TWO_PI);

    // Eccentric anomaly
    let e = kepler(m, ecc);

    // True anomaly
    let nu = 2.0 * (((1.0 + ecc) / (1.0 - ecc)).sqrt() * (e / 2.0).tan()).atan();

    // Distance
    let r = a * (1.0 - ecc * e.cos());

    // Heliocentric ecliptic coordinates (IAU 76 reference plane)
    // Argument of latitude
    let u = peri + nu;
    let (sin_u, cos_u) = u.sin_cos();
    let (sin_i, cos_i) = inc.sin_cos();
    let (sin_n, cos_n) = node.sin_cos();

    // 3-D heliocentric coordinates
    let x = r * (-sin_n * sin_u).mul_add(cos_i, cos_n * cos_u);
    let y = r * (cos_n * sin_u).mul_add(cos_i, sin_n * cos_u);
    let z = r * sin_u * sin_i;

    let lon = to_deg(y.atan2(x)).rem_euclid(360.0);
    let lat = to_deg((z / r).asin());

    (lon, lat, r)
}

/// Speed of Chiron (deg/day) via numerical differentiation.
/// Uses geocentric (`chiron_geocentric`) so consumers see the apparent
/// motion of Chiron as seen from Earth — matching how
/// `chiron_speed` is consumed by `calc_chiron`.
#[must_use]
pub fn chiron_speed(jd: JulianDay) -> (f64, f64, f64) {
    let jd: f64 = jd.into();
    let h = 0.5;
    let (l0, b0, r0) = chiron_geocentric(jd - h);
    let (l1, b1, r1) = chiron_geocentric(jd + h);
    let dl = ((l1 - l0 + 540.0) % 360.0) - 180.0; // handle 0/360 wrap
    (dl, b1 - b0, r1 - r0)
}

/// Geocentric ecliptic position of Chiron at JDE.
///
/// `chiron_pos` returns HELIOCENTRIC coordinates. For chart use we need
/// what Chiron looks like from Earth: subtract Earth's heliocentric
/// position vector. Mirrors `astronomy::pluto::pluto_geocentric`.
///
/// Before this conversion existed, `calc_chiron` returned heliocentric
/// coordinates labelled as geocentric, giving 10–40° errors at every
/// chart date (J2000: 27° off vs published; Diana 1961: 10° off; 2024:
/// 13° off; Geraldo Netto PDF 1986: 41° off).
#[must_use]
pub fn chiron_geocentric(jde: f64) -> (f64, f64, f64) {
    use crate::astronomy::vsop87::{heliocentric, Planet};

    let (ch_lon, ch_lat, ch_r) = chiron_pos(JulianDay::new(jde));
    let earth = heliocentric(Planet::Earth, jde);

    // Chiron heliocentric → rectangular ecliptic.
    let plon_r = ch_lon.to_radians();
    let plat_r = ch_lat.to_radians();
    let (sin_plon, cos_plon) = plon_r.sin_cos();
    let (sin_plat, cos_plat) = plat_r.sin_cos();
    let px = ch_r * cos_plat * cos_plon;
    let py = ch_r * cos_plat * sin_plon;
    let pz = ch_r * sin_plat;

    // Earth heliocentric (VSOP87 returns lon/lat in RADIANS already).
    let (sin_elon, cos_elon) = earth.lon.sin_cos();
    let (sin_elat, cos_elat) = earth.lat.sin_cos();
    let ex = earth.rad * cos_elat * cos_elon;
    let ey = earth.rad * cos_elat * sin_elon;
    let ez = earth.rad * sin_elat;

    // Geocentric position vector (Chiron − Earth).
    let dx = px - ex;
    let dy = py - ey;
    let dz = pz - ez;

    let dist = dx.hypot(dy).hypot(dz);
    let lon = dy.atan2(dx).to_degrees().rem_euclid(360.0);
    let lat = (dz / dist).asin().to_degrees();
    (lon, lat, dist)
}

#[cfg(test)]
mod cov_tests {
    use super::*;

    #[test]
    fn norm360_wraps() {
        assert!((norm360(361.0) - 1.0).abs() < 1e-9);
        assert!((norm360(-1.0) - 359.0).abs() < 1e-9);
    }

    #[test]
    fn geocentric_position_and_speed_match_regression_vectors() {
        let cases = [
            (
                1_721_425.5,
                [
                    353.856_071_833_356_17,
                    3.695_837_339_007_048_7,
                    18.878_173_353_844_677,
                    0.025_022_906_417_916_6,
                    -0.004_209_203_034_547_304,
                    0.016_922_137_780_259_305,
                ],
            ),
            (
                J2000,
                [
                    250.777_080_645_625,
                    3.984_617_154_410_26,
                    10.577_608_742_139_182,
                    0.114_693_592_874_800_74,
                    0.006_279_641_217_276_755,
                    -0.006_675_457_573_217_969,
                ],
            ),
            (
                2_463_456.789,
                [
                    55.082_625_987_681_816,
                    -2.611_554_029_901_816,
                    16.260_702_032_527_156,
                    0.017_508_213_009_932_66,
                    -0.004_105_368_138_216_381,
                    -0.017_994_451_649_084_198,
                ],
            ),
            (
                3_182_045.0,
                [
                    52.972_101_009_587_94,
                    -3.232_883_876_786_869,
                    15.240_127_249_033_964,
                    -0.016_546_972_626_088_063,
                    0.001_724_831_371_064_272_6,
                    0.013_862_870_649_566_261,
                ],
            ),
        ];
        for (jd, expected) in cases {
            let (lon, lat, dist) = chiron_geocentric(jd);
            let (speed_lon, speed_lat, speed_dist) = chiron_speed(JulianDay::new(jd));
            assert_eq!([lon, lat, dist, speed_lon, speed_lat, speed_dist], expected);
        }
    }

    #[test]
    fn kepler_solver_matches_regression_vectors() {
        let cases = [
            (0.0, 0.0),
            (1.25, 1.632_227_637_717_615_3),
            (-2.75, 3.425_805_408_883_046_5),
            (7.0, 1.048_757_387_221_363_6),
        ];
        for (mean_anomaly, expected) in cases {
            assert!((kepler(mean_anomaly, 0.382_95) - expected).abs() < 1e-14);
        }
    }
}
