//! Crossings, rise/set/transit, eclipses, and angle-transit searches.
//!
//! # Examples
//!
//! ```no_run
//! use celestial_core::motion::{RiseTransOptions, SearchOptions, solcross_ut};
//! use celestial_core::body::{Body, CalcFlags, HouseSystem};
//!
//! // Next time the Sun crosses 0° (Aries ingress)
//! let jd = solcross_ut(0.0, 2_451_545.0, CalcFlags::BUILTIN).unwrap();
//!
//! // Moon rise using the builder
//! let rise = RiseTransOptions::new(2_451_545.0, Body::MOON, [2.35, 48.85, 35.0])
//!     .event(1) // CALC_RISE
//!     .search()
//!     .unwrap();
//! println!("Moon rises at JD {:.4}", rise.tret);
//!
//! // Saturn transiting the natal MC
//! let jd_mc = SearchOptions::new(Body::SATURN, 2_460_000.0)
//!     .natal_chart(2_451_545.0, 48.85, 2.35, HouseSystem::PLACIDUS)
//!     .search_mc_transit()
//!     .unwrap();
//! ```

pub use crate::functions::eclipses::*;
pub use crate::functions::motion::RiseTransOptions;
pub use crate::functions::motion::*;
pub use crate::functions::searches::SearchOptions;
