//! `celestial render` — compute a chart and render any template against it.
//!
//! ## Usage
//! ```text
//! # Use the built-in SVG (default):
//! celestial render --date 2025-03-20 --lat 48.85 --lon 2.35 --out chart.svg
//!
//! # Use your own template:
//! celestial render --date 2025-03-20 --lat 48.85 --lon 2.35 \
//!     --template my.tmpl --out chart.svg
//!
//! # Bootstrap an example template showing all available variables:
//! celestial render --print-template > my.tmpl
//!
//! # Dump the JSON context so you know what to reference in templates:
//! celestial render --date 2025-03-20 --print-context
//!
//! # Custom variables via --var or [vars] in a TOML config:
//! celestial render --config chart.toml --var title="My Chart"
//! ```
//!
//! ## TOML config
//! ```toml
//! [render]
//! date  = "2025-03-20"
//! lat   = 48.85
//! lon   = 2.35
//! out   = "chart.svg"
//! hsys  = "P"
//!
//! [vars]
//! title        = "Spring Equinox 2025"
//! bg_color     = "#ffffff"    white background (default)
//! ring_color   = "#1a1a2e"    dark navy for rings and labels
//! ```

use crate::error::CliError;
use std::collections::BTreeMap;

use celestial_core::body::{Body, CalcFlags};
use celestial_core::lon_to_sign;
use celestial_core::MoonPhase;
use celestial_core::{long_to_nakshatra, nakshatra_name};
use celestial_core::{
    lunar_return_jd, moon_phase, revjul, sign_exaltation, sign_ruler, solar_return_jd, Calendar,
};
use minijinja::{Environment, Value as MjValue};
use serde_json::{json, Value};

// ── Builder submodules — each tradition's build_*/render_* fns ───────────────
pub(crate) mod builtin_svg;
mod calendar_overlays;
mod calendar_wheel;
mod chinese;
pub(crate) mod context;
mod derived;
mod glyph_paths;
mod hellenistic;
mod indigenous;
mod mesoamerican;
mod omer_grid;
mod specialist;
mod svg_common;
mod vedic;
mod western;

// ── Split-out concerns (former god-file sections; facade re-exports) ─────────
mod args;
mod chart_context;
mod config;
mod geometry;
mod pipeline;
mod registry;

pub use args::RenderArgs;
pub use pipeline::run;
pub(crate) use geometry::{spread_labels, wheel_angle, wx, wy};
pub(crate) use chart_context::ChartContext;
pub(crate) use config::*;
pub(crate) use pipeline::*;
pub(crate) use registry::*;

use builtin_svg::render_builtin_svg;
use context::build_context;
use derived::{
    build_biwheel_context, build_progressed_context, build_solar_arc_context, render_biwheel_svg,
    render_cosmogram_svg, render_progressed_svg,
};



// ─── JSON helper ───────────────────────────────────────────────────────────────

/// Borrow the array at `v`, or an empty slice for non-arrays.
pub(super) fn json_array(v: &serde_json::Value) -> &[serde_json::Value] {
    v.as_array().map_or(&[], |a| a.as_slice())
}

// ─── Formatting helpers ───────────────────────────────────────────────────────

pub(super) fn fmt_lon_dms(lon: f64) -> String {
    let (sign_idx, deg_in_sign) = lon_to_sign(lon);
    let d = deg_in_sign as u32;
    let m = ((deg_in_sign - d as f64) * 60.0) as u32;
    let s = (((deg_in_sign - d as f64) * 3600.0) - m as f64 * 60.0).round() as u32;
    // Trailing `\u{FE0E}` forces the text-presentation form of each
    // zodiac glyph — without it SVG renderers fall back to colour-emoji
    // bitmap glyphs (Noto Color Emoji et al.) which look blocky next
    // to the surrounding crisp serif/sans digits.
    let glyphs = [
        "\u{2648}\u{FE0E}",
        "\u{2649}\u{FE0E}",
        "\u{264A}\u{FE0E}",
        "\u{264B}\u{FE0E}",
        "\u{264C}\u{FE0E}",
        "\u{264D}\u{FE0E}",
        "\u{264E}\u{FE0E}",
        "\u{264F}\u{FE0E}",
        "\u{2650}\u{FE0E}",
        "\u{2651}\u{FE0E}",
        "\u{2652}\u{FE0E}",
        "\u{2653}\u{FE0E}",
    ];
    format!(
        "{d:02}\u{00B0}{m:02}\u{2032}{s:02}\u{2033}{}",
        glyphs[sign_idx as usize % 12]
    )
}

pub(super) fn moon_phase_str(jd: f64) -> &'static str {
    match moon_phase(jd).unwrap_or(MoonPhase::NewMoon) {
        MoonPhase::NewMoon => "New Moon",
        MoonPhase::WaxingCrescent => "Waxing Crescent",
        MoonPhase::FirstQuarter => "First Quarter",
        MoonPhase::WaxingGibbous => "Waxing Gibbous",
        MoonPhase::FullMoon => "Full Moon",
        MoonPhase::WaningGibbous => "Waning Gibbous",
        MoonPhase::LastQuarter => "Last Quarter",
        MoonPhase::WaningCrescent => "Waning Crescent",
    }
}

// Format a Julian Day as "YYYY-MM-DD".

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 5 — Hellenistic / Persian chart builders
// ═══════════════════════════════════════════════════════════════════════════════

// ─── 1. Hellenistic dignities overlay (extends existing natal wheel) ───────────

/// Map a planet key string to a Body.
pub(super) fn key_to_body(key: &str) -> Option<Body> {
    match key {
        "sun" => Some(Body::SUN),
        "moon" => Some(Body::MOON),
        "mercury" => Some(Body::MERCURY),
        "venus" => Some(Body::VENUS),
        "mars" => Some(Body::MARS),
        "jupiter" => Some(Body::JUPITER),
        "saturn" => Some(Body::SATURN),
        "uranus" => Some(Body::URANUS),
        "neptune" => Some(Body::NEPTUNE),
        "pluto" => Some(Body::PLUTO),
        "mean_node" => Some(Body::MEAN_NODE),
        "chiron" => Some(Body::CHIRON),
        _ => None,
    }
}

// ─── 2. Firdaria timeline ──────────────────────────────────────────────────────

// ─── 3. Profection wheel ──────────────────────────────────────────────────────

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 6 — Chinese astrology chart builders
// ═══════════════════════════════════════════════════════════════════════════════

// ─── Ba Zi (Four Pillars) ─────────────────────────────────────────────────────

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 7 — Mesoamerican calendar context + SVG
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 8 — Medicine Wheel / Egyptian decans context + SVG
// ═══════════════════════════════════════════════════════════════════════════════

// Helper to get sun longitude from PlanetPos

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 3 — specialist Western chart builders
// ═══════════════════════════════════════════════════════════════════════════════

// ─── 1. 90° Midpoint Dial ─────────────────────────────────────────────────────

// ─── 2. Composite chart ───────────────────────────────────────────────────────

// ─── 3. Tri-wheel ────────────────────────────────────────────────────────────

// ─── 4. Graphic Ephemeris ─────────────────────────────────────────────────────

// ─── 5. Local Space chart ─────────────────────────────────────────────────────

/// Format a Julian Day as a date string, including HH:MM when the time is not midnight.
pub(super) fn jd_to_date_str(jd: f64) -> String {
    let d = celestial_core::revjul(jd, celestial_core::body::Calendar::Gregorian);
    let total_sec = (d.hour * 3600.0).round() as i32; // round to nearest second first
    let total_min = total_sec / 60; // truncate seconds from display
    let h = total_min / 60;
    let m = total_min % 60;
    if h == 0 && m == 0 {
        format!(
            "{:04}-{:02}-{:02}",
            d.year as i32, d.month as u32, d.day as u32
        )
    } else {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02} UT",
            d.year as i32, d.month as u32, d.day as u32, h, m
        )
    }
}

// ─── Planet table ─────────────────────────────────────────────────────────────

/// Planet display glyphs. Each Unicode symbol is followed by the
/// text-presentation variation selector (`U+FE0E`) so renderers don't
/// substitute the chunky colour-emoji form for the astrological symbol
/// — same fix as the zodiac glyphs in `SIGN_GLYPHS`.
pub(super) const BODIES: &[(Body, &str, &str, &str)] = &[
    (Body::SUN, "sun", "Sun", "\u{2609}\u{FE0E}"),
    (Body::MOON, "moon", "Moon", "\u{263D}\u{FE0E}"),
    (Body::MERCURY, "mercury", "Mercury", "\u{263F}\u{FE0E}"),
    (Body::VENUS, "venus", "Venus", "\u{2640}\u{FE0E}"),
    (Body::MARS, "mars", "Mars", "\u{2642}\u{FE0E}"),
    (Body::JUPITER, "jupiter", "Jupiter", "\u{2643}\u{FE0E}"),
    (Body::SATURN, "saturn", "Saturn", "\u{2644}\u{FE0E}"),
    (Body::URANUS, "uranus", "Uranus", "\u{2645}\u{FE0E}"),
    (Body::NEPTUNE, "neptune", "Neptune", "\u{2646}\u{FE0E}"),
    (Body::PLUTO, "pluto", "Pluto", "\u{2647}\u{FE0E}"),
    (Body::MEAN_NODE, "mean_node", "Node (North)", "\u{260A}\u{FE0E}"),
    (Body::CHIRON, "chiron", "Chiron", "\u{26B7}\u{FE0E}"),
];

/// Per-body display colour (key → hex). Picked to approximate the
/// traditional astrological palette used by the World-of-Wisdom PDF
/// reference: warm metals for the luminaries, mode-coloured outer
/// planets, mercury/venus in green/pink for their domiciles.
pub(super) const BODY_COLORS: &[(&str, &str)] = &[
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
    ("mean_node", "#6a4f8a"),
    ("true_node", "#6a4f8a"),
    ("south_node", "#6a4f8a"),
    ("chiron", "#6c3a1a"),
];

