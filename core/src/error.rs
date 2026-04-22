//! Error types for the celestial-core engine.
//!
//! All errors implement [`std::fmt::Display`] with a human-readable message,
//! so callers that only need `.to_string()` are unaffected by new variants.
//!
//! The enum is marked `#[non_exhaustive]` — match arms must include a `_`
//! wildcard so that adding variants in future is non-breaking.
//!
//! # Structured vs string variants
//!
//! Structured variants (`BodyNotImplemented`, `StarNotFound`, etc.) carry
//! typed fields for programmatic inspection. The legacy string variants
//! (`Calc`, `Houses`, `Eclipse`, `RiseTrans`, `Date`) remain for call sites
//! where a free-form message is more convenient.

/// All errors returned by `celestial-core`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
    // ── Structured variants ─────────────────────────────────────────────────
    /// A body number is outside the range the engine supports.
    BodyNotImplemented {
        /// Raw body index (e.g. `Body::SUN.as_raw()` = 0).
        body: i32,
    },

    /// A fixed-star name could not be found in the built-in catalog.
    StarNotFound {
        /// The name that was looked up.
        name: String,
    },

    /// A moon-phase search converged on a JD that lies outside the
    /// expected time window (numerical instability or bad input JD).
    PhaseNotFound {
        /// Description of which phase was sought.
        phase: String,
        /// The JD at which the search was started.
        from_jd: f64,
    },

    /// An eclipse or occultation search found no event in the search window.
    NoEclipseFound {
        /// The JD at which the search was started.
        from_jd: f64,
    },

    /// A rise/transit/set calculation determined that the body never
    /// crosses the horizon at the given latitude (circumpolar or sub-horizon).
    CircumpolarBody {
        /// Raw body index.
        body: i32,
        /// Observer geographic latitude in degrees.
        lat: f64,
    },

    /// A house-system calculation failed at the given latitude or with
    /// the given system byte.
    HouseSystemFailed {
        /// House system byte code (e.g. `b'P'` for Placidus).
        system: u8,
        /// Observer geographic latitude in degrees.
        lat: f64,
    },

    // ── Legacy string variants (kept for backward compatibility) ────────────
    /// A planetary calculation failed or is not yet implemented.
    Calc(String),
    /// A house-system calculation failed.
    Houses(String),
    /// An eclipse or occultation search failed.
    Eclipse(String),
    /// A rise/transit search failed.
    RiseTrans(String),
    /// A date/time conversion failed.
    ///
    /// Reserved for future use — the pure-Rust engine currently
    /// returns `Calc` errors for invalid date inputs.
    Date(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BodyNotImplemented { body } => {
                write!(f, "body {body} is not implemented by this engine")
            }
            Self::StarNotFound { name } => write!(f, "fixed star '{name}' not found in catalog"),
            Self::PhaseNotFound { phase, from_jd } => {
                write!(f, "moon phase '{phase}' not found from JD {from_jd:.2}")
            }
            Self::NoEclipseFound { from_jd } => write!(f, "no eclipse found from JD {from_jd:.2}"),
            Self::CircumpolarBody { body, lat } => write!(
                f,
                "body {body} is circumpolar or never rises at latitude {lat:.2}°"
            ),
            Self::HouseSystemFailed { system, lat } => write!(
                f,
                "house system '{}' failed at latitude {lat:.2}°",
                char::from(*system)
            ),
            Self::Calc(s) => write!(f, "calculation error: {s}"),
            Self::Houses(s) => write!(f, "house calculation error: {s}"),
            Self::Eclipse(s) => write!(f, "eclipse search error: {s}"),
            Self::RiseTrans(s) => write!(f, "rise/transit error: {s}"),
            Self::Date(s) => write!(f, "date conversion error: {s}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience `Result` alias — equivalent to `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
