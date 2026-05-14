//! House cusp systems — Placidus, Koch, Equal, Whole-sign, Regiomontanus, …
//!
//! # Examples
//!
//! ```
//! use celestial_core::houses::{houses_ex, houses};
//! use celestial_core::body::{CalcFlags, HouseSystem};
//!
//! let h = houses_ex(2_451_545.0, CalcFlags::BUILTIN,
//!                   48.85, 2.35, HouseSystem::PLACIDUS).unwrap();
//! println!("ASC = {:.4}°  MC = {:.4}°", h.ascmc[0], h.ascmc[1]);
//! ```

pub use crate::astronomy::houses::{mean_sidereal_time_deg, sidereal_time_deg};
pub use crate::functions::houses::*;
