#![allow(dead_code, unused_imports)]
//! Mesoamerican chart builders — split from render.rs.

use celestial_core::AzAlt;

use celestial_core::{calendar_round, haab, tonalpohualli, tzolkin, xiuhpohualli};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn build_mesoamerican_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Mesoamerican Calendars".to_string());

    let (trecena, sign_idx, tonal_name, tonal_en) = tonalpohualli(jd);
    let (xiu_month, xiu_day, xiu_month_name, xiu_month_en) = xiuhpohualli(jd);
    let (tzol_trecena, tzol_idx, tzol_name, tzol_en) = tzolkin(jd);
    let (haab_month, haab_day, haab_month_name) = haab(jd);
    let (cr_trecena, cr_sign, cr_haab_day, cr_haab_month) = calendar_round(jd);

    let palette = super::palette_with_defaults(
        &[
        ("bg_color", "#1a0a00"),
        ("border_color", "#d4a800"),
        ("text_color", "#f0e0c0"),
        ("planet_color", "#ffd070"),
        ],
        &vars,
    );

    Ok(json!({
        "date": date_str, "jd": jd, "lat": lat, "lon": lon,
        // Tonalpohualli (Aztec 260-day)
        "tonal_trecena":  trecena,
        "tonal_sign_idx": sign_idx,
        "tonal_name":     tonal_name,
        "tonal_english":  tonal_en,
        // Xiuhpohualli (Aztec 365-day)
        "xiu_month":      xiu_month,
        "xiu_day":        xiu_day,
        "xiu_month_name": xiu_month_name,
        "xiu_month_en":   xiu_month_en,
        // Tzolkin (Maya 260-day)
        "tzol_trecena":   tzol_trecena,
        "tzol_sign_idx":  tzol_idx,
        "tzol_name":      tzol_name,
        "tzol_english":   tzol_en,
        // Haab (Maya 365-day)
        "haab_month":     haab_month,
        "haab_day":       haab_day,
        "haab_month_name":haab_month_name,
        // Calendar Round
        "cr_trecena":     cr_trecena,
        "cr_sign":        cr_sign,
        "cr_haab_day":    cr_haab_day,
        "cr_haab_month":  cr_haab_month,
        "vars": Value::Object(palette.into_iter().collect())}))
}

pub fn render_mesoamerican_svg(ctx: &Value) -> String {
    use std::fmt::Write;

    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#1a0a00");
    let gold = ctx["vars"]["border_color"].as_str().unwrap_or("#d4a800");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#f0e0c0");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Mesoamerican");
    let date = ctx["date"].as_str().unwrap_or("");

    // Day sign glyphs (simple text representations)
    let tonal_name = ctx["tonal_name"].as_str().unwrap_or("?");
    let tonal_en = ctx["tonal_english"].as_str().unwrap_or("?");
    let trecena = ctx["tonal_trecena"].as_u64().unwrap_or(1);
    let xiu_name = ctx["xiu_month_name"].as_str().unwrap_or("?");
    let xiu_en = ctx["xiu_month_en"].as_str().unwrap_or("?");
    let xiu_day = ctx["xiu_day"].as_u64().unwrap_or(1);
    let cr_tre = ctx["cr_trecena"].as_u64().unwrap_or(1);
    let cr_sign = ctx["cr_sign"].as_str().unwrap_or("?");
    let cr_hday = ctx["cr_haab_day"].as_u64().unwrap_or(0);
    let cr_hmonth = ctx["cr_haab_month"].as_str().unwrap_or("?");
    let sign_idx = ctx["tonal_sign_idx"].as_u64().unwrap_or(0);

    let mut s = String::with_capacity(8 * 1024);
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 700 580" width="700" height="580">
  <rect width="700" height="580" fill="{bg}"/>
  <text x="350" y="28" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{gold}">{title}</text>
  <text x="350" y="46" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".7">{date}</text>"##
    );

    // Tonalpohualli panel (left)
    let _ = writeln!(
        s,
        r##"
  <!-- Aztec Tonalpohualli -->
  <rect x="20" y="65" width="310" height="130" rx="6" fill="none" stroke="{gold}" stroke-width="1.5" opacity=".6"/>
  <text x="175" y="82" text-anchor="middle" font-size="11" font-weight="600"
        fill="{gold}">Tonalpohualli (Aztec 260-day)</text>
  <text x="175" y="106" text-anchor="middle" font-size="32" font-weight="700"
        fill="{gold}">{trecena} {tonal_name}</text>
  <text x="175" y="128" text-anchor="middle" font-size="11"
        fill="{txt}" opacity=".8">{trecena}-{tonal_en}</text>
  <text x="175" y="148" text-anchor="middle" font-size="9"
        fill="{txt}" opacity=".5">Day sign #{}: {tonal_en}</text>
  <text x="175" y="164" text-anchor="middle" font-size="8"
        fill="{txt}" opacity=".4">Trecena (week) {trecena} of 13</text>"##,
        sign_idx + 1
    );

    // Xiuhpohualli panel (right)
    let _ = writeln!(
        s,
        r##"
  <!-- Aztec Xiuhpohualli -->
  <rect x="370" y="65" width="310" height="130" rx="6" fill="none" stroke="{gold}" stroke-width="1.5" opacity=".6"/>
  <text x="525" y="82" text-anchor="middle" font-size="11" font-weight="600"
        fill="{gold}">Xiuhpohualli (Aztec 365-day)</text>
  <text x="525" y="106" text-anchor="middle" font-size="28" font-weight="700"
        fill="{gold}">Day {xiu_day}</text>
  <text x="525" y="128" text-anchor="middle" font-size="14"
        fill="{txt}">{xiu_name}</text>
  <text x="525" y="148" text-anchor="middle" font-size="11"
        fill="{txt}" opacity=".7">{xiu_en}</text>"##
    );

    // Tzolkin (Maya) panel

    // Haab (Maya) panel

    // Calendar Round (bottom centre)
    let _ = writeln!(
        s,
        r##"
  <!-- Calendar Round (52-year cycle) -->
  <rect x="150" y="365" width="400" height="80" rx="6" fill="none" stroke="{gold}" stroke-width="1.5" opacity=".5"/>
  <text x="350" y="382" text-anchor="middle" font-size="11" font-weight="600"
        fill="{gold}">Calendar Round (52-year cycle)</text>
  <text x="350" y="406" text-anchor="middle" font-size="18" font-weight="700"
        fill="{gold}">{cr_tre} {cr_sign} — {cr_hday} {cr_hmonth}</text>
  <text x="350" y="426" text-anchor="middle" font-size="8"
        fill="{txt}" opacity=".45">Tzolkin + Haab combination, repeats every 18,980 days (≈52 years)</text>"##
    );

    // Note
    s.push_str("  <text x=\"350\" y=\"525\" text-anchor=\"middle\" font-size=\"9\" fill=\"#4a9a6a\" opacity=\".5\">Maya</text>\n");
    s.push_str("  <rect x=\"310\" y=\"516\" width=\"12\" height=\"12\" fill=\"#4a9a6a\" opacity=\".4\" rx=\"2\"/>\n");
    let _ = writeln!(s, "  <text x=\"370\" y=\"525\" text-anchor=\"middle\" font-size=\"9\" fill=\"{gold}\" opacity=\".5\">Aztec</text>");
    let _ = writeln!(s, "  <rect x=\"330\" y=\"516\" width=\"12\" height=\"12\" fill=\"{gold}\" opacity=\".4\" rx=\"2\"/>");

    let _ = writeln!(s, "</svg>");
    s
}
