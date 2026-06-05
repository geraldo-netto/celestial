//! Shared SVG scaffolding for the per-tradition chart renderers.
//!
//! Only the genuinely invariant part of every renderer's preamble lives
//! here: the XML declaration, the opening `<svg>` element and the
//! full-canvas background `<rect>`. The title/date lines deliberately
//! stay in each renderer — they differ per tradition (colour, size,
//! position, extra annotations) by design, not by copy-paste.

use celestial_core::JulianDay;
use serde_json::Value;

/// Read one `ctx["vars"]` colour/title string and XML-escape it
/// (SEC-10: user-controlled `--var`/config values flow into SVG text
/// and attributes in every specialist renderer; the SEC-1/SEC-9 fix
/// never reached them). Plain hex colours and default titles have no
/// escapable chars → byte-identical output for non-malicious input.
pub(super) fn esc_var(vars: &Value, key: &str, default: &str) -> String {
    crate::format::xml_escape(vars[key].as_str().unwrap_or(default))
}

/// Background / accent / text colours pulled from `ctx["vars"]`, each
/// with a per-tradition default, XML-escaped at the boundary (SEC-10).
/// Replaces the repeated trio of `ctx["vars"]["…"].as_str()` lookups.
pub(super) struct SvgPalette {
    pub bg: String,
    pub accent: String,
    pub text: String,
}

impl SvgPalette {
    /// `accent_var` is the var key the tradition uses for its accent
    /// colour (`"border_color"` or `"accent_color"`).
    pub fn from_ctx(
        ctx: &Value,
        bg_default: &str,
        accent_var: &str,
        accent_default: &str,
        text_default: &str,
    ) -> Self {
        let v = &ctx["vars"];
        SvgPalette {
            bg: esc_var(v, "bg_color", bg_default),
            accent: esc_var(v, accent_var, accent_default),
            text: esc_var(v, "text_color", text_default),
        }
    }
}

/// The invariant document preamble (`<?xml …?>`, `<svg …>`, background
/// rect), loaded once from the `fragments/svg_open.svg` template so every
/// renderer composes the same bytes. `{w}`/`{h}` are rendered with `:.0`
/// (integer canvases). No trailing newline — callers append their
/// title/date lines directly. Byte-identical to the hand-written form
/// previously inlined in each renderer (DUP-8).
///
/// `bg` must already be XML-escaped by the caller (every renderer pulls it
/// via `esc_var`/`SvgPalette`, so SEC-9 escaping happens once at that
/// boundary — this helper does not re-escape).
const SVG_OPEN_TEMPLATE: &str = include_str!("fragments/svg_open.svg");

pub(super) fn svg_doc_open(w: f64, h: f64, bg: &str) -> String {
    SVG_OPEN_TEMPLATE
        .trim_end_matches('\n')
        .replace("{w}", &format!("{w:.0}"))
        .replace("{h}", &format!("{h:.0}"))
        .replace("{bg}", bg)
}

/// A bordered panel card: rounded rect + centred heading text. Covers the
/// `<rect rx=…/><text …>heading</text>` pair repeated across renderers.
#[allow(clippy::too_many_arguments)]
pub(super) fn panel_card(
    s: &mut String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    stroke: &str,
    stroke_opacity: &str,
    heading: &str,
    heading_color: &str,
) {
    use std::fmt::Write;
    let cx = x + w / 2.0;
    // SEC-9: stroke / heading_color are user-controlled palette colours
    // and heading is rendered into <text>; escape all three (no-op for
    // plain colours / tradition labels → byte-identical).
    let stroke = crate::format::xml_escape(stroke);
    let heading_color = crate::format::xml_escape(heading_color);
    let heading = crate::format::xml_escape(heading);
    let _ = write!(
        s,
        "\n  <rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" rx=\"6\" \
         fill=\"none\" stroke=\"{stroke}\" stroke-width=\"1.5\" \
         opacity=\"{stroke_opacity}\"/>\
         \n  <text x=\"{cx}\" y=\"{ty}\" text-anchor=\"middle\" font-size=\"11\" \
         font-weight=\"600\" fill=\"{heading_color}\">{heading}</text>",
        ty = y + 17.0
    );
}

/// Snapshot of a [`HouseResult::cusps`](celestial_core::HouseResult) array
/// into an owned `[f64; 13]`. `cusps` is already a `Copy` array — calling
/// this helper documents the intent at the rendering layer (DUP-11).
#[inline]
#[must_use]
pub(super) fn cusps_to_array(h: &celestial_core::HouseResult) -> [f64; 13] {
    h.cusps
}

/// Year-axis gridlines for the horizontal timeline renderers (firdaria,
/// vimśottarī daśā). Emits a 5-yearly `<line>`+`<text>` pair from the
/// birth year to the end of `span`. Geometry (`lm`/`w`/`tm`/`axis_bottom`),
/// `clamp_pad`, `color` and `font_size` differ per tradition; everything
/// else was verbatim-duplicated (DUP-7).
#[allow(clippy::too_many_arguments)]
pub(super) fn write_year_axis(
    s: &mut String,
    jd_birth: f64,
    jd_start: f64,
    span: f64,
    lm: f64,
    w: f64,
    tm: f64,
    axis_bottom: f64,
    clamp_pad: f64,
    color: &str,
    font_size: u32,
) {
    use std::fmt::Write;
    let birth_year = {
        let d = celestial_core::revjul(
            JulianDay::new(jd_birth),
            celestial_core::body::Calendar::Gregorian,
        );
        d.year as i32
    };
    let end_year = birth_year + (span / 365.25) as i32 + 1;
    for yr in (birth_year..=end_year).step_by(5) {
        let jd_yr =
            celestial_core::julday(yr, 1, 1, 0.0, celestial_core::body::Calendar::Gregorian);
        let x = lm + (jd_yr - jd_start) / span * w;
        if !(lm - clamp_pad..=lm + w + clamp_pad).contains(&x) {
            continue;
        }
        let _ = writeln!(
            s,
            r##"  <line x1="{x:.1}" y1="{tm:.1}" x2="{x:.1}" y2="{axis_bottom:.1}" stroke="{color}" stroke-width="0.5" opacity=".2"/>
  <text x="{x:.1}" y="{:.1}" text-anchor="middle" font-size="{font_size}" fill="{color}" opacity=".5">{yr}</text>"##,
            tm - 6.0
        );
    }
}
