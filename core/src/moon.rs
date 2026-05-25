//! Moon phases, illumination, esbats, Vesak, and sabbats.
//!
//! **Layering.** This module is the *canonical public facade* for the
//! lunar/calendar domain. The implementation lives in the private
//! `functions` layer (`pub(crate)`); this module curates and
//! re-exports the supported surface. Import from `celestial_core::moon`
//! (or the crate root / `prelude`), never from `functions::` directly.
//!
//! # Examples
//!
//! ```
//! use celestial_core::moon::{moon_phase, moon_illumination, moon_phases_for_month};
//! use celestial_core::JulianDay;
//!
//! let jd    = JulianDay::new(2_451_545.0);
//! let phase = moon_phase(jd).unwrap();
//! let illum = moon_illumination(jd).unwrap();
//! println!("{phase:?}  {illum:.1}%");
//!
//! // All phases in a given month
//! let phases = moon_phases_for_month(2025, 4).unwrap();
//! for p in &phases { println!("{} JD {:.2}", p.phase.name(), p.jd); }
//! ```

#[cfg(feature = "calendar-traditions")]
pub use crate::functions::esbats::*;
pub use crate::functions::moon_phases::*;
#[cfg(feature = "calendar-traditions")]
pub use crate::functions::sabbats::{next_sabbat, sabbat_jd, sabbats_for_year, Sabbat, SabbatKind};
#[cfg(feature = "calendar-traditions")]
pub use crate::functions::vesak::*;
