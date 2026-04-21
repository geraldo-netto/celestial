//! Error types for the celestial-core engine.

/// All errors returned by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A planetary calculation failed or is not yet implemented.
    Calc(String),
    /// A house-system calculation failed.
    Houses(String),
    /// An eclipse or occultation search is not yet implemented.
    Eclipse(String),
    /// A rise/transit search failed.
    RiseTrans(String),
    /// A date/time conversion failed.
    Date(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Calc(s) => write!(f, "Calculation error: {s}"),
            Self::Houses(s) => write!(f, "House calculation error: {s}"),
            Self::Eclipse(s) => write!(f, "Eclipse search error: {s}"),
            Self::RiseTrans(s) => write!(f, "Rise/transit error: {s}"),
            Self::Date(s) => write!(f, "Date conversion error: {s}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience `Result` alias — equivalent to `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
