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
pub struct Body(pub i32);

impl Body {
    // ── Main planets & luminaries ─────────────────────────────────────────
    /// The Sun.
    pub const SUN: Self = Body(0);
    /// The Moon.
    pub const MOON: Self = Body(1);
    /// Mercury.
    pub const MERCURY: Self = Body(2);
    /// Venus.
    pub const VENUS: Self = Body(3);
    /// Mars.
    pub const MARS: Self = Body(4);
    /// Jupiter.
    pub const JUPITER: Self = Body(5);
    /// Saturn.
    pub const SATURN: Self = Body(6);
    /// Uranus.
    pub const URANUS: Self = Body(7);
    /// Neptune.
    pub const NEPTUNE: Self = Body(8);
    /// Pluto.
    pub const PLUTO: Self = Body(9);

    // ── Lunar nodes & apsides ─────────────────────────────────────────────
    /// Mean lunar node (Rahu/Ketu, mean position).
    pub const MEAN_NODE: Self = Body(10);
    /// True (osculating) lunar node.
    pub const TRUE_NODE: Self = Body(11);
    /// Mean lunar apogee (mean Black Moon Lilith).
    pub const MEAN_APOGEE: Self = Body(12);
    /// Osculating lunar apogee (true Black Moon Lilith).
    pub const OSCULATING_APOGEE: Self = Body(13);
    /// Earth — for heliocentric calculations.
    pub const EARTH: Self = Body(14);

    // ── Asteroids (also available at ASTEROID_OFFSET + n) ────────────────
    /// Chiron.
    pub const CHIRON: Self = Body(15);
    /// Pholus.
    pub const PHOLUS: Self = Body(16);
    /// Ceres.
    pub const CERES: Self = Body(17);
    /// Pallas.
    pub const PALLAS: Self = Body(18);
    /// Juno.
    pub const JUNO: Self = Body(19);
    /// Vesta.
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
    #[must_use]
    pub const fn as_raw(self) -> i32 {
        self.0
    }

    /// Returns `true` if `n` falls in a documented body-id range.
    ///
    /// Recognised ranges:
    /// * `-10` → fixed-star sentinel ([`FIXED_STAR`](Self::FIXED_STAR))
    /// * `-1`  → ecliptic / nutation pseudo-body ([`ECL_NUT`](Self::ECL_NUT))
    /// * `0..=20` → main planets + luminaries + classical asteroids
    /// * `FICTITIOUS_OFFSET..FICTITIOUS_OFFSET + 100` → Uranian / Hamburg bodies
    /// * `MOON_OFFSET..MOON_OFFSET + 1_000` → planetary moons
    /// * `ASTEROID_OFFSET..ASTEROID_OFFSET + 1_000_000` → numbered asteroids
    ///
    /// Anything else is rejected by [`Body::try_from_raw`] so that FFI / CLI
    /// callers fail fast instead of letting an obscure error surface inside
    /// `calc_ut` (REL-8 defensive-depth guard).
    #[must_use]
    pub const fn is_known_id(n: i32) -> bool {
        if n == -10 || n == -1 {
            return true;
        }
        if n >= 0 && n <= 20 {
            return true;
        }
        if n >= Self::FICTITIOUS_OFFSET && n < Self::FICTITIOUS_OFFSET + 100 {
            return true;
        }
        if n >= Self::MOON_OFFSET && n < Self::ASTEROID_OFFSET + 1_000_000 {
            return true;
        }
        false
    }

    /// Validating ctor — returns [`BodyError::OutOfRange`] for ids that fall
    /// outside every documented range. Bindings call this at the FFI seam.
    ///
    /// `Body::from_raw` remains unchecked for hot paths and `const` use.
    ///
    /// # Errors
    /// `BodyError::OutOfRange { id }` when `n` matches no known range.
    pub const fn try_from_raw(n: i32) -> Result<Self, BodyError> {
        if Self::is_known_id(n) {
            Ok(Body(n))
        } else {
            Err(BodyError::OutOfRange { id: n })
        }
    }