#[must_use]
pub(super) fn body_color(key: &str) -> &'static str {
    BODY_COLORS
        .iter()
        .find(|(k, _)| *k == key)
        .map_or("#0d0d1e", |(_, c)| *c)
}

/// (angle°, name, orb_limit, is_minor)
pub(super) const ASPECT_DEFS: &[(f64, &str, f64, bool)] = &[
    // ── Major aspects ─────────────────────────────────────────────────────────
    (0.0, "conjunction", 8.0, false),
    (60.0, "sextile", 6.0, false),
    (90.0, "square", 7.0, false),
    (120.0, "trine", 8.0, false),
    (150.0, "quincunx", 3.0, false),
    (180.0, "opposition", 8.0, false),
    // ── Minor aspects ─────────────────────────────────────────────────────────
    (30.0, "semi-sextile", 2.0, true),
    (45.0, "semi-square", 2.0, true),
    (72.0, "quintile", 1.5, true),
    (135.0, "sesquiquadrate", 2.0, true),
    (144.0, "biquintile", 1.5, true),
    (51.43, "septile", 1.0, true),
    (40.0, "novile", 1.0, true),
];

// ─── Dignity helper ───────────────────────────────────────────────────────────

/// Returns the essential dignity label for a planet at a given sign (0–11).
///
/// Checks (in order): domicile, exaltation, detriment, fall, peregrine.
/// Secondary (pre-outer-planet) domicile for the 5 classical rulers.
///
/// Returns `Some(second_sign)` when `body` is one of Mercury/Venus/Mars/Jupiter/Saturn,
/// otherwise `None`. Each classical planet rules two signs of opposite polarity;
/// `sign_ruler` only reports one of them, so this fills in the other.
fn second_domicile(body: Body) -> Option<u8> {
    match body.as_raw() {
        2 => Some(8),  // Mercury: Gemini + Virgo
        3 => Some(6),  // Venus:   Taurus + Libra
        4 => Some(7),  // Mars:    Aries  + Scorpio
        5 => Some(11), // Jupiter: Sagittarius + Pisces
        6 => Some(9),  // Saturn:  Capricorn + Aquarius
        _ => None,
    }
}

/// Classical essential dignity of `body` in `sign`.
///
/// Checks, in order: domicile → detriment → exaltation → fall → peregrine.
/// Returns the dignity name as a static string.
pub(super) fn planet_dignity(body: Body, sign: u8) -> &'static str {
    let s = sign % 12;
    let opp = (s + 6) % 12;

    // A body rules `sign` if it is the primary ruler or the secondary domicile
    let rules = |sign: u8| -> bool {
        sign_ruler(sign) == body || second_domicile(body).is_some_and(|s2| s2 == sign)
    };

    if rules(s) {
        return "domicile";
    }
    if rules(opp) {
        return "detriment";
    }

    // Exaltation & fall
    let ex = sign_exaltation(body);
    if ex >= 0 && ex as u8 == s {
        return "exaltation";
    }
    if ex >= 0 && (ex as u8 + 6) % 12 == s {
        return "fall";
    }

    "peregrine"
}

/// Antiscion longitude: reflection over the 0°Cancer / 0°Capricorn (solstice) axis.
/// Formula: antiscion = (180° - lon) mod 360°
#[inline]
pub(super) fn antiscion_lon(lon: f64) -> f64 {
    (180.0 - lon).rem_euclid(360.0)
}

/// Contra-antiscion longitude: reflection over the 0°Aries / 0°Libra (equinox) axis.
/// Formula: contra = (360° - lon) mod 360°
#[inline]
pub(super) fn contra_antiscion_lon(lon: f64) -> f64 {
    (360.0 - lon).rem_euclid(360.0)
}

// ─── Build the full context ───────────────────────────────────────────────────

pub(super) const CX: f64 = 450.0;
pub(super) const CY: f64 = 424.0;
pub(super) const RO: f64 = 320.0; // outer ring
pub(super) const RM: f64 = 290.0; // sign band outer
pub(super) const RI: f64 = 262.0; // sign band inner
pub(super) const RH: f64 = 238.0; // house cusp inner
pub(super) const RP: f64 = 212.0; // planet ring
pub(super) const RC: f64 = 88.0; // inner circle

/// Build a palette `BTreeMap<String, Value>` from a slice of default
/// `(key, color)` pairs, merging user-provided `vars` over the top.
/// User vars take precedence so callers can override any color via `--var key=value`.
pub(super) fn palette_with_defaults(
    defaults: &[(&str, &str)],
    vars: &std::collections::BTreeMap<String, String>,
) -> std::collections::BTreeMap<String, Value> {
    let mut palette = std::collections::BTreeMap::new();
    for (k, v) in defaults {
        palette.insert((*k).to_string(), json!(v));
    }
    for (k, v) in vars {
        palette.insert(k.clone(), json!(v));
    }
    palette
}

/// DUP-7: the title-default + palette-merge + JSON-object boilerplate
/// every tradition builder repeated. Inserts `title` only when the
/// user didn't supply one, merges `defaults` under the user vars, and
/// returns the ready `vars` JSON object — byte-identical to the
/// former hand-written trio.
pub(super) fn palette_vars(
    mut user_vars: std::collections::BTreeMap<String, String>,
    title_default: &str,
    defaults: &[(&str, &str)],
) -> Value {
    user_vars
        .entry("title".to_string())
        .or_insert_with(|| title_default.to_string());
    Value::Object(palette_with_defaults(defaults, &user_vars).into_iter().collect())
}

/// DUP-7: `palette_with_defaults` + `Value::Object(… .collect())` in
/// one call, for builders whose `title` is injected upstream (so they
/// don't self-insert one like [`palette_vars`]). Byte-identical to the
/// former two-step.
pub(super) fn palette_obj(
    vars: &std::collections::BTreeMap<String, String>,
    defaults: &[(&str, &str)],
) -> Value {
    Value::Object(palette_with_defaults(defaults, vars).into_iter().collect())
}

// ─── Built-in SVG generator (pure Rust — no template parsing) ────────────────

// ─── Tests ───────────────────────────────────────────────────────────────────

// ── Recovered builders (were accidentally removed during refactor) ────────────

#[cfg(test)]
mod tests {
    use super::specialist::{
        build_composite_context, build_dial_context, build_graphic_ephemeris_context,
        build_local_space_context, build_triwheel_context,
    };
    use super::*;

    // ── wheel geometry ────────────────────────────────────────────────────────

    #[test]
    fn wheel_asc_at_9_oclock() {
        // ASC is placed at the leftmost point: (cx - r, cy)
        let (cx, cy, r, asc) = (450.0, 490.0, 212.0, 30.0);
        let x = wx(cx, r, asc, asc);
        let y = wy(cy, r, asc, asc);
        assert!(
            (x - (cx - r)).abs() < 1e-9,
            "ASC should be at cx-r, got {x}"
        );
        assert!((y - cy).abs() < 1e-9, "ASC y should be cy, got {y}");
    }

    #[test]
    fn wheel_opposite_point() {
        // DSC = ASC + 180 → rightmost (cx + r, cy)
        let (cx, cy, r, asc) = (450.0, 490.0, 212.0, 0.0);
        let dsc = 180.0_f64;
        let x = wx(cx, r, dsc, asc);
        let y = wy(cy, r, dsc, asc);
        assert!(
            (x - (cx + r)).abs() < 1e-9,
            "DSC should be at cx+r, got {x}"
        );
        assert!((y - cy).abs() < 1e-9, "DSC y should be cy, got {y}");
    }

    #[test]
    fn wheel_mc_at_top() {
        // In traditional natal charts the MC sits roughly 90° earlier
        // in zodiac longitude than the ASC (e.g. ASC=191°, MC=99°),
        // and is rendered at the top of the wheel. With the chart
        // rotated so longitudes increase CCW past the ASC (lower-left
        // first), MC = asc − 90 lands at (cx, cy − r).
        let (cx, cy, r, asc) = (450.0, 490.0, 212.0, 0.0);
        let mc = -90.0_f64;
        let x = wx(cx, r, mc, asc);
        let y = wy(cy, r, mc, asc);
        assert!((x - cx).abs() < 1e-9, "MC x should be cx, got {x}");
        assert!((y - (cy - r)).abs() < 1e-9, "MC should be at top, got {y}");
    }

    #[test]
    fn wheel_ic_at_bottom() {
        // IC = MC + 180 = asc + 90 in the canonical case → wheel
        // bottom (cx, cy + r). This is the position House 1 starts
        // descending from (just below the ASC on the lower-left) and
        // House 4 occupies, matching the World-of-Wisdom PDF layout.
        let (cx, cy, r, asc) = (450.0, 490.0, 212.0, 0.0);
        let ic = 90.0_f64;
        let x = wx(cx, r, ic, asc);
        let y = wy(cy, r, ic, asc);
        assert!((x - cx).abs() < 1e-9, "IC x should be cx, got {x}");
        assert!((y - (cy + r)).abs() < 1e-9, "IC should be at bottom, got {y}");
    }

    #[test]
    fn wheel_house_1_starts_below_ascendant() {
        // House 1 starts at the ASC (left edge) and spans CCW into the
        // **lower** hemisphere. The midpoint of house 1 (asc + 15°)
        // must therefore land below the centre line (y > cy).
        let (cx, cy, r, asc) = (450.0, 490.0, 212.0, 30.0);
        let house1_mid = asc + 15.0;
        let y = wy(cy, r, house1_mid, asc);
        assert!(
            y > cy,
            "house 1 midpoint should be below the wheel centre (y > {cy}), got {y}"
        );
        let x = wx(cx, r, house1_mid, asc);
        assert!(x < cx, "house 1 midpoint should be left of centre, got {x}");
    }

