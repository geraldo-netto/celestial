//! Moon phases, illumination, esbats, Vesak, and sabbats.
//!
//! # Examples
//!
//! ```
//! use celestial_core::moon::{moon_phase, moon_illumination, moon_phases_for_month};
//!
//! let jd    = 2_451_545.0;
//! let phase = moon_phase(jd).unwrap();
//! let illum = moon_illumination(jd).unwrap();
//! println!("{phase:?}  {illum:.1}%");
//!
//! // All phases in a given month
//! let phases = moon_phases_for_month(2025, 4).unwrap();
//! for p in &phases { println!("{} JD {:.2}", p.phase.name(), p.jd); }
//! ```

pub use crate::functions::esbats::*;
pub use crate::functions::moon_phases::*;
pub use crate::functions::sabbats::{next_sabbat, sabbat_jd, sabbats_for_year, Sabbat, SabbatKind};
pub use crate::functions::vesak::*;
