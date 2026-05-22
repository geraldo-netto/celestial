//! Byte-identical snapshot tests for the specialist / vedic / calendar SVG
//! renderers (the ones outside the natal-wheel gate in
//! `natal_builtin_render.rs`).
//!
//! These lock the exact rendered bytes for a fixed birth input so the DUP-8
//! preamble refactor — and any future renderer tweak — cannot silently
//! change output. Goldens live in `tests/fixtures/svg/<type>.svg`; regenerate
//! them by rendering each type with the args below if an intentional change
//! is made.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

fn celestial_binary() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_celestial"))
}

static CLI_LOCK: Mutex<()> = Mutex::new(());

/// Chart types whose renderers carry an inline SVG preamble (DUP-8 sites).
const SNAPSHOT_TYPES: &[&str] = &[
    "dial",
    "local-space",
    "rasi",
    "navamsa",
    "dasha",
    "north-indian",
    "ashtakavarga",
    "shadbala",
    "hellenistic",
    "firdaria",
    "bazi",
    "mesoamerican",
    "medicine-wheel",
    "wheel-of-year",
    "omer-grid",
    "calendar",
];

/// Render one chart type for the canonical 1986-05-30 reference birth and
/// return the SVG bytes as a string.
fn render(chart_type: &str) -> String {
    let _guard = CLI_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let out = std::env::temp_dir().join(format!("celestial_snap_{chart_type}.svg"));
    let _ = fs::remove_file(&out);

    let output = Command::new(celestial_binary())
        .args([
            "render",
            "--chart-type",
            chart_type,
            "--date",
            "1986-05-30 09:00",
            "--tz",
            "UTC",
            "--lat=-23.55",
            "--lon=-46.63",
            "--out",
        ])
        .arg(&out)
        .output()
        .expect("failed to spawn celestial");
    assert!(
        output.status.success(),
        "render {chart_type} failed.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(fs::read(&out).expect("output unreadable")).expect("output not UTF-8")
}

#[test]
fn svg_renderers_byte_identical_to_golden() {
    for &chart_type in SNAPSHOT_TYPES {
        let got = render(chart_type);
        let golden = format!(
            "{}/tests/fixtures/svg/{chart_type}.svg",
            env!("CARGO_MANIFEST_DIR")
        );
        let want = fs::read_to_string(&golden)
            .unwrap_or_else(|_| panic!("missing golden fixture: {golden}"));
        assert_eq!(got, want, "SVG output drifted for chart-type '{chart_type}'");
    }
}
