//! Solar (Schwabe) cycle calculation.
//!
//! See [`solar_cycle`] for the per-JD lookup and [`grand_solar_epoch`] for the
//! long-term (centuries-scale) escape hatch.
//!
//! # Example
//!
//! ```
//! use celestial_core::solar::{solar_cycle, SolarCyclePhase};
//! use celestial_core::JulianDay;
//!
//! // Mid-2022 lands in cycle 25, ascending phase.
//! let info = solar_cycle(JulianDay::new(2_459_726.0)).unwrap();
//! assert_eq!(info.cycle_num, 25);
//! assert_eq!(info.phase_name, SolarCyclePhase::Rising);
//! ```

pub use crate::astronomy::solar_cycle::{
    cycle_nickname, grand_solar_epoch, solar_cycle, GrandSolarEpoch, SolarCycleInfo,
    SolarCyclePhase,
};
