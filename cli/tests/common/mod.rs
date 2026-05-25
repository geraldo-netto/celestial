//! Shared scaffolding for the cli integration tests (DUP-9).
//!
//! Each `cli/tests/*.rs` file is its own crate, so anything they need in
//! common has to live in a sub-module compiled into each test crate. The
//! pieces shared by every harness:
//!
//! - [`celestial_binary`] — path to the `celestial` binary Cargo built
//!   for the current test run, via the `CARGO_BIN_EXE_celestial` env var
//!   that Cargo sets at compile time.
//! - [`CLI_LOCK`] — global mutex that serialises CLI spawns so the
//!   underlying swissdata file-locking can't deadlock under parallel
//!   integration tests (regression seen on macOS).
//!
//! The actual per-test render helpers stay in each file because they
//! differ in shape (snapshot vs. template vs. natal-geometry vs. --help
//! locale).

// Each integration test file is its own crate and compiles this module
// independently. Tests that don't use the lock (e.g. `i18n_help`, which
// runs `--help` and never writes a scratch file) would trip the
// `dead_code` lint otherwise.
#![allow(dead_code)]

use std::path::Path;
use std::sync::Mutex;

/// Path to the `celestial` binary built for the current test crate.
pub fn celestial_binary() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_celestial"))
}

/// Serialises CLI invocations to avoid swissdata file-locking contention
/// when several integration tests in the same crate run in parallel.
pub static CLI_LOCK: Mutex<()> = Mutex::new(());
