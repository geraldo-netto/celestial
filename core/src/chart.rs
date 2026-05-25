//! Chart analysis: aspects, progressions, returns, Arabic parts,
//! Hellenistic dignities, Chinese astrology, Mesoamerican and Indigenous calendars.
//!
//! # Examples
//!
//! ```no_run
//! use celestial_core::chart::{
//!     AspectOrbs, calc_chart_aspects,
//!     full_dignity, firdaria,
//!     four_pillars, tonalpohualli, medicine_wheel_totem,
//! };
//! use celestial_core::body::Body;
//! use celestial_core::{JulianDay, Longitude};
//!
//! // Aspect matching with separate applying/separating orbs
//! let m = AspectOrbs::new(2.0, 1.5)
//!     .check(280.0, 0.98, 100.0, -0.45, 120.0); // trine
//! if m.matched { println!("Trine  orb = {:.2}°", m.diff.abs()); }
//!
//! // Hellenistic dignity
//! let (dig, score) = full_dignity(Body::SUN, Longitude::new(280.4), false);
//! println!("Sun dignity: {dig:?}  score: {score}");
//!
//! // Chinese Ba Zi four pillars
//! let pillars = four_pillars(JulianDay::new(2_451_545.0), 12.0, Longitude::new(280.4));
//! println!("Year: {} {}", pillars[0].stem_name, pillars[0].branch_name);
//!
//! // Aztec Tonalpohualli
//! let (trecena, sign, name, _) = tonalpohualli(JulianDay::new(2_451_545.0));
//! println!("{trecena} {name}");
//!
//! // Medicine Wheel birth totem
//! let (animal, element, clan, season) = medicine_wheel_totem(Longitude::new(280.4));
//! println!("{animal}  {element}  {clan}  {season}");
//! ```

pub use crate::functions::aspects::AspectOrbs;
pub use crate::functions::aspects::*;
pub use crate::functions::chart::*;
pub use crate::functions::searches::*;
