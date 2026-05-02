//! Chinese chart builders — split from render.rs.

use celestial_core::body::{Body, CalcFlags};
use celestial_core::{calc_ut, four_pillars, solar_term_position, SOLAR_TERMS};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn build_bazi_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Four Pillars of Destiny (八字)".to_string());

    // Compute Sun's ecliptic longitude for solar-term-based month pillar
    let sun_pos = calc_ut(jd, Body::SUN, flags).map_err(|e| e.to_string())?;

    // Extract hour from fractional JD (JD starts at noon)
    let day_frac = (jd + 0.5).fract(); // fraction of day since midnight UT
    let hour_ut = day_frac * 24.0;

    let pillars = four_pillars(jd, hour_ut, sun_pos.lon);
    let pillar_vals: Vec<Value> = pillars
        .iter()
        .map(|p| {
            json!({
                "stem":           p.stem,
                "branch":         p.branch,
                "stem_name":      p.stem_name,
                "branch_name":    p.branch_name,
                "animal":         p.animal,
                "stem_element":   p.stem_element,
                "branch_element": p.branch_element,
                "yang":           p.yang,
                "polarity":       if p.yang { "Yang" } else { "Yin" },
                "name":           format!("{}-{}", p.stem_name, p.branch_name)})
        })
        .collect();

    // Solar term context
    let (current_term, deg_into, next_term, deg_to) = solar_term_position(sun_pos.lon);
    let (ct_pinyin, ct_english) = (SOLAR_TERMS[current_term].1, SOLAR_TERMS[current_term].2);
    let (nt_pinyin, nt_english) = (SOLAR_TERMS[next_term].1, SOLAR_TERMS[next_term].2);

    // Element count from pillars (useful for balance analysis)
    let elements = ["Wood", "Fire", "Earth", "Metal", "Water"];
    let mut element_counts = [0u8; 5];
    for p in pillars {
        for (i, &el) in elements.iter().enumerate() {
            if p.stem_element == el {
                element_counts[i] += 1;
            }
            if p.branch_element == el {
                element_counts[i] += 1;
            }
        }
    }
    let element_vals: Vec<Value> = elements
        .iter()
        .zip(element_counts.iter())
        .map(|(&name, &count)| json!({ "element": name, "count": count }))
        .collect();

    let palette = super::palette_with_defaults(
        &[
        ("bg_color", "#ffffff"),
        ("border_color", "#8b0000"),
        ("text_color", "#1a0a00"),
        ("planet_color", "#2a1a60"),
        ],
        &vars,
    );

    Ok(json!({
        "date": date_str, "jd": jd, "lat": lat, "lon": lon,
        "pillars":   pillar_vals,
        "elements":  element_vals,
        "solar_term_current":    ct_pinyin,
        "solar_term_current_en": ct_english,
        "solar_term_next":       nt_pinyin,
        "solar_term_next_en":    nt_english,
        "degrees_into_term":     (deg_into * 100.0).round() / 100.0,
        "degrees_to_next":       (deg_to   * 100.0).round() / 100.0,
        "sun_lon":    (sun_pos.lon * 1e4).round() / 1e4,
        "vars": Value::Object(palette.into_iter().collect())}))
}