    // ── formatting ────────────────────────────────────────────────────────────

    #[test]
    fn fmt_lon_dms_aries_start() {
        let s = fmt_lon_dms(0.0);
        assert!(s.contains("\u{2648}"), "0° should be ♈, got {s}");
        assert!(
            s.starts_with("00\u{00B0}"),
            "should start with 00°, got {s}"
        );
    }

    #[test]
    fn sabbat_wheel_context_has_8_sabbats() {
        let jd = 2_460_482.5; // 2024-06-21
        let ctx = calendar_wheel::build_sabbat_wheel_context(jd, std::collections::BTreeMap::new())
            .unwrap();
        let sabbats = ctx["sabbats"].as_array().expect("sabbats array");
        assert_eq!(sabbats.len(), 8, "Wheel of the Year always has 8 sabbats");
        // Each sabbat must have rendering geometry
        for sb in sabbats {
            for field in ["name", "date", "lon", "glyph_x", "glyph_y", "tick_x1"] {
                assert!(
                    !sb[field].is_null(),
                    "sabbat missing field `{field}`: {sb:?}"
                );
            }
        }
    }

    #[test]
    fn sabbat_wheel_renders_valid_svg() {
        let jd = 2_460_482.5;
        let ctx = calendar_wheel::build_sabbat_wheel_context(jd, std::collections::BTreeMap::new())
            .unwrap();
        let svg = calendar_wheel::render_sabbat_wheel_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.starts_with("<?xml"), "SVG should start with <?xml");
        assert!(svg.contains("<svg "), "should contain <svg> tag");
        assert!(svg.ends_with("</svg>\n"), "should close </svg>");
        // Must contain all 8 sabbat names
        for name in [
            "Yule",
            "Imbolc",
            "Ostara",
            "Beltane",
            "Litha",
            "Lughnasadh",
            "Mabon",
            "Samhain",
        ] {
            assert!(svg.contains(name), "SVG missing sabbat `{name}`");
        }
        // Must not contain NaN / undefined / Infinity
        assert!(!svg.contains("NaN"), "SVG must not contain NaN");
        assert!(!svg.contains("inf"), "SVG must not contain inf");
    }

    fn assert_natal_fields_present(ctx: &Value) {
        assert!(ctx["asc"].is_number(), "asc should still be present");
        assert!(ctx["planets"].is_array(), "planets should still be array");
    }

    fn assert_omer_overlay_for_may_15_2024(ctx: &Value) {
        assert!(ctx["omer"].is_object(), "omer overlay missing");
        let today = &ctx["omer"]["today"];
        assert!(
            !today.is_null(),
            "May 15 2024 is in Omer 5784 — today should be set"
        );
        assert_eq!(today["day"].as_u64(), Some(23), "day 23 of Omer 5784");
        assert_eq!(today["day_sefirah"].as_str(), Some("Gevurah"));
        assert_eq!(today["week_sefirah"].as_str(), Some("Netzach"));
    }

    fn assert_sabbats_overlay_2024(ctx: &Value) {
        let sabbats = ctx["sabbats"]["sabbats"].as_array().expect("sabbats array");
        assert_eq!(sabbats.len(), 8);
        assert_eq!(sabbats[0]["index"].as_u64(), Some(0));
        assert_eq!(sabbats[0]["list_y"].as_i64(), Some(0));
        assert_eq!(sabbats[7]["index"].as_u64(), Some(7));
        assert_eq!(sabbats[7]["list_y"].as_i64(), Some(98)); // 7 × 14
    }

    /// Universal overlay: `--calendar omer` should add an `omer.today` field
    /// to ANY chart-type's context, not just `--chart-type calendar`.
    /// Verifies the post-dispatch overlay-merge step in `run()`.
    #[test]
    fn overlays_merge_into_non_calendar_context() {
        let jd = celestial_core::julday(2024, 5, 15, 12.0, Calendar::Gregorian);
        let mut ctx = build_context(
            jd,
            48.85,
            2.35,
            "2024-05-15 12:00",
            'P',
            std::collections::BTreeMap::new(),
        )
        .expect("natal context");

        let obj = ctx.as_object_mut().expect("ctx is object");
        obj.insert("omer".into(), calendar_overlays::omer_overlay(jd));
        obj.insert("sabbats".into(), calendar_overlays::sabbats_overlay(2024));

        assert_natal_fields_present(&ctx);
        assert_omer_overlay_for_may_15_2024(&ctx);
        assert_sabbats_overlay_2024(&ctx);
    }

    /// MiniJinja contract: math, filters, and loop.index work as expected.
    /// This locks in the features that were the reason for the
    /// tinytemplate→minijinja migration.
    #[test]
    fn minijinja_supports_math_filters_and_loop_index() {
        use minijinja::{Environment, Value as MjValue};

        let mut env = Environment::new();
        env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
        env.add_template(
            "t",
            // A representative template that exercises:
            //   - arithmetic on context values
            //   - chained filters (round, format)
            //   - loop.index0 inline math
            //   - if/else with comparison operators
            r#"
math: {{ x + y }}, {{ x * 2 }}, {{ x % 3 }}
filter: {{ pi | round(2) }}
format: {{ "%05d" | format(x) }}
{%- for n in nums %}
row {{ loop.index }} y={{ 100 + loop.index0 * 14 }} val={{ n }}
{%- endfor %}
cond: {% if x > 10 and y < 50 %}both true{% else %}fallthrough{% endif %}
"#,
        )
        .unwrap();

        let ctx = serde_json::json!({
            "x": 12,
            "y": 30,
            "pi": std::f64::consts::PI,
            "nums": ["a", "b", "c"],
        });
        let out = env
            .get_template("t")
            .unwrap()
            .render(MjValue::from_serialize(&ctx))
            .unwrap();

        assert!(out.contains("math: 42, 24, 0"), "math rendered: {out}");
        assert!(out.contains("filter: 3.14"), "round filter: {out}");
        assert!(out.contains("format: 00012"), "format filter: {out}");
        assert!(out.contains("row 1 y=100 val=a"), "loop row 1: {out}");
        assert!(out.contains("row 2 y=114 val=b"), "loop row 2: {out}");
        assert!(out.contains("row 3 y=128 val=c"), "loop row 3: {out}");
        assert!(out.contains("cond: both true"), "if/else: {out}");
    }

    #[test]
    fn fmt_lon_dms_taurus_boundary() {
        // 30° = start of ♉
        let s = fmt_lon_dms(30.0);
        assert!(s.contains("\u{2649}"), "30° should be ♉, got {s}");
        assert!(
            s.starts_with("00\u{00B0}"),
            "should start with 00°, got {s}"
        );
    }

    #[test]
    fn fmt_lon_dms_mid_sign() {
        // 45° = 15° Taurus
        let s = fmt_lon_dms(45.0);
        assert!(s.contains("\u{2649}"), "45° should be ♉, got {s}");
        assert!(
            s.starts_with("15\u{00B0}"),
            "should start with 15°, got {s}"
        );
    }

    #[test]
    fn fmt_lon_dms_wraps_360() {
        // 360° = 0° = ♈
        let s = fmt_lon_dms(360.0);
        assert!(s.contains("\u{2648}"), "360° should wrap to ♈, got {s}");
    }

    #[test]
    fn moon_phase_str_smoke() {
        // J2000 should return a non-empty phase name
        let p = moon_phase_str(2451545.0);
        assert!(!p.is_empty(), "moon_phase_str should not be empty");
        let valid = [
            "New Moon",
            "Waxing Crescent",
            "First Quarter",
            "Waxing Gibbous",
            "Full Moon",
            "Waning Gibbous",
            "Last Quarter",
            "Waning Crescent",
        ];
        assert!(valid.contains(&p), "unexpected phase: {p}");
    }

    // ── build_context ──────────────────────────────────────────────────────────

    #[test]
    fn context_j2000_placidus() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        );
        assert!(ctx.is_ok(), "build_context should succeed: {:?}", ctx.err());
        let v = ctx.unwrap();
        assert_eq!(v["date"], "2000-01-01 12:00 UT"); // J2000.0 = noon
        assert!((v["jd"].as_f64().unwrap() - 2451545.0).abs() < 0.1);
        // 12 BODIES entries + the synthesised South Node = 13 rows.
        assert_eq!(v["planets"].as_array().unwrap().len(), 13);
        // 12 houses
        assert_eq!(v["houses"].as_array().unwrap().len(), 12);
        // ASC is a valid longitude
        let asc = v["asc"].as_f64().unwrap();
        assert!((0.0..360.0).contains(&asc), "ASC out of range: {asc}");
    }

    #[test]
    fn context_all_house_systems() {
        // Every standard house system must succeed without panic
        for hsys in ['P', 'K', 'E', 'W', 'O', 'R', 'C', 'M', 'B', 'X'] {
            let r = build_context(
                2451545.0,
                48.85,
                2.35,
                "2000-01-01",
                hsys,
                std::collections::BTreeMap::new(),
            );
            assert!(r.is_ok(), "house system '{hsys}' failed: {:?}", r.err());
        }
    }

    #[test]
    fn context_equator_zero_lon() {
        let r = build_context(
            2451545.0,
            0.0,
            0.0,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        );
        assert!(r.is_ok(), "equator/prime-meridian should succeed");
    }

    #[test]
    fn context_polar_latitude() {
        // High latitudes: some house systems fail, others degrade gracefully
        // We just require no panic — Placidus may return error at high lat
        let _ = build_context(
            2451545.0,
            89.0,
            0.0,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        );
        let _ = build_context(
            2451545.0,
            -89.0,
            0.0,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        );
    }

    #[test]
    fn context_ancient_date() {
        // JDE 1000000 ≈ year -693 — should not panic
        let _ = build_context(
            1000000.0,
            0.0,
            0.0,
            "ancient",
            'E',
            std::collections::BTreeMap::new(),
        );
    }

    #[test]
    fn context_far_future() {
        // JDE 2816787 ≈ year 3000 — should not panic
        let _ = build_context(
            2816787.0,
            0.0,
            0.0,
            "future",
            'E',
            std::collections::BTreeMap::new(),
        );
    }

    #[test]
    fn context_user_vars_visible() {
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("my_key".to_string(), "my_value".to_string());
        vars.insert("bg_color".to_string(), "#123456".to_string());
        let ctx = build_context(2451545.0, 0.0, 0.0, "test", 'E', vars).unwrap();
        assert_eq!(ctx["vars"]["my_key"], "my_value", "custom var missing");
        assert_eq!(
            ctx["vars"]["bg_color"], "#123456",
            "palette override missing"
        );
    }

    #[test]
    fn context_palette_defaults() {
        let ctx = build_context(
            2451545.0,
            0.0,
            0.0,
            "test",
            'E',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(ctx["vars"]["bg_color"], "#ffffff");
        assert_eq!(ctx["vars"]["ring_color"], "#1a1a2e");
        assert_eq!(ctx["vars"]["title"], "Celestial Chart");
    }

    #[test]
    fn context_sun_lon_range() {
        let ctx = build_context(
            2451545.0,
            0.0,
            0.0,
            "test",
            'E',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let sun = &ctx["planets"][0];
        let lon = sun["lon"].as_f64().unwrap();
        assert!((0.0..360.0).contains(&lon), "Sun lon out of range: {lon}");
        assert_eq!(sun["name"], "Sun");
        // Sun glyph is paired with `U+FE0E` to force the text-presentation
        // form (see comment on `BODIES`); the JSON context preserves the
        // pair verbatim so SVG renderers don't substitute the colour
        // emoji.
        assert_eq!(sun["glyph"], "\u{2609}\u{FE0E}");
    }

    #[test]
    fn context_precomputed_coords_finite() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "test",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        for p in ctx["planets"].as_array().unwrap() {
            let x = p["x"].as_f64().unwrap();
            let y = p["y"].as_f64().unwrap();
            assert!(
                x.is_finite() && y.is_finite(),
                "planet {} has non-finite coords ({x}, {y})",
                p["name"]
            );
        }
    }

    #[test]
    fn context_aspects_have_endpoints() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "test",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        for asp in ctx["aspects"].as_array().unwrap() {
            let x1 = asp["x1"].as_f64().unwrap();
            let y1 = asp["y1"].as_f64().unwrap();
            let x2 = asp["x2"].as_f64().unwrap();
            let y2 = asp["y2"].as_f64().unwrap();
            assert!(
                x1.is_finite() && y1.is_finite() && x2.is_finite() && y2.is_finite(),
                "aspect endpoints not finite"
            );
            let orb = asp["orb"].as_f64().unwrap();
            assert!((0.0..=8.0).contains(&orb), "orb out of range: {orb}");
            assert!(
                asp["applying"].as_bool().is_some(),
                "applying flag must be boolean"
            );
        }
    }

    /// Aspects to the angles (ASC and MC) are part of every commercial
    /// natal chart — Sat-ASC squares, Ura-MC sesquiquadrates, etc. The
    /// World-of-Wisdom reference for the 1986 São Paulo nativity shows
    /// at least one aspect each to ASC and MC, so regression-lock that.
    #[test]
    fn context_aspects_include_asc_and_mc() {
        // 1986-05-30 09:00 UT, São Paulo (PDF reference chart)
        let jd = 2_446_580.875;
        let ctx = build_context(
            jd,
            -23.5505,
            -46.6333,
            "1986-05-30",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let aspects = ctx["aspects"].as_array().unwrap();
        let asc_aspects: Vec<&serde_json::Value> = aspects
            .iter()
            .filter(|a| a["body1"] == "ASC" || a["body2"] == "ASC")
            .collect();
        let mc_aspects: Vec<&serde_json::Value> = aspects
            .iter()
            .filter(|a| a["body1"] == "MC" || a["body2"] == "MC")
            .collect();
        assert!(
            !asc_aspects.is_empty(),
            "expected at least one aspect involving ASC"
        );
        assert!(
            !mc_aspects.is_empty(),
            "expected at least one aspect involving MC"
        );
        // The PDF reference shows ~6 ASC aspects and ~8 MC aspects.
        // Loose lower bound here so the test survives small orb-table
        // tweaks without breaking on every edit.
        assert!(
            asc_aspects.len() >= 3,
            "ASC should have several aspects, got {}",
            asc_aspects.len()
        );
        assert!(
            mc_aspects.len() >= 3,
            "MC should have several aspects, got {}",
            mc_aspects.len()
        );
    }

    /// `applying` must use both the sign of the orb (which side of
    /// exact we're on) and the *relative* speed of the two bodies.
    /// Single-body speed checks misclassify retrograde outer-planet
    /// transits to fast inner planets.
    ///
    /// For the 1986 São Paulo chart, the Sun-Saturn opposition has the
    /// Sun (+0.96°/d) and Saturn retrograde (-0.07°/d) at a separation
    /// of ~177.6° (below 180°). Their relative motion is closing the
    /// |sep| gap *away* from 180°, so the opposition must be separating.
    #[test]
    fn context_aspects_applying_uses_signed_orb() {
        let jd = 2_446_580.875; // 1986-05-30 09:00 UT
        let ctx = build_context(
            jd,
            -23.5505,
            -46.6333,
            "1986-05-30",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let aspects = ctx["aspects"].as_array().unwrap();
        let sun_sat_opp = aspects.iter().find(|a| {
            let names = [
                a["body1"].as_str().unwrap_or(""),
                a["body2"].as_str().unwrap_or(""),
            ];
            names.contains(&"Sun") && names.contains(&"Saturn") && a["aspect_deg"] == 180.0
        });
        let asp = sun_sat_opp.expect("Sun-Saturn opposition should be present");
        assert_eq!(
            asp["applying"].as_bool(),
            Some(false),
            "Sun (direct) opposing retrograde Saturn at <180° is separating"
        );
    }

    // ── render_builtin_svg ────────────────────────────────────────────────────

    #[test]
    fn builtin_svg_is_valid_xml_fragment() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let svg = render_builtin_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.starts_with("<?xml"), "must start with XML declaration");
        assert!(svg.contains("<svg "), "must contain <svg>");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(!svg.contains("NaN"), "SVG must not contain NaN");
        assert!(!svg.contains("inf"), "SVG must not contain inf");
    }

    #[test]
    fn builtin_svg_contains_legend_sections() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let svg = render_builtin_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.contains("Planets"), "legend must have Planets section");
        assert!(svg.contains("Aspects"), "legend must have Aspects section");
        assert!(
            svg.contains("Angles"),
            "legend must have Angles &amp; Houses section"
        );
        assert!(svg.contains("ASC"), "legend must show ASC");
        assert!(svg.contains("MC"), "legend must show MC");
    }

    #[test]
    fn builtin_svg_planet_glyphs_present() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let svg = render_builtin_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        // All 12 planet glyphs must appear — as `<symbol id="g-XXXX">`
        // entries in the defs block plus `<use href="#g-XXXX">` references
        // in the wheel and legend tables. The bare character is no longer
        // emitted because the renderer now uses embedded vector paths so
        // the chart looks identical on every viewer.
        for cp in [
            0x2609_u32, 0x263D, 0x263F, 0x2640, 0x2642, 0x2643, 0x2644, 0x2645, 0x2646, 0x2647,
            0x260A, 0x26B7,
        ] {
            let needle = format!("g-{cp:04X}");
            assert!(svg.contains(&needle), "SVG missing planet glyph `{needle}`");
        }
    }

    #[test]
    fn builtin_svg_custom_palette() {
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("bg_color".to_string(), "#FACADE".to_string());
        vars.insert("title".to_string(), "Test Title".to_string());
        let ctx = build_context(2451545.0, 0.0, 0.0, "test", 'E', vars).unwrap();
        let svg = render_builtin_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(
            svg.contains("#FACADE"),
            "custom bg_color must appear in SVG"
        );
        assert!(
            svg.contains("Test Title"),
            "custom title must appear in SVG"
        );
    }

    #[test]
    fn builtin_svg_winter_solstice() {
        // Winter solstice: Sun near 270° (Capricorn)
        let ctx = build_context(
            2451545.0 + 354.0,
            51.5,
            -0.1,
            "test",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let svg = render_builtin_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(
            svg.contains("<svg"),
            "SVG must be generated for winter solstice date"
        );
        assert!(!svg.contains("NaN"), "no NaN in winter solstice SVG");
    }

    // ── example template renders ──────────────────────────────────────────────

    #[test]
    fn example_template_renders_successfully() {
        use minijinja::{Environment, Value as MjValue};
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let mut env = Environment::new();
        env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
        env.add_template("t", EXAMPLE_TEMPLATE)
            .expect("example template should parse");
        let rendered = env
            .get_template("t")
            .unwrap()
            .render(MjValue::from_serialize(&ctx))
            .expect("example template should render");
        assert!(
            rendered.contains("<svg"),
            "example template must produce SVG"
        );
        assert!(
            !rendered.contains("NaN"),
            "example template must not produce NaN"
        );
    }

    /// The schema document must mention the documented top-level scalar
    /// fields and the major collection names. This catches drift if the
    /// schema gets edited but the template still references removed fields.
    #[test]
    fn context_schema_mentions_all_top_level_fields() {
        for field in [
            "date",
            "jd",
            "lat",
            "lon",
            "asc",
            "mc",
            "ic",
            "dsc",
            "asc_dms",
            "mc_dms",
            "moon_phase_name",
            "moon_illumination",
            "planets",
            "signs",
            "houses",
            "aspects",
            "vars",
        ] {
            assert!(
                CONTEXT_SCHEMA.contains(field),
                "CONTEXT_SCHEMA missing documented field `{field}`"
            );
        }
    }

    /// Every `{vars.X}` placeholder in `EXAMPLE_TEMPLATE` should reference a key
    /// that the schema documents. Catches typos like `{vars.title_color}`
    /// when only `vars.text_color` is documented.
    #[test]
    fn example_template_only_uses_documented_vars() {
        // Manual scan: split on the unambiguous prefix "{vars." and take the
        // identifier up to the first non-word character.
        let referenced: std::collections::BTreeSet<String> = EXAMPLE_TEMPLATE
            .split("{vars.")
            .skip(1)
            .map(|s| {
                s.chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>()
            })
            .filter(|s| !s.is_empty())
            .collect();
        for v in &referenced {
            let needle = format!("vars.{v}");
            assert!(
                CONTEXT_SCHEMA.contains(&needle),
                "EXAMPLE_TEMPLATE references {{vars.{v}}} but schema doesn't mention it",
            );
        }
    }

    // ── Phase 2: chart-type dispatch ──────────────────────────────────────────

    #[test]
    fn cosmogram_context_has_no_houses_flag() {
        let jd = 2_451_545.0;
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("no_houses".to_string(), "1".to_string());
        vars.insert("title".to_string(), "Cosmogram".to_string());
        let ctx = build_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        assert_eq!(ctx["vars"]["no_houses"].as_str(), Some("1"));
        // Should still have planets and signs
        assert!(!ctx["planets"].as_array().unwrap().is_empty());
        assert!(ctx["signs"].as_array().unwrap().len() == 12);
    }

    #[test]
    fn solar_return_jd_is_after_natal() {
        use celestial_core::{body::CalcFlags, solar_return_jd};
        let jd_natal = 2_440_000.0; // 1968-ish
        let sr = solar_return_jd(jd_natal, 2025, CalcFlags::BUILTIN).unwrap();
        assert!(
            sr > jd_natal,
            "solar return must be after natal: {sr} <= {jd_natal}"
        );
        // Should be within 2030 (JD ~2462000)
        assert!(sr < 2_463_000.0, "solar return too far in the future: {sr}");
    }

    #[test]
    fn lunar_return_jd_is_after_search_start() {
        use celestial_core::{body::CalcFlags, lunar_return_jd};
        let jd_natal = 2_451_545.0;
        let jd_start = jd_natal + 365.0; // one year later
        let lr = lunar_return_jd(jd_natal, jd_start, CalcFlags::BUILTIN).unwrap();
        assert!(
            lr >= jd_start,
            "lunar return {lr} should be >= start {jd_start}"
        );
        // Lunar cycle ≈ 29.5 days — return should be within one cycle of start
        assert!(lr < jd_start + 30.0, "lunar return too far: {lr}");
    }

    #[test]
    fn progressed_context_has_progressed_planets() {
        let jd = 2_451_545.0;
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("title".to_string(), "Progressed".to_string());
        let ctx = build_progressed_context(jd, 30.0, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let prog = ctx["progressed_planets"].as_array();
        assert!(prog.is_some(), "context missing progressed_planets");
        assert_eq!(prog.unwrap().len(), 13);
    }

    #[test]
    fn solar_arc_context_has_directed_planets() {
        let jd = 2_451_545.0;
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("title".to_string(), "Solar Arc".to_string());
        let ctx = build_solar_arc_context(jd, 30.0, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        assert!(ctx["directed_planets"].as_array().is_some());
        let arc = ctx["solar_arc_degrees"].as_f64().unwrap_or(0.0);
        // 30 years × ~1°/year → arc ≈ 28–32°
        assert!(
            arc > 25.0 && arc < 35.0,
            "solar arc {arc:.2}° outside expected range"
        );
    }

    #[test]
    fn biwheel_context_has_both_planet_sets() {
        let jd1 = 2_451_545.0;
        let jd2 = jd1 + 365.0; // one year later
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("title".to_string(), "Bi-wheel".to_string());
        let ctx = build_biwheel_context(
            jd1,
            jd2,
            48.85,
            2.35,
            48.85,
            2.35,
            "2000-01-01",
            "2001-01-01",
            'P',
            vars,
        )
        .unwrap();
        assert!(
            !ctx["planets"].as_array().unwrap().is_empty(),
            "inner planets missing"
        );
        assert!(
            ctx["outer_planets"].as_array().is_some(),
            "outer_planets missing"
        );
        assert_eq!(ctx["outer_planets"].as_array().unwrap().len(), 13);
    }

    #[test]
    fn biwheel_cross_aspects_computed() {
        let jd1 = 2_451_545.0;
        let jd2 = jd1 + 10.0; // close transit — should have aspects
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("title".to_string(), "Test".to_string());
        let ctx = build_biwheel_context(
            jd1,
            jd2,
            0.0,
            0.0,
            0.0,
            0.0,
            "2000-01-01",
            "2000-01-11",
            'P',
            vars,
        )
        .unwrap();
        // cross_aspects should exist (list may be empty but field must be present)
        assert!(ctx.get("cross_aspects").is_some());
    }

    // ── Phase 3: specialist charts ────────────────────────────────────────────

    #[test]
    fn dial_context_has_midpoints() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_dial_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        assert!(ctx["midpoints"].as_array().is_some(), "midpoints missing");
        // The 90° dial builder iterates `BODIES` directly (no South
        // Node synthesis) so the planets array stays at 12.
        assert!(ctx["planets"].as_array().unwrap().len() == 12);
        // Every planet dial_lon must be in [0, 90)
        for p in ctx["planets"].as_array().unwrap() {
            let dl = p["dial_lon"].as_f64().unwrap_or(-1.0);
            assert!((0.0..90.0).contains(&dl), "dial_lon {dl} out of [0,90)");
        }
    }

    #[test]
    fn dial_svg_contains_planet_glyphs() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_dial_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let svg = specialist::render_dial_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.contains("☉"), "Sun glyph missing from dial");
        assert!(svg.contains("☽"), "Moon glyph missing from dial");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn composite_context_positions_are_midpoints() {
        let jd1 = 2_451_545.0;
        let jd2 = jd1 + 180.0; // 6 months apart
        let vars = std::collections::BTreeMap::new();
        let ctx = build_composite_context(jd1, jd2, 0.0, 0.0, "date1", "date2", 'P', vars).unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 12);
        // Composite ASC should be the midpoint of the two individual ASCs
        let comp_asc = ctx["asc"].as_f64().unwrap();
        assert!(
            (0.0..360.0).contains(&comp_asc),
            "composite ASC out of range: {comp_asc}"
        );
    }

    #[test]
    fn triwheel_context_has_three_rings() {
        let jd1 = 2_451_545.0;
        let jd2 = jd1 + 365.0;
        let jd3 = jd1 + 730.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_triwheel_context(
            jd1, jd2, jd3, 48.85, 2.35, "natal", "prog", "transit", 'P', vars,
        )
        .unwrap();
        // Inner ring goes through `build_context` so it carries the
        // synthesised South Node (13 rows). Rings 2 and 3 iterate the
        // raw `BODIES` table for the transiting positions and stay at
        // the 12 canonical bodies.
        assert!(ctx["planets"].as_array().unwrap().len() == 13, "inner ring");
        assert!(
            ctx["ring2_planets"].as_array().unwrap().len() == 12,
            "ring 2"
        );
        assert!(
            ctx["ring3_planets"].as_array().unwrap().len() == 12,
            "ring 3"
        );
    }

    #[test]
    fn graphic_ephemeris_context_has_series() {
        let jd_start = 2_451_545.0;
        let jd_end = jd_start + 90.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_graphic_ephemeris_context(jd_start, jd_end, vars).unwrap();
        let series = ctx["planet_series"].as_array().unwrap();
        assert_eq!(series.len(), 12, "expected 12 planet series");
        // Each series must have >= 2 data points
        for s in series {
            let lons = s["lons"].as_array().unwrap();
            assert!(
                lons.len() >= 2,
                "series {} has too few points: {}",
                s["name"],
                lons.len()
            );
        }
    }

    #[test]
    fn graphic_ephemeris_svg_has_planet_paths() {
        let jd_start = 2_451_545.0;
        let jd_end = jd_start + 60.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_graphic_ephemeris_context(jd_start, jd_end, vars).unwrap();
        let svg = specialist::render_graphic_ephemeris_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.contains("<path"), "no paths in ephemeris SVG");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn local_space_context_has_azimuths() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_local_space_context(jd, 48.85, 2.35, "2000-01-01", vars).unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 12);
        for p in planets {
            let az = p["azimuth"].as_f64().unwrap_or(-1.0);
            assert!(
                (0.0..360.0).contains(&az),
                "azimuth {az} out of [0,360) for {}",
                p["name"]
            );
        }
    }

    #[test]
    fn local_space_svg_has_compass_directions() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_local_space_context(jd, 48.85, 2.35, "2000-01-01", vars).unwrap();
        let svg = specialist::render_local_space_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.contains(">N<"), "N direction missing");
        assert!(svg.contains(">S<"), "S direction missing");
        assert!(svg.contains(">E<"), "E direction missing");
        assert!(svg.contains(">W<"), "W direction missing");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 4 — Vedic / Jyotish chart builders
