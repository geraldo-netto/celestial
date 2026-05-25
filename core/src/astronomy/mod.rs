//! Pure-Rust astronomical engine.
//!
//! Provides planetary positions, house systems, ayanamsa, and rise/set calculations.
//! but implemented entirely in Rust without any external dependencies or data
//! files. It uses:
//!
//! - **VSOP87** (Bretagnon & Francou 1988) — truncated planetary theory
//! - **ELP2000-82** (Chapront & Chapront-Touzé 1983) — truncated lunar theory
//! - **IAU 1980** nutation series (106 terms, Wahr 1981)
//! - **Laskar 1986** obliquity formula
//! - **Espenak & Meeus 2006** ΔT polynomials
//! - All major house systems (pure trigonometry)
//! - 36 ayanamsa modes (Lahiri, Fagan-Bradley, Raman, …)
//! - Rise/set/transit via the Meeus iterative algorithm
//!
//! ## Accuracy
//!
//! | Body        | Longitude     | Latitude      | Distance   |
//! |-------------|---------------|---------------|------------|
//! | Sun         | ~1"           | ~0.1"         | ~0.00001AU |
//! | Moon        | ~10"          | ~4"           | ~4 km      |
//! | Inner planets| ~1"–30"      | ~1"–10"       | ~0.001 AU  |
//! | Outer planets| ~1"–60"      | ~1"–30"       | ~0.01 AU   |
//!
//! Accuracy degrades beyond ±3000 years from J2000.
//!
//! ## Planet body numbers
//!
//! ```text
//! 0 = Sun      4 = Jupiter   8 = Moon
//! 1 = Mercury  5 = Saturn    9 = True Node
//! 2 = Venus    6 = Uranus   10 = Mean Node
//! 3 = Mars     7 = Neptune  11 = Chiron
//! ```

pub mod ayanamsa;
pub mod constants;
pub mod delta_t;
pub mod engine;
pub mod houses;
pub mod moon;
pub mod nutation;
pub mod planetary;
pub(crate) mod pluto;
pub mod rise_set;
pub mod vsop87;

use crate::astronomy::{
    ayanamsa::{ayanamsa as calc_ayanamsa, ayanamsa_name, SidMode},
    houses::{houses as calc_houses, houses_armc as calc_houses_armc, HouseResult, HouseSystem},
    nutation::{mean_obliquity, nutation, true_obliquity},
    rise_set::{moon_rise_set, planet_rise_set, sun_rise_set, RiseSetEvent},
    vsop87::Planet,
};
use crate::units::{Degrees, JulianDay, Latitude, Longitude};

// ─── Public body-number constants ─────────────────────────────────────────────

/// Swiss Ephemeris body numbers, for use with [`calc_ut`] etc.
pub mod body {
    pub const SUN: i32 = 0;
    pub const MOON: i32 = 1;
    pub const MERCURY: i32 = 2;
    pub const VENUS: i32 = 3;
    pub const MARS: i32 = 4;
    pub const JUPITER: i32 = 5;
    pub const SATURN: i32 = 6;
    pub const URANUS: i32 = 7;
    pub const NEPTUNE: i32 = 8;
    pub const PLUTO: i32 = 9;
    pub const MEAN_NODE: i32 = 10;
    pub const TRUE_NODE: i32 = 11;
    pub const CHIRON: i32 = 15;
}

/// Return flags (subset matching Swiss Ephemeris).
pub mod flag {
    #![allow(dead_code)]
    pub const FLG_BUILTIN: u32 = 2;
    pub const FLG_JPL: u32 = 1;
    pub const FLG_MOSHIER: u32 = 4;
    pub const FLG_SPEED: u32 = 256;
    pub const FLG_EQUATORIAL: u32 = 2048;
    pub const FLG_HELCTR: u32 = 8;
    pub const FLG_NONUT: u32 = 512;
    pub const FLG_SIDEREAL: u32 = 65536;
    pub const FLG_RADIANS: u32 = 1024;
    pub const FLG_XYZ: u32 = 16;
}

// ─── Planetary position — delegates to engine ───────────────────────────────────

pub use engine::calc_tt;
/// Compute apparent geocentric position using UT.
pub use engine::calc_ut;
pub use houses::houses_from_armc;

// ─── House calculations// ─── House calculations ───────────────────────────────────────────────────────

/// Compute house cusps and special angles.
///
/// - `jd_ut` — Julian day (Universal Time)
/// - `geolat` — geographic latitude (degrees, N positive)
/// - `geolon` — geographic longitude (degrees, E positive)
/// - `hsys`   — house system byte: `b'P'` Placidus, `b'K'` Koch, `b'E'` Equal, etc.
pub fn houses(jd_ut: JulianDay, geolat: Latitude, geolon: Longitude, hsys: u8) -> HouseResult {
    let jd_ut: f64 = jd_ut.into();
    let geolat: f64 = geolat.into();
    let geolon: f64 = geolon.into();
    calc_houses(JulianDay::new(jd_ut), Latitude::new(geolat), Longitude::new(geolon), hsys)
}

