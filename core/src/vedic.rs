//! Vedic / Jyotish helpers: nakshatra, navamsa, Panchānga, Vimshottari dasha.
//!
//! # Examples
//!
//! ```
//! use celestial_core::vedic::{long_to_nakshatra, panchanga};
//! use celestial_core::Longitude;
//!
//! let (nak, pada) = long_to_nakshatra(Longitude::new(123.456));
//! println!("Nakshatra {nak}, pada {pada}");
//! ```

pub use crate::functions::panchanga::*;
pub use crate::functions::vedic::*;
