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
    let _guard = CLI_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let out: PathBuf = std::env::temp_dir().join(out_name);
    let _ = fs::remove_file(&out);

    // Natal charts now require an explicit birth time + timezone. These
    // geometry tests don't care about the instant, so pin a fixed UTC time.
    let date = if date.contains(' ') {
        date.to_string()
    } else {
        format!("{date} 12:00")
    };

    let output = Command::new(celestial_binary())
        .args([
            "render",
            "--chart-type",
            "natal",
            "--date",
            &date,
            "--tz",
            "UTC",
            "--lat",
            lat,
            "--lon",
            lon,
            "--out",
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
fn builtin_natal_emits_glyph_symbol_defs_exactly_once() {
    // The `<defs>` block carrying the embedded `<symbol id="g-XXXX">`
    // entries must be emitted exactly once per chart so each glyph is
    // defined a single time and referenced via `<use href="#g-XXXX">`
    // from every wheel and legend position. Multiple defs blocks
    // would bloat the SVG and cause `id` collisions in some renderers.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_defs.svg");
    assert_eq!(
        svg.matches("<defs>").count(),
        1,
        "<defs> block should appear exactly once"
    );
    // 25 entries from `GLYPH_PATHS` (12 zodiac + Sun, Moon, Mercury,
    // Venus, Earth-stub, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto,
    // Mean Node, Chiron) plus the synthesised South Node (`g-260B`,
    // emitted by `emit_defs` as a vertically-flipped `<use>` of the
    // North Node) → 26 unique `<symbol id="g-XXXX">` defs.
    let symbol_count = svg.matches("<symbol id=\"g-").count();
    assert_eq!(
        symbol_count, 26,
        "expected 26 <symbol> definitions (25 from glyph_paths + South Node); got {symbol_count}"
    );
}

#[test]
fn builtin_natal_wheel_uses_vector_paths_for_zodiac_glyphs() {
    // The twelve sign-band glyphs on the wheel are now `<use>` elements
    // referencing the embedded `<symbol>` defs. Every zodiac code-point
    // (U+2648..U+2653) must have at least one `<use href="#g-264X">`
    // emission — otherwise the wheel is rendering text fallback.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_uses.svg");
    for cp in 0x2648_u32..=0x2653 {
        let needle = format!(r##"<use href="#g-{cp:04X}""##);
        let count = svg.matches(&needle).count();
        assert!(
            count >= 1,
            "expected at least one <use> reference for glyph U+{cp:04X}; got {count}"
        );
    }
}

#[test]
fn builtin_natal_glyphs_use_text_presentation_selector() {
    // Every astrological glyph (zodiac signs + planet symbols) must
    // carry the trailing `U+FE0E` text-presentation variation selector,
    // otherwise SVG renderers substitute the colour-emoji form (Noto
    // Color Emoji et al.) which looks chunky next to the surrounding
    // text. Counting glyphs paired with VS15 confirms the selector is
    // applied to all of them.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_text_vs.svg");
    let glyphs = [
        "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓", // zodiac
        "☉", "☽", "☿", "♀", "♂", "♃", "♄", "♅", "♆", "♇", "☊", "⚷", // planets
    ];
    for g in glyphs {
        let bare = format!(">{g}<");
        let paired = format!(">{g}\u{FE0E}<");
        let bare_only = svg.matches(&bare).count();
        let paired_count = svg.matches(&paired).count();
        // The `bare` pattern matches both `>{g}<` and `>{g}\u{FE0E}<` (it
        // is a prefix match). Real bare emissions = bare_only − paired.
        let unprotected = bare_only.saturating_sub(paired_count);
        assert_eq!(
            unprotected, 0,
            "glyph `{g}` appears without `U+FE0E` text-presentation \
             selector in the rendered SVG ({paired_count} protected, \
             {bare_only} total occurrences)"
        );
    }
}

#[test]
fn builtin_natal_legend_signs_use_tspan_colour_and_symbol_font() {
    // Table rows (planet legend, angles, houses, arabic parts) wrap the
    // trailing zodiac glyph in a `<tspan>` carrying the symbol-font
    // stack and the element colour, so glyphs in tables render with
    // the same fidelity as those on the wheel.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_table_signs.svg");
    let tspans: Vec<&str> = svg.lines().filter(|l| l.contains("<tspan")).collect();
    assert!(
        tspans.len() >= 12,
        "expected ≥ 12 sign tspans across the legend tables, got {}",
        tspans.len()
    );
    for needle in [
        "fill=\"#c1272d\"",
        "fill=\"#5a7a30\"",
        "fill=\"#c4a017\"",
        "fill=\"#1a5fb4\"",
    ] {
        assert!(
            tspans.iter().any(|t| t.contains(needle)),
            "no table sign tspan painted in element colour `{needle}`"
        );
    }
    assert!(
        tspans.iter().all(|t| t.contains("Segoe UI Symbol")),
        "every legend tspan must use the symbol-font stack"
    );
    // And the chart must contain zero `font-family="serif"` declarations —
    // every glyph (sign + planet) now routes through the symbol stack.
    assert!(
        !svg.contains("font-family=\"serif\""),
        "no glyph should fall back to plain `serif`"
    );
}

#[test]
fn builtin_natal_signs_use_element_colors_and_symbol_font() {
    // Zodiac glyphs render in their classical element colour (fire=red
    // c1272d, earth=green 5a7a30, air=gold c4a017, water=blue 1a5fb4)
    // and prefer dedicated symbol fonts so the U+2648..U+2653 glyphs
    // come out as vectors rather than chunky bitmap-style fallbacks.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_sign_colors.svg");
    let element_colors = ["#c1272d", "#5a7a30", "#c4a017", "#1a5fb4"];
    for col in element_colors {
        let needle = format!("fill=\"{col}\"");
        let count = svg.matches(&needle).count();
        assert!(
            count >= 3,
            "expected ≥ 3 sign glyphs in element colour `{col}`, got {count}"
        );
    }
    assert!(
        svg.contains("Segoe UI Symbol"),
        "sign glyph font stack must lead with a dedicated symbol font"
    );
    // The legacy generic serif fallback alone must not be the planet/sign
    // family — confirm at least one sign uses the rich font stack.
    assert!(
        svg.contains("font-family=\"'Segoe UI Symbol'"),
        "sign glyphs must use the rich symbol-font stack, not generic serif"
    );
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
    let centre = (450.0_f64, 424.0_f64);
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

#[test]
fn builtin_natal_renders_symbol_legend_table() {
    // The bottom of the chart now carries a "Symbol reference" table
    // pairing every wheel glyph with a plain-English label, replacing
    // the Arabic-Parts and Solar-Cycle blocks. The underlying values
    // remain in the JSON context for custom templates, but the
    // built-in renderer no longer surfaces them as tables.
    let svg = render_builtin("2000-01-01", "48.8566", "2.3522", "natal_legend.svg");
    assert!(
        svg.contains("Symbol reference"),
        "header `Symbol reference` missing"
    );
    for sub in ["Planets", "Signs", "Angles", "Aspects"] {
        let needle = format!(">{sub}<");
        assert!(svg.contains(&needle), "sub-heading `{sub}` missing");
    }
    // A representative cross-section of the descriptions: one body, one
    // sign, one angle, plus the retrograde marker, plus a couple of
    // aspect rows so the new Aspects column is pinned.
    for desc in [
        "Sun",
        "Aries",
        "Ascendant — rising sign",
        "Midheaven — culminating point",
        "Retrograde motion",
        "Conjunction (0°)",
        "Square (90°)",
        "Sesquiquadrate (135°)",
    ] {
        assert!(svg.contains(desc), "legend row `{desc}` missing");
    }
    // The replaced tables must NOT appear in the built-in SVG.
    assert!(
        !svg.contains(">Arabic Parts<"),
        "Arabic Parts table should be removed from the built-in chart"
    );
    assert!(
        !svg.contains(">Solar Cycle<"),
        "Solar Cycle table should be removed from the built-in chart"
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