pub fn render_bazi_svg(ctx: &Value) -> String {
    use std::fmt::Write;

    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#ffffff");
    let border = ctx["vars"]["border_color"].as_str().unwrap_or("#8b0000");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#1a0a00");
    let pcol = ctx["vars"]["planet_color"].as_str().unwrap_or("#2a1a60");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Ba Zi");
    let date = ctx["date"].as_str().unwrap_or("");

    // Element colours
    const ELEM_COLORS: &[(&str, &str)] = &[
        ("Wood", "#2d6a2d"),
        ("Fire", "#c0392b"),
        ("Earth", "#a0722a"),
        ("Metal", "#707070"),
        ("Water", "#1a4a8a"),
    ];

    let pillars = super::json_array(&ctx["pillars"]);
    let elements = super::json_array(&ctx["elements"]);
    let solar_term = ctx["solar_term_current_en"].as_str().unwrap_or("—");
    let solar_term_cn = ctx["solar_term_current"].as_str().unwrap_or("—");
    let next_term_en = ctx["solar_term_next_en"].as_str().unwrap_or("—");
    let deg_to = ctx["degrees_to_next"].as_f64().unwrap_or(0.0);

    const CW: f64 = 160.0; // column width
    const CH: f64 = 280.0; // column height
    const OX: f64 = 60.0; // left margin
    const OY: f64 = 70.0; // top margin

    let total_w = OX * 2.0 + 4.0 * CW;
    let total_h = OY + CH + 200.0;

    let mut s = String::with_capacity(8 * 1024);
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w:.0} {total_h:.0}" width="{total_w:.0}" height="{total_h:.0}">
  <rect width="{total_w:.0}" height="{total_h:.0}" fill="{bg}"/>
  <text x="{:.1}" y="26" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="{:.1}" y="44" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"##,
        total_w / 2.0,
        total_w / 2.0
    );

    // Column labels
    let col_labels = ["Hour 時", "Day 日", "Month 月", "Year 年"];
    for (ci, label) in col_labels.iter().enumerate() {
        let cx = OX + ci as f64 * CW + CW / 2.0;
        let _ = writeln!(
            s,
            r##"  <text x="{cx:.1}" y="{:.1}" font-size="11" font-weight="600" text-anchor="middle"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{border}" opacity=".8">{label}</text>"##,
            OY - 8.0
        );
    }

    // Four pillar columns
    for (ci, p) in pillars.iter().enumerate() {
        let cx = OX + ci as f64 * CW;
        let stem_n = p["stem_name"].as_str().unwrap_or("?");
        let branch_n = p["branch_name"].as_str().unwrap_or("?");
        let animal = p["animal"].as_str().unwrap_or("?");
        let stem_el = p["stem_element"].as_str().unwrap_or("?");
        let br_el = p["branch_element"].as_str().unwrap_or("?");
        let pol = p["polarity"].as_str().unwrap_or("?");

        let stem_col = ELEM_COLORS
            .iter()
            .find(|(e, _)| *e == stem_el)
            .map(|(_, c)| *c)
            .unwrap_or(txt);
        let branch_col = ELEM_COLORS
            .iter()
            .find(|(e, _)| *e == br_el)
            .map(|(_, c)| *c)
            .unwrap_or(txt);

        // Column background
        let _ = writeln!(
            s,
            r##"  <rect x="{cx:.1}" y="{OY:.1}" width="{CW:.1}" height="{CH:.1}" rx="6" fill="none" stroke="{border}" stroke-width="1.5" opacity=".5"/>"##
        );

        // Divider line (stem / branch)
        let div_y = OY + CH * 0.5;
        let _ = writeln!(
            s,
            r##"  <line x1="{cx:.1}" y1="{div_y:.1}" x2="{:.1}" y2="{div_y:.1}" stroke="{border}" stroke-width="0.8" opacity=".4"/>"##,
            cx + CW
        );

        let cx_c = cx + CW / 2.0;

        // Heavenly Stem (upper half)
        let _ = writeln!(
            s,
            r##"  <text x="{cx_c:.1}" y="{:.1}" font-size="28" font-weight="700" text-anchor="middle"
        dominant-baseline="central" font-family="serif" fill="{stem_col}">{stem_n}</text>
  <text x="{cx_c:.1}" y="{:.1}" font-size="10" text-anchor="middle"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{stem_col}" opacity=".8">{stem_el} · {pol}</text>"##,
            OY + CH * 0.25,
            OY + CH * 0.40
        );

        // Earthly Branch (lower half)
        let _ = writeln!(
            s,
            r##"  <text x="{cx_c:.1}" y="{:.1}" font-size="22" font-weight="700" text-anchor="middle"
        dominant-baseline="central" font-family="serif" fill="{branch_col}">{branch_n}</text>
  <text x="{cx_c:.1}" y="{:.1}" font-size="11" text-anchor="middle"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{branch_col}">{animal}</text>
  <text x="{cx_c:.1}" y="{:.1}" font-size="9" text-anchor="middle"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{branch_col}" opacity=".7">{br_el}</text>"##,
            OY + CH * 0.65,
            OY + CH * 0.78,
            OY + CH * 0.90
        );
    }

    // Element balance row
    let ey = OY + CH + 18.0;
    let _ = writeln!(
        s,
        r##"  <text x="{:.1}" y="{ey:.1}" font-size="11" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">Element balance:</text>"##,
        OX
    );
    let ex_start = OX + 130.0;
    for (i, el) in elements.iter().enumerate() {
        let name = el["element"].as_str().unwrap_or("?");
        let count = el["count"].as_u64().unwrap_or(0);
        let ec = ELEM_COLORS
            .iter()
            .find(|(e, _)| *e == name)
            .map(|(_, c)| *c)
            .unwrap_or(txt);
        let ex = ex_start + i as f64 * 90.0;
        // Bar: width proportional to count (max 8)
        let bw = count as f64 * 16.0;
        let _ = writeln!(
            s,
            r##"  <rect x="{ex:.1}" y="{:.1}" width="{bw:.1}" height="12" rx="3" fill="{ec}" opacity=".7"/>
  <text x="{:.1}" y="{:.1}" font-size="9" font-family="'Segoe UI',system-ui,sans-serif" fill="{ec}">{name} {count}</text>"##,
            ey + 10.0,
            ex + bw + 4.0,
            ey + 21.0
        );
    }

    // Solar term row
    let sy = ey + 50.0;
    let _ = writeln!(
        s,
        r##"  <text x="{:.1}" y="{sy:.1}" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{pcol}">
        Solar term: <tspan font-weight="600">{solar_term_cn} — {solar_term}</tspan>
        · Next: {next_term_en} in {deg_to:.1}°</text>"##,
        OX
    );

    let _ = writeln!(s, "</svg>");
    s
}
