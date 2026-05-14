//! Regression tests for the built-in natal SVG (rendered when `celestial
//! render` is invoked without a `--template` flag).
//!
//! These pin down the wheel geometry — concentric rings, house spokes
//! anchored at the inner disc, house numbers inside the house band,
//! angular cusps extended through the sign band — so future renderer
//! tweaks can't silently regress the layout described in the
//! World-of-Wisdom natal-chart reference.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

fn celestial_binary() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_celestial"))
}

static CLI_LOCK: Mutex<()> = Mutex::new(());

/// Render a built-in natal SVG (no `--template`) and return its bytes.
fn render_builtin(date: &str, lat: &str, lon: &str, out_name: &str) -> String {
    let _guard = CLI_LOCK.lock().expect("CLI_LOCK poisoned");
    let out: PathBuf = std::env::temp_dir().join(out_name);
    let _ = fs::remove_file(&out);

    let output = Command::new(celestial_binary())
        .args([
            "render", "--chart-type", "natal", "--date", date, "--lat", lat, "--lon", lon, "--out",
        ])
        .arg(&out)
        .output()
        .expect("failed to spawn celestial");
    assert!(
        output.status.success(),
        "celestial render (built-in) failed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let bytes = fs::read(&out).expect("output unreadable");
    String::from_utf8(bytes).expect("output not UTF-8")
}

/// Count occurrences of a substring. Cheap enough for these tests.
fn count(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

#[test]
fn builtin_natal_has_three_wheels() {
    // Three concentric wheels — signs (RO→RI), houses (RI→RH), aspect
    // area (inside RH) — drawn with three ring boundaries. The
    // decorative RM (sign-band divider), RP (planet anchor) and RC
    // (centre disc) circles were removed to reduce graphical clutter.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_rings.svg");
    for r in ["r=\"320\"", "r=\"262\"", "r=\"238\""] {
        assert!(
            svg.contains(r),
            "expected concentric ring `{r}` missing from built-in natal SVG"
        );
    }
    for r in ["r=\"290\"", "r=\"212\"", "r=\"88\""] {
        assert!(
            !svg.contains(r),
            "removed decorative ring `{r}` should not be in built-in natal SVG"
        );
    }
}

#[test]
fn builtin_natal_planets_use_named_colors() {
    // Each named body must render in its traditional astrological hue
    // (BODY_COLORS table) — not the generic ring/planet fill. This pins
    // the colour palette against accidental regression to monochrome.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_colors.svg");
    let expected = [
        ("sun", "#d4a017"),
        ("moon", "#6b7888"),
        ("mercury", "#2c9c4f"),
        ("venus", "#d65a9e"),
        ("mars", "#c1272d"),
        ("jupiter", "#5d3f8e"),
        ("saturn", "#4a4036"),
        ("uranus", "#0085c7"),
        ("neptune", "#1ba89d"),
        ("pluto", "#7c1a1a"),
    ];
    for (name, col) in expected {
        let needle = format!("fill=\"{col}\"");
        assert!(
            svg.contains(&needle),
            "planet `{name}` colour `{col}` missing from built-in natal SVG"
        );
    }
}

#[test]
fn builtin_natal_no_glow_filter_on_planets() {
    // The Gaussian-blur glow filter caused pixelation in some SVG
    // renderers. The redesigned wheel drops the `<defs>` block and the
    // `filter="url(#glow)"` attribute entirely; planet glyphs render
    // crisp at any zoom level.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_no_glow.svg");
    assert!(
        !svg.contains("filter=\"url(#glow)\""),
        "planet glyphs must not reference the glow filter"
    );
    assert!(
        !svg.contains("<filter id=\"glow\""),
        "the glow <defs> block should be removed"
    );
}

#[test]
fn builtin_natal_moon_phase_in_subtitle_not_centre() {
    // The moon-phase disc used to sit at the wheel centre, occluding
    // aspect-line crossings. It now lives in the subtitle row instead.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_moon_phase.svg");
    // The moon-phase rect (formerly centred at CX-50, CY-22, w=100,
    // h=36) is gone — the only top-level <rect> is the page background.
    let rect_count = svg.matches("<rect").count();
    assert_eq!(
        rect_count, 1,
        "expected exactly one <rect> (page background); found {rect_count}"
    );
    // The phase information now appears in the subtitle row alongside
    // the date / location / JD.
    assert!(
        svg.contains("☽ "),
        "moon phase glyph should appear in subtitle row"
    );
}

#[test]
fn builtin_natal_numbers_all_twelve_houses() {
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_house_nums.svg");
    // The house-number text element follows the pattern: opacity=".95">N<
    // for angular cusps and opacity=".80">N< for intermediate ones. We
    // assert each numeric label (1..12) appears as a >N< text node.
    for n in 1..=12u32 {
        let needle = format!(">{n}</text>");
        assert!(
            svg.contains(&needle),
            "house number `{n}` missing from built-in natal SVG"
        );
    }
}

#[test]
fn builtin_natal_marks_four_angular_axes() {
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_axes.svg");
    for label in ["ASC", "DSC", "MC", "IC"] {
        let needle = format!(">{label}</text>");
        assert!(
            svg.contains(&needle),
            "angle label `{label}` missing from built-in natal SVG"
        );
    }
    // Exactly 4 angular-strength spokes — line stroke-width="3.0" is reserved
    // for the ASC/DSC/MC/IC cusp lines and nothing else in this template.
    let strong = count(&svg, "stroke-width=\"3.0\"");
    assert_eq!(
        strong, 4,
        "expected exactly 4 angular cusp spokes, found {strong}"
    );
}

#[test]
fn builtin_natal_is_well_formed_svg() {
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_wellformed.svg");
    assert!(svg.starts_with("<?xml"), "missing XML prolog");
    assert!(svg.contains("<svg"), "missing <svg> open tag");
    assert!(svg.contains("</svg>"), "missing </svg> close tag");
    // The inner disc (formerly drawn at r=88 with bg fill to occlude
    // aspect-line crossings) is gone — confirm no stray bg-filled disc
    // at the wheel centre remains in the output.
    assert!(
        !svg.contains("r=\"88\" fill=\"#ffffff\""),
        "centre disc should not be drawn (it has been removed along with the moon-phase badge)"
    );
}

#[test]
fn builtin_natal_angular_cusps_run_from_centre_to_outer_ring() {
    // Each angular spoke (ASC/DSC/MC/IC) now runs from the wheel
    // centre to the outer wheel radius (RO=320) → span = 320 px. The
    // four spokes meet at (CX, CY) so the chart shows a full ASC-DSC
    // and MC-IC axis cross without a disc occluding the centre.
    let svg = render_builtin("2000-01-01", "0", "0", "natal_angular_span.svg");
    let lines: Vec<&str> = svg
        .lines()
        .filter(|l| l.contains("stroke-width=\"3.0\""))
        .collect();
    assert_eq!(
        lines.len(),
        4,
        "expected 4 stroke-width=3.0 spokes; got {}",
        lines.len()
    );
    let expected_span = 320.0; // RO
    for ln in &lines {
        let span = max_axis_span(ln);
        assert!(
            (span - expected_span).abs() < 5.0,
            "angular cusp spoke spans {span:.1}px (expected ≈ {expected_span:.1}, RO): {ln}"
        );
    }
}

#[test]
fn builtin_natal_intermediate_cusps_run_from_centre_to_sign_band() {
    // Non-angular cusps (houses 2/3/5/6/8/9/11/12) start at the wheel
    // centre and end at the sign-band inner ring (RI=262), so they
    // don't intrude on the sign glyphs. Span = 262 px.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_intermediate.svg");
    let lines: Vec<&str> = svg
        .lines()
        .filter(|l| l.contains("stroke-width=\"1.2\"") && l.contains("opacity=\".50\""))
        .collect();
    assert_eq!(
        lines.len(),
        8,
        "expected 8 intermediate cusp spokes; got {}",
        lines.len()
    );
    let expected_span = 262.0; // RI
    for ln in &lines {
        let span = max_axis_span(ln);
        assert!(
            (span - expected_span).abs() < 10.0,
            "intermediate cusp spoke spans {span:.1}px (expected ≈ {expected_span:.1}, RI): {ln}"
        );
    }
}

#[test]
fn builtin_natal_house_numbers_sit_inside_house_band() {
    // House numbers must land at radius (RH + RI)/2 = 250 px from the
    // wheel centre (CX=450, CY=490) — i.e. inside the house band, clear
    // of both the planet ring (RP=212) and the sign band (RI=262 outward).
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_num_radii.svg");
    let centre = (450.0_f64, 490.0_f64);
    let expected_r = (238.0_f64 + 262.0_f64) / 2.0; // (RH + RI)/2
    let mut count_inside = 0;
    for ln in svg.lines() {
        if !ln.contains("font-weight=\"700\"") && !ln.contains("font-weight=\"800\"") {
            continue;
        }
        if !ln.contains("font-size=\"12\"") && !ln.contains("font-size=\"13\"") {
            continue;
        }
        let Some(x) = extract_attr(ln, "x") else {
            continue;
        };
        let Some(y) = extract_attr(ln, "y") else {
            continue;
        };
        let r = ((x - centre.0).powi(2) + (y - centre.1).powi(2)).sqrt();
        if (r - expected_r).abs() < 5.0 {
            count_inside += 1;
        }
    }
    assert_eq!(
        count_inside, 12,
        "expected 12 house-number labels at radius ≈ {expected_r:.0} px from centre, found {count_inside}"
    );
}

fn extract_attr(line: &str, attr: &str) -> Option<f64> {
    let key = format!("{attr}=\"");
    let i = line.find(&key)? + key.len();
    let j = i + line[i..].find('"')?;
    line[i..j].parse().ok()
}

/// Euclidean length of a `<line>` element. Used to verify a cusp spoke
/// runs the radial distance between two known wheel radii.
fn max_axis_span(line: &str) -> f64 {
    let x1 = extract_attr(line, "x1").unwrap_or(0.0);
    let y1 = extract_attr(line, "y1").unwrap_or(0.0);
    let x2 = extract_attr(line, "x2").unwrap_or(0.0);
    let y2 = extract_attr(line, "y2").unwrap_or(0.0);
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}