/// Compute house cusps from ARMC, latitude and obliquity directly.
pub fn houses_armc(armc: Degrees, geolat: Latitude, eps: Degrees, hsys: u8) -> HouseResult {
    let armc: f64 = armc.into();
    let geolat: f64 = geolat.into();
    let eps: f64 = eps.into();
    calc_houses_armc(Degrees::new(armc), Latitude::new(geolat), Degrees::new(eps), hsys)
}

/// Return the display name for a house system byte.
#[must_use]
pub fn house_name(hsys: u8) -> &'static str {
    HouseSystem::from_char(hsys).map_or("Unknown", houses::HouseSystem::name)
}

// ─── Ayanamsa ─────────────────────────────────────────────────────────────────

/// Compute ayanamsa (degrees) for a given Julian Ephemeris Day and sidereal mode.
///
/// `sid_mode` corresponds to the `SIDM_*` constants (0 = Fagan-Bradley, 1 = Lahiri, …).
#[must_use]
pub fn get_ayanamsa(jde: f64, sid_mode: i32) -> f64 {
    let mode = SidMode::from_i32(sid_mode).unwrap_or(SidMode::Lahiri);
    calc_ayanamsa(jde, mode)
}

/// Return the name of a sidereal mode.
#[must_use]
pub fn get_ayanamsa_name(sid_mode: i32) -> &'static str {
    ayanamsa_name(sid_mode)
}

// ─── Delta T / time ───────────────────────────────────────────────────────────

/// Compute ΔT = TT − UT1 (seconds) for a given UT Julian day.
#[must_use]
pub fn deltat(jd_ut: JulianDay) -> f64 {
    let jd_ut: f64 = jd_ut.into();
    delta_t::delta_t(JulianDay::new(jd_ut))
}

/// Compute mean obliquity of the ecliptic (degrees) for a JDE (TT).
#[must_use]
pub fn obliquity(jde: f64) -> f64 {
    mean_obliquity(jde)
}

/// Compute true obliquity of the ecliptic, including nutation in obliquity.
#[must_use]
pub fn obliquity_true(jde: f64) -> f64 {
    true_obliquity(jde)
}

/// Compute nutation in longitude and obliquity (arcseconds each).
#[must_use]
pub fn get_nutation(jde: f64) -> (f64, f64) {
    let n = nutation(jde);
    (n.dpsi, n.deps)
}

// ─── Rise / set / transit ────────────────────────────────────────────────────

/// Compute rise, transit or set time (UT Julian day) for the Sun.
///
/// - `event` — 0 = rise, 1 = transit, 2 = set
pub fn sun_rise_transit_set(jd_ut: JulianDay, geolat: Latitude, geolon: Longitude, event: u8) -> Option<f64> {
    let jd_ut: f64 = jd_ut.into();
    let geolat: f64 = geolat.into();
    let geolon: f64 = geolon.into();
    let ev = match event {
        0 => RiseSetEvent::Rise,
        1 => RiseSetEvent::Transit,
        2 => RiseSetEvent::Set,
        _ => return None,
    };
    let r = sun_rise_set(JulianDay::new(jd_ut), Latitude::new(geolat), Longitude::new(geolon), ev);
    r.found.then_some(r.jd_ut)
}

/// Compute rise, transit or set time (UT Julian day) for the Moon.
pub fn moon_rise_transit_set(jd_ut: JulianDay, geolat: Latitude, geolon: Longitude, event: u8) -> Option<f64> {
    let jd_ut: f64 = jd_ut.into();
    let geolat: f64 = geolat.into();
    let geolon: f64 = geolon.into();
    let ev = match event {
        0 => RiseSetEvent::Rise,
        1 => RiseSetEvent::Transit,
        2 => RiseSetEvent::Set,
        _ => return None,
    };
    let r = moon_rise_set(JulianDay::new(jd_ut), Latitude::new(geolat), Longitude::new(geolon), ev);
    r.found.then_some(r.jd_ut)
}

/// Compute rise, transit or set time (UT Julian day) for a planet.
///
/// `body_num` follows the [`body`] constants.
pub fn planet_rise_transit_set(
    jd_ut: JulianDay,
    geolat: Latitude,
    geolon: Longitude,
    body_num: i32,
    event: u8,
) -> Option<f64> {
    let jd_ut: f64 = jd_ut.into();
    let geolat: f64 = geolat.into();
    let geolon: f64 = geolon.into();
    let planet = body_to_vsop87(body_num)?;
    let ev = match event {
        0 => RiseSetEvent::Rise,
        1 => RiseSetEvent::Transit,
        2 => RiseSetEvent::Set,
        _ => return None,
    };
    let r = planet_rise_set(JulianDay::new(jd_ut), Latitude::new(geolat), Longitude::new(geolon), ev, planet);
    r.found.then_some(r.jd_ut)
}

