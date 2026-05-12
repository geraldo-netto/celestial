//! Integration tests for `--help` localization.
//!
//! Spawns the actual `celestial` binary with different `LANG` /
//! `CELESTIAL_LANG` environment variables and asserts that the printed
//! `--help` output contains the expected localized strings.
//!
//! These tests guard against regressions where:
//! - the locale-detection wiring breaks (env var precedence change)
//! - a translation table entry is missing or misrouted at runtime
//! - clap's `mut_subcommand` API stops applying overrides correctly

use std::path::Path;
use std::process::Command;

fn celestial_binary() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_celestial"))
}

/// Run `celestial --help` with the given environment, returning stdout.
fn help_output(envs: &[(&str, &str)]) -> String {
    // We must clear inherited locale variables to avoid contamination from
    // the test runner's environment — otherwise an `LC_ALL=en_US.UTF-8` set
    // by CI would override anything we pass for `LANG`.
    let mut cmd = Command::new(celestial_binary());
    cmd.arg("--help")
        .env_remove("LANG")
        .env_remove("LC_ALL")
        .env_remove("CELESTIAL_LANG");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("failed to spawn celestial");
    assert!(
        output.status.success(),
        "celestial --help exited non-zero (stderr: {})",
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn english_default_when_no_locale_set() {
    let out = help_output(&[]);
    assert!(
        out.contains("Astronomical calculations"),
        "English about line missing.\nGot:\n{out}"
    );
    assert!(
        out.contains("Calculate geocentric planetary positions"),
        "English calc subcommand description missing.\nGot:\n{out}"
    );
}

#[test]
fn english_for_unsupported_locale() {
    let out = help_output(&[("LANG", "fr_FR.UTF-8")]);
    assert!(
        out.contains("Astronomical calculations"),
        "Unsupported locale should fall back to English.\nGot:\n{out}"
    );
}

#[test]
fn brazilian_portuguese_lang_var() {
    let out = help_output(&[("LANG", "pt_BR.UTF-8")]);
    assert!(
        out.contains("Cálculos astronômicos"),
        "Portuguese about line missing.\nGot:\n{out}"
    );
    assert!(
        out.contains("Calcular posições geocêntricas"),
        "Portuguese calc description missing.\nGot:\n{out}"
    );
}

#[test]
fn spanish_lang_var() {
    let out = help_output(&[("LANG", "es_ES.UTF-8")]);
    assert!(
        out.contains("Cálculos astronómicos"),
        "Spanish about line missing.\nGot:\n{out}"
    );
    assert!(
        out.contains("Calcular cúspides de casas"),
        "Spanish houses description missing.\nGot:\n{out}"
    );
}

#[test]
fn italian_lang_var() {
    let out = help_output(&[("LANG", "it_IT.UTF-8")]);
    assert!(
        out.contains("Calcoli astronomici"),
        "Italian about line missing.\nGot:\n{out}"
    );
    assert!(
        out.contains("Ruota dell'Anno Celtica"),
        "Italian sabbats description missing.\nGot:\n{out}"
    );
}

#[test]
fn german_lang_var() {
    let out = help_output(&[("LANG", "de_DE.UTF-8")]);
    assert!(
        out.contains("Astronomische Berechnungen"),
        "German about line missing.\nGot:\n{out}"
    );
    assert!(
        out.contains("Geozentrische Planetenpositionen"),
        "German calc description missing.\nGot:\n{out}"
    );
}

/// `CELESTIAL_LANG` must override both `LC_ALL` and `LANG`.
#[test]
fn celestial_lang_overrides_lang() {
    let out = help_output(&[
        ("CELESTIAL_LANG", "de"),
        ("LANG", "pt_BR.UTF-8"),
        ("LC_ALL", "es_ES.UTF-8"),
    ]);
    assert!(
        out.contains("Astronomische Berechnungen"),
        "CELESTIAL_LANG should win over LC_ALL+LANG.\nGot:\n{out}"
    );
}

/// `LC_ALL` must override `LANG` (POSIX behavior).
#[test]
fn lc_all_overrides_lang() {
    let out = help_output(&[("LC_ALL", "it_IT.UTF-8"), ("LANG", "pt_BR.UTF-8")]);
    assert!(
        out.contains("Calcoli astronomici"),
        "LC_ALL should win over LANG.\nGot:\n{out}"
    );
}

/// Subcommand --help also picks up the locale.
#[test]
fn subcommand_help_localized() {
    let mut cmd = Command::new(celestial_binary());
    cmd.args(["calc", "--help"])
        .env_remove("LC_ALL")
        .env_remove("CELESTIAL_LANG")
        .env("LANG", "pt_BR.UTF-8");
    let out = cmd.output().expect("failed to spawn celestial");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Calcular posições geocêntricas"),
        "subcommand help didn't localize.\nGot:\n{stdout}"
    );
}

/// `pt_PT` (European Portuguese) routes to Brazilian — only variant we ship.
#[test]
fn pt_pt_routes_to_brazilian() {
    let out = help_output(&[("LANG", "pt_PT.UTF-8")]);
    assert!(
        out.contains("Cálculos astronômicos"),
        "pt_PT should route to Brazilian Portuguese.\nGot:\n{out}"
    );
}
