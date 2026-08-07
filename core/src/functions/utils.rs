//! Utility calculations: azimuth/altitude, refraction, coordinate transforms,
//! phenomena, Gauquelin sectors, and heliacal events.

use crate::units::{Degrees, JulianDay};

// ─── Azimuth / altitude ───────────────────────────────────────────────────────

/// Result of [`azalt`].
#[derive(Debug, Clone, PartialEq)]
pub struct AzAlt {
    /// Azimuth (degrees).
    pub azimuth: f64,
    /// True altitude above horizon (degrees).
    pub true_alt: f64,
    /// Apparent (refracted) altitude (degrees).
    pub apparent_alt: f64,
}

/// Convert ecliptic/equatorial coordinates to azimuth and altitude.
///
/// - `calc_flag = 0` (`SE_ECL2HOR`): `xin` = ecliptic `[lon°, lat°, dist]`
/// - `calc_flag = 1` (`SE_EQU2HOR`): `xin` = equatorial `[RA°, Dec°, dist]`
/// - `geopos` = observer `[lon°, lat°, alt_m]` (east positive)
/// - `pressure_mb` = atmospheric pressure (mbar); `temp_c` = temperature (°C)
///
/// Returns azimuth (0° N, clockwise) and true / apparent altitude above horizon.
#[must_use]
pub fn azalt(
    jd_ut: JulianDay,
    direction: i32,
    geopos: [f64; 3],
    pressure_mb: f64,
    temp_c: f64,
    xin: [f64; 3],
) -> AzAlt {
    let jd_ut: f64 = jd_ut.into();
    let geolon = geopos[0]; // observer longitude (°E)
    let geolat = geopos[1]; // observer latitude  (°N)

    // ── Step 1: get equatorial RA / Dec (degrees) ──────────────────────────
    let (ra, dec) = if direction == 0 {
        // Ecliptic → equatorial using true obliquity
        let eps = crate::astronomy::obliquity_true(jd_ut).to_radians();
        let lon = xin[0].to_radians();
        let lat = xin[1].to_radians();

        let (sin_lon, cos_lon) = lon.sin_cos();
        let (sin_lat, cos_lat) = lat.sin_cos();
        let (sin_eps, cos_eps) = eps.sin_cos();
        let sin_dec = sin_lat.mul_add(cos_eps, cos_lat * sin_eps * sin_lon);
        let dec_rad = sin_dec.clamp(-1.0, 1.0).asin();

        let y = (-lat.tan()).mul_add(sin_eps, sin_lon * cos_eps);
        let ra_rad = y.atan2(cos_lon);

        (ra_rad.to_degrees().rem_euclid(360.0), dec_rad.to_degrees())
    } else {
        (xin[0], xin[1]) // already equatorial
    };

    // ── Step 2: Local Sidereal Time → Hour Angle ───────────────────────────
    let gmst_deg = crate::astronomy::houses::sidereal_time_deg(JulianDay::new(jd_ut));
    let lst_deg = (gmst_deg + geolon).rem_euclid(360.0);
    let ha_deg = (lst_deg - ra).rem_euclid(360.0);

    // ── Step 3: equatorial → horizontal ───────────────────────────────────
    let ha_r = ha_deg.to_radians();
    let dec_r = dec.to_radians();
    let lat_r = geolat.to_radians();

    let (sin_ha, cos_ha) = ha_r.sin_cos();
    let (sin_dec, cos_dec) = dec_r.sin_cos();
    let (sin_lat, cos_lat) = lat_r.sin_cos();
    let sin_alt = sin_lat.mul_add(sin_dec, cos_lat * cos_dec * cos_ha);
    let true_alt_rad = sin_alt.clamp(-1.0, 1.0).asin();
    let true_alt = true_alt_rad.to_degrees();

    // Azimuth: North-based clockwise (N=0°, E=90°, S=180°, W=270°)
    let cos_az = (-sin_lat).mul_add(sin_alt, sin_dec) / (cos_lat * true_alt_rad.cos().max(1e-10));
    let az_base = cos_az.clamp(-1.0, 1.0).acos().to_degrees();
    let az_north = if sin_ha > 0.0 {
        360.0 - az_base
    } else {
        az_base
    };

    // Swiss Ephemeris convention: azimuth measured from South, clockwise
    // (S=0°, W=90°, N=180°, E=270°).  South-based = (North-based + 180°) % 360°
    let azimuth = (az_north + 180.0).rem_euclid(360.0);

    let apparent_alt = apparent_altitude(true_alt, pressure_mb, temp_c);

    AzAlt {
        azimuth,
        true_alt,
        apparent_alt,
    }
}

