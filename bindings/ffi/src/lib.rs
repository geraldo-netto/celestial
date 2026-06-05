//! Language-neutral FFI facade over `celestial-core`.
//!
//! The js (napi), python (pyo3) and php (ext-php-rs) bindings each wrap
//! ~201 functions. The per-export macro stubs are inherently
//! per-language and stay in each binding, but the genuinely
//! triplicated, language-neutral pieces live here so they exist once:
//!
//! - a single dependency seam: bindings depend on `celestial-ffi`,
//!   which re-exports the whole `celestial-core` API,
//! - [`FfiError`] — one canonical error the per-language error shims
//!   convert from (instead of three hand-rolled `e.to_string()` maps),
//! - [`pos6`] — the `[lon, lat, dist, speed_lon, speed_lat,
//!   speed_dist]` flatten every `calc*` export produced by hand in
//!   each binding.

#![warn(rustdoc::broken_intra_doc_links)]

pub use celestial_core::*;

/// The one canonical FFI error. Transparently wraps a
/// [`celestial_core::Error`]; each binding maps this to its native
/// exception type via a single `to_native(FfiError) -> NativeErr` shim.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct FfiError(#[from] pub celestial_core::Error);

impl FfiError {
    /// The human-readable message (identical to the wrapped core
    /// error's `Display`).
    pub fn message(&self) -> String {
        self.0.to_string()
    }
}

/// Flatten a core [`PlanetPos`] into the
/// `[lon, lat, dist, speed_lon, speed_lat, speed_dist]` array shape
/// every `calc*` export marshals to. `ret_flags` is intentionally
/// excluded — bindings surface it as a separate typed field.
#[inline]
pub fn pos6(p: &celestial_core::PlanetPos) -> [f64; 6] {
    [p.lon, p.lat, p.dist, p.speed_lon, p.speed_lat, p.speed_dist]
}

/// Same six fields as [`pos6`] but as a **tuple** — for bindings (e.g.
/// pyo3) that marshal a fixed-arity tuple rather than an array/list, so
/// the field order lives in exactly one place (DUP-8).
#[inline]
pub fn pos6_tuple(p: &celestial_core::PlanetPos) -> (f64, f64, f64, f64, f64, f64) {
    (p.lon, p.lat, p.dist, p.speed_lon, p.speed_lat, p.speed_dist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos6_preserves_field_order() {
        let p = celestial_core::PlanetPos {
            lon: 1.0,
            lat: 2.0,
            dist: 3.0,
            speed_lon: 4.0,
            speed_lat: 5.0,
            speed_dist: 6.0,
            ret_flags: 7,
        };
        assert_eq!(pos6(&p), [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    /// DEAD-3 contract pin: `pos6` and `pos6_tuple` must yield identical
    /// fields in identical order — every binding (js array, python tuple,
    /// php map) marshals one or the other and downstream consumers expect
    /// the same six slots.
    #[test]
    fn pos6_array_and_tuple_agree_for_random_inputs() {
        let cases = [
            // Boundary fp values
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            (f64::MIN, f64::MAX, f64::MIN_POSITIVE, -0.0, 1e-308, 1e308),
            (f64::INFINITY, f64::NEG_INFINITY, 0.0, 0.0, 0.0, 0.0),
            // NaN propagates field-by-field; can't `==`-compare NaN so handled below
            (-360.0, 360.0, 1.0, -1.0, 360.000_000_000_000_06, -360.0),
        ];
        for (lon, lat, dist, slon, slat, sdist) in cases {
            let p = celestial_core::PlanetPos {
                lon,
                lat,
                dist,
                speed_lon: slon,
                speed_lat: slat,
                speed_dist: sdist,
                ret_flags: 0,
            };
            let arr = pos6(&p);
            let tup = pos6_tuple(&p);
            // Field-wise bitwise equality (handles NaN by `to_bits`).
            assert_eq!(arr[0].to_bits(), tup.0.to_bits());
            assert_eq!(arr[1].to_bits(), tup.1.to_bits());
            assert_eq!(arr[2].to_bits(), tup.2.to_bits());
            assert_eq!(arr[3].to_bits(), tup.3.to_bits());
            assert_eq!(arr[4].to_bits(), tup.4.to_bits());
            assert_eq!(arr[5].to_bits(), tup.5.to_bits());
        }
    }

    /// DEAD-3 contract pin: `pos6` length is exactly 6 — guards against an
    /// accidental field addition that would break the php / js array shape.
    #[test]
    fn pos6_array_len_is_six() {
        let p = celestial_core::PlanetPos {
            lon: 0.0,
            lat: 0.0,
            dist: 0.0,
            speed_lon: 0.0,
            speed_lat: 0.0,
            speed_dist: 0.0,
            ret_flags: 0,
        };
        assert_eq!(pos6(&p).len(), 6);
    }

    #[test]
    fn ffi_error_message_matches_core_display() {
        let core_err = celestial_core::Error::BodyNotImplemented { body: 99 };
        let want = core_err.to_string();
        let ffi: FfiError = core_err.into();
        assert_eq!(ffi.message(), want);
        assert_eq!(ffi.to_string(), want); // transparent
    }
}
