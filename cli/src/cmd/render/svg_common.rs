//! Shared SVG scaffolding for the per-tradition chart renderers.
//!
//! Only the genuinely invariant part of every renderer's preamble lives
//! here: the XML declaration, the opening `<svg>` element and the
//! full-canvas background `<rect>`. The title/date lines deliberately
//! stay in each renderer — they differ per tradition (colour, size,
//! position, extra annotations) by design, not by copy-paste.

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

/// The invariant document preamble: `<?xml …?>`, `<svg …>` and the
/// background rect, for an integer-dimensioned canvas. Byte-identical to
/// the hand-written form used by the static-size renderers (no trailing
/// newline — callers append their title/date lines directly).
pub(super) fn svg_doc_open(w: u32, h: u32, bg: &str) -> String {
    // SEC-9: `bg` is a user-controlled palette colour (--var/config);
    // escape it as SEC-1 did for builtin_svg. Plain hex colours have no
    // escapable chars → byte-identical output.
    let bg = crate::format::xml_escape(bg);
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" \
         width=\"{w}\" height=\"{h}\">\n  \
         <rect width=\"{w}\" height=\"{h}\" fill=\"{bg}\"/>"
    )
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
        let d = celestial_core::revjul(jd_birth, celestial_core::body::Calendar::Gregorian);
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