fn apparent_altitude(true_alt: f64, pressure_mb: f64, temp_c: f64) -> f64 {
    if true_alt > -5.0 {
        refrac(true_alt, pressure_mb, temp_c, 0)
    } else {
        true_alt
    }
}

/// Convert azimuth/altitude back to ecliptic/equatorial coordinates.
///
/// This is the exact inverse of [`azalt`].
///
/// - `calc_flag = 0` (`SE_HOR2ECL`): returns ecliptic `[lon°, lat°, 1.0]`
/// - `calc_flag = 1` (`SE_HOR2EQU`): returns equatorial `[RA°, Dec°, 1.0]`
/// - `xin` = `[azimuth°, true_altitude°]`  (South-based azimuth, S=0° clockwise)
/// - `geopos` = observer `[lon°, lat°, alt_m]`
#[must_use]
pub fn azalt_rev(jd_ut: JulianDay, direction: i32, geopos: [f64; 3], xin: [f64; 2]) -> [f64; 3] {
    let jd_ut: f64 = jd_ut.into();
    let geolon = geopos[0];
    let geolat = geopos[1];

    // Convert SE South-based azimuth → North-based (N=0°, E=90°)
    let az_north = south_to_north_azimuth(xin[0]);
    let alt = xin[1];

    let az_r = az_north.to_radians();
    let alt_r = alt.to_radians();
    let lat_r = geolat.to_radians();

    // Horizontal → equatorial
    let (sin_lat, cos_lat) = lat_r.sin_cos();
    let (sin_alt, cos_alt) = alt_r.sin_cos();
    let (sin_az, cos_az) = az_r.sin_cos();
    let sin_dec = sin_lat.mul_add(sin_alt, cos_lat * cos_alt * cos_az);
    let dec_rad = sin_dec.clamp(-1.0, 1.0).asin();
    let dec = dec_rad.to_degrees();

    let cos_ha = (-sin_lat).mul_add(sin_dec, sin_alt) / (cos_lat * dec_rad.cos().max(1e-10));
    // Quadrant rule (inverse of azalt's forward convention):
    //   az_north in (180°,360°) — sin < 0 — means ha was in (0°,180°)   → ha = ha_base
    //   az_north in (0°,180°)  — sin > 0 — means ha was in (180°,360°) → ha = 360°-ha_base
    let ha_base = cos_ha.clamp(-1.0, 1.0).acos().to_degrees();
    let ha_deg = hour_angle_from_azimuth(sin_az, ha_base);

    // Hour angle → right ascension via Local Sidereal Time
    let gmst_deg = crate::astronomy::houses::sidereal_time_deg(JulianDay::new(jd_ut));
    let lst_deg = (gmst_deg + geolon).rem_euclid(360.0);
    let ra = (lst_deg - ha_deg).rem_euclid(360.0);

    if direction == 1 {
        // Return equatorial [RA, Dec, 1.0]
        return [ra, dec, 1.0];
    }

    // Convert equatorial → ecliptic
    let eps = crate::astronomy::obliquity_true(jd_ut).to_radians();
    let ra_r = ra.to_radians();
    let dec_r = dec_rad;

    let (sin_ra, cos_ra) = ra_r.sin_cos();
    let (sin_dec_e, cos_dec_e) = dec_r.sin_cos();
    let (sin_eps, cos_eps) = eps.sin_cos();
    let sin_lat = (-cos_dec_e).mul_add(sin_eps * sin_ra, sin_dec_e * cos_eps);
    let lat_ecl = sin_lat.clamp(-1.0, 1.0).asin().to_degrees();

    let y = dec_r.tan().mul_add(sin_eps, sin_ra * cos_eps);
    let lon_ecl = y.atan2(cos_ra).to_degrees().rem_euclid(360.0);

    [lon_ecl, lat_ecl, 1.0]
}

fn south_to_north_azimuth(azimuth: f64) -> f64 {
    (azimuth - 180.0).rem_euclid(360.0)
}

fn hour_angle_from_azimuth(sin_azimuth: f64, base: f64) -> f64 {
    if sin_azimuth < 0.0 {
        base
    } else {
        360.0 - base
    }
}

// ─── Refraction ───────────────────────────────────────────────────────────────

