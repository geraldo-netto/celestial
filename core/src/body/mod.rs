//! Celestial body identifiers and calculation flags.
//!
//! Instead of passing raw `i32` constants, use [`Body`] and [`CalcFlags`]:
//!
//! ```
//! use celestial_core::body::{Body, CalcFlags};
//! use celestial_core::position::calc_ut;
//!
//! // Old style (still works via prelude):
//! // calc_ut(jd, 0, 2 | 256)
//!
//! // New style — type-safe, IDE-friendly:
//! // calc_ut(jd, Body::SUN, CalcFlags::BUILTIN | CalcFlags::SPEED)
//! ```

use std::ops::{BitAnd, BitOr, BitOrAssign, Not};

// ── Body ──────────────────────────────────────────────────────────────────────

/// A celestial body or point, identified by number.
///
/// Associated constants cover all bodies supported by the engine.
/// Use [`Body::from_raw`] or `Body(n)` for asteroid / custom indices.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
/// A celestial body or point identified by its index.
///
/// Use the associated constants (`Body::SUN`, `Body::MOON`, …)
/// or `Body(n)` for custom asteroid indices.
pub struct Body(pub i32);

impl Body {
    // ── Main planets & luminaries ─────────────────────────────────────────
    pub const SUN: Self = Body(0);
    pub const MOON: Self = Body(1);
    pub const MERCURY: Self = Body(2);
    pub const VENUS: Self = Body(3);
    pub const MARS: Self = Body(4);
    pub const JUPITER: Self = Body(5);
    pub const SATURN: Self = Body(6);
    pub const URANUS: Self = Body(7);
    pub const NEPTUNE: Self = Body(8);
    pub const PLUTO: Self = Body(9);

    // ── Lunar nodes & apsides ─────────────────────────────────────────────
    /// Mean lunar node (Rahu/Ketu, mean position).
    pub const MEAN_NODE: Self = Body(10);
    /// True (osculating) lunar node.
    pub const TRUE_NODE: Self = Body(11);
    pub const MEAN_APOGEE: Self = Body(12);
    pub const OSCULATING_APOGEE: Self = Body(13);
    /// Earth — for heliocentric calculations.
    pub const EARTH: Self = Body(14);

    // ── Asteroids (also available at ASTEROID_OFFSET + n) ────────────────
    /// Chiron.
    pub const CHIRON: Self = Body(15);
    pub const PHOLUS: Self = Body(16);
    /// Ceres.
    pub const CERES: Self = Body(17);
    pub const PALLAS: Self = Body(18);
    /// Juno.
    pub const JUNO: Self = Body(19);
    pub const VESTA: Self = Body(20);

    // ── Special markers ───────────────────────────────────────────────────
    /// Sentinel value for ecliptic/nutation-only calculations.
    pub const ECL_NUT: Self = Body(-1);
    /// Sentinel value indicating a fixed star rather than a solar system body.
    pub const FIXED_STAR: Self = Body(-10);

    // ── Offsets for extended catalogs ─────────────────────────────────────
    /// Base index for numbered asteroids (add the asteroid number).
    pub const ASTEROID_OFFSET: i32 = 10_000;
    /// Base index for Uranian / Hamburg fictitious planets.
    pub const FICTITIOUS_OFFSET: i32 = 40;
    /// Base index for planetary moons.
    pub const MOON_OFFSET: i32 = 9_000;

    /// Construct a `Body` from a raw integer (asteroid / custom use).
    #[inline]
    pub const fn from_raw(n: i32) -> Self {
        Body(n)
    }

    /// Return the raw integer representation.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }

    /// Returns `true` if this body is one of the nine classical planets or Moon.
    pub fn is_planet(self) -> bool {
        (0..=9).contains(&self.0)
    }

    /// Returns `true` if this is a lunar node or apside.
    pub fn is_node(self) -> bool {
        matches!(self.0, 10..=13)
    }

    /// Human-readable name for well-known bodies; `"Body(N)"` otherwise.
    pub fn name(self) -> &'static str {
        /// Display names for built-in body codes 0–20. Contiguous array so
        /// `Body(n)` looks up in O(1) via indexing; unknown values fall back
        /// to `"Unknown"` below.
        const BODY_NAMES: [&str; 21] = [
            "Sun",         // 0
            "Moon",        // 1
            "Mercury",     // 2
            "Venus",       // 3
            "Mars",        // 4
            "Jupiter",     // 5
            "Saturn",      // 6
            "Uranus",      // 7
            "Neptune",     // 8
            "Pluto",       // 9
            "Mean Node",   // 10
            "True Node",   // 11
            "Mean Apogee", // 12
            "Osc. Apogee", // 13
            "Earth",       // 14
            "Chiron",      // 15
            "Pholus",      // 16
            "Ceres",       // 17
            "Pallas",      // 18
            "Juno",        // 19
            "Vesta",       // 20
        ];
        BODY_NAMES
            .get(self.0 as usize)
            .copied()
            .unwrap_or("Unknown")
    }
}

