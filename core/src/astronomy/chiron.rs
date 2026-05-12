//! Chiron (2060 Chiron / 95P/Chiron) position.
//!
//! Chiron orbits between Saturn and Uranus with a ~50-year period.
//! We use the mean orbital elements from the JPL small-body database
//! and propagate them with Kepler's equation for a good approximation.
//!
//! Accuracy: ~0.5° over 1800–2200 CE; degrades outside that range.

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
    for _ in 0..50 {
        let (sin_e, cos_e) = e.sin_cos();
        let de = (m - e + ecc * sin_e) / (1.0 - ecc * cos_e);
        e += de;
        if de.abs() < 1e-12 {
            break;
        }
    }
    e
}

/// Compute Chiron's heliocentric ecliptic longitude and latitude (degrees)
/// and distance (AU) for a given Julian day (UT ≈ ET for this purpose).
///
/// Elements: epoch J2000.0, from MPC / AstDys.
#[must_use]
pub fn chiron_pos(jd: f64) -> (f64, f64, f64) {
    // Mean elements at epoch J2000.0
    let a = 13.648_16_f64; // AU
    let ecc = 0.382_95_f64;
    let inc = to_rad(6.930_2_f64);
    let node = to_rad(209.386_7_f64); // ascending node
    let peri = to_rad(339.534_3_f64); // argument of perihelion (from node)
    let m0 = to_rad(48.5_f64); // mean anomaly at J2000.0 (from ephemeris)
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
#[must_use]
pub fn chiron_speed(jd: f64) -> (f64, f64, f64) {
    let h = 0.5;
    let (l0, b0, r0) = chiron_pos(jd - h);
    let (l1, b1, r1) = chiron_pos(jd + h);
    let dl = ((l1 - l0 + 540.0) % 360.0) - 180.0; // handle 0/360 wrap
    (
        (dl / (2.0 * h)),
        (b1 - b0) / (2.0 * h),
        (r1 - r0) / (2.0 * h),
    )
}
