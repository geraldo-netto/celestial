//! # celestial-core
//!
//! Pure-Rust astronomical engine — no C compiler, no data files, no external dependencies.
//!
//! ## Quick start
//!
//! ```no_run
//! use celestial_core::body::{Body, CalcFlags};
//! use celestial_core::time::jdnow;
//! use celestial_core::position::calc_ut;
//!
//! let sun = calc_ut(jdnow(), Body::SUN, CalcFlags::BUILTIN | CalcFlags::SPEED).unwrap();
//! println!("Sun lon = {:.4}°", sun.lon);
//! ```
//!
//! ## Module layout
//!
//! | Module | Contents |
//! |---|---|
//! | [`body`] | [`Body`], [`CalcFlags`], [`HouseSystem`], [`SiderealMode`], [`Calendar`] |
//! | [`time`] | Julian day, UTC, calendar conversion |
//! | [`position`] | Planetary positions, fixed stars, ayanamsa |
//! | [`houses`] | House cusp systems |
//! | [`motion`] | Crossings, rise/set/transit, eclipses |
//! | [`moon`] | Phase, illumination, principal phases, esbats |
//! | [`chart`] | Aspects, progressions, returns, Arabic parts |
//! | [`vedic`] | Jyotish helpers, Panchānga |
//! | [`geo`] | Atlas, coordinate formatting, timezones |
//! | [`calendar`] | Hebrew, Christian, Islamic, Hindu, Buddhist, Persian, Celtic calendars |
//!
//! A [`prelude`] module re-exports the most-used items for convenience.

// ── Crate-private physics engine (unchanged) ──────────────────────────────────
pub(crate) mod astronomy;
pub(crate) mod constants; // raw i32 constants kept for internal use

// ── Core types ─────────────────────────────────────────────────────────────────
pub mod error;
pub(crate) mod types;

// ── New domain modules ─────────────────────────────────────────────────────────
pub mod body;
pub mod calendar;
pub mod chart;
pub mod geo;
pub mod houses;
pub mod moon;
pub mod motion;
pub mod position;
pub mod time;
pub mod vedic;

// ── Legacy flat re-exports (for backward compatibility and bindings) ───────────
// These delegate to the new module structure.
// Deprecated style: `use celestial_core::calc_ut`
// Preferred style:  `use celestial_core::position::calc_ut`
mod flat;
pub use flat::*;

// ── Public types from types.rs ─────────────────────────────────────────────────
pub use types::{FixStarPos, NodAps, OrbitalDistances, OrbitalElements, PlanetPos};

// ── Error ──────────────────────────────────────────────────────────────────────
pub use error::{Error, Result};

// ── Prelude ────────────────────────────────────────────────────────────────────
pub mod prelude {
    //! The most commonly used items, all in one place.
    //!
    //! ```
    //! use celestial_core::prelude::*;
    //! ```
    pub use crate::body::{Body, CalcFlags, Calendar, HouseSystem, SiderealMode};
    pub use crate::error::{Error, Result};
    pub use crate::houses::houses;
    pub use crate::moon::{moon_illumination, moon_phase, MoonPhase};
    pub use crate::position::{ayanamsa, calc_ut, planet_name, set_sid_mode};
    pub use crate::time::{jdnow, julday, revjul, CalDate};
    pub use crate::types::PlanetPos;
}

// ── Keep functions/ as private implementation detail ──────────────────────────
pub(crate) mod functions;