impl From<i32> for Body {
    fn from(n: i32) -> Self {
        Body(n)
    }
}

impl From<Body> for i32 {
    fn from(b: Body) -> Self {
        b.0
    }
}

impl std::fmt::Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ── CalcFlags ─────────────────────────────────────────────────────────────────

/// Bitmask flags controlling how [`position::calc_ut`] computes a position.
///
/// Combine with `|`:
/// ```
/// use celestial_core::body::CalcFlags;
/// let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CalcFlags(pub i32);

impl CalcFlags {
    // ── Ephemeris source (only BUILTIN is fully implemented) ──────────────
    /// Use the built-in VSOP87/ELP2000 engine (default; always works).
    pub const BUILTIN: Self = CalcFlags(2);
    /// Use JPL ephemeris data (falls back to BUILTIN if no file is set).
    pub const JPL: Self = CalcFlags(1);
    /// Use Moshier approximation (falls back to BUILTIN).
    pub const MOSHIER: Self = CalcFlags(4);

    // ── Reference frame ───────────────────────────────────────────────────
    /// Heliocentric coordinates instead of geocentric.
    pub const HELIOCENTRIC: Self = CalcFlags(8);
    /// J2000 coordinates (skip precession to current epoch).
    pub const J2000: Self = CalcFlags(32);
    /// Equatorial (RA/Dec) instead of ecliptic.
    pub const EQUATORIAL: Self = CalcFlags(2048);
    /// Barycentric instead of geocentric.
    pub const BARYCENTRIC: Self = CalcFlags(16384);
    /// Topocentric (requires `set_topo` to be called first).
    pub const TOPOCENTRIC: Self = CalcFlags(32768);
    /// ICRS reference frame.
    pub const ICRS: Self = CalcFlags(131072);

    // ── Corrections ───────────────────────────────────────────────────────
    /// Suppress gravitational deflection.
    pub const NO_DEFLECTION: Self = CalcFlags(512);
    /// Suppress aberration correction.
    pub const NO_ABERRATION: Self = CalcFlags(1024);
    /// Astrometric coordinates (no deflection, no aberration).
    pub const ASTROMETRIC: Self = CalcFlags(1024 | 512);
    /// Suppress nutation correction.
    pub const NO_NUTATION: Self = CalcFlags(64);

    // ── Output options ────────────────────────────────────────────────────
    /// Include daily velocity in output (populates `speed_*` fields).
    pub const SPEED: Self = CalcFlags(256);
    /// Include velocity via numerical differentiation (slower, same result).
    pub const SPEED3: Self = CalcFlags(128);
    /// Apply sidereal (ayanamsa) correction — requires `set_sid_mode` first.
    pub const SIDEREAL: Self = CalcFlags(65536);

    // ── Convenience combinations ──────────────────────────────────────────
    /// Most common usage: built-in engine with velocity.
    pub const DEFAULT: Self = CalcFlags(2 | 256); // BUILTIN | SPEED

    /// Return the raw `i32` bit-pattern.
    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }

    /// Returns `true` if the sidereal flag is set.
    #[inline]
    pub fn is_sidereal(self) -> bool {
        self.0 & Self::SIDEREAL.0 != 0
    }

    /// Returns `true` if the speed flag is set.
    #[inline]
    pub fn is_speed(self) -> bool {
        self.0 & Self::SPEED.0 != 0
    }
}

impl BitOr for CalcFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        CalcFlags(self.0 | rhs.0)
    }
}

impl BitOrAssign for CalcFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for CalcFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        CalcFlags(self.0 & rhs.0)
    }
}

impl Not for CalcFlags {
    type Output = Self;
    fn not(self) -> Self {
        CalcFlags(!self.0)
    }
}

impl From<i32> for CalcFlags {
    fn from(n: i32) -> Self {
        CalcFlags(n)
    }
}

impl From<CalcFlags> for i32 {
    fn from(f: CalcFlags) -> Self {
        f.0
    }
}

impl std::fmt::Display for CalcFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CalcFlags(0x{:x})", self.0)
    }
}

// ── House system ──────────────────────────────────────────────────────────────

/// A house system identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HouseSystem(pub u8);

