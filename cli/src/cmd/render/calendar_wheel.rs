//! Calendar-wheel renderers — Celtic Wheel of the Year (sabbats).

use super::ChartContext;
use crate::error::CliError;
use celestial_core::{revjul, sabbats_for_year, Calendar};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Build the context for a Celtic Wheel of the Year diagram.
///
/// Places each of the 8 sabbats around a circle at the angle corresponding to
/// the Sun's ecliptic longitude at that moment. Each entry has pre-computed
/// SVG geometry (`x` / `y` for the glyph, `tick_x1` / `tick_y1` / `tick_x2` /
/// `tick_y2` for radial ticks).
pub fn build_sabbat_wheel_context(
    jd: f64,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, CliError> {
    let date = revjul(jd, Calendar::Gregorian);
    let year = date.year;

    let sabbats =
        sabbats_for_year(year).map_err(|e| format!("sabbats_for_year({year}) failed: {e}"))?;

    // SVG layout — same conventions as the natal wheel
    const CX: f64 = 450.0;
    const CY: f64 = 420.0;
    const RING_OUTER: f64 = 320.0;
    const RING_INNER: f64 = 240.0;
    const GLYPH_R: f64 = 285.0;
    const NAME_R: f64 = 200.0;

    let mut entries = Vec::with_capacity(8);
    for s in &sabbats {
        let lon = s.kind.solar_longitude();
        // Convention: 0° solar lon (Aries / spring equinox) at the LEFT (9 o'clock),
        // matching natal-chart orientation. Angles increase counter-clockwise.
        // SVG has y-axis pointing DOWN, so we negate sin.
        let theta = (180.0 - lon).to_radians();

        let glyph_x = CX + GLYPH_R * theta.cos();
        let glyph_y = CY - GLYPH_R * theta.sin();
        let name_x = CX + NAME_R * theta.cos();
        let name_y = CY - NAME_R * theta.sin();
        let tick_x1 = CX + RING_INNER * theta.cos();
        let tick_y1 = CY - RING_INNER * theta.sin();
        let tick_x2 = CX + RING_OUTER * theta.cos();
        let tick_y2 = CY - RING_OUTER * theta.sin();

        let d = revjul(s.jd, Calendar::Gregorian);
        let date_str = format!("{:04}-{:02}-{:02}", d.year, d.month, d.day);

        entries.push(json!({
            "name": s.name,
            "alt_names": s.kind.alt_names(),
            "lon": lon,
            "jd": s.jd,
            "date": date_str,
            "is_quarter_day": s.kind.is_quarter_day(),
            "is_cross_quarter": s.kind.is_cross_quarter(),
            "glyph_x": glyph_x, "glyph_y": glyph_y,
            "name_x": name_x,   "name_y": name_y,
            "tick_x1": tick_x1, "tick_y1": tick_y1,
            "tick_x2": tick_x2, "tick_y2": tick_y2,
        }));
    }

    // Default vars
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert_with(|| format!("Wheel of the Year — {year}"));
    vars.entry("bg_color".to_string())
        .or_insert_with(|| "#ffffff".to_string());
    vars.entry("text_color".to_string())
        .or_insert_with(|| "#222".to_string());
    vars.entry("ring_color".to_string())
        .or_insert_with(|| "#888".to_string());
    vars.entry("quarter_color".to_string())
        .or_insert_with(|| "#b8860b".to_string());
    vars.entry("cross_color".to_string())
        .or_insert_with(|| "#3d6b35".to_string());

    let mut vars_json = serde_json::Map::new();
    for (k, v) in vars {
        vars_json.insert(k, json!(v));
    }

    // ARCH-9/DP-5: typed context (field names == JSON keys).
    let ctx = SabbatWheelContext {
        year,
        jd,
        sabbats: entries,
        vars: Value::Object(vars_json),
    };
    serde_json::to_value(&ctx).map_err(|e| CliError::Msg(e.to_string()))
}

/// Typed Wheel-of-the-Year context (ARCH-9/DP-5).
#[derive(serde::Serialize)]
struct SabbatWheelContext {
    year: i32,
    jd: f64,
    sabbats: Vec<Value>,
    vars: Value,
}

/// Built-in SVG renderer for the Wheel of the Year.
///
/// Draws an outer ring, two cross-axis lines (solstice/equinox axes), 8 radial
/// ticks at each sabbat, the sabbat name and date, and a centre-piece title.
pub fn render_sabbat_wheel_svg(ctx: &ChartContext) -> String {
    use std::fmt::Write;

    let vars = &ctx["vars"];
    let bg = super::svg_common::esc_var(vars, "bg_color", "#ffffff");
    let txt = super::svg_common::esc_var(vars, "text_color", "#222");
    let ring = super::svg_common::esc_var(vars, "ring_color", "#888");
    let quarter = super::svg_common::esc_var(vars, "quarter_color", "#b8860b");
    let cross = super::svg_common::esc_var(vars, "cross_color", "#3d6b35");
    let title = super::svg_common::esc_var(vars, "title", "Wheel of the Year");
    let year = ctx["year"].as_i64().unwrap_or(0);

    let mut s = String::with_capacity(4096);

    // Header
    s.push_str(&super::svg_common::svg_doc_open(900, 800, &bg));
    let _ = write!(
        s,
        r#"
  <text x="450" y="36" text-anchor="middle" font-size="20" font-weight="600"
        font-family="Georgia,serif" fill="{txt}">{title}</text>
  <text x="450" y="58" text-anchor="middle" font-size="11"
        font-family="system-ui,sans-serif" fill="{ring}" opacity=".75">{year}</text>
"#
    );

    // Concentric rings
    let _ = writeln!(
        s,
        r#"  <circle cx="450" cy="420" r="320" fill="none" stroke="{ring}" stroke-width="1.2"/>"#
    );
    let _ = writeln!(
        s,
        r#"  <circle cx="450" cy="420" r="240" fill="none" stroke="{ring}" stroke-width=".8" opacity=".6"/>"#
    );
    let _ = writeln!(
        s,
        r#"  <circle cx="450" cy="420" r="120" fill="none" stroke="{ring}" stroke-width=".5" opacity=".4"/>"#
    );

    // Cardinal axes (solstice/equinox lines: through 0/180 and 90/270 solar lon)
    let _ = writeln!(
        s,
        r#"  <line x1="130" y1="420" x2="770" y2="420" stroke="{ring}" stroke-width=".6" opacity=".4" stroke-dasharray="4,4"/>"#
    );
    let _ = writeln!(
        s,
        r#"  <line x1="450" y1="100" x2="450" y2="740" stroke="{ring}" stroke-width=".6" opacity=".4" stroke-dasharray="4,4"/>"#
    );

    // Sabbat tick + glyph + name + date
    if let Some(sabbats) = ctx["sabbats"].as_array() {
        for sb in sabbats {
            let name = sb["name"].as_str().unwrap_or("?");
            let date = sb["date"].as_str().unwrap_or("");
            let is_quarter = sb["is_quarter_day"].as_bool().unwrap_or(false);
            let colour = if is_quarter { &quarter } else { &cross };

            let tx1 = sb["tick_x1"].as_f64().unwrap_or(0.0);
            let ty1 = sb["tick_y1"].as_f64().unwrap_or(0.0);
            let tx2 = sb["tick_x2"].as_f64().unwrap_or(0.0);
            let ty2 = sb["tick_y2"].as_f64().unwrap_or(0.0);

            let _ = writeln!(
                s,
                r#"  <line x1="{tx1:.1}" y1="{ty1:.1}" x2="{tx2:.1}" y2="{ty2:.1}" stroke="{colour}" stroke-width="2.2"/>"#
            );

            let gx = sb["glyph_x"].as_f64().unwrap_or(0.0);
            let gy = sb["glyph_y"].as_f64().unwrap_or(0.0);
            let _ = writeln!(
                s,
                r#"  <text x="{gx:.1}" y="{gy:.1}" text-anchor="middle" dominant-baseline="central" font-size="16" font-weight="600" font-family="Georgia,serif" fill="{txt}">{name}</text>"#
            );
            let _ = writeln!(
                s,
                r#"  <text x="{gx:.1}" y="{ny:.1}" text-anchor="middle" dominant-baseline="central" font-size="10" font-family="system-ui,sans-serif" fill="{colour}" opacity=".85">{date}</text>"#,
                ny = gy + 18.0
            );
        }
    }

    // Centre legend
    let _ = writeln!(
        s,
        r#"  <text x="450" y="412" text-anchor="middle" font-size="11" font-family="Georgia,serif" fill="{ring}" opacity=".7">solstice / equinox</text>"#
    );
    let _ = writeln!(
        s,
        r#"  <text x="450" y="430" text-anchor="middle" font-size="11" font-family="Georgia,serif" fill="{cross}" opacity=".75">cross-quarter</text>"#
    );

    // Footer legend
    let _ = writeln!(
        s,
        r#"  <g font-family="system-ui,sans-serif" font-size="10" fill="{txt}">"#
    );
    let _ = writeln!(
        s,
        r#"    <rect x="160" y="755" width="14" height="3" fill="{quarter}"/>"#
    );
    let _ = writeln!(
        s,
        r#"    <text x="180" y="760">Quarter days (solstice / equinox)</text>"#
    );
    let _ = writeln!(
        s,
        r#"    <rect x="500" y="755" width="14" height="3" fill="{cross}"/>"#
    );
    let _ = writeln!(s, r#"    <text x="520" y="760">Cross-quarter days</text>"#);
    let _ = writeln!(s, r#"  </g>"#);

    s.push_str("</svg>\n");
    s
}
