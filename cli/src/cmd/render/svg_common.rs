//! Shared SVG scaffolding for the per-tradition chart renderers.
//!
//! Only the genuinely invariant part of every renderer's preamble lives
//! here: the XML declaration, the opening `<svg>` element and the
//! full-canvas background `<rect>`. The title/date lines deliberately
//! stay in each renderer — they differ per tradition (colour, size,
//! position, extra annotations) by design, not by copy-paste.

use serde_json::Value;

/// Background / accent / text colours pulled from `ctx["vars"]`, each
/// with a per-tradition default. Replaces the repeated trio of
/// `ctx["vars"]["…"].as_str().unwrap_or("…")` lookups.
pub(super) struct SvgPalette<'a> {
    pub bg: &'a str,
    pub accent: &'a str,
    pub text: &'a str,
}

impl<'a> SvgPalette<'a> {
    /// `accent_var` is the var key the tradition uses for its accent
    /// colour (`"border_color"` or `"accent_color"`).
    pub fn from_ctx(
        ctx: &'a Value,
        bg_default: &'a str,
        accent_var: &str,
        accent_default: &'a str,
        text_default: &'a str,
    ) -> Self {
        let v = &ctx["vars"];
        SvgPalette {
            bg: v["bg_color"].as_str().unwrap_or(bg_default),
            accent: v[accent_var].as_str().unwrap_or(accent_default),
            text: v["text_color"].as_str().unwrap_or(text_default),
        }
    }
}

/// The invariant document preamble: `<?xml …?>`, `<svg …>` and the
/// background rect, for an integer-dimensioned canvas. Byte-identical to
/// the hand-written form used by the static-size renderers (no trailing
/// newline — callers append their title/date lines directly).
pub(super) fn svg_doc_open(w: u32, h: u32, bg: &str) -> String {
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