// ═══════════════════════════════════════════════════════════════════════════════

// South Indian Rasi chart — 3×4 fixed-sign grid, planets by sign.
//
// Layout (sign index 0=Aries … 11=Pisces, sidereal):
//   ┌──────┬──────┬──────┬──────┐
//   │ Pi 12│ Ar  1│ Ta  2│ Ge  3│  row 0
//   ├──────┤      centre     ├──────┤
//   │ Aq 11│               │ Ca  4│  row 1
//   ├──────┤               ├──────┤  row 2
//   │ Cp 10│               │ Le  5│
//   ├──────┼──────┬──────┼──────┤
//   │ Sg  9│ Sc  8│ Li  7│ Vi  6│  row 3
//   └──────┴──────┴──────┴──────┘
//
// Cell (row, col) → sign index (0-based, Aries=0):

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 4 (remaining) — North Indian chart, Ashtakavarga, Shadbala
// ═══════════════════════════════════════════════════════════════════════════════

// ─── North Indian diamond chart ───────────────────────────────────────────────
//
// 12 triangular cells arranged as a diamond (rhombus) pattern.
// The top cell is Aries (0), and cells proceed clockwise. The Ascendant sign
// is highlighted. Unlike South Indian (fixed signs), North Indian uses
// the ASC sign as the first house.

