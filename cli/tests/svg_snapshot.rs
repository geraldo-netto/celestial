//! Byte-identical snapshot tests for every `render --chart-type` SVG.
//!
//! These lock the exact rendered bytes for a fixed birth input so renderer
//! refactors (e.g. the DUP-8 preamble extraction) cannot silently change
//! output. The natal-wheel *geometry* is additionally asserted in
//! `natal_builtin_render.rs`; this file is the full-byte gate for the whole
//! chart-type registry — wheel family + specialist/vedic/calendar.
//!
//! Goldens live in `tests/fixtures/svg/<type>.svg`. To regenerate after an
//! intentional change, render each type with the args in `SNAPSHOTS` below.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

fn celestial_binary() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_celestial"))
}

static CLI_LOCK: Mutex<()> = Mutex::new(());

/// `(chart_type, extra_args)` — every registered chart type. Most need only
/// the common birth args; a few require an extra flag (return span, second
/// or third ring date).
const SNAPSHOTS: &[(&str, &[&str])] = &[
    // ── Wheel family ──
    ("natal", &[]),
    ("cosmogram", &[]),
    ("solar-return", &[]),
    ("lunar-return", &[]),
    ("progressed", &["--years", "30"]),
    ("solar-arc", &["--years", "30"]),
    ("biwheel", &["--date2", "1990-07-01 12:00"]),
    ("composite", &["--date2", "1990-07-01 12:00"]),
    (
        "triwheel",
        &["--date2", "1990-07-01 12:00", "--date3", "2000-01-01 12:00"],
    ),
    ("ephemeris", &[]),
    ("profection", &[]),
    // ── Specialist / vedic / calendar ──
    ("dial", &[]),
    ("local-space", &[]),
    ("rasi", &[]),
    ("navamsa", &[]),
    ("dasha", &[]),
    ("north-indian", &[]),
    ("ashtakavarga", &[]),
    ("shadbala", &[]),
    ("hellenistic", &[]),
    ("firdaria", &[]),
    ("bazi", &[]),
    ("mesoamerican", &[]),
    ("medicine-wheel", &[]),
    ("wheel-of-year", &[]),
    ("omer-grid", &[]),
    ("calendar", &[]),
];

/// Render one chart type for the canonical 1986-05-30 reference birth and
/// return the SVG bytes as a string.
fn render(chart_type: &str, extra: &[&str]) -> String {
    let _guard = CLI_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let out = std::env::temp_dir().join(format!("celestial_snap_{chart_type}.svg"));
    let _ = fs::remove_file(&out);

    let mut cmd = Command::new(celestial_binary());
    cmd.args([
        "render",
        "--chart-type",
        chart_type,
        "--date",
        "1986-05-30 09:00",
        "--tz",
        "UTC",
        "--lat=-23.55",
        "--lon=-46.63",
    ]);
    cmd.args(extra);
    cmd.arg("--out").arg(&out);

    let output = cmd.output().expect("failed to spawn celestial");
    assert!(
        output.status.success(),
        "render {chart_type} failed.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(fs::read(&out).expect("output unreadable")).expect("output not UTF-8")
}

#[test]
fn svg_renderers_byte_identical_to_golden() {
    for &(chart_type, extra) in SNAPSHOTS {
        let got = render(chart_type, extra);
        let golden = format!(
            "{}/tests/fixtures/svg/{chart_type}.svg",
            env!("CARGO_MANIFEST_DIR")
        );
        let want = fs::read_to_string(&golden)
            .unwrap_or_else(|_| panic!("missing golden fixture: {golden}"));
        assert_eq!(got, want, "SVG output drifted for chart-type '{chart_type}'");
    }
}
