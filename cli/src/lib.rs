//! Library half of the `celestial` CLI.
//!
//! The main `celestial` binary lives in `main.rs`; this `lib.rs` exposes
//! the same modules so that integration tests and fuzz harnesses can
//! exercise the parsers without spawning the full CLI process.

#![warn(rustdoc::broken_intra_doc_links)]

pub mod cmd;
pub mod error;
pub mod format;
pub mod i18n;
pub mod parse;
pub mod plugin;