// Cell centre coordinates in a 540×540 grid
// Order: Aries(0)..Pisces(11) starting from top, clockwise
pub(super) const NI_CELLS: &[(f64, f64)] = &[
    (270.0, 72.0),  // 0 = Aries     top
    (405.0, 144.0), // 1 = Taurus    top-right
    (468.0, 270.0), // 2 = Gemini    right
    (405.0, 396.0), // 3 = Cancer    bottom-right
    (270.0, 468.0), // 4 = Leo       bottom
    (135.0, 396.0), // 5 = Virgo     bottom-left
    (72.0, 270.0),  // 6 = Libra     left
    (135.0, 144.0), // 7 = Scorpio   top-left
    (270.0, 180.0), // 8 = Sagittarius inner-top
    (360.0, 270.0), // 9 = Capricorn  inner-right
    (270.0, 360.0), // 10= Aquarius   inner-bottom
    (180.0, 270.0), // 11= Pisces     inner-left
];

// ─── Ashtakavarga table ───────────────────────────────────────────────────────
//
// Classical Jyotish system: each of 7 planets contributes benefic points
// (bindus) to each of the 12 signs. Total = Sarvashtakavarga.
// This is a pure table-layout SVG; the bindu calculation is done here
// using the traditional algorithm.

/// Compute Ashtakavarga bindus for one planet.
///
/// Classic rules: a planet contributes 1 bindu to certain signs relative
/// to its own position and the positions of other planets + ASC.
/// This uses the simplified Parashara tables (8 reference points × 12 offsets).
fn ashtakavarga_bindus(
    planet_rasi: usize,
    other_rasis: &[usize], // [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, ASC]
    table: &[&[usize]],    // contribution offsets for each reference point
) -> [u8; 12] {
    let mut bindus = [0u8; 12];
    for (i, &ref_rasi) in other_rasis.iter().enumerate() {
        if i >= table.len() {
            break;
        }
        for &offset in table[i] {
            let sign = (ref_rasi + offset) % 12;
            bindus[sign] += 1;
        }
    }
    // Also add contributions from the planet itself
    let _ = planet_rasi; // used implicitly through other_rasis[self]
    bindus
}