/// Compute atmospheric refraction.
#[must_use]
pub fn refrac(altitude: f64, pressure_mb: f64, temp_c: f64, direction: i32) -> f64 {
    {
        // Simple Bennett formula
        let a = altitude + 7.31 / (altitude + 4.4);
        let r = 1.02 / a.to_radians().tan();
        let r_corrected = r * (pressure_mb / 1010.0) * (283.0 / (273.0 + temp_c)) / 60.0;
        if direction == 0 {
            altitude + r_corrected
        } else {
            altitude - r_corrected
        }
    }
}

/// Extended refraction calculation.
#[must_use]
pub fn refrac_extended(
    altitude: f64,
    _geoalt: f64,
    pressure_mb: f64,
    temp_c: f64,
    _lapse_rate: f64,
    calc_flag: i32,
) -> (f64, [f64; 4]) {
    let result = refrac(altitude, pressure_mb, temp_c, calc_flag);
    (result, [result, altitude, 0.0, 0.0])
}

// ─── Coordinate transforms ────────────────────────────────────────────────────

/// Transform ecliptic ↔ equatorial coordinates.
///
/// `coords` = `[lon, lat, dist]`, `eps` = obliquity in degrees.
/// Positive `eps` converts ecliptic → equatorial; negative reverses.
#[must_use]
pub fn coord_transform(coords: [f64; 3], eps: Degrees) -> [f64; 3] {
    let eps: f64 = eps.into();
    let (lon, lat, dist) = (coords[0], coords[1], coords[2]);
    let eps_r = eps.to_radians();
    let lon_r = lon.to_radians();
    let lat_r = lat.to_radians();
    // Swiss Ephemeris sign convention (swe_cotrans):
    // y' = y*cos(eps) + z*sin(eps),  z' = -y*sin(eps) + z*cos(eps)
    let (sin_lat, cos_lat) = lat_r.sin_cos();
    let (sin_lon, cos_lon) = lon_r.sin_cos();
    let (sin_eps, cos_eps) = eps_r.sin_cos();
    let x = cos_lon * cos_lat;
    let y = sin_lon * cos_lat;
    let z = sin_lat;
    let yp = z.mul_add(sin_eps, y * cos_eps);
    let zp = (-y).mul_add(sin_eps, z * cos_eps);
    let out_lon = yp.atan2(x).to_degrees().rem_euclid(360.0);
    let out_lat = zp.clamp(-1.0, 1.0).asin().to_degrees();
    [out_lon, out_lat, dist]
}

/// Transform with speeds (coord_transform_with_speed).
#[must_use]
pub fn coord_transform_with_speed(coords: [f64; 6], eps: Degrees) -> [f64; 6] {
    let pos = [coords[0], coords[1], coords[2]];
    let out = coord_transform(pos, eps);
    [out[0], out[1], out[2], coords[3], coords[4], coords[5]]
}

// ─── Math utilities ───────────────────────────────────────────────────────────

/// Normalise degrees to [0, 360).
#[inline]
#[must_use]
pub fn norm_deg(x: f64) -> f64 {
    x.rem_euclid(360.0)
}

/// Normalise radians to [0, 2π).
#[inline]
#[must_use]
pub fn norm_rad(x: f64) -> f64 {
    x.rem_euclid(std::f64::consts::TAU)
}

/// Midpoint of two ecliptic degrees (accounts for 360° wrap).
#[must_use]
pub fn midpoint_deg(x1: f64, x0: f64) -> f64 {
    let d = diff_deg_signed(x1, x0);
    norm_deg(x0 + d / 2.0)
}

/// Midpoint of two radian values.
#[must_use]
pub fn midpoint_rad(x1: f64, x0: f64) -> f64 {
    let d = diff_rad_signed(x1, x0);
    norm_rad(x0 + d / 2.0)
}

/// Wrap an angle delta into (−180, +180]. Handles arbitrary input magnitude
/// via `rem_euclid` then shifts the [0, 360) result into (−180, 180].
#[inline]
#[must_use]
pub fn wrap_signed_180(d: f64) -> f64 {
    let r = d.rem_euclid(360.0);
    if r > 180.0 {
        r - 360.0
    } else {
        r
    }
}

/// Signed difference of degrees, result in (−180, +180].
#[inline]
#[must_use]
pub fn diff_deg_signed(p1: f64, p2: f64) -> f64 {
    wrap_signed_180(norm_deg(p1) - norm_deg(p2))
}

/// Unsigned difference of degrees, result in [0, 360).
#[inline]
#[must_use]
pub fn diff_deg(p1: f64, p2: f64) -> f64 {
    norm_deg(p1 - p2)
}

