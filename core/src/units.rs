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