    /// Returns `true` if this body is one of the nine classical planets or Moon.
    #[must_use]
    pub fn is_planet(self) -> bool {
        (0..=9).contains(&self.0)
    }

    /// Approximate mean-motion window (days) for retrograde / sign-ingress
    /// searches. Faster bodies use shorter windows; outer planets need years.
    ///
    /// Used by [`crate::functions::chart::retrograde_station_ut`] and
    /// [`crate::functions::chart::sign_ingress_ut`].
    #[must_use]
    pub fn retrograde_search_window(self) -> f64 {
        match self.0 {
            4 => 800.0,    // Mars
            5 => 1500.0,   // Jupiter
            6 => 2000.0,   // Saturn
            7 => 5000.0,   // Uranus
            8 => 10_000.0, // Neptune
            9 => 30_000.0, // Pluto
            _ => 200.0,
        }
    }

    /// Approximate window (days) for sign-ingress crossing search.
    /// Wider than `retrograde_search_window` to cover multi-sign hops for
    /// outer planets.
    #[must_use]
    pub fn ingress_search_window(self) -> f64 {
        match self.0 {
            0..=3 => 400.0,
            4 => 750.0,
            5 => 4_500.0,
            6 => 11_000.0,
            7 => 31_000.0,
            8 => 61_000.0,
            9 => 95_000.0,
            _ => 800.0,
        }
    }

    /// Orb tier (multiplier) for aspect calculations: luminaries get the
    /// widest orbs, outer planets the tightest.
    /// Used by [`crate::functions::chart::default_orb`].
    #[must_use]
    pub fn orb_weight(self) -> f64 {
        match self.0 {
            0 | 1 => 2.0, // Sun, Moon
            2..=4 => 1.5, // Mercury, Venus, Mars
            5 | 6 => 1.0, // Jupiter, Saturn
            _ => 0.75,    // outer planets, nodes, asteroids
        }
    }

    /// Returns `true` if this is a lunar node or apside.
    #[must_use]
    pub fn is_node(self) -> bool {
        matches!(self.0, 10..=13)
    }

    /// Human-readable name for well-known bodies; `"Body(N)"` otherwise.
    #[must_use]
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

/// Failure modes for [`Body::try_from_raw`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyError {
    /// Raw integer did not match any documented body-id range.
    OutOfRange {
        /// The rejected raw id.
        id: i32,
    },
}

impl std::fmt::Display for BodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyError::OutOfRange { id } => write!(
                f,
                "body id {id} is outside every documented range \
                 (-10, -1, 0..=20, 40..140, 9000..10000, 10000..1010000)"
            ),
        }
    }
}

impl std::error::Error for BodyError {}

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