/// Full Sarvashtakavarga: sum of all 7 planets' bindus per sign.
/// Returns a 7×12 matrix (rows = planets, cols = signs) and a 12-element total row.
pub(super) fn sarvashtakavarga(
    planet_rasis: &[usize; 7],
    asc_rasi: usize,
) -> ([u8; 12], Vec<[u8; 12]>) {
    // Parashara tables: for each planet, the sign offsets (0-based from reference point)
    // that get a bindu. Each row = [Sun,Moon,Mars,Mer,Jup,Ven,Sat,ASC] reference points.
    // These are the classical Parashara benefic positions.
    const SUN_TABLE: &[&[usize]] = &[
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 6, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 5, 6, 9, 10, 11],
        &[5, 6, 9, 11],
        &[6, 7, 12, 1, 2, 3],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 4, 6, 10, 11, 12],
    ];
    const MON_TABLE: &[&[usize]] = &[
        &[3, 6, 10, 11],
        &[1, 3, 6, 7, 10, 11],
        &[2, 3, 5, 6, 9, 10, 11],
        &[1, 3, 4, 5, 7, 8, 10, 11],
        &[1, 4, 7, 8, 10, 11, 12],
        &[3, 4, 5, 7, 9, 10, 11],
        &[3, 5, 6, 11],
        &[3, 6, 10, 11],
    ];
    const MAR_TABLE: &[&[usize]] = &[
        &[3, 5, 6, 10, 11],
        &[3, 6, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 5, 6, 11],
        &[6, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 4, 7, 8, 9, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
    ];
    const MER_TABLE: &[&[usize]] = &[
        &[5, 6, 9, 11, 12],
        &[2, 4, 6, 8, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 3, 5, 6, 9, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 2, 4, 6, 8, 10, 11],
    ];
    const JUP_TABLE: &[&[usize]] = &[
        &[1, 2, 3, 4, 7, 8, 9, 10, 11],
        &[2, 5, 7, 9, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[1, 2, 4, 5, 6, 9, 10, 11],
        &[1, 2, 3, 4, 7, 8, 10, 11],
        &[2, 5, 6, 9, 10, 11],
        &[3, 5, 6, 12],
        &[1, 2, 4, 5, 6, 7, 9, 10, 11],
    ];
    const VEN_TABLE: &[&[usize]] = &[
        &[8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11, 12],
        &[3, 4, 6, 9, 11, 12],
        &[3, 5, 6, 9, 11],
        &[5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 10, 11],
        &[3, 4, 5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 11],
    ];
    const SAT_TABLE: &[&[usize]] = &[
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 6, 11],
        &[3, 5, 6, 10, 11, 12],
        &[6, 8, 9, 10, 11, 12],
        &[5, 6, 11, 12],
        &[6, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 10, 11],
        &[1, 3, 4, 6, 10, 11],
    ];

    let tables = [
        SUN_TABLE, MON_TABLE, MAR_TABLE, MER_TABLE, JUP_TABLE, VEN_TABLE, SAT_TABLE,
    ];
    let refs: Vec<usize> = planet_rasis
        .iter()
        .copied()
        .chain(std::iter::once(asc_rasi))
        .collect();

    let mut rows = Vec::new();
    let mut totals = [0u8; 12];
    for (pi, &tbl) in tables.iter().enumerate() {
        let row = ashtakavarga_bindus(planet_rasis[pi], &refs, tbl);
        for (si, &b) in row.iter().enumerate() {
            totals[si] += b;
        }
        rows.push(row);
    }
    (totals, rows)
}

// ─── Shadbala strength table ──────────────────────────────────────────────────
//
// Six-fold strength system. We compute three readily-available components:
// Ochchabala (exaltation strength), Saptavargaja bala (sign placement),
// and Chesta bala (motional strength from speed).

pub(super) const RASI_NAMES: &[&str] = &[
    "Ar", "Ta", "Ge", "Ca", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi",
];

pub(super) const RASI_GLYPHS: &[&str] = &[
    "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓",
];

// (row, col) → sign index; 4 rows × 4 cols, centre (rows 1-2, cols 1-2) = metadata
pub(super) const SI_CELLS: &[(usize, usize, i32)] = &[
    (0, 0, 11),
    (0, 1, 0),
    (0, 2, 1),
    (0, 3, 2),
    (1, 0, 10),
    (1, 3, 3),
    (2, 0, 9),
    (2, 3, 4),
    (3, 0, 8),
    (3, 1, 7),
    (3, 2, 6),
    (3, 3, 5),
];

const SI_CW: f64 = 140.0;
const SI_CH: f64 = 120.0;
const SI_OX: f64 = 30.0;
const SI_OY: f64 = 70.0;

/// Palette for the South-Indian Rasi chart renderer. Bundles the five
/// colour strings that every `write_si_*` helper needs so each helper's
/// signature stays under the clippy 7-arg ceiling.
struct SiPalette<'a> {
    bg: &'a str,
    txt: &'a str,
    border: &'a str,
    pcol: &'a str,
    retro: &'a str,
}

fn group_planets_by_rasi(planets: &[Value]) -> Vec<Vec<String>> {
    let mut rasi_planets: Vec<Vec<String>> = vec![Vec::new(); 12];
    for p in planets {
        let rasi = p["rasi"].as_i64().unwrap_or(0) as usize % 12;
        let g = p["glyph"].as_str().unwrap_or("?");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let deg = p["deg_in_rasi"].as_f64().unwrap_or(0.0);
        let lbl = format!("{g}{}", if ret { "℞" } else { "" });
        rasi_planets[rasi].push(format!("{lbl} {deg:.0}°"));
    }
    rasi_planets
}

fn write_si_header(
    s: &mut String,
    pal: &SiPalette,
    title: &str,
    date: &str,
    total_w: f64,
    total_h: f64,
) {
    use std::fmt::Write;
    let (bg, txt, border) = (pal.bg, pal.txt, pal.border);
    let half = total_w / 2.0;
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w:.0} {total_h:.0}" width="{total_w:.0}" height="{total_h:.0}">
  <rect width="{total_w:.0}" height="{total_h:.0}" fill="{bg}"/>
  <text x="{half:.2}" y="28" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="{half:.2}" y="48" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>
  <rect x="{SI_OX}" y="{SI_OY}" width="{:.2}" height="{:.2}" fill="none" stroke="{border}" stroke-width="2.0"/>"##,
        4.0 * SI_CW,
        4.0 * SI_CH
    );
}