/// Convert a body number to a `Planet` enum value, returning `None` for non-planet bodies.
fn body_to_vsop87(body_num: i32) -> Option<Planet> {
    Some(match body_num {
        body::MERCURY => Planet::Mercury,
        body::VENUS => Planet::Venus,
        body::MARS => Planet::Mars,
        body::JUPITER => Planet::Jupiter,
        body::SATURN => Planet::Saturn,
        body::URANUS => Planet::Uranus,
        body::NEPTUNE => Planet::Neptune,
        _ => return None,
    })
}

// ─── Planet names ─────────────────────────────────────────────────────────────

/// Return the canonical name for a body number.
#[must_use]
pub fn planet_name(body_num: i32) -> &'static str {
    match body_num {
        body::SUN => "Sun",
        body::MOON => "Moon",
        body::MERCURY => "Mercury",
        body::VENUS => "Venus",
        body::MARS => "Mars",
        body::JUPITER => "Jupiter",
        body::SATURN => "Saturn",
        body::URANUS => "Uranus",
        body::NEPTUNE => "Neptune",
        body::PLUTO => "Pluto",
        _ => "Unknown",
    }
}

/// Return the engine version string.
#[must_use]
pub fn engine_version() -> &'static str {
    "2.10.03"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference: calc_ut(2452275.5, SUN, FLG_BUILTIN|FLG_SPEED)
    /// lon ≈ 280.38°
    #[test]
    fn sun_lon_2002_jan_1() {
        let pos = calc_ut(
            JulianDay::new(2_452_275.5),
            body::SUN,
            (flag::FLG_BUILTIN | flag::FLG_SPEED) as i32,
        )
        .expect("calc_ut failed");
        assert!((pos.lon - 280.38).abs() < 0.1, "Sun lon = {}", pos.lon);
        assert!(pos.ret_flags as u32 & flag::FLG_BUILTIN != 0);
    }

    #[test]
    fn moon_position_j2000() {
        let pos = calc_ut(JulianDay::new(2_451_545.0), body::MOON, flag::FLG_BUILTIN as i32).expect("Moon failed");
        assert!(pos.lon >= 0.0 && pos.lon < 360.0);
        assert!(pos.dist > 0.002 && pos.dist < 0.003); // Moon ~0.00257 AU
    }

    #[test]
    fn all_planets_j2000() {
        let bodies = [
            body::SUN,
            body::MERCURY,
            body::VENUS,
            body::MARS,
            body::JUPITER,
            body::SATURN,
            body::URANUS,
            body::NEPTUNE,
            body::MOON,
        ];
        for &b in &bodies {
            let pos = calc_ut(JulianDay::new(2_451_545.0), b, flag::FLG_BUILTIN as i32).expect("calc_ut");
            assert!(
                pos.lon >= 0.0 && pos.lon < 360.0,
                "body {b} lon out of range"
            );
            assert!(pos.dist > 0.0, "body {b} dist <= 0");
        }
    }

    #[test]
    fn speed_flag_gives_nonzero_speeds() {
        let pos = calc_ut(
            JulianDay::new(2_451_545.0),
            body::SUN,
            (flag::FLG_BUILTIN | flag::FLG_SPEED) as i32,
        )
        .expect("calc_ut");
        assert!(pos.speed_lon.abs() > 0.0, "Sun speed_lon = 0");
        // Sun moves ~1°/day
        assert!(
            pos.speed_lon.abs() < 1.1 && pos.speed_lon.abs() > 0.9,
            "Sun speed_lon unreasonable: {}",
            pos.speed_lon
        );
    }

    #[test]
    fn unknown_body_returns_error() {
        assert!(calc_ut(JulianDay::new(2_451_545.0), 99, flag::FLG_BUILTIN as i32).is_err());
    }

    #[test]
    fn house_system_names() {
        assert_eq!(house_name(b'P'), "Placidus");
        assert_eq!(house_name(b'K'), "Koch");
        assert_eq!(house_name(b'Z'), "Unknown");
    }

    #[test]
    fn ayanamsa_lahiri_reasonable() {
        let ay = get_ayanamsa(2_451_545.0, 1);
        assert!((ay - 23.85).abs() < 0.1, "Lahiri ayanamsa = {ay}");
    }

    #[test]
    fn deltat_j2000() {
        let dt = deltat(JulianDay::new(2_451_545.0));
        assert!((dt - 63.8).abs() < 3.0, "ΔT = {dt}");
    }
}

// ─── New sub-modules ──────────────────────────────────────────────────────────
pub mod chiron;
pub mod crossings;
pub mod eclipses;
pub mod fixstars;
pub mod heliacal;
pub mod nodes;
pub mod phenomena;
pub mod solar_cycle;
