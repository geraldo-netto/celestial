//! House cusp systems — Placidus, Koch, Equal, Whole-sign, Regiomontanus, …
//!
//! **Layering.** This module is the *canonical public facade* for the
//! house-systems domain. The implementation lives in the private
//! `functions` / `astronomy` layers (`pub(crate)`);
//! this module curates and re-exports the supported surface. Import
//! from `celestial_core::houses` (or the crate root / `prelude`), never
//! from `functions::` / `astronomy::` directly.
//!
//! # Examples
//!
//! ```
//! use celestial_core::houses::{houses_ex, houses};
//! use celestial_core::body::{CalcFlags, HouseSystem};
//! use celestial_core::{JulianDay, Latitude, Longitude};
//!
//! let h = houses_ex(JulianDay::new(2_451_545.0), CalcFlags::BUILTIN,
//!                   Latitude::new(48.85), Longitude::new(2.35), HouseSystem::PLACIDUS).unwrap();
//! println!("ASC = {:.4}°  MC = {:.4}°", h.ascmc[0], h.ascmc[1]);
//! ```

pub use crate::astronomy::houses::{mean_sidereal_time_deg, sidereal_time_deg};
pub use crate::functions::houses::*;