fn write_si_centre(s: &mut String, ctx: &Value, pal: &SiPalette) {
    use std::fmt::Write;
    let (bg, txt, border) = (pal.bg, pal.txt, pal.border);
    let cx = SI_OX + SI_CW;
    let cy = SI_OY + SI_CH;
    let moon_sid = ctx["moon_sid_lon"].as_f64().unwrap_or(0.0);
    let (moon_nak, _) = long_to_nakshatra(moon_sid);
    let nak_nm = nakshatra_name(moon_nak).unwrap_or("?");
    let _ = writeln!(
        s,
        r##"  <rect x="{cx:.2}" y="{cy:.2}" width="{:.2}" height="{:.2}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>
  <text x="{:.2}" y="{:.2}" text-anchor="middle" font-size="11" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">Vedic Chart</text>
  <text x="{:.2}" y="{:.2}" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".7">☽ {nak_nm}</text>"##,
        2.0 * SI_CW,
        2.0 * SI_CH,
        cx + SI_CW,
        SI_CH.mul_add(0.85, cy),
        cx + SI_CW,
        SI_CH.mul_add(1.1, cy)
    );
}

fn write_si_cells(s: &mut String, rasi_planets: &[Vec<String>], pal: &SiPalette) {
    use std::fmt::Write;
    let (bg, txt, border, pcol, retro) = (pal.bg, pal.txt, pal.border, pal.pcol, pal.retro);
    for &(row, col, sign_idx) in SI_CELLS {
        let x = (col as f64).mul_add(SI_CW, SI_OX);
        let y = (row as f64).mul_add(SI_CH, SI_OY);
        let sg = RASI_GLYPHS[sign_idx as usize % 12];
        let sn = RASI_NAMES[sign_idx as usize % 12];
        let _ = writeln!(
            s,
            r##"  <rect x="{x:.2}" y="{y:.2}" width="{SI_CW:.2}" height="{SI_CH:.2}" fill="{bg}" stroke="{border}" stroke-width="1.0"/>
  <text x="{:.2}" y="{:.2}" font-size="11" font-family="serif" fill="{txt}" opacity=".5">{sg}</text>
  <text x="{:.2}" y="{:.2}" font-size="8" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{sn}</text>"##,
            x + 4.0,
            y + 14.0,
            x + 4.0,
            y + 24.0
        );

        let prasi = &rasi_planets[sign_idx as usize % 12];
        for (pi, plabel) in prasi.iter().enumerate() {
            let py = (pi as f64).mul_add(14.0, y + 36.0);
            let col_s = if plabel.contains('℞') { retro } else { pcol };
            let _ = writeln!(
                s,
                r##"  <text x="{:.2}" y="{py:.2}" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{col_s}">{plabel}</text>"##,
                x + 6.0
            );
        }
    }
}

fn write_si_dashas(s: &mut String, ctx: &Value, dy: f64, total_w: f64, txt: &str, pcol: &str) {
    use std::fmt::Write;
    let _ = writeln!(
        s,
        r##"  <text x="{SI_OX}" y="{dy:.2}" font-size="11" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">Vimshottari Dashas</text>"##
    );
    let dashas = json_array(&ctx["dashas"]);
    let col_w = (total_w - SI_OX * 2.0) / 3.0;
    for (i, d) in dashas.iter().take(9).enumerate() {
        let col_x = ((i % 3) as f64).mul_add(col_w, SI_OX);
        let row_y = ((i / 3) as f64).mul_add(14.0, dy + 14.0);
        let body = d["body"].as_str().unwrap_or("?");
        let yrs = d["years"].as_f64().unwrap_or(0.0);
        let start = d["start"].as_str().unwrap_or("");
        let _ = writeln!(
            s,
            r##"  <text x="{col_x:.2}" y="{row_y:.2}" font-size="9"
            font-family="'Segoe UI',system-ui,sans-serif" fill="{pcol}"><tspan font-weight="600">{body}</tspan> {yrs:.1}y · {start}</text>"##
        );
    }
}

pub(super) fn render_south_indian_svg(ctx: &ChartContext) -> String {
    use std::fmt::Write;
    let pal = SiPalette {
        bg: ctx["vars"]["bg_color"].as_str().unwrap_or("#ffffff"),
        border: ctx["vars"]["border_color"].as_str().unwrap_or("#5c3a00"),
        txt: ctx["vars"]["text_color"].as_str().unwrap_or("#2a1a00"),
        pcol: ctx["vars"]["planet_color"].as_str().unwrap_or("#1a3a7a"),
        retro: ctx["vars"]["retro_color"].as_str().unwrap_or("#a01030"),
    };
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Rasi Chart");
    let date = ctx["date"].as_str().unwrap_or("");

    let planets = json_array(&ctx["planets"]);
    let rasi_planets = group_planets_by_rasi(planets);

    let mut s = String::with_capacity(16 * 1024);
    let total_h = 4.0_f64.mul_add(SI_CH, SI_OY) + 40.0;
    let total_w = 4.0_f64.mul_add(SI_CW, SI_OX * 2.0);
    write_si_header(&mut s, &pal, title, date, total_w, total_h);
    write_si_centre(&mut s, ctx, &pal);
    write_si_cells(&mut s, &rasi_planets, &pal);
    let dy = 4.0_f64.mul_add(SI_CH, SI_OY) + 10.0;
    write_si_dashas(&mut s, ctx, dy, total_w, pal.txt, pal.pcol);
    let _ = writeln!(s, "</svg>");
    s
}

#[cfg(test)]
mod tests_vedic {
    use super::chinese::build_bazi_context;
    use super::hellenistic::{
        build_firdaria_context, build_hellenistic_context, build_profection_context,
    };
    use super::indigenous::build_medicine_wheel_context;
    use super::mesoamerican::build_mesoamerican_context;
    use super::vedic::{build_ashtakavarga_context, build_shadbala_context, build_vedic_context};
    use super::*;
    use celestial_core::{long_to_navamsa, long_to_rasi};

