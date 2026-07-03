//! Coverage-expansion smoke tests for the `celestial` binary.
//!
//! Complements `cli_smoke.rs`. Each test drives a CLI path that the
//! existing suite leaves cold: JSON branches, `--next` shortcuts, the
//! sidereal/mode argument plumbing, the Mesoamerican render specialist,
//! and the top-level error/exit paths in `main`. The goal is to lift the
//! `cmd::{calc,moon,sabbats,jd,phenomena}` dispatchers and the render
//! pipeline above the 80% line.

use assert_cmd::Command;
use predicates::prelude::*;

fn celestial() -> Command {
    Command::cargo_bin("celestial").expect("celestial binary built")
}

// ─── calc: json + sidereal/mode plumbing ──────────────────────────────────────

#[test]
fn calc_multi_body_json() {
    celestial()
        .args([
            "calc",
            "-b",
            "sun,moon,mars",
            "-d",
            "2000-01-01 12:00",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

#[test]
fn calc_sidereal_mode_json() {
    celestial()
        .args([
            "calc",
            "-b",
            "mars",
            "-d",
            "2025-03-20",
            "--sidereal",
            "--mode",
            "lahiri",
            "--json",
        ])
        .assert()
        .success();
}

#[test]
fn calc_unknown_sid_mode_fails() {
    celestial()
        .args([
            "calc",
            "-b",
            "sun",
            "-d",
            "now",
            "--sidereal",
            "--mode",
            "not_a_mode",
        ])
        .assert()
        .failure();
}

// ─── jd: json branch + inverse json ───────────────────────────────────────────

#[test]
fn jd_date_json() {
    celestial()
        .args(["jd", "2000-01-01 12:00", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2451545"));
}

#[test]
fn jd_from_jd_json() {
    celestial()
        .args(["jd", "--from-jd", "2451545.0", "--json"])
        .assert()
        .success();
}

// ─── moon: every phase mode + json ────────────────────────────────────────────

#[test]
fn moon_first_quarter_runs() {
    celestial()
        .args(["moon", "-d", "2025-03-20", "--first-quarter"])
        .assert()
        .success();
}

#[test]
fn moon_last_quarter_runs() {
    celestial()
        .args(["moon", "-d", "2025-03-20", "--last-quarter"])
        .assert()
        .success();
}

#[test]
fn moon_combined_phases_json() {
    celestial()
        .args([
            "moon",
            "-d",
            "2025-03-20",
            "--new",
            "--full",
            "--first-quarter",
            "--last-quarter",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

#[test]
fn moon_month_json() {
    celestial()
        .args(["moon", "--month", "2025-03", "--json"])
        .assert()
        .success();
}

#[test]
fn moon_bad_month_fails() {
    celestial()
        .args(["moon", "--month", "2025-13"])
        .assert()
        .failure();
}

#[test]
fn moon_month_with_phase_flag_fails() {
    celestial()
        .args(["moon", "--month", "2025-03", "--new"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--month cannot be combined"));
}

// ─── sabbats / esbats: next + json ────────────────────────────────────────────

#[test]
fn sabbats_year_json() {
    celestial()
        .args(["sabbats", "--year", "2025", "--json"])
        .assert()
        .success();
}

#[test]
fn sabbats_next_text_and_json() {
    celestial().args(["sabbats", "--next"]).assert().success();
    celestial()
        .args(["sabbats", "--next", "--json"])
        .assert()
        .success();
}

#[test]
fn esbats_year_json() {
    celestial()
        .args(["esbats", "--year", "2025", "--json"])
        .assert()
        .success();
}

#[test]
fn esbats_next_text_and_json() {
    celestial().args(["esbats", "--next"]).assert().success();
    celestial()
        .args(["esbats", "--next", "--json"])
        .assert()
        .success();
}

// ─── phenomena: time-merge branch + json ──────────────────────────────────────

#[test]
fn phenomena_time_flag_json() {
    celestial()
        .args([
            "phenomena",
            "-b",
            "mars",
            "-d",
            "2025-03-20",
            "-t",
            "12:00",
            "--json",
        ])
        .assert()
        .success();
}

// ─── render: Mesoamerican specialist (registry alias `maya`) ──────────────────

#[test]
fn render_mesoamerican_svg() {
    let mut out = std::env::temp_dir();
    out.push("celestial_meso_test.svg");
    celestial()
        .args([
            "render",
            "--chart-type",
            "maya",
            "--date",
            "2025-03-20 12:00",
            "--lat=19.43",
            "--lon=-99.13",
            "--tz=-06:00",
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let svg = std::fs::read_to_string(&out).expect("svg written");
    assert!(svg.contains("<svg"), "expected SVG output, got {svg:.80}");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn render_profection_return_year_derives_age() {
    celestial()
        .args([
            "render",
            "--chart-type",
            "profection",
            "--date",
            "2000-01-01 12:00",
            "--tz",
            "UTC",
            "--return-year",
            "2026",
            "--print-context",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"profection_age\": 26"));
}

// ─── SEC-10: user `--var` title/colors must be XML-escaped ────────────────────

#[test]
fn render_specialist_var_injection_escaped() {
    let mut out = std::env::temp_dir();
    out.push("celestial_sec10_test.svg");
    celestial()
        .args([
            "render",
            "--chart-type",
            "maya",
            "--date",
            "2025-03-20 12:00",
            "--lat=19.43",
            "--lon=-99.13",
            "--tz=-06:00",
            "--var",
            "title=</text><script>alert(1)</script>",
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let svg = std::fs::read_to_string(&out).expect("svg written");
    let _ = std::fs::remove_file(&out);
    // The raw injection must NOT appear; the escaped form must.
    assert!(
        !svg.contains("<script>alert(1)</script>"),
        "SEC-10: unescaped injection reached SVG"
    );
    assert!(
        svg.contains("&lt;script&gt;"),
        "SEC-10: title not XML-escaped"
    );
}

#[test]
fn render_south_indian_var_injection_escaped() {
    let mut out = std::env::temp_dir();
    out.push("celestial_sec10_rasi_test.svg");
    celestial()
        .args([
            "render",
            "--chart-type",
            "rasi",
            "--date",
            "2025-03-20 12:00",
            "--lat=19.43",
            "--lon=-99.13",
            "--tz=-06:00",
            "--var",
            "title=</text><script>alert(1)</script>",
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let svg = std::fs::read_to_string(&out).expect("svg written");
    let _ = std::fs::remove_file(&out);
    assert!(
        !svg.contains("<script>alert(1)</script>"),
        "SEC-10: unescaped injection reached South-Indian SVG"
    );
    assert!(
        svg.contains("&lt;script&gt;"),
        "SEC-10: South-Indian title not XML-escaped"
    );
}

// ─── main.rs: top-level error / exit paths ────────────────────────────────────

#[test]
fn unknown_subcommand_fails() {
    celestial()
        .args(["definitely-not-a-command"])
        .assert()
        .failure();
}

#[test]
fn calc_invalid_date_fails() {
    celestial()
        .args(["calc", "-b", "sun", "-d", "not-a-date"])
        .assert()
        .failure();
}

#[test]
fn no_args_prints_help_or_fails() {
    // Bare invocation: clap either prints help (exit 2) or usage; never panics.
    celestial().assert().code(predicate::ne(101));
}