/// Signed difference of radians, result in (−π, +π].
#[must_use]
pub fn diff_rad_signed(p1: f64, p2: f64) -> f64 {
    let pi = std::f64::consts::PI;
    let d = norm_rad(p1) - norm_rad(p2);
    if d <= -pi {
        d + 2.0 * pi
    } else if d > pi {
        d - 2.0 * pi
    } else {
        d
    }
}

/// Signed difference of centiseconds (long), result in (−648000000, +648000000].
#[must_use]
pub fn diff_cs_signed(p1: i32, p2: i32) -> i64 {
    let full = 360 * 360_000i64;
    let d = norm_cs(p1) - norm_cs(p2);
    if d <= -(full / 2) {
        d + full
    } else if d > full / 2 {
        d - full
    } else {
        d
    }
}

/// Unsigned difference of centiseconds.
#[must_use]
pub fn diff_cs(p1: i32, p2: i32) -> i64 {
    norm_cs(p1 - p2)
}

/// Normalise centiseconds to [0, 360°).
#[must_use]
pub fn norm_cs(p: i32) -> i64 {
    p.rem_euclid(360 * 360_000) as i64
}

/// Round centiseconds to the nearest second.
#[must_use]
pub fn cs_round_sec(x: i32) -> i64 {
    let r = x % 100;
    if r >= 50 {
        (x - r + 100) as i64
    } else {
        (x - r) as i64
    }
}

/// Convert a float to a long integer (floor).
#[must_use]
pub fn deg_to_cs(x: f64) -> i64 {
    x.floor() as i64
}

// ─── Split degrees ────────────────────────────────────────────────────────────

