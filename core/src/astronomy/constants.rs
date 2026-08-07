#![allow(dead_code)]
//! Fundamental astronomical constants.

use std::f64::consts::PI;

/// Two π.
pub const TWO_PI: f64 = 2.0 * PI;

/// Arcseconds per radian.
pub const ARCSEC_PER_RAD: f64 = 206_264.806_247_09;

/// Degrees per radian.
pub const DEG_PER_RAD: f64 = 180.0 / PI;

/// Radians per degree.
pub const RAD_PER_DEG: f64 = PI / 180.0;

/// Radians per arcsecond.
pub const RAD_PER_ARCSEC: f64 = PI / (180.0 * 3600.0);

/// Julian date of J2000.0.
pub const J2000: f64 = 2_451_545.0;

/// Days per Julian century.
pub const DAYS_PER_CENTURY: f64 = 36_525.0;

/// Days per Julian millennium.
pub const DAYS_PER_MILLENNIUM: f64 = 365_250.0;

/// Speed of light (AU/day).
pub const LIGHT_SPEED_AU_DAY: f64 = 173.144_632_674_240;

/// Astronomical unit in km.
pub const AU_KM: f64 = 149_597_870.7;

/// Earth's equatorial radius (km).
pub const EARTH_RADIUS_KM: f64 = 6378.137;

/// Solar parallax (arcseconds).
pub const SOLAR_PARALLAX: f64 = 8.794_148;

/// Earth's obliquity at J2000.0 (degrees).
pub const OBLIQUITY_J2000: f64 = 23.439_291_111;

/// Convert degrees to radians.
#[inline]
pub(crate) fn to_rad(deg: f64) -> f64 {
    deg * RAD_PER_DEG
}

/// Convert radians to degrees.
#[inline]
#[must_use]
pub fn to_deg(rad: f64) -> f64 {
    rad * DEG_PER_RAD
}

/// Normalise an angle in radians to [0, 2π).
#[inline]
#[must_use]
pub fn norm_rad(r: f64) -> f64 {
    r.rem_euclid(TWO_PI)
}

/// Normalise an angle in degrees to [0, 360).
#[inline]
#[must_use]
pub fn norm_deg(d: f64) -> f64 {
    d.rem_euclid(360.0)
}

/// Julian centuries from J2000.0.
#[inline]
#[must_use]
pub fn julian_centuries(jde: f64) -> f64 {
    (jde - J2000) / DAYS_PER_CENTURY
}

/// Julian millennia from J2000.0.
#[inline]
#[must_use]
pub fn julian_millennia(jde: f64) -> f64 {
    (jde - J2000) / DAYS_PER_MILLENNIUM
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radians_per_arcsecond_matches_reference() {
        assert!((RAD_PER_ARCSEC - 4.848_136_811_095_36e-6).abs() < 1e-18);
    }
}
