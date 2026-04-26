#![allow(dead_code, unused_imports)]
//! Sefirat HaOmer 7×7 sefirot grid — the canonical visual layout for the
//! 49-day count, with each cell showing one day's pairing of sefirot.
//!
//! Layout: 7 rows (weeks) × 7 columns (days within the week). Cell (week, day)
//! holds day number `(week - 1) * 7 + day`. Lag Ba'Omer (day 33) is highlighted.

use celestial_core::{
    approx_hebrew_year, omer_days, omer_period, revjul, Calendar, OmerDay,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Build the context for an Omer 7×7 grid.
///
/// Pre-computes all 49 cell positions (`x`, `y`, `width`, `height`) plus the
/// header geometry. Each cell carries the day number, the week-sefirah and
/// day-sefirah pair, the Gregorian date, and a `is_lag_baomer` flag.
pub fn build_omer_grid_context(
    jd: f64,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    // Determine which Hebrew year's Omer we're rendering. If the requested
    // JD is inside an Omer period use that one; otherwise use the upcoming
    // Omer that starts after this date.
    let period = omer_period(jd);
    let hebrew_year = period.hebrew_year;
    let days = omer_days(hebrew_year);
    if days.len() != 49 {
        return Err(format!(
            "expected 49 Omer days, got {} for Hebrew year {hebrew_year}",
            days.len()
        ));
    }

    // SVG layout — 7 wide × 7 tall grid, generous cell size for readability.
    const MARGIN_X: f64 = 60.0;
    const HEADER_Y: f64 = 130.0;     // top of grid (below title block)
    const COL_HEADER_H: f64 = 36.0;  // height of "Day 1..7" header strip
    const ROW_HEADER_W: f64 = 64.0;  // width of "Week 1..7" header strip
    const CELL_W: f64 = 110.0;
    const CELL_H: f64 = 100.0;

    let grid_x0 = MARGIN_X + ROW_HEADER_W;
    let grid_y0 = HEADER_Y + COL_HEADER_H;

    let mut cells = Vec::with_capacity(49);
    for d in &days {
        let col = (d.day_of_week - 1) as f64; // 0..6
        let row = (d.week - 1) as f64;        // 0..6
        let x = grid_x0 + col * CELL_W;
        let y = grid_y0 + row * CELL_H;

        let revdate = revjul(d.jd, Calendar::Gregorian);
        let date_str = format!("{:04}-{:02}-{:02}", revdate.year, revdate.month, revdate.day);

        cells.push(json!({
            "day":            d.day,
            "week":           d.week,
            "day_of_week":    d.day_of_week,
            "week_sefirah":   d.week_sefirah,
            "day_sefirah":    d.day_sefirah,
            "hebrew_text":    d.hebrew_text,
            "is_lag_baomer":  d.is_lag_baomer,
            "jd":             d.jd,
            "date":           date_str,
            "x": x, "y": y,
            "w": CELL_W, "h": CELL_H,
            // Helpful pre-computed text positions
            "day_num_x":      x + 8.0,
            "day_num_y":      y + 18.0,
            "sefirah_x":      x + CELL_W * 0.5,
            "week_sefirah_y": y + 44.0,
            "of_y":           y + 56.0,
            "day_sefirah_y":  y + 70.0,
            "date_x":         x + CELL_W * 0.5,
            "date_y":         y + CELL_H - 10.0,
        }));
    }

    // Column headers (day-of-week sefirot in row order: Chesed, Gevurah, …).
    // Match the spelling used in `core::omer::SEFIROT` so headers and cell
    // text are visually aligned.
    let col_sefirot = [
        "Chesed", "Gevurah", "Tiferet", "Netzach", "Hod", "Yesod", "Malkhut",
    ];
    let mut col_headers = Vec::with_capacity(7);
    for (i, name) in col_sefirot.iter().enumerate() {
        col_headers.push(json!({
            "name": name,
            "x": grid_x0 + (i as f64 + 0.5) * CELL_W,
            "y": HEADER_Y + COL_HEADER_H * 0.6,
        }));
    }

    // Row headers (week sefirot — same names, applied in week order)
    let mut row_headers = Vec::with_capacity(7);
    for (i, name) in col_sefirot.iter().enumerate() {
        row_headers.push(json!({
            "name": name,
            "x": MARGIN_X + ROW_HEADER_W * 0.5,
            "y": grid_y0 + (i as f64 + 0.5) * CELL_H,
        }));
    }

    // Grid bounds (for outer rect)
    let grid_w = 7.0 * CELL_W;
    let grid_h = 7.0 * CELL_H;
    let total_w = ROW_HEADER_W + grid_w + 2.0 * MARGIN_X;
    let total_h = grid_y0 + grid_h + 50.0;

    let date_now = revjul(jd, Calendar::Gregorian);
    let date_str = format!(
        "{:04}-{:02}-{:02}",
        date_now.year, date_now.month, date_now.day
    );

    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert_with(|| format!("Sefirat HaOmer · Hebrew Year {hebrew_year}"));
    vars.entry("bg_color".to_string()).or_insert("#fffff8".to_string());
    vars.entry("text_color".to_string()).or_insert("#222".to_string());
    vars.entry("ring_color".to_string()).or_insert("#888".to_string());
    vars.entry("header_color".to_string())
        .or_insert("#5c4a8a".to_string());
    vars.entry("lag_color".to_string()).or_insert("#c87f32".to_string());

    let mut vars_json = serde_json::Map::new();
    for (k, v) in vars {
        vars_json.insert(k, json!(v));
    }

    Ok(json!({
        "hebrew_year":  hebrew_year,
        "jd":           jd,
        "date":         date_str,
        "start_jd":     period.start_jd,
        "end_jd":       period.end_jd,
        "cells":        cells,
        "col_headers":  col_headers,
        "row_headers":  row_headers,
        "grid_x0":      grid_x0,
        "grid_y0":      grid_y0,
        "grid_w":       grid_w,
        "grid_h":       grid_h,
        "viewbox_w":    total_w,
        "viewbox_h":    total_h,
        "vars":         Value::Object(vars_json),
    }))
}

/// Built-in SVG renderer for the Omer grid.
pub fn render_omer_grid_svg(ctx: &Value) -> String {
    use std::fmt::Write;

    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fffff8");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#222");
    let ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#888");
    let header = ctx["vars"]["header_color"].as_str().unwrap_or("#5c4a8a");
    let lag = ctx["vars"]["lag_color"].as_str().unwrap_or("#c87f32");
    let title = ctx["vars"]["title"].as_str().unwrap_or("Sefirat HaOmer");

    let vw = ctx["viewbox_w"].as_f64().unwrap_or(900.0);
    let vh = ctx["viewbox_h"].as_f64().unwrap_or(800.0);
    let gx = ctx["grid_x0"].as_f64().unwrap_or(124.0);
    let gy = ctx["grid_y0"].as_f64().unwrap_or(166.0);
    let gw = ctx["grid_w"].as_f64().unwrap_or(770.0);
    let gh = ctx["grid_h"].as_f64().unwrap_or(700.0);

    let mut s = String::with_capacity(8192);

    // Header
    let _ = write!(
        s,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {vw:.0} {vh:.0}" width="{vw:.0}" height="{vh:.0}">
  <rect width="{vw:.0}" height="{vh:.0}" fill="{bg}"/>
  <text x="{cx:.0}" y="42" text-anchor="middle" font-size="22" font-weight="600"
        font-family="Georgia,serif" fill="{txt}">{title}</text>
  <text x="{cx:.0}" y="68" text-anchor="middle" font-size="11"
        font-family="system-ui,sans-serif" fill="{ring}" opacity=".75">7 weeks × 7 days · sefirah-of-sefirah pairings</text>
"#,
        cx = vw / 2.0
    );

    // Column headers (Day-of-week sefirot)
    if let Some(headers) = ctx["col_headers"].as_array() {
        for h in headers {
            let name = h["name"].as_str().unwrap_or("?");
            let x = h["x"].as_f64().unwrap_or(0.0);
            let y = h["y"].as_f64().unwrap_or(0.0);
            let _ = writeln!(
                s,
                r#"  <text x="{x:.1}" y="{y:.1}" text-anchor="middle" dominant-baseline="central" font-size="12" font-weight="600" font-family="Georgia,serif" fill="{header}">{name}</text>"#
            );
        }
    }

    // Row headers (Week sefirot)
    if let Some(headers) = ctx["row_headers"].as_array() {
        for h in headers {
            let name = h["name"].as_str().unwrap_or("?");
            let x = h["x"].as_f64().unwrap_or(0.0);
            let y = h["y"].as_f64().unwrap_or(0.0);
            let _ = writeln!(
                s,
                r#"  <text x="{x:.1}" y="{y:.1}" text-anchor="middle" dominant-baseline="central" font-size="12" font-weight="600" font-family="Georgia,serif" fill="{header}">{name}</text>"#
            );
        }
    }

    // Outer grid rect
    let _ = writeln!(
        s,
        r#"  <rect x="{gx:.1}" y="{gy:.1}" width="{gw:.1}" height="{gh:.1}" fill="none" stroke="{ring}" stroke-width="1.2"/>"#
    );

    // Cells
    if let Some(cells) = ctx["cells"].as_array() {
        for c in cells {
            let x = c["x"].as_f64().unwrap_or(0.0);
            let y = c["y"].as_f64().unwrap_or(0.0);
            let w = c["w"].as_f64().unwrap_or(110.0);
            let h = c["h"].as_f64().unwrap_or(100.0);
            let day = c["day"].as_u64().unwrap_or(0);
            let week_sef = c["week_sefirah"].as_str().unwrap_or("?");
            let day_sef = c["day_sefirah"].as_str().unwrap_or("?");
            let date = c["date"].as_str().unwrap_or("");
            let is_lag = c["is_lag_baomer"].as_bool().unwrap_or(false);

            let cell_fill = if is_lag {
                format!(r#"fill="{lag}" fill-opacity=".18""#)
            } else {
                "fill=\"none\"".to_string()
            };

            let _ = writeln!(
                s,
                r#"  <rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" {cell_fill} stroke="{ring}" stroke-width=".6"/>"#
            );

            // Day number (top-left)
            let dx = c["day_num_x"].as_f64().unwrap_or(x + 8.0);
            let dy = c["day_num_y"].as_f64().unwrap_or(y + 18.0);
            let day_color = if is_lag { lag } else { ring };
            let _ = writeln!(
                s,
                r#"  <text x="{dx:.1}" y="{dy:.1}" font-size="11" font-weight="700" font-family="system-ui,sans-serif" fill="{day_color}">Day {day}</text>"#
            );

            // Sefirot pairing (centred)
            let sx = c["sefirah_x"].as_f64().unwrap_or(x + w / 2.0);
            let wsy = c["week_sefirah_y"].as_f64().unwrap_or(y + 44.0);
            let oy = c["of_y"].as_f64().unwrap_or(y + 56.0);
            let dsy = c["day_sefirah_y"].as_f64().unwrap_or(y + 70.0);
            let _ = writeln!(
                s,
                r#"  <text x="{sx:.1}" y="{wsy:.1}" text-anchor="middle" font-size="13" font-weight="600" font-family="Georgia,serif" fill="{txt}">{day_sef}</text>"#
            );
            let _ = writeln!(
                s,
                r#"  <text x="{sx:.1}" y="{oy:.1}" text-anchor="middle" font-size="9" font-style="italic" font-family="Georgia,serif" fill="{ring}">of</text>"#
            );
            let _ = writeln!(
                s,
                r#"  <text x="{sx:.1}" y="{dsy:.1}" text-anchor="middle" font-size="13" font-weight="600" font-family="Georgia,serif" fill="{txt}">{week_sef}</text>"#
            );

            // Date (bottom)
            let datex = c["date_x"].as_f64().unwrap_or(x + w / 2.0);
            let datey = c["date_y"].as_f64().unwrap_or(y + h - 10.0);
            let _ = writeln!(
                s,
                r#"  <text x="{datex:.1}" y="{datey:.1}" text-anchor="middle" font-size="9" font-family="system-ui,sans-serif" fill="{ring}" opacity=".85">{date}</text>"#
            );

            // Lag Ba'Omer star
            if is_lag {
                let star_x = x + w - 14.0;
                let star_y = y + 18.0;
                let _ = writeln!(
                    s,
                    r#"  <text x="{star_x:.1}" y="{star_y:.1}" text-anchor="middle" font-size="14" fill="{lag}">★</text>"#
                );
            }
        }
    }

    // Footer note
    let _ = writeln!(s, r#"  <g font-family="system-ui,sans-serif" font-size="10" fill="{txt}">"#);
    let _ = writeln!(
        s,
        r#"    <rect x="{lx:.0}" y="{ly:.0}" width="14" height="10" fill="{lag}" fill-opacity=".18" stroke="{lag}" stroke-width=".6"/>"#,
        lx = gx,
        ly = vh - 30.0
    );
    let _ = writeln!(
        s,
        r#"    <text x="{lx:.0}" y="{ly:.0}">Lag Ba'Omer (Day 33) — traditional festive day</text>"#,
        lx = gx + 22.0,
        ly = vh - 21.0
    );
    let _ = writeln!(s, r#"  </g>"#);

    s.push_str("</svg>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omer_grid_context_has_49_cells() {
        // Pick a JD inside an Omer period: Hebrew year 5784 = April 24, 2024
        let jd = celestial_core::julday(2024, 5, 1, 12.0, Calendar::Gregorian);
        let ctx = build_omer_grid_context(jd, BTreeMap::new()).unwrap();
        let cells = ctx["cells"].as_array().expect("cells array");
        assert_eq!(cells.len(), 49, "Omer is always 49 days");
    }

    #[test]
    fn omer_grid_renders_valid_svg() {
        let jd = celestial_core::julday(2024, 5, 1, 12.0, Calendar::Gregorian);
        let ctx = build_omer_grid_context(jd, BTreeMap::new()).unwrap();
        let svg = render_omer_grid_svg(&ctx);
        assert!(svg.starts_with("<?xml"), "SVG should start with <?xml");
        assert!(svg.contains("<svg "), "should contain <svg> tag");
        assert!(svg.ends_with("</svg>\n"), "should close </svg>");
        // Must show all 7 sefirot names at least 14 times (7 row + 7 col headers,
        // plus once in each of 49 cells = much more)
        for sef in ["Chesed", "Gevurah", "Tiferet", "Netzach", "Hod", "Yesod", "Malkhut"] {
            assert!(svg.contains(sef), "SVG missing sefirah `{sef}`");
        }
        // Lag Ba'Omer star should be present
        assert!(svg.contains("★"), "SVG should contain Lag Ba'Omer star");
        assert!(svg.contains("Day 33"), "SVG should label Day 33");
        // No NaN/inf
        assert!(!svg.contains("NaN"), "SVG must not contain NaN");
    }
}