/// Split a decimal degree value into degrees, minutes, seconds, fraction, sign.
///
/// Returns `(deg, min, sec, sec_fraction, sign)` where `sign` is +1 or −1,
/// or a zodiac sign number (1–12) when `SPLIT_DEG_ZODIACAL` is set in `round_flag`.
#[must_use]
pub fn split_deg(deg: f64, round_flag: i32) -> (i32, i32, i32, f64, i32) {
    use crate::constants::SPLIT_DEG_ZODIACAL;
    let sign = if deg < 0.0 { -1i32 } else { 1i32 };
    let abs = deg.abs();
    // Total arcseconds
    let total_sec = abs * 3600.0;
    let (d, m, s, frac) = if round_flag & 1 != 0 {
        // Round to nearest second
        let rounded = total_sec.round() as i64;
        let d = (rounded / 3600) as i32;
        let m = ((rounded % 3600) / 60) as i32;
        let s = (rounded % 60) as i32;
        (d, m, s, 0.0_f64)
    } else {
        let d = total_sec as i64 / 3600;
        let rem = total_sec - d as f64 * 3600.0;
        let m_i = rem as i64 / 60;
        let s_f = rem - m_i as f64 * 60.0;
        let s = s_f.floor() as i32;
        (d as i32, m_i as i32, s, s_f - s as f64)
    };
    if round_flag & SPLIT_DEG_ZODIACAL != 0 {
        let sign_num = d / 30; // 0-indexed: 0=Aries, 1=Taurus, ..., 4=Leo
        let d_in_sign = d % 30;
        (d_in_sign, m, s, frac, sign_num)
    } else {
        (d, m, s, frac, sign)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic xorshift PRNG for property fuzz inside unit tests.
    struct Rng(u64);
    impl Rng {
        fn new(seed: u64) -> Self {
            Self(seed | 1)
        }
        fn next_u64(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn range(&mut self, lo: f64, hi: f64) -> f64 {
            let u = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
            lo + u * (hi - lo)
        }
    }

    /// `wrap_signed_180` is idempotent: applying twice equals applying once
    /// (output already in (-180, 180]).
    #[test]
    fn wrap_signed_180_idempotent_property() {
        let mut rng = Rng::new(0xCAFEBABE);
        for _ in 0..2048 {
            let x = rng.range(-3600.0, 3600.0);
            let once = wrap_signed_180(x);
            let twice = wrap_signed_180(once);
            assert!(
                (twice - once).abs() < 1e-12,
                "x={x} once={once} twice={twice}"
            );
        }
    }

    /// `wrap_signed_180` output is in (-180, 180].
    #[test]
    fn wrap_signed_180_range_property() {
        let mut rng = Rng::new(0xDEADBEEF);
        for _ in 0..4096 {
            let x = rng.range(-1e6, 1e6);
            let w = wrap_signed_180(x);
            assert!(w > -180.0 && w <= 180.0, "x={x} w={w}");
        }
    }

    /// `wrap_signed_180(x + 360k)` == `wrap_signed_180(x)` for all integer k.
    /// Mod-360 invariance.
    #[test]
    fn wrap_signed_180_mod_360_invariant() {
        let mut rng = Rng::new(0xF00DCAFE);
        for _ in 0..1024 {
            let x = rng.range(-180.0, 180.0);
            for k in [-5_i32, -3, -1, 1, 2, 4, 10] {
                let shifted = (k as f64).mul_add(360.0, x);
                let w = wrap_signed_180(shifted);
                let direct = wrap_signed_180(x);
                assert!(
                    (w - direct).abs() < 1e-9,
                    "x={x} k={k} w={w} direct={direct}"
                );
            }
        }
    }

    /// `diff_deg_signed` reproduces `wrap_signed_180` on already-normalised inputs.
    #[test]
    fn diff_deg_signed_consistency() {
        let mut rng = Rng::new(0xBADC0DE);
        for _ in 0..1024 {
            let p1 = rng.range(0.0, 360.0);
            let p2 = rng.range(0.0, 360.0);
            let direct = diff_deg_signed(p1, p2);
            let expect = wrap_signed_180(p1 - p2);
            assert!((direct - expect).abs() < 1e-9, "p1={p1} p2={p2}");
        }
    }

    #[test]
    fn azalt_regressions_are_exact() {
        let jd = JulianDay::new(2_451_545.0);
        let lst = crate::astronomy::houses::sidereal_time_deg(jd);
        let ecliptic = azalt(jd, 0, [12.1, 49.0, 330.0], 1010.0, 15.0, [120.0, 5.0, 1.0]);
        assert_eq!(
            ecliptic,
            AzAlt {
                azimuth: 169.86785954068296,
                true_alt: -15.342945848546734,
                apparent_alt: -15.342945848546734,
            }
        );

        let equatorial = azalt(
            jd,
            1,
            [0.0, 30.0, 0.0],
            850.0,
            -5.0,
            [(lst - 30.0).rem_euclid(360.0), 20.0, 1.0],
        );
        assert_eq!(
            equatorial,
            AzAlt {
                azimuth: 76.74230673519315,
                true_alt: 61.137368271806594,
                apparent_alt: 61.145657010028806,
            }
        );
    }

    #[test]
    fn azalt_branch_boundaries_are_exact() {
        let jd = JulianDay::new(2_451_545.0);
        let lst = crate::astronomy::houses::sidereal_time_deg(jd);
        assert_eq!(
            azalt(jd, 1, [0.0, 0.0, 0.0], 1010.0, 15.0, [lst, 0.0, 1.0]),
            AzAlt {
                azimuth: 270.0,
                true_alt: 90.0,
                apparent_alt: 89.99997742301815,
            }
        );
        assert_eq!(apparent_altitude(-5.0, 1010.0, 15.0), -5.0);
        assert_ne!(apparent_altitude(-4.999, 1010.0, 15.0), -4.999);
    }

    #[test]
    fn azalt_reverse_regressions_are_exact() {
        let jd = JulianDay::new(2_451_545.0);
        assert_eq!(south_to_north_azimuth(0.0), 180.0);
        assert_eq!(south_to_north_azimuth(180.0), 0.0);
        assert_eq!(hour_angle_from_azimuth(-1.0, 30.0), 30.0);
        assert_eq!(hour_angle_from_azimuth(0.0, 30.0), 330.0);
        assert_eq!(hour_angle_from_azimuth(1.0, 30.0), 330.0);
        assert_eq!(
            azalt_rev(jd, 1, [12.1, 49.0, 330.0], [30.0, 20.0]),
            [263.2952821649003, -16.008111300982545, 1.0]
        );
        assert_eq!(
            azalt_rev(jd, 1, [12.1, 49.0, 330.0], [300.0, -10.0]),
            [5.744614658115211, -27.00703852286642, 1.0]
        );
        assert_eq!(
            azalt_rev(jd, 0, [12.1, 49.0, 330.0], [30.0, 20.0]),
            [263.50382882092117, 7.27850978034093, 1.0]
        );
    }

    #[test]
    fn azalt_reverse_preserves_azimuth_neighbor() {
        let jd = JulianDay::new(2_451_545.0);
        let below_south = f64::from_bits(180.0_f64.to_bits() - 1);
        assert_eq!(
            azalt_rev(jd, 1, [12.1, 49.0, 330.0], [below_south, 20.0]),
            [112.55706881489692, 61.0, 1.0]
        );
    }

    #[test]
    fn refraction_contracts_are_exact() {
        assert_eq!(refrac(1.0, 1013.25, 15.0, 0), 1.407722383503609);
        assert_eq!(refrac(1.0, 800.0, -10.0, 1), 0.647487380950574);
        assert_eq!(
            refrac_extended(1.0, 50.0, 1013.25, 15.0, 0.0065, 0),
            (1.407722383503609, [1.407722383503609, 1.0, 0.0, 0.0])
        );
    }

    #[test]
    fn coordinate_transforms_are_exact() {
        assert_eq!(
            coord_transform([121.34, 43.57, 2.5], Degrees::new(23.4393)),
            [114.113_119_067_227_97, 22.719_051_586_665_874, 2.5]
        );
        assert_eq!(
            coord_transform([210.25, -12.75, 0.75], Degrees::new(-23.4393)),
            [203.309_477_198_617_6, -23.449_125_374_957_653, 0.75]
        );
        assert_eq!(
            coord_transform_with_speed(
                [121.34, 43.57, 2.5, -0.2, 0.03, 0.004],
                Degrees::new(23.4393)
            ),
            [
                114.113_119_067_227_97,
                22.719_051_586_665_874,
                2.5,
                -0.2,
                0.03,
                0.004,
            ]
        );
    }

    #[test]
    fn signed_angle_boundaries_are_exact() {
        let pi = std::f64::consts::PI;
        assert_eq!(norm_deg(-1.0), 359.0);
        assert_eq!(norm_rad(-0.25), std::f64::consts::TAU - 0.25);
        assert_eq!(midpoint_deg(100.0, 20.0), 60.0);
        assert_eq!(midpoint_deg(10.0, 350.0), 0.0);
        assert_eq!(midpoint_rad(1.0, 0.2), 0.600_000_000_000_000_1);
        assert_eq!(wrap_signed_180(0.0), 0.0);
        assert_eq!(wrap_signed_180(180.0), 180.0);
        assert_eq!(wrap_signed_180(-180.0), 180.0);
        assert_eq!(wrap_signed_180(200.0), -160.0);
        assert_eq!(diff_deg(10.0, 350.0), 20.0);
        assert_eq!(diff_deg(350.0, 10.0), 340.0);
        assert_eq!(diff_rad_signed(0.0, pi), pi);
        assert_eq!(diff_rad_signed(pi, 0.0), pi);
        assert_eq!(diff_rad_signed(0.25, 6.0), 0.5331853071795862);
        assert_eq!(diff_rad_signed(6.0, 0.25), -0.5331853071795862);
    }

    #[test]
    fn centisecond_boundaries_are_exact() {
        let half = 180 * 360_000;
        assert_eq!(diff_cs_signed(0, half), half as i64);
        assert_eq!(diff_cs_signed(half, 0), half as i64);
        assert_eq!(diff_cs_signed(100, 200), -100);
        assert_eq!(diff_cs_signed(200, 100), 100);
        assert_eq!(diff_cs_signed(-1, 0), -1);
        assert_eq!(diff_cs(100, 200), 129_599_900);
        assert_eq!(diff_cs(200, 100), 100);
        assert_eq!(norm_cs(-1), 129_599_999);
        assert_eq!(norm_cs(129_600_000), 0);
        assert_eq!(cs_round_sec(149), 100);
        assert_eq!(cs_round_sec(150), 200);
        assert_eq!(cs_round_sec(-149), -100);
        assert_eq!(cs_round_sec(-150), -100);
        assert_eq!(deg_to_cs(12.75), 12);
        assert_eq!(deg_to_cs(-12.25), -13);
    }

    #[test]
    fn split_degree_contracts_are_exact() {
        assert_eq!(split_deg(0.0, 0), (0, 0, 0, 0.0, 1));
        assert_eq!(
            split_deg(123.123, 0),
            (123, 7, 22, 0.799_999_999_988_358_5, 1)
        );
        assert_eq!(split_deg(-10.5, 0), (10, 30, 0, 0.0, -1));
        assert_eq!(
            split_deg(123.123, crate::constants::SPLIT_DEG_ROUND_SEC),
            (123, 7, 23, 0.0, 1)
        );
        assert_eq!(
            split_deg(123.123, crate::constants::SPLIT_DEG_ZODIACAL),
            (3, 7, 22, 0.799_999_999_988_358_5, 4)
        );
    }
}
