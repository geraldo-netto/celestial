//! Pluto position via Meeus "Astronomical Algorithms" ch. 37.
//!
//! Uses a low-accuracy table valid 1885–2099, giving positions
//! accurate to roughly 0.07° in longitude and 0.02° in latitude.
//! Good enough for astrological use (orbs > 0.5°).

use crate::astronomy::constants::to_rad;

/// Heliocentric ecliptic coordinates of Pluto (J2000.0 ecliptic).
/// Returns `(lon_deg, lat_deg, radius_au)`.
///
/// Valid range: 1885–2099. Outside this range, results degrade gracefully.
#[must_use]
pub fn pluto_pos(jde: f64) -> (f64, f64, f64) {
    let t = (jde - 2_451_545.0) / 36525.0; // Julian centuries from J2000
    let j = to_rad(34.35 + 3034.9057 * t);
    let s = to_rad(50.08 + 1222.1138 * t);
    let p = to_rad(238.96 + 144.9600 * t);

    // Meeus table 37.a — 43 terms
    type PlutoTerm = (f64, f64, f64, f64, f64, f64, f64, f64, f64);
    let coeffs: &[PlutoTerm] = &[
        (
            0., 0., 1., -19799805., 19850055., -5452852., -14974862., 66865439., 68951812.,
        ),
        (
            0., 0., 2., 897144., -4954829., 3527812., 1672790., -11827535., -332538.,
        ),
        (
            0., 0., 3., 611149., 1211027., -1050748., 327647., 1593179., -1438890.,
        ),
        (
            0., 0., 4., -341243., -189585., 178690., -292153., -18444., 483220.,
        ),
        (
            0., 0., 5., 129287., -34992., 18650., 100340., -65977., -85879.,
        ),
        (
            0., 0., 6., -38164., 30893., -30697., -25823., 31174., -6973.,
        ),
        (0., 1., 0., 20442., -9987., 4878., 11248., -5765., -1940.),
        (0., 1., 1., -4063., -9402., -678., -2572., 3538., 6498.),
        (0., 1., 2., -6016., -3416., 462., -5870., 2802., -2327.),
        (0., 1., 3., -3956., 790., -423., -1817., 531., -484.),
        (0., 1., 4., -667., 852., -127., 426., -743., -43.),
        (0., 2., 0., 1247., -1228., -1897., -1310., 2141., 2573.),
        (0., 2., 1., 647., -1515., 230., -632., -309., -856.),
        (0., 2., 2., -243., 1195., -172., 637., 199., -244.),
        (0., 3., 0., 185., -228., 67., 45., -96., -102.),
        (0., 3., 1., -152., -142., -78., -16., 32., 30.),
        (0., 3., 2., -122., 30., 80., -14., 71., 7.),
        (0., 4., 0., -60., 30., 11., -17., 5., 6.),
        (1., 0., 0., 1265., -130., 50., 1251., -822., -314.),
        (1., 0., 1., -180., -165., -50., -19., -10., -33.),
        (1., 0., 2., 62., -27., 9., -7., 6., -38.),
        (1., 1., 0., -9., -6., -3., 2., -3., -3.),
        (1., 2., 0., 1., 1., -1., 0., -1., -1.),
        (2., 0., 0., -4., 1., 0., 0., -1., -2.),
        (2., 0., 1., 1., -1., 0., 0., 0., 0.),
        (2., 1., 0., 0., 0., 0., 0., 0., 0.),
        (3., 0., 0., 0., 0., 0., 0., 0., 0.),
        (1., 0., 3., -0., -0., 0., -0., 0., 0.),
        (0., 0., 0., 0., 0., 0., 0., 0., 0.),
    ];

    let mut lon_sum = 0.0_f64;
    let mut lat_sum = 0.0_f64;
    let mut rad_sum = 0.0_f64;

    for &(jc, sc, pc, la, lb, ba, bb, ra, rb) in coeffs {
        let arg = jc * j + sc * s + pc * p;
        let (sa, ca) = arg.sin_cos();
        lon_sum += la * sa + lb * ca;
        lat_sum += ba * sa + bb * ca;
        rad_sum += ra * sa + rb * ca;
    }

    // Convert from Meeus units (1e-6 degrees, 1e-7 AU)
    let mut lon = 238.958_116 + 144.960_455 * t + lon_sum * 1e-6;
    let lat = -3.908_239 + lat_sum * 1e-6;
    let rad = 40.7241346 + rad_sum * 1e-7;

    lon = lon.rem_euclid(360.0);
    (lon, lat, rad)
}

/// Geocentric ecliptic position of Pluto at JDE.
///
/// Converts heliocentric → geocentric using Earth's heliocentric position.
#[must_use]
pub fn pluto_geocentric(jde: f64) -> (f64, f64, f64) {
    use crate::astronomy::vsop87::{heliocentric, Planet};

    let (pl_lon, pl_lat, pl_r) = pluto_pos(jde);
    let earth = heliocentric(Planet::Earth, jde);

    // Convert to rectangular heliocentric (each cos/sin pair → one sin_cos call)
    let plon_r = pl_lon.to_radians();
    let plat_r = pl_lat.to_radians();
    let (sin_plon, cos_plon) = plon_r.sin_cos();
    let (sin_plat, cos_plat) = plat_r.sin_cos();
    let px = pl_r * cos_plat * cos_plon;
    let py = pl_r * cos_plat * sin_plon;
    let pz = pl_r * sin_plat;

    let elon_r = earth.lon;
    let elat_r = earth.lat;
    let (sin_elon, cos_elon) = elon_r.sin_cos();
    let (sin_elat, cos_elat) = elat_r.sin_cos();
    let ex = earth.rad * cos_elat * cos_elon;
    let ey = earth.rad * cos_elat * sin_elon;
    let ez = earth.rad * sin_elat;

    // Geocentric vector
    let dx = px - ex;
    let dy = py - ey;
    let dz = pz - ez;

    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let lon = dy.atan2(dx).to_degrees().rem_euclid(360.0);
    let lat = (dz / dist).asin().to_degrees();

    (lon, lat, dist)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pluto_j2000_reasonable() {
        let (lon, lat, r) = pluto_pos(2_451_545.0);
        assert!((0.0..360.0).contains(&lon), "lon={lon}");
        assert!(lat.abs() < 20.0, "lat={lat}");
        assert!(r > 28.0 && r < 50.0, "r={r} AU");
        // Pluto was at ~246° (Sagittarius) around J2000
        assert!(
            lon > 240.0 && lon < 260.0,
            "Pluto lon={lon:.2}°, expected ~246-250°"
        );
    }
}
