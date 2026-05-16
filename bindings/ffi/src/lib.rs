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

pub use celestial_core as core;
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

/// Flatten a core [`PlanetPos`](celestial_core::PlanetPos) into the
/// `[lon, lat, dist, speed_lon, speed_lat, speed_dist]` array shape
/// every `calc*` export marshals to. `ret_flags` is intentionally
/// excluded — bindings surface it as a separate typed field.
#[inline]
pub fn pos6(p: &celestial_core::PlanetPos) -> [f64; 6] {
    [
        p.lon,
        p.lat,
        p.dist,
        p.speed_lon,
        p.speed_lat,
        p.speed_dist,
    ]
}
