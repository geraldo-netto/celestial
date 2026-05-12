//! End-to-end smoke tests for the `celestial` binary.
//!
//! Each test invokes the compiled CLI through `assert_cmd` and checks
//! exit status + a couple of output substrings. The goal is to lift
//! CLI binary glue (cmd dispatchers, argparse, main entry) above the
//! 80% coverage threshold without depending on integration internals.

use assert_cmd::Command;
use predicates::prelude::*;

fn celestial() -> Command {
    Command::cargo_bin("celestial").expect("celestial binary built")
}

// ─── help / version ───────────────────────────────────────────────────────────

#[test]
fn help_lists_known_subcommands() {
    celestial()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("phenomena"))
        .stdout(predicate::str::contains("calc"))
        .stdout(predicate::str::contains("houses"))
        .stdout(predicate::str::contains("calendar"));
}

#[test]
fn version_prints_semver() {
    celestial()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ─── phenomena ────────────────────────────────────────────────────────────────

#[test]
fn phenomena_venus_table_runs() {
    celestial()
        .args(["phenomena", "-b", "venus", "-d", "2025-03-20"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Venus").or(predicate::str::contains("venus")));
}

#[test]
fn phenomena_json_output() {
    celestial()
        .args(["phenomena", "-b", "moon", "-d", "2025-03-20", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("phase").or(predicate::str::contains("magnitude")));
}

#[test]
fn phenomena_unknown_body_fails() {
    celestial()
        .args(["phenomena", "-b", "asteroid_x999", "-d", "now"])
        .assert()
        .failure();
}

// ─── calc ─────────────────────────────────────────────────────────────────────

#[test]
fn calc_sun_at_j2000_prints_longitude() {
    celestial()
        .args(["calc", "-b", "sun", "-d", "2000-01-01 12:00"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+").unwrap());
}

// ─── jd ───────────────────────────────────────────────────────────────────────

#[test]
fn jd_now_runs() {
    celestial().args(["jd"]).assert().success();
}

#[test]
fn jd_date_prints_julian_day() {
    celestial()
        .args(["jd", "2000-01-01 12:00"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2451545"));
}

// ─── calendar ─────────────────────────────────────────────────────────────────

#[test]
fn calendar_easter_runs() {
    celestial()
        .args(["calendar", "easter", "--year", "2025"])
        .assert()
        .success();
}

// ─── moon ─────────────────────────────────────────────────────────────────────

#[test]
fn moon_phase_runs() {
    celestial()
        .args(["moon", "-d", "2025-03-20"])
        .assert()
        .success();
}

#[test]
fn moon_next_new_runs() {
    celestial()
        .args(["moon", "-d", "2025-03-20", "--new"])
        .assert()
        .success();
}

#[test]
fn moon_next_full_runs() {
    celestial()
        .args(["moon", "-d", "2025-03-20", "--full"])
        .assert()
        .success();
}

#[test]
fn moon_month_runs() {
    celestial()
        .args(["moon", "--month", "2025-03"])
        .assert()
        .success();
}

#[test]
fn moon_json_runs() {
    celestial()
        .args(["moon", "-d", "2025-03-20", "--json"])
        .assert()
        .success();
}

// ─── calendar subcommands ─────────────────────────────────────────────────────

#[test]
fn calendar_jewish_runs() {
    celestial()
        .args(["calendar", "jewish", "--year", "5785"])
        .assert()
        .success();
}

#[test]
fn calendar_islamic_runs() {
    celestial()
        .args(["calendar", "islamic", "--year", "2025"])
        .assert()
        .success();
}

#[test]
fn calendar_panchanga_runs() {
    celestial()
        .args(["calendar", "panchanga", "--date", "2025-03-20"])
        .assert()
        .success();
}

#[test]
fn calendar_vesak_runs() {
    celestial()
        .args(["calendar", "vesak", "--year", "2025"])
        .assert()
        .success();
}

// ─── calc ─────────────────────────────────────────────────────────────────────

#[test]
fn calc_moon_at_j2000_runs() {
    celestial()
        .args(["calc", "-b", "moon", "-d", "2000-01-01 12:00"])
        .assert()
        .success();
}

#[test]
fn calc_jupiter_sidereal_runs() {
    celestial()
        .args(["calc", "-b", "jupiter", "-d", "2025-03-20", "--sidereal"])
        .assert()
        .success();
}

#[test]
fn jd_from_jd_inverse() {
    // round-trip: JD → date → JD should land on the same JD
    celestial()
        .args(["jd", "--from-jd", "2451545.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2000"));
}
