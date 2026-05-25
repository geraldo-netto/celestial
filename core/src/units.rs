//! Zero-cost unit newtypes for the public compute API (DP-2).
//!
//! Each is `#[repr(transparent)]` over `f64`, so wrapping/unwrapping is
//! a no-op at runtime and the precision-tested compute paths see the
//! identical bit pattern. The newtypes exist only to stop callers
//! transposing a Julian Day, a latitude and a longitude at the API
//! boundary — every public entry point destructures back to `f64` on
//! its first line, leaving the numeric core unchanged.

/// Generate a transparent `f64` newtype with the conversion glue every
/// boundary needs. Each generated item is CC 1 (trivial), so this stays
/// well within the project's complexity ceiling.
macro_rules! f64_newtype {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
        pub struct $name(pub f64);

        impl $name {
            /// Wrap a raw `f64`.
            #[inline]
            #[must_use]
            pub const fn new(v: f64) -> Self {
                Self(v)
            }

            /// Unwrap to the raw `f64`.
            #[inline]
            #[must_use]
            pub const fn get(self) -> f64 {
                self.0
            }
        }

        impl From<f64> for $name {
            #[inline]
            fn from(v: f64) -> Self {
                Self(v)
            }
        }

        impl From<$name> for f64 {
            #[inline]
            fn from(v: $name) -> Self {
                v.0
            }
        }
    };
}

f64_newtype!(
    /// A Julian Day number (ET or UT, per the call site).
    JulianDay
);
f64_newtype!(
    /// Geographic latitude in degrees, north positive.
    Latitude
);
f64_newtype!(
    /// Geographic longitude in degrees, east positive.
    Longitude
);
f64_newtype!(
    /// A bare angle in degrees (ARMC, obliquity, …).
    Degrees
);

#[cfg(test)]
mod tests {
    use super::{Degrees, JulianDay, Latitude, Longitude};

    /// TEST-6: `From<f64>` / `From<$T> for f64` / `Default` are part of
    /// the public boundary contract — exercise them so the macro-generated
    /// impls are actually covered (otherwise `units.rs` lands below the
    /// 80% per-file floor).
    #[test]
    fn julian_day_round_trip_and_default() {
        let jd: JulianDay = 2_446_950.875_f64.into();
        assert!((f64::from(jd) - 2_446_950.875_f64).abs() < f64::EPSILON);
        assert_eq!(JulianDay::default().get(), 0.0);
    }

    #[test]
    fn latitude_round_trip_and_default() {
        let lat: Latitude = (-23.45_f64).into();
        assert!((f64::from(lat) - (-23.45_f64)).abs() < f64::EPSILON);
        assert_eq!(Latitude::default().get(), 0.0);
    }

    #[test]
    fn longitude_round_trip_and_default() {
        let lon: Longitude = 46.6333_f64.into();
        assert!((f64::from(lon) - 46.6333_f64).abs() < f64::EPSILON);
        assert_eq!(Longitude::default().get(), 0.0);
    }

    #[test]
    fn degrees_round_trip_and_default() {
        let d: Degrees = 180.0_f64.into();
        assert!((f64::from(d) - 180.0_f64).abs() < f64::EPSILON);
        assert_eq!(Degrees::default().get(), 0.0);
    }
}
