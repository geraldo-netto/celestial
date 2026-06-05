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
///
/// `#[non_exhaustive]` (NV-1): future error categories can be added
/// without breaking external matchers; `main` only ever needs the
/// `Display` string. Within-crate exhaustive matching still works.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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

impl From<crate::parse::ParseError> for CliError {
    fn from(e: crate::parse::ParseError) -> Self {
        CliError::Parse(e.to_string())
    }
}

/// Result alias used throughout the CLI crate.
pub type CliResult<T> = std::result::Result<T, CliError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_parse_variant() {
        let e = CliError::Parse("bad arg".to_owned());
        assert_eq!(e.to_string(), "bad arg");
    }

    #[test]
    fn display_config_variant() {
        let e = CliError::Config("bad toml".to_owned());
        assert_eq!(e.to_string(), "bad toml");
    }

    #[test]
    fn display_msg_variant() {
        let e = CliError::Msg("free form".to_owned());
        assert_eq!(e.to_string(), "free form");
    }

    #[test]
    fn display_compute_variant_transparent() {
        let inner = celestial_core::Error::Calc("engine boom".to_owned());
        let e = CliError::Compute(inner.clone());
        // `#[error(transparent)]` => identical to the wrapped error's Display.
        assert_eq!(e.to_string(), inner.to_string());
    }

    #[test]
    fn display_io_variant_transparent() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "missing file");
        let e = CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "missing file",
        ));
        assert_eq!(e.to_string(), inner.to_string());
    }

    #[test]
    fn from_core_error_yields_compute() {
        let inner = celestial_core::Error::Calc("core failed".to_owned());
        let e: CliError = inner.clone().into();
        match &e {
            CliError::Compute(c) => assert_eq!(c, &inner),
            other => panic!("expected Compute, got {other:?}"),
        }
        assert_eq!(e.to_string(), inner.to_string());
    }

    #[test]
    fn from_io_error_yields_io() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let e: CliError = io.into();
        match &e {
            CliError::Io(i) => assert_eq!(i.kind(), std::io::ErrorKind::PermissionDenied),
            other => panic!("expected Io, got {other:?}"),
        }
    }

    #[test]
    fn from_string_yields_msg() {
        let e: CliError = String::from("owned msg").into();
        match &e {
            CliError::Msg(m) => assert_eq!(m, "owned msg"),
            other => panic!("expected Msg, got {other:?}"),
        }
        assert_eq!(e.to_string(), "owned msg");
    }

    #[test]
    fn from_str_yields_msg() {
        let e: CliError = "borrowed msg".into();
        match &e {
            CliError::Msg(m) => assert_eq!(m, "borrowed msg"),
            other => panic!("expected Msg, got {other:?}"),
        }
        assert_eq!(e.to_string(), "borrowed msg");
    }

    #[test]
    fn from_parse_error_yields_parse() {
        let pe = crate::parse::ParseError::Date("expected YYYY-MM-DD".to_owned());
        let e: CliError = pe.clone().into();
        match &e {
            CliError::Parse(m) => assert_eq!(m, &pe.to_string()),
            other => panic!("expected Parse, got {other:?}"),
        }
        assert_eq!(e.to_string(), "expected YYYY-MM-DD");
    }

    fn question_core() -> CliResult<()> {
        let r: Result<(), celestial_core::Error> =
            Err(celestial_core::Error::Houses("no houses".to_owned()));
        r?;
        Ok(())
    }

    fn question_io() -> CliResult<()> {
        let r: Result<(), std::io::Error> = Err(std::io::Error::other("io via ?"));
        r?;
        Ok(())
    }

    fn question_parse() -> CliResult<()> {
        let r: Result<(), crate::parse::ParseError> =
            Err(crate::parse::ParseError::Tz("unknown tz".to_owned()));
        r?;
        Ok(())
    }

    #[test]
    fn question_mark_core_is_compute() {
        match question_core() {
            Err(CliError::Compute(celestial_core::Error::Houses(m))) => {
                assert_eq!(m, "no houses");
            }
            other => panic!("expected Compute(Houses), got {other:?}"),
        }
    }

    #[test]
    fn question_mark_io_is_io() {
        match question_io() {
            Err(CliError::Io(i)) => assert_eq!(i.to_string(), "io via ?"),
            other => panic!("expected Io, got {other:?}"),
        }
    }

    #[test]
    fn question_mark_parse_is_parse() {
        match question_parse() {
            Err(CliError::Parse(m)) => assert_eq!(m, "unknown tz"),
            other => panic!("expected Parse, got {other:?}"),
        }
    }
}
