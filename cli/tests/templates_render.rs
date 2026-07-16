//! Integration tests for every bundled template under `cli/templates/`.
//!
//! Each test:
//! 1. Spawns the `celestial render` binary with the appropriate
//!    `--chart-type` and any required ancillary flags (date2, calendar
//!    overlays, etc.).
//! 2. Points `--template` at the template under test.
//! 3. Verifies the binary exits 0 and writes valid SVG output.
//!
//! These tests guard the bundled templates against context-shape drift
//! when the underlying chart-type builder gains/loses fields. If a
//! template starts referencing an undefined field, MiniJinja errors and
//! the test fails.
//!
//! The CLI binary path is provided by Cargo via the `CARGO_BIN_EXE_celestial`
//! env-var at compile time. No PATH or system-binary assumptions.

mod common;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use common::{celestial_binary, CLI_LOCK};

/// Locate the workspace's `cli/templates/` directory relative to this
/// test file's manifest. `CARGO_MANIFEST_DIR` is the cli crate root.
fn templates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn add_default_time(args: &mut [String], flag: &str) {
    let Some(i) = args.iter().position(|a| a == flag) else {
        return;
    };
    let Some(date) = args.get_mut(i + 1) else {
        return;
    };
    if !date.contains(' ') && date.parse::<f64>().is_err() && date != "now" {
        date.push_str(" 12:00");
    }
}

