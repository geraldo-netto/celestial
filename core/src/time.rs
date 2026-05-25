//! Julian Day conversion, UTC, calendar arithmetic, and `CalDate`.
//!
//! # Examples
//!
//! ```
//! use celestial_core::time::{julday, revjul, jdnow, CalDate};
//! use celestial_core::body::Calendar;
//! use celestial_core::JulianDay;
//!
//! let jd   = julday(2025, 3, 20, 9.0, Calendar::Gregorian);
//! let date = revjul(JulianDay::new(jd), Calendar::Gregorian);
//! assert_eq!(date.year, 2025);
//! ```

pub use crate::functions::time::*;