impl HouseSystem {
    pub const PLACIDUS: Self = HouseSystem(b'P');
    pub const KOCH: Self = HouseSystem(b'K');
    pub const EQUAL: Self = HouseSystem(b'E');
    pub const WHOLE_SIGN: Self = HouseSystem(b'W');
    pub const PORPHYRY: Self = HouseSystem(b'O');
    pub const REGIOMONTANUS: Self = HouseSystem(b'R');
    pub const CAMPANUS: Self = HouseSystem(b'C');
    pub const MORINUS: Self = HouseSystem(b'M');
    pub const ALCABITUS: Self = HouseSystem(b'B');
    pub const AXIAL_ROTATION: Self = HouseSystem(b'X');
    pub const GAUQUELIN: Self = HouseSystem(b'G');
    pub const VEHLOW_EQUAL: Self = HouseSystem(b'V');
    pub const WHOLE_SIGN_MERIDIAN: Self = HouseSystem(b'Y');

    #[inline]
    pub const fn as_raw(self) -> u8 {
        self.0
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            b'P' => "Placidus",
            b'K' => "Koch",
            b'E' => "Equal",
            b'W' => "Whole-Sign",
            b'O' => "Porphyry",
            b'R' => "Regiomontanus",
            b'C' => "Campanus",
            b'M' => "Morinus",
            b'B' => "Alcabitus",
            b'X' => "Axial Rotation",
            b'G' => "Gauquelin",
            b'V' => "Vehlow Equal",
            b'Y' => "Whole-Sign Meridian",
            _ => "Unknown",
        }
    }
}

impl From<u8> for HouseSystem {
    fn from(b: u8) -> Self {
        HouseSystem(b)
    }
}

impl From<char> for HouseSystem {
    fn from(c: char) -> Self {
        HouseSystem(c as u8)
    }
}

impl std::fmt::Display for HouseSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ── Sidereal mode ─────────────────────────────────────────────────────────────

/// A sidereal (ayanamsa) mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SiderealMode(pub i32);

impl SiderealMode {
    pub const FAGAN_BRADLEY: Self = SiderealMode(0);
    pub const LAHIRI: Self = SiderealMode(1);
    pub const DELUCE: Self = SiderealMode(2);
    pub const RAMAN: Self = SiderealMode(3);
    pub const KRISHNAMURTI: Self = SiderealMode(5);
    pub const SASSANIAN: Self = SiderealMode(11);
    /// User-defined ayanamsa (supply epoch and value via `set_sid_mode`).
    pub const USER_DEFINED: Self = SiderealMode(255);

    #[inline]
    pub const fn as_raw(self) -> i32 {
        self.0
    }

    pub fn name(self) -> &'static str {
        crate::functions::config::ayanamsa_name(self.0)
    }
}

impl From<i32> for SiderealMode {
    fn from(n: i32) -> Self {
        SiderealMode(n)
    }
}

impl std::fmt::Display for SiderealMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ── Calendar type ─────────────────────────────────────────────────────────────

/// Which calendar system to use for date conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Calendar {
    /// Julian calendar (default before 1582).
    Julian,
    /// Proleptic Gregorian calendar (default after 1582).
    Gregorian,
}

impl Calendar {
    /// Convert to the raw `i32` used internally (`0` = Julian, `1` = Gregorian).
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Julian => 0,
            Self::Gregorian => 1,
        }
    }
}

impl From<i32> for Calendar {
    fn from(n: i32) -> Self {
        match n {
            0 => Self::Julian,
            _ => Self::Gregorian,
        }
    }
}

impl std::fmt::Display for Calendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Julian => write!(f, "Julian"),
            Self::Gregorian => write!(f, "Gregorian"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_names() {
        assert_eq!(Body::SUN.name(), "Sun");
        assert_eq!(Body::MOON.name(), "Moon");
        assert_eq!(Body::CHIRON.name(), "Chiron");
    }

    #[test]
    fn body_is_planet() {
        assert!(Body::SUN.is_planet());
        assert!(Body::NEPTUNE.is_planet());
        assert!(!Body::CHIRON.is_planet());
        assert!(!Body::MEAN_NODE.is_planet());
    }

    #[test]
    fn calc_flags_bitor() {
        let f = CalcFlags::BUILTIN | CalcFlags::SPEED;
        assert_eq!(f.as_raw(), 2 | 256);
        assert!(f.is_speed());
    }

    #[test]
    fn house_system_names() {
        assert_eq!(HouseSystem::PLACIDUS.name(), "Placidus");
        assert_eq!(HouseSystem::KOCH.name(), "Koch");
    }

    #[test]
    fn sidereal_mode_display() {
        assert_eq!(SiderealMode::LAHIRI.as_raw(), 1);
    }

    #[test]
    fn calendar_raw() {
        assert_eq!(Calendar::Gregorian.as_raw(), 1);
        assert_eq!(Calendar::Julian.as_raw(), 0);
    }

    #[test]
    fn body_from_raw_roundtrip() {
        let b = Body::from_raw(15);
        assert_eq!(b, Body::CHIRON);
        assert_eq!(i32::from(b), 15);
    }
}