/// Render a template and return the SVG bytes. Panics with a useful
/// message on any of: missing template, non-zero exit, missing output,
/// non-SVG content.
fn render_template(template_name: &str, out_name: &str, extra_args: &[&str]) -> Vec<u8> {
    let _guard = CLI_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tpl = templates_dir().join(template_name);
    assert!(
        tpl.exists(),
        "template `{}` not found at {}",
        template_name,
        tpl.display()
    );

    let out = std::env::temp_dir().join(out_name);
    let _ = fs::remove_file(&out);

    // Natal/derived charts now require an explicit birth time + timezone.
    // These template tests only assert SVG shape, so normalise any date args
    // to carry a time and append a fixed UTC timezone.
    let mut args: Vec<String> = extra_args.iter().map(|s| s.to_string()).collect();
    for flag in ["--date", "--date2", "--date3"] {
        add_default_time(&mut args, flag);
    }
    if !args.iter().any(|a| a == "--tz" || a == "--timezone") {
        args.push("--tz".into());
        args.push("UTC".into());
    }

    let mut cmd = Command::new(celestial_binary());
    cmd.arg("render")
        .args(&args)
        .arg("--template")
        .arg(&tpl)
        .arg("--out")
        .arg(&out);

    let output = cmd
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn celestial: {e}"));

    assert!(
        output.status.success(),
        "celestial render failed for template `{}`\nstdout: {}\nstderr: {}",
        template_name,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let bytes = fs::read(&out).unwrap_or_else(|e| {
        panic!(
            "could not read output `{}` after rendering `{}`: {e}",
            out.display(),
            template_name
        )
    });

    let head: &[u8] = if bytes.len() >= 64 {
        &bytes[..64]
    } else {
        &bytes
    };
    let head_str = String::from_utf8_lossy(head);
    assert!(
        head_str.contains("<?xml") || head_str.contains("<svg"),
        "template `{template_name}` produced output that doesn't look like SVG (first 64 bytes: {head_str:?})"
    );

    assert!(
        bytes.len() > 500,
        "template `{}` produced suspiciously small output ({} bytes)",
        template_name,
        bytes.len()
    );

    bytes
}

/// Convenience: assert the rendered SVG contains a particular substring.
fn assert_contains(svg: &[u8], needle: &str, template: &str) {
    let s = String::from_utf8_lossy(svg);
    assert!(
        s.contains(needle),
        "template `{template}` rendered SVG missing expected content `{needle}`",
    );
}

// ─── Template tests ──────────────────────────────────────────────────────

/// `example.svg.tt` is the minimal demo template — natal chart, no overlays.
#[test]
fn renders_example_template() {
    let svg = render_template(
        "example.svg.tt",
        "test_example.svg",
        &[
            "--chart-type",
            "natal",
            "--date",
            "2024-01-01",
            "--lat=40.71",
            "--lon=-74.0",
        ],
    );
    // The example template renders sign glyphs and planet glyphs.
    assert_contains(&svg, "<svg", "example.svg.tt");
}

/// `natal_with_overlays.svg.tt` — natal wheel tagged with omer/sabbats/moon.
/// May 15 2024 falls inside the Omer count, so the title block must
/// produce the Omer banner.
#[test]
fn renders_natal_with_overlays() {
    let svg = render_template(
        "natal_with_overlays.svg.tt",
        "test_natal_overlays.svg",
        &[
            "--chart-type",
            "natal",
            "--date",
            "2024-05-15 12:00",
            "--lat",
            "48.85",
            "--lon",
            "2.35",
            "--calendar",
            "omer",
            "--calendar",
            "sabbats",
            "--calendar",
            "moon",
        ],
    );
    // The Omer-day banner appears only if the omer overlay is non-null.
    assert_contains(&svg, "Omer", "natal_with_overlays.svg.tt");
}

/// `year_calendar.svg.tt` — 12-month grid with all overlays.
#[test]
fn renders_year_calendar() {
    let svg = render_template(
        "year_calendar.svg.tt",
        "test_year.svg",
        &[
            "--chart-type",
            "natal",
            "--date",
            "2024-04-15",
            "--lat",
            "0",
            "--lon",
            "0",
            "--calendar",
            "gregorian-year",
            "--calendar",
            "omer",
            "--calendar",
            "moon",
            "--calendar",
            "sabbats",
            "--calendar",
            "hebrew",
        ],
    );
    // 12 month names should appear in the SVG.
    for month in &["January", "February", "March", "December"] {
        assert_contains(&svg, month, "year_calendar.svg.tt");
    }
    // Year heading
    assert_contains(&svg, "2024", "year_calendar.svg.tt");
}

/// `full_astral_map.svg.tt` — complete reference natal chart.
#[test]
fn renders_full_astral_map() {
    let svg = render_template(
        "full_astral_map.svg.tt",
        "test_astral.svg",
        &[
            "--chart-type",
            "natal",
            "--date",
            "1990-05-15 14:30",
            "--lat=40.71",
            "--lon=-74.0",
        ],
    );
    // The astral map has an "Aspects" table heading and an "Arabic parts" heading.
    assert_contains(&svg, "Planets", "full_astral_map.svg.tt");
    assert_contains(&svg, "Aspects", "full_astral_map.svg.tt");
    assert_contains(&svg, "Arabic parts", "full_astral_map.svg.tt");
    assert_contains(&svg, "Almuten Figuris", "full_astral_map.svg.tt");
    assert_contains(&svg, "Fixed-star conjunctions", "full_astral_map.svg.tt");
}

/// `bazi_chart.svg.tt` — Chinese 4-pillars chart.
#[test]
fn renders_bazi_chart() {
    let svg = render_template(
        "bazi_chart.svg.tt",
        "test_bazi.svg",
        &[
            "--chart-type",
            "bazi",
            "--date",
            "1990-05-15 14:30",
            "--lat=40.71",
            "--lon=-74.0",
        ],
    );
    // The four pillar labels (in the labels array) should be in the SVG
    assert_contains(&svg, "Year Pillar", "bazi_chart.svg.tt");
    assert_contains(&svg, "Day Pillar", "bazi_chart.svg.tt");
    assert_contains(&svg, "Five Elements", "bazi_chart.svg.tt");
}

/// `vedic_rasi.svg.tt` — Indian rasi chart with planets in their sidereal sign.
#[test]
fn renders_vedic_rasi() {
    let svg = render_template(
        "vedic_rasi.svg.tt",
        "test_rasi.svg",
        &[
            "--chart-type",
            "rasi",
            "--date",
            "1990-05-15 14:30",
            "--lat=40.71",
            "--lon=-74.0",
        ],
    );
    assert_contains(&svg, "Vedic Rasi Chart", "vedic_rasi.svg.tt");
    assert_contains(&svg, "Vimshottari Dasha", "vedic_rasi.svg.tt");
    // 12 sign names
    assert_contains(&svg, "Aries", "vedic_rasi.svg.tt");
    assert_contains(&svg, "Pisces", "vedic_rasi.svg.tt");
}

/// `mesoamerican_calendars.svg.tt` — Tonalpohualli + Tzolk'in + Haab + Xiuhpohualli.
#[test]
fn renders_mesoamerican_calendars() {
    let svg = render_template(
        "mesoamerican_calendars.svg.tt",
        "test_meso.svg",
        &[
            "--chart-type",
            "mesoamerican",
            "--date",
            "1990-05-15 14:30",
            "--lat=40.71",
            "--lon=-74.0",
        ],
    );
    // The four panel headings
    assert_contains(&svg, "Tzolk", "mesoamerican_calendars.svg.tt"); // Tzolk'in
    assert_contains(&svg, "Tonalpohualli", "mesoamerican_calendars.svg.tt");
    assert_contains(&svg, "Haab", "mesoamerican_calendars.svg.tt");
    assert_contains(&svg, "Xiuhpohualli", "mesoamerican_calendars.svg.tt");
    assert_contains(&svg, "Calendar Round", "mesoamerican_calendars.svg.tt");
}

/// `medicine_wheel.svg.tt` — Indigenous 4-direction wheel.
#[test]
fn renders_medicine_wheel() {
    let svg = render_template(
        "medicine_wheel.svg.tt",
        "test_wheel.svg",
        &[
            "--chart-type",
            "medicine-wheel",
            "--date",
            "1990-05-15 14:30",
            "--lat=40.71",
            "--lon=-74.0",
        ],
    );
    // The four cardinal direction labels
    assert_contains(&svg, "EAST", "medicine_wheel.svg.tt");
    assert_contains(&svg, "SOUTH", "medicine_wheel.svg.tt");
    assert_contains(&svg, "WEST", "medicine_wheel.svg.tt");
    assert_contains(&svg, "NORTH", "medicine_wheel.svg.tt");
    assert_contains(&svg, "Medicine Wheel", "medicine_wheel.svg.tt");
}

/// `hellenistic_dignities.svg.tt` — Essential dignities + Lots.
#[test]
fn renders_hellenistic_dignities() {
    let svg = render_template(
        "hellenistic_dignities.svg.tt",
        "test_hell.svg",
        &[
            "--chart-type",
            "hellenistic",
            "--date",
            "1990-05-15 14:30",
            "--lat=40.71",
            "--lon=-74.0",
        ],
    );
    assert_contains(&svg, "Essential Dignities", "hellenistic_dignities.svg.tt");
    assert_contains(&svg, "Lots", "hellenistic_dignities.svg.tt");
    // Sect badge — depends on whether Sun is above horizon
    let s = String::from_utf8_lossy(&svg);
    assert!(
        s.contains("Day Chart") || s.contains("Night Chart"),
        "hellenistic_dignities.svg.tt missing sect badge",
    );
}

/// `dial_90.svg.tt` — Uranian 90° midpoint dial.
#[test]
fn renders_dial_90() {
    let svg = render_template(
        "dial_90.svg.tt",
        "test_dial.svg",
        &[
            "--chart-type",
            "dial",
            "--date",
            "2000-01-01",
            "--lat",
            "0",
            "--lon",
            "0",
        ],
    );
    assert_contains(&svg, "Midpoint Dial", "dial_90.svg.tt");
    assert_contains(&svg, "midpoints", "dial_90.svg.tt");
}

/// `biwheel_synastry.svg.tt` — Inner natal + outer transit ring.
#[test]
fn renders_biwheel_synastry() {
    let svg = render_template(
        "biwheel_synastry.svg.tt",
        "test_biwheel.svg",
        &[
            "--chart-type",
            "biwheel",
            "--date",
            "1990-05-15 14:30",
            "--lat=40.71",
            "--lon=-74.0",
            "--date2",
            "2025-03-20",
        ],
    );
    assert_contains(&svg, "Bi-wheel", "biwheel_synastry.svg.tt");
    assert_contains(&svg, "Cross-aspects", "biwheel_synastry.svg.tt");
    assert_contains(&svg, "Inner ring", "biwheel_synastry.svg.tt");
    assert_contains(&svg, "Outer ring", "biwheel_synastry.svg.tt");
}