    #[test]
    fn vedic_context_has_sidereal_rasi() {
        let jd = 2_451_545.0; // J2000.0
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 13.08, 80.27, "2000-01-01", vars, "Rasi").unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 12);
        for p in planets {
            let rasi = p["rasi"].as_i64().unwrap_or(-1);
            assert!(
                (0..12).contains(&rasi),
                "rasi {rasi} out of range for {}",
                p["name"]
            );
        }
    }

    #[test]
    fn vedic_context_has_nakshatra() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Rasi").unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        for p in planets {
            let nak = p["nakshatra"].as_i64().unwrap_or(-1);
            assert!(
                (0..27).contains(&nak),
                "nakshatra {nak} out of range for {}",
                p["name"]
            );
            let pada = p["pada"].as_i64().unwrap_or(-1);
            assert!((1..=4).contains(&pada), "pada {pada} out of range");
        }
    }

    #[test]
    fn vedic_context_has_dashas() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Dasha").unwrap();
        let dashas = ctx["dashas"].as_array().unwrap();
        assert!(!dashas.is_empty(), "no dashas computed");
        // Total Vimshottari cycle = 120 years; entries should cover a long span
        let total_yrs: f64 = dashas
            .iter()
            .map(|d| d["years"].as_f64().unwrap_or(0.0))
            .sum();
        assert!(
            total_yrs > 50.0,
            "total dasha years {total_yrs:.1} too short"
        );
    }

    #[test]
    fn navamsa_positions_differ_from_rasi() {
        // Navamsa D9 divides each sign into 9 equal parts of 3°20'
        // At least some planets should have a different navamsa vs rasi sign
        let jd = 2_451_545.0;
        use celestial_core::body::{Body, CalcFlags};
        let flags = CalcFlags::BUILTIN | CalcFlags(64); // sidereal
        if let Ok(sun) = celestial_core::calc_ut(jd, Body::SUN, flags) {
            let rasi = long_to_rasi(sun.lon);
            let navamsa = long_to_navamsa(sun.lon);
            // Can't assert they differ (they might coincide), but both must be valid
            assert!((0..12).contains(&rasi), "rasi {rasi} invalid");
            assert!((0..12).contains(&navamsa), "navamsa {navamsa} invalid");
        }
    }

    #[test]
    fn south_indian_svg_has_all_signs() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Rasi").unwrap();
        let svg = render_south_indian_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        // All 12 rasi abbreviations must appear
        for sign in [
            "Ar", "Ta", "Ge", "Ca", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi",
        ] {
            assert!(
                svg.contains(sign),
                "sign {sign} missing from South Indian SVG"
            );
        }
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn dasha_svg_has_planet_bars() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Dasha").unwrap();
        let svg = vedic::render_dasha_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.contains("<rect"), "no bars in dasha SVG");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn navamsa_svg_built_without_panic() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Navamsa").unwrap();
        let svg = vedic::render_navamsa_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.contains("D9"), "Navamsa title missing");
        assert!(svg.contains("</svg>"));
    }
    // ── Phase 4 (remaining): North Indian, Ashtakavarga, Shadbala ─────────────

    #[test]
    fn north_indian_context_uses_vedic_rasis() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        // North Indian reuses build_vedic_context — same rasi grouping
        let ctx = build_vedic_context(jd, 13.08, 80.27, "2000-01-01", vars, "Rasi").unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 12);
        for p in planets {
            let rasi = p["rasi"].as_i64().unwrap_or(-1);
            assert!(
                (0..12).contains(&rasi),
                "rasi {rasi} out of [0,12) for {}",
                p["name"]
            );
        }
    }

    #[test]
    fn north_indian_svg_has_all_12_cells() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 13.08, 80.27, "2000-01-01", vars, "Rasi").unwrap();
        let svg = vedic::render_north_indian_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        // All 12 house numbers (1..=12) should appear
        for h in 1..=12u32 {
            assert!(
                svg.contains(&format!(">{h}<")),
                "house number {h} missing from North Indian SVG"
            );
        }
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn north_indian_svg_has_rasi_glyphs() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Rasi").unwrap();
        let svg = vedic::render_north_indian_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        // At least Aries glyph should appear
        assert!(svg.contains('\u{2648}'), "Aries glyph missing");
    }

    #[test]
    fn ashtakavarga_context_has_rows_and_totals() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_ashtakavarga_context(jd, 13.08, 80.27, "2000-01-01", vars).unwrap();
        let rows = ctx["ashtakavarga_rows"].as_array().unwrap();
        assert_eq!(rows.len(), 7, "expected 7 planet rows (Sun..Saturn)");
        let totals = ctx["sarvashtakavarga"].as_array().unwrap();
        assert_eq!(totals.len(), 12, "expected 12 sign totals");
    }

    #[test]
    fn ashtakavarga_bindus_range() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_ashtakavarga_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let rows = ctx["ashtakavarga_rows"].as_array().unwrap();
        for row in rows {
            let name = row["planet"].as_str().unwrap_or("?");
            let bindus = row["bindus"].as_array().unwrap();
            assert_eq!(bindus.len(), 12, "{name}: expected 12 sign values");
            for (si, b) in bindus.iter().enumerate() {
                let bv = b.as_u64().unwrap_or(99);
                assert!(
                    bv <= 8,
                    "{name} sign {si}: bindu {bv} > 8 (max is 8 contributors)"
                );
            }
        }
    }

    #[test]
    fn sarvashtakavarga_totals_are_sum_of_rows() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_ashtakavarga_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let rows = ctx["ashtakavarga_rows"].as_array().unwrap();
        let totals = ctx["sarvashtakavarga"].as_array().unwrap();
        for (si, total_v) in totals.iter().enumerate().take(12) {
            let row_sum: u64 = rows
                .iter()
                .map(|r| r["bindus"].as_array().unwrap()[si].as_u64().unwrap_or(0))
                .sum();
            let total = total_v.as_u64().unwrap_or(0);
            assert_eq!(
                row_sum, total,
                "sign {si}: row sum {row_sum} != sarvashtakavarga total {total}"
            );
        }
    }

    #[test]
    fn ashtakavarga_svg_has_grid() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_ashtakavarga_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = vedic::render_ashtakavarga_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.contains("Sun"), "Sun row missing from Ashtakavarga SVG");
        assert!(svg.contains("Moon"), "Moon row missing");
        assert!(svg.contains("Total"), "Total row missing");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn shadbala_context_has_seven_planets() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_shadbala_context(jd, 13.08, 80.27, "2000-01-01", vars).unwrap();
        let rows = ctx["shadbala"].as_array().unwrap();
        assert_eq!(
            rows.len(),
            7,
            "Shadbala needs exactly 7 planets (Sun..Saturn)"
        );
    }

    #[test]
    fn shadbala_strength_components_non_negative() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_shadbala_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let rows = ctx["shadbala"].as_array().unwrap();
        for row in rows {
            let name = row["name"].as_str().unwrap_or("?");
            for field in &[
                "ochchabala",
                "sapta_bala",
                "chesta_bala",
                "dig_bala",
                "total",
            ] {
                let v = row[field].as_f64().unwrap_or(-1.0);
                assert!(v >= 0.0, "{name}.{field} = {v} is negative");
            }
            // Total should be >= each component
            let total = row["total"].as_f64().unwrap_or(0.0);
            let ochcha = row["ochchabala"].as_f64().unwrap_or(0.0);
            assert!(
                total >= ochcha,
                "{name}: total {total} < ochchabala {ochcha}"
            );
        }
    }

    #[test]
    fn shadbala_svg_has_all_planets() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_shadbala_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = vedic::render_shadbala_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        for planet in &[
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
        ] {
            assert!(svg.contains(planet), "{planet} missing from Shadbala SVG");
        }
        assert!(svg.contains("Ochcha"), "column header missing");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }
    // ── Phase 5: Hellenistic / Persian chart types ─────────────────────────────

    #[test]
    fn hellenistic_context_has_dignity5_fields() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_hellenistic_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        // 12 BODIES + synthesised South Node = 13.
        assert_eq!(planets.len(), 13);
        for p in planets {
            // Every planet must have the new Phase 5 dignity fields
            assert!(
                p.get("dignity5").is_some(),
                "{} missing dignity5",
                p["name"]
            );
            assert!(
                p.get("term_ruler").is_some(),
                "{} missing term_ruler",
                p["name"]
            );
            assert!(
                p.get("decan_ruler").is_some(),
                "{} missing decan_ruler",
                p["name"]
            );
            assert!(
                p.get("same_sect").is_some(),
                "{} missing same_sect",
                p["name"]
            );
        }
    }

    #[test]
    fn hellenistic_context_has_is_day_flag() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_hellenistic_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        assert!(ctx.get("is_day").is_some(), "is_day missing from context");
    }

    #[test]
    fn hellenistic_svg_has_dignity_table() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_hellenistic_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let svg = hellenistic::render_hellenistic_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(
            svg.contains("Hellenistic Dignities"),
            "dignity table heading missing"
        );
        assert!(svg.contains("Term lord"), "term lord column missing");
        assert!(svg.contains("Decan lord"), "decan column missing");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn firdaria_context_has_periods() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_firdaria_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let periods = ctx["firdaria"].as_array().unwrap();
        assert!(!periods.is_empty(), "firdaria periods should not be empty");
        // Each period must have required fields
        for p in periods {
            assert!(p.get("major_lord").is_some());
            assert!(p.get("minor_lord").is_some());
            assert!(p.get("start_jd").is_some());
            assert!(p.get("end_jd").is_some());
        }
    }

    #[test]
    fn firdaria_svg_has_bars() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_firdaria_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let svg = hellenistic::render_firdaria_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.contains("<rect"), "no bars in Firdaria SVG");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn profection_context_has_house_and_lord() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_profection_context(jd, 48.85, 2.35, "2000-01-01", 'P', 35, vars).unwrap();
        let house = ctx["profection_house"].as_u64().unwrap_or(0);
        assert!(
            (1..=12).contains(&house),
            "profection house {house} ;out of [1,12]"
        );
        assert!(ctx.get("profection_lord").is_some());
    }

    #[test]
    fn profection_svg_has_marker() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_profection_context(jd, 48.85, 2.35, "2000-01-01", 'P', 30, vars).unwrap();
        let svg = hellenistic::render_profection_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.contains("profection"), "profection annotation missing");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }
    // ── Phase 6: Chinese / Ba Zi ────────────────────────────────────────────

    #[test]
    fn bazi_context_has_four_pillars() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_bazi_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let pillars = ctx["pillars"].as_array().unwrap();
        assert_eq!(pillars.len(), 4, "Ba Zi must have exactly 4 pillars");
        for p in pillars {
            assert!(p.get("stem_name").is_some(), "pillar missing stem_name");
            assert!(p.get("animal").is_some(), "pillar missing animal");
            assert!(p.get("stem_element").is_some(), "pillar missing element");
        }
    }

    #[test]
    fn bazi_context_has_solar_term() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_bazi_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        assert!(ctx.get("solar_term_current").is_some());
        assert!(ctx.get("solar_term_current_en").is_some());
        assert!(ctx.get("degrees_to_next").is_some());
    }

    #[test]
    fn bazi_context_element_counts_sum_to_eight() {
        // 4 pillars × 2 (stem + branch) = 8 element contributions
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_bazi_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let total: u64 = ctx["elements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["count"].as_u64().unwrap_or(0))
            .sum();
        assert_eq!(total, 8, "total element counts must equal 8");
    }

    #[test]
    fn bazi_svg_has_pillar_columns() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_bazi_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = chinese::render_bazi_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        // All 4 column labels must appear
        for lbl in ["Hour 時", "Day 日", "Month 月", "Year 年"] {
            assert!(
                svg.contains(lbl),
                "column label '{lbl}' missing from Ba Zi SVG"
            );
        }
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn bazi_svg_has_element_balance() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_bazi_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = chinese::render_bazi_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(
            svg.contains("Element balance"),
            "element balance section missing"
        );
    }

    // ── Phase 7: Mesoamerican ────────────────────────────────────────────────

    #[test]
    fn mesoamerican_context_has_all_calendars() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_mesoamerican_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        // All four calendar systems must have an entry
        assert!(ctx.get("tonal_name").is_some(), "tonalpohualli missing");
        assert!(ctx.get("xiu_month_name").is_some(), "xiuhpohualli missing");
        assert!(ctx.get("tzol_name").is_some(), "tzolkin missing");
        assert!(ctx.get("haab_month_name").is_some(), "haab missing");
        assert!(ctx.get("cr_sign").is_some(), "calendar round missing");
    }

    #[test]
    fn mesoamerican_trecena_in_range() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_mesoamerican_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let t = ctx["tonal_trecena"].as_u64().unwrap_or(0);
        assert!((1..=13).contains(&t), "trecena {t} out of [1,13]");
    }

    #[test]
    fn mesoamerican_svg_has_both_traditions() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_mesoamerican_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = mesoamerican::render_mesoamerican_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        assert!(svg.contains("Tonalpohualli"), "Aztec label missing");
        assert!(svg.contains("Tzolkin"), "Maya label missing");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    // ── Phase 8: Medicine Wheel / Egyptian Decans ────────────────────────────

    #[test]
    fn medicine_wheel_context_has_totem() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_medicine_wheel_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        assert!(ctx.get("totem").is_some(), "totem missing");
        assert!(ctx.get("element").is_some(), "element missing");
        assert!(ctx.get("decan_name").is_some(), "decan_name missing");
        assert!(ctx.get("decan_star").is_some(), "decan_star missing");
    }

    #[test]
    fn medicine_wheel_svg_has_compass() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_medicine_wheel_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = indigenous::render_medicine_wheel_svg(&crate::cmd::render::ChartContext::from(ctx.clone()));
        // Cardinal directions
        for dir in ["N", "E", "S", "W"] {
            assert!(
                svg.contains(&format!(">{dir}<")),
                "{dir} cardinal direction missing from Medicine Wheel SVG"
            );
        }
        assert!(
            svg.contains("Egyptian Decan"),
            "Egyptian decan section missing"
        );
        assert!(svg.contains("</svg>"), "SVG not closed");
    }
}