/// Bitmask flags controlling how [`calc_ut`](crate::calc_ut) computes a position.
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
    #[must_use]
    pub const fn as_raw(self) -> i32 {
        self.0
    }

    /// Returns `true` if the sidereal flag is set.
    #[inline]
    #[must_use]
    pub fn is_sidereal(self) -> bool {
        self.0 & Self::SIDEREAL.0 != 0
    }

    /// Returns `true` if the speed flag is set.
    #[inline]
    #[must_use]
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
    /// Placidus house system.
    pub const PLACIDUS: Self = HouseSystem(b'P');
    /// Koch house system.
    pub const KOCH: Self = HouseSystem(b'K');
    /// Equal house system.
    pub const EQUAL: Self = HouseSystem(b'E');
    /// Whole-sign house system.
    pub const WHOLE_SIGN: Self = HouseSystem(b'W');
    /// Porphyry house system.
    pub const PORPHYRY: Self = HouseSystem(b'O');
    /// Regiomontanus house system.
    pub const REGIOMONTANUS: Self = HouseSystem(b'R');
    /// Campanus house system.
    pub const CAMPANUS: Self = HouseSystem(b'C');
    /// Morinus house system.
    pub const MORINUS: Self = HouseSystem(b'M');
    /// Alcabitus house system.
    pub const ALCABITUS: Self = HouseSystem(b'B');
    /// Axial-rotation (meridian) house system.
    pub const AXIAL_ROTATION: Self = HouseSystem(b'X');
    /// Gauquelin sectors.
    pub const GAUQUELIN: Self = HouseSystem(b'G');
    /// Vehlow equal house system.
    pub const VEHLOW_EQUAL: Self = HouseSystem(b'V');
    /// Whole-sign system anchored on the meridian.
    pub const WHOLE_SIGN_MERIDIAN: Self = HouseSystem(b'Y');

    /// Return the raw house-system code byte.
    #[inline]
    #[must_use]
    pub const fn as_raw(self) -> u8 {
        self.0
    }

    /// Human-readable name of this house system (e.g. `"Placidus"`, `"Koch"`).
    ///
    /// Returns `"Unknown"` for non-standard bytes. For a complete listing
    /// see the `HOUSE_SYSTEMS` table in `functions::geoformat`.
    #[must_use]
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
    /// Fagan-Bradley ayanamsa.
    pub const FAGAN_BRADLEY: Self = SiderealMode(0);
    /// Lahiri (Chitrapaksha) ayanamsa.
    pub const LAHIRI: Self = SiderealMode(1);
    /// De Luce ayanamsa.
    pub const DELUCE: Self = SiderealMode(2);
    /// Raman ayanamsa.
    pub const RAMAN: Self = SiderealMode(3);
    /// Krishnamurti (KP) ayanamsa.
    pub const KRISHNAMURTI: Self = SiderealMode(5);
    /// Sassanian ayanamsa.
    pub const SASSANIAN: Self = SiderealMode(11);
    /// User-defined ayanamsa (supply epoch and value via `set_sid_mode`).
    pub const USER_DEFINED: Self = SiderealMode(255);

    /// Return the raw ayanamsa-mode code.
    #[inline]
    #[must_use]
    pub const fn as_raw(self) -> i32 {
        self.0
    }

    /// Human-readable name of this sidereal mode (e.g. `"Lahiri"`,
    /// `"Fagan-Bradley"`). Delegates to [`crate::functions::config::ayanamsa_name`]
    /// which also handles user-defined overrides.
    #[must_use]
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
    #[must_use]
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
    fn calc_flags_from_i32_roundtrip() {
        let n: i32 = 0x4242;
        let f: CalcFlags = n.into();
        let back: i32 = f.into();
        assert_eq!(back, n);
        let zero: CalcFlags = 0.into();
        let zero_back: i32 = zero.into();
        assert_eq!(zero_back, 0);
    }

    #[test]
    fn house_system_names() {
        assert_eq!(HouseSystem::PLACIDUS.name(), "Placidus");
        assert_eq!(HouseSystem::KOCH.name(), "Koch");
    }

    #[test]
    fn sidereal_mode_display() {
        assert_eq!(SiderealMode::LAHIRI.as_raw(), 1);
        assert_eq!(SiderealMode::USER_DEFINED.as_raw(), 255);
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

    #[test]
    fn special_body_sentinels_keep_negative_ids() {
        assert_eq!(Body::ECL_NUT.as_raw(), -1);
        assert_eq!(Body::FIXED_STAR.as_raw(), -10);
    }

    #[test]
    fn combined_calc_flags_keep_every_bit() {
        assert_eq!(CalcFlags::ASTROMETRIC.as_raw(), 1_536);
        assert_eq!(CalcFlags::DEFAULT.as_raw(), 258);
    }

    #[test]
    fn known_body_id_boundaries_are_exact() {
        let known = [-10, -1, 0, 20, 40, 139, 9_000, 10_000, 1_009_999];
        let unknown = [-11, -9, 21, 39, 140, 8_999, 1_010_000];
        for id in known {
            assert!(Body::is_known_id(id), "expected known id {id}");
        }
        for id in unknown {
            assert!(!Body::is_known_id(id), "expected unknown id {id}");
        }
        assert_eq!(
            Body::try_from_raw(21).unwrap_err().to_string(),
            "body id 21 is outside every documented range (-10, -1, 0..=20, 40..140, 9000..10000, 10000..1010000)"
        );
    }

    #[test]
    fn calc_flags_bitor_preserves_overlapping_bits() {
        assert_eq!(CalcFlags::BUILTIN | CalcFlags::BUILTIN, CalcFlags::BUILTIN);
        assert_eq!(CalcFlags::SPEED | CalcFlags::SPEED, CalcFlags::SPEED);
    }

    #[test]
    fn every_house_system_has_its_canonical_name() {
        let cases = [
            (HouseSystem::PLACIDUS, "Placidus"),
            (HouseSystem::KOCH, "Koch"),
            (HouseSystem::EQUAL, "Equal"),
            (HouseSystem::WHOLE_SIGN, "Whole-Sign"),
            (HouseSystem::PORPHYRY, "Porphyry"),
            (HouseSystem::REGIOMONTANUS, "Regiomontanus"),
            (HouseSystem::CAMPANUS, "Campanus"),
            (HouseSystem::MORINUS, "Morinus"),
            (HouseSystem::ALCABITUS, "Alcabitus"),
            (HouseSystem::AXIAL_ROTATION, "Axial Rotation"),
            (HouseSystem::GAUQUELIN, "Gauquelin"),
            (HouseSystem::VEHLOW_EQUAL, "Vehlow Equal"),
            (HouseSystem::WHOLE_SIGN_MERIDIAN, "Whole-Sign Meridian"),
        ];

        for (system, expected) in cases {
            assert_eq!(system.name(), expected);
        }
    }

    #[test]
    fn sidereal_mode_name_is_canonical() {
        assert_eq!(SiderealMode::LAHIRI.name(), "Lahiri");
    }

    fn assert_is_node(bodies: &[Body], expect: bool) {
        for b in bodies {
            assert_eq!(b.is_node(), expect, "is_node mismatch for {b:?}");
        }
    }

    /// `Body::is_node` matches the four node/apside body codes (10..=13).
    /// These are: Mean Node, True Node, Mean Apogee, Osculating Apogee.
    /// Everything else — planets, Sun, Moon, Earth, Chiron, asteroids — must
    /// return `false`.
    #[test]
    fn is_node_matches_only_lunar_nodes_and_apsides() {
        // Mean Node, True Node, Mean Apogee, Osculating Apogee
        assert_is_node(
            &[
                Body::MEAN_NODE,
                Body::from_raw(11),
                Body::from_raw(12),
                Body::from_raw(13),
            ],
            true,
        );
        // Planets and luminaries must NOT be flagged as nodes
        assert_is_node(
            &[
                Body::SUN,
                Body::MOON,
                Body::MERCURY,
                Body::JUPITER,
                Body::PLUTO,
                Body::CHIRON,
            ],
            false,
        );
        // Out-of-range codes must NOT be flagged
        assert_is_node(
            &[Body::from_raw(9), Body::from_raw(14), Body::from_raw(15)],
            false,
        );
    }

    /// `retrograde_search_window` returns body-tuned window sizes. Mars and
    /// inner planets get short windows; outer planets need years.
    #[test]
    fn retrograde_search_window_per_body() {
        assert_eq!(Body::MARS.retrograde_search_window(), 800.0);
        assert_eq!(Body::JUPITER.retrograde_search_window(), 1500.0);
        assert_eq!(Body::SATURN.retrograde_search_window(), 2000.0);
        assert_eq!(Body::URANUS.retrograde_search_window(), 5000.0);
        assert_eq!(Body::NEPTUNE.retrograde_search_window(), 10_000.0);
        assert_eq!(Body::PLUTO.retrograde_search_window(), 30_000.0);
        // Sun/Moon/Mercury/Venus get the default
        assert_eq!(Body::SUN.retrograde_search_window(), 200.0);
        assert_eq!(Body::MOON.retrograde_search_window(), 200.0);
        assert_eq!(Body::CHIRON.retrograde_search_window(), 200.0);
    }

    /// `ingress_search_window` ramps up with orbital period. Pluto's 248-year
    /// orbit means a 95 000-day window is required to bracket a sign hop.
    #[test]
    fn ingress_search_window_per_body() {
        assert_eq!(Body::SUN.ingress_search_window(), 400.0);
        assert_eq!(Body::MARS.ingress_search_window(), 750.0);
        assert_eq!(Body::JUPITER.ingress_search_window(), 4_500.0);
        assert_eq!(Body::SATURN.ingress_search_window(), 11_000.0);
        assert_eq!(Body::URANUS.ingress_search_window(), 31_000.0);
        assert_eq!(Body::NEPTUNE.ingress_search_window(), 61_000.0);
        assert_eq!(Body::PLUTO.ingress_search_window(), 95_000.0);
        assert_eq!(Body::CHIRON.ingress_search_window(), 800.0);
    }

    /// Property: all three body-tuning methods produce strictly positive
    /// finite values for every defined body, including asteroids and nodes.
    #[test]
    fn body_methods_positive_finite_for_all_codes() {
        for code in [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, // planets
            10, 11, 12, 13, // nodes / apsides
            14, // Earth
            15, 16, 17, 18, 19, 20, // asteroids
            -1, 50, 100, 10_000, // sentinels + numbered asteroids
        ] {
            let b = Body::from_raw(code);
            assert!(b.retrograde_search_window().is_finite());
            assert!(b.retrograde_search_window() > 0.0);
            assert!(b.ingress_search_window().is_finite());
            assert!(b.ingress_search_window() > 0.0);
            assert!(b.orb_weight().is_finite());
            assert!(b.orb_weight() > 0.0);
        }
    }

    /// Property: ingress windows are non-decreasing for Sun→Pluto order
    /// (orbital period grows monotonically). Mars excluded because the
    /// table jumps from 400 d (inner) directly to 750 d at Mars.
    #[test]
    fn ingress_window_non_decreasing_outer_planets() {
        let bodies = [
            Body::JUPITER,
            Body::SATURN,
            Body::URANUS,
            Body::NEPTUNE,
            Body::PLUTO,
        ];
        for pair in bodies.windows(2) {
            let a = pair[0].ingress_search_window();
            let b = pair[1].ingress_search_window();
            assert!(
                b >= a,
                "ingress not monotone: {:?}→{:?} ({} → {})",
                pair[0],
                pair[1],
                a,
                b
            );
        }
    }

    /// Property: orb_weight is non-increasing from luminaries outward.
    #[test]
    fn orb_weight_non_increasing_luminaries_to_outers() {
        let ordered = [
            Body::SUN,
            Body::MOON, // 2.0
            Body::MERCURY,
            Body::VENUS,
            Body::MARS, // 1.5
            Body::JUPITER,
            Body::SATURN, // 1.0
            Body::URANUS,
            Body::NEPTUNE,
            Body::PLUTO,
            Body::CHIRON, // 0.75
        ];
        for pair in ordered.windows(2) {
            let a = pair[0].orb_weight();
            let b = pair[1].orb_weight();
            assert!(
                b <= a,
                "orb weight not monotone: {:?}→{:?} ({} → {})",
                pair[0],
                pair[1],
                a,
                b
            );
        }
    }

    /// `orb_weight` follows the Ptolemaic tradition: luminaries widest,
    /// outer planets tightest. Values used in aspect-orb calculation.
    #[test]
    fn orb_weight_tiers() {
        const EXPECTED: &[(Body, f64)] = &[
            (Body::SUN, 2.0),
            (Body::MOON, 2.0),
            (Body::MERCURY, 1.5),
            (Body::VENUS, 1.5),
            (Body::MARS, 1.5),
            (Body::JUPITER, 1.0),
            (Body::SATURN, 1.0),
            (Body::URANUS, 0.75),
            (Body::NEPTUNE, 0.75),
            (Body::PLUTO, 0.75),
            (Body::CHIRON, 0.75),
            (Body::MEAN_NODE, 0.75),
        ];
        for &(body, expected) in EXPECTED {
            assert_eq!(body.orb_weight(), expected, "{body:?}");
        }
    }

    #[test]
    fn body_display_uses_name() {
        // Display impl falls back to `name()`.
        assert_eq!(format!("{}", Body::SUN), "Sun");
        assert_eq!(format!("{}", Body::CHIRON), "Chiron");
    }

    #[test]
    fn body_from_i32_via_into() {
        let b: Body = 5i32.into();
        assert_eq!(b, Body::JUPITER);
        let n: i32 = Body::JUPITER.into();
        assert_eq!(n, 5);
    }

    #[test]
    fn calc_flags_bitand_and_not() {
        let combined = CalcFlags::BUILTIN | CalcFlags::SPEED;
        // AND with SPEED isolates just the SPEED bit
        let isolated = combined & CalcFlags::SPEED;
        assert!(isolated.is_speed());
        // NOT clears the SPEED bit when AND-ed back
        let cleared = combined & !CalcFlags::SPEED;
        assert!(!cleared.is_speed());
    }

    #[test]
    fn calc_flags_bitor_assign() {
        let mut f = CalcFlags::BUILTIN;
        f |= CalcFlags::SPEED;
        assert!(f.is_speed());
    }

    #[test]
    fn calc_flags_display() {
        // Display impl produces a non-empty string for any flag combination.
        let s = format!("{}", CalcFlags::BUILTIN | CalcFlags::SPEED);
        assert!(!s.is_empty());
    }

    #[test]
    fn house_system_from_u8_and_char() {
        let p: HouseSystem = b'P'.into();
        assert_eq!(HouseSystem::PLACIDUS.as_raw(), b'P');
        assert_eq!(HouseSystem::WHOLE_SIGN_MERIDIAN.as_raw(), b'Y');
        assert_eq!(p.name(), "Placidus");
        let k: HouseSystem = 'K'.into();
        assert_eq!(k.name(), "Koch");
    }

    #[test]
    fn house_system_display_uses_name() {
        assert_eq!(format!("{}", HouseSystem::PLACIDUS), "Placidus");
    }

    #[test]
    fn sidereal_mode_from_i32_and_display() {
        let m: SiderealMode = 1.into();
        assert_eq!(m, SiderealMode::LAHIRI);
        assert!(!format!("{m}").is_empty());
    }

    #[test]
    fn calendar_from_i32_round_trip() {
        let g: Calendar = 1.into();
        assert_eq!(g, Calendar::Gregorian);
        let j: Calendar = 0.into();
        assert_eq!(j, Calendar::Julian);
        // Display impl
        assert!(!format!("{g}").is_empty());
    }

    /// `CalcFlags::is_sidereal` checks the SIDEREAL bit. This flag tells the
    /// engine to apply the configured ayanamsa to all longitudes — used to
    /// switch a calculation from tropical to sidereal zodiac.
    #[test]
    fn is_sidereal_flag_round_trip() {
        // Plain BUILTIN flags do not have SIDEREAL set
        assert!(!CalcFlags::BUILTIN.is_sidereal());
        assert!(!(CalcFlags::BUILTIN | CalcFlags::SPEED).is_sidereal());

        // SIDEREAL on its own is sidereal
        assert!(CalcFlags::SIDEREAL.is_sidereal());

        // OR-ing SIDEREAL with BUILTIN keeps the sidereal bit observable
        let combined = CalcFlags::BUILTIN | CalcFlags::SIDEREAL;
        assert!(combined.is_sidereal());

        // Explicit zero flags has nothing set
        assert!(!CalcFlags(0).is_sidereal());
    }
}
