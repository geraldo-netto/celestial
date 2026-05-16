//! Unified CLI error type.
//!
//! Every `cmd::*::run` and the context/render pipeline return
//! [`CliResult`]. Typed boundaries are preserved where they carry
//! information worth inspecting:
//!
//! * [`CliError::Compute`] wraps a [`celestial_core::Error`] verbatim
//!   (`#[from]`), so a `core::Result` propagates with plain `?` — no
//!   more `.map_err(|e| e.to_string())`.
//! * [`CliError::Io`] wraps [`std::io::Error`] (`#[from]`).
//! * [`CliError::Parse`] / [`CliError::Config`] tag argument- and
//!   config-file problems.
//!
//! [`CliError::Msg`] is the bridge for the many call sites that still
//! build an ad-hoc `format!(..)` string; `From<String>`/`From<&str>`
//! let those flow through `?` unchanged. `main` only ever needs the
//! `Display` string.

/// All errors surfaced by the `celestial` CLI.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// Invalid command-line argument or value.
    #[error("{0}")]
    Parse(String),

    /// Problem loading or interpreting the TOML `--config` file.
    #[error("{0}")]
    Config(String),

    /// An error from the `celestial-core` engine, preserved verbatim.
    #[error(transparent)]
    Compute(#[from] celestial_core::Error),

    /// Filesystem / output I/O failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Free-form message (legacy `format!`-built strings).
    #[error("{0}")]
    Msg(String),
}

impl From<String> for CliError {
    fn from(s: String) -> Self {
        CliError::Msg(s)
    }
}

impl From<&str> for CliError {
    fn from(s: &str) -> Self {
        CliError::Msg(s.to_owned())
    }
}

/// Result alias used throughout the CLI crate.
pub type CliResult<T> = std::result::Result<T, CliError>;
