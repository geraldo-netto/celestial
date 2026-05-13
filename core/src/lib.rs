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
//! | [`constants`] | Numeric constants (body indices, flags, sidereal modes) |
//! | [`position`] | `calc_ut`, `calc_many`, `CalcOptions`, fixed stars, ayanamsa |
//! | [`time`] | Julian day, UTC, calendar conversion |
//! | [`houses`] | House cusp systems |
//! | [`motion`] | Crossings, rise/set/transit, eclipses, `RiseTransOptions`, `SearchOptions` |
//! | [`moon`] | Phase, illumination, principal phases, esbats, sabbats |
//! | [`chart`] | Aspects, `AspectOrbs`, progressions, returns, Arabic parts, traditions |
//! | [`vedic`] | Jyotish helpers, Panchānga |
//! | [`geo`] | Coordinate formatting, timezones |
//! | [`calendar`] | Hebrew, Christian, Islamic, Hindu, Buddhist, Persian, Celtic |
//!
//! All public symbols are re-exported at the crate root, so
//! `use celestial_core::calc_ut` continues to work alongside
//! the preferred `use celestial_core::position::calc_ut`.
//!
//! A [`prelude`] module re-exports the most-used items for convenience.

// ── Crate-private physics engine (unchanged) ──────────────────────────────────
pub(crate) mod astronomy;
pub mod constants; // raw i32 constants kept for internal use

// ── Core types ─────────────────────────────────────────────────────────────────
pub mod error;
pub(crate) mod types;

// ── New domain modules ─────────────────────────────────────────────────────────
pub mod body;
#[cfg(feature = "calendar-traditions")]
pub mod calendar;
pub mod chart;
pub mod geo;
pub mod houses;
pub mod moon;
pub mod motion;
pub mod position;
pub mod solar;
pub mod time;
pub mod vedic;

// ── Flat re-exports from domain modules ─────────────────────────────────────────
// Everything accessible as `celestial_core::calc_ut` etc. for backward compatibility
// and for the language bindings. Prefer using domain modules directly:
//   use celestial_core::position::calc_ut;
//   use celestial_core::moon::moon_phase;
pub use body::*;
#[cfg(feature = "calendar-traditions")]
pub use calendar::*;
pub use chart::*;
pub use constants::*;
pub use geo::*;
pub use houses::*;
pub use moon::*;
pub use motion::*;
pub use position::*;
pub use solar::*;
pub use time::*;
pub use vedic::*;

// ── Public types from types.rs ─────────────────────────────────────────────────
pub use types::{FixStarPos, NodAps, OrbitalDistances, OrbitalElements, PlanetPos};

// ── Error ──────────────────────────────────────────────────────────────────────
pub use error::{Error, Result};

// ── Prelude ────────────────────────────────────────────────────────────────────
pub mod prelude {
    //! The most commonly used items, all in one place.
    //!
    //! ```no_run
    //! use celestial_core::prelude::*;
    //!
    //! let jd  = julday(2025, 3, 20, 9.0, Calendar::Gregorian);
    //! let sun = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN).unwrap();
    //! println!("Sun: {:.4}°", sun.lon);
    //! ```
    pub use crate::body::{Body, CalcFlags, Calendar, HouseSystem, SiderealMode};
    pub use crate::chart::AspectOrbs;
    pub use crate::chart::{
        calc_chart_aspects, calc_chart_aspects_auto, lunar_return_jd, solar_return_jd,
    };
    pub use crate::error::{Error, Result};
    pub use crate::houses::houses;
    pub use crate::moon::{moon_illumination, moon_phase, moon_phases_for_month, MoonPhase};
    pub use crate::motion::{
        mooncross_ut, rise_trans, solcross_ut, RiseTransOptions, SearchOptions,
    };
    pub use crate::position::{
        ayanamsa, calc_many, calc_ut, calc_ut_many, planet_name, set_sid_mode, CalcOptions,
        CalcStrategy, PlanetPos,
    };
    pub use crate::time::{jdnow, julday, revjul, CalDate};
}

// ── Implementation detail — not part of the public API ──────────────────────────
pub(crate) mod functions;
