//! Indigenous chart builders — split from render.rs.


use celestial_core::body::{Body, CalcFlags};
use celestial_core::calc_ut;
use celestial_core::{egyptian_decan, medicine_wheel_totem};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn build_medicine_wheel_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Medicine Wheel".to_string());

    let sun_pos = calc_ut(jd, Body::SUN, flags).map_err(|e| e.to_string())?;
    let (animal, element, clan, season) = medicine_wheel_totem(sun_pos.lon);
    let (decan_idx, decan_name, decan_star) = egyptian_decan(sun_pos.lon);

    let palette = super::palette_with_defaults(
        &[
        ("bg_color", "#0a1a0a"),
        ("border_color", "#a0c040"),
        ("text_color", "#d0e8a0"),
        ("planet_color", "#80c060"),
        ],
        &vars,
    );

    Ok(json!({
        "date": date_str, "jd": jd, "lat": lat, "lon": lon,
        "sun_lon":    (sun_pos.lon * 1e4).round() / 1e4,
        "totem":      animal,
        "element":    element,
        "clan":       clan,
        "season":     season,
        "decan_idx":  decan_idx,
        "decan_name": decan_name,
        "decan_star": decan_star,
        "vars": Value::Object(palette.into_iter().collect())}))
}

pub fn render_medicine_wheel_svg(ctx: &Value) -> String {
    use std::fmt::Write;

    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#0a1a0a");
    let green = ctx["vars"]["border_color"].as_str().unwrap_or("#a0c040");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#d0e8a0");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Medicine Wheel");
    let date = ctx["date"].as_str().unwrap_or("");
    let totem = ctx["totem"].as_str().unwrap_or("?");
    let element = ctx["element"].as_str().unwrap_or("?");
    let clan = ctx["clan"].as_str().unwrap_or("?");
    let season = ctx["season"].as_str().unwrap_or("?");
    let sun_lon = ctx["sun_lon"].as_f64().unwrap_or(0.0);
    let decan_name = ctx["decan_name"].as_str().unwrap_or("?");
    let decan_star = ctx["decan_star"].as_str().unwrap_or("?");
    let decan_idx = ctx["decan_idx"].as_u64().unwrap_or(0);

    const CX: f64 = 350.0;
    const CY: f64 = 300.0;
    const R: f64 = 200.0;

    let mut s = String::with_capacity(8 * 1024);
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 700 600" width="700" height="600">
  <rect width="700" height="600" fill="{bg}"/>
  <text x="350" y="26" text-anchor="middle" font-size="15" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{green}">{title}</text>
  <text x="350" y="43" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".55">{date}</text>
  <text x="350" y="55" text-anchor="middle" font-size="7" fill="{txt}" opacity=".35">
    Sun Bear Medicine Wheel synthesis (1980) — modern system, not traditional indigenous</text>"##
    );

    // Outer circle
    let _ = writeln!(
        s,
        r##"  <circle cx="{CX}" cy="{CY}" r="{R}" fill="none" stroke="{green}" stroke-width="2.0" opacity=".6"/>"##
    );

    // Cardinal directions
    for (ang, dir, col) in [
        (90.0_f64, "N", "#ffffff"),
        (0.0, "E", "#ffff40"),
        (270.0, "S", "#c06020"),
        (180.0, "W", "#404040"),
    ] {
        let a = ang.to_radians();
        let x = CX + (R + 18.0) * a.cos();
        let y = CY - (R + 18.0) * a.sin();
        let _ = writeln!(
            s,
            r##"  <text x="{x:.1}" y="{y:.1}" font-size="14" font-weight="700" text-anchor="middle"
          dominant-baseline="central" fill="{col}">{dir}</text>"##
        );
    }

    // 12 totem positions (every 30°)
    const TOTEMS_12: &[(&str, &str)] = &[
        ("Snow Goose", "Earth"),
        ("Otter", "Air"),
        ("Cougar", "Air"),
        ("Red Hawk", "Fire"),
        ("Beaver", "Earth"),
        ("Deer", "Air"),
        ("Flicker", "Water"),
        ("Sturgeon", "Fire"),
        ("Brown Bear", "Earth"),
        ("Raven", "Air"),
        ("Snake", "Water"),
        ("Elk", "Fire"),
    ];
    for (i, &(totem_i, elem_i)) in TOTEMS_12.iter().enumerate() {
        let ang = (i as f64 * 30.0 + 90.0).to_radians();
        let tx = CX + (R - 28.0) * ang.cos();
        let ty = CY - (R - 28.0) * ang.sin();
        let col = match elem_i {
            "Fire" => "#c04030",
            "Water" => "#3060c0",
            "Earth" => "#806020",
            _ => "#406040",
        };
        let _ = writeln!(
            s,
            r##"  <text x="{tx:.1}" y="{ty:.1}" font-size="8" text-anchor="middle"
          dominant-baseline="central" fill="{col}" opacity=".7">{totem_i}</text>"##
        );
    }

    // Sun marker
    let sun_ang = (sun_lon + 90.0).to_radians();
    let sx = CX + R * sun_ang.cos();
    let sy = CY - R * sun_ang.sin();
    let _ = writeln!(
        s,
        r##"  <circle cx="{sx:.2}" cy="{sy:.2}" r="8" fill="#ffd040" opacity=".9"/>"##
    );
    let _ = writeln!(s, "  <text x=\"{sx:.2}\" y=\"{sy:.2}\" font-size=\"10\" text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"#1a1a00\">☉</text>");

    // Centre
    let _ = writeln!(
        s,
        r##"  <text x="{CX}" y="{:.1}" text-anchor="middle" font-size="20" font-weight="700"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{green}">{totem}</text>
  <text x="{CX}" y="{:.1}" text-anchor="middle" font-size="11" fill="{txt}">{element} · {clan} Clan</text>
  <text x="{CX}" y="{:.1}" text-anchor="middle" font-size="10" fill="{txt}" opacity=".7">{season}</text>"##,
        CY - 10.0,
        CY + 12.0,
        CY + 28.0
    );

    // Egyptian decan section
    let _ = writeln!(s,
        "  <text x=\"350\" y=\"520\" text-anchor=\"middle\" font-size=\"12\" font-weight=\"600\" fill=\"#c8a030\">Egyptian Decan {} — {decan_name}</text>",
        decan_idx + 1);
    let _ = writeln!(s,
        "  <text x=\"350\" y=\"538\" text-anchor=\"middle\" font-size=\"10\" fill=\"{txt}\" opacity=\".7\">Rising star: {decan_star}</text>");

    let _ = writeln!(s, "</svg>");
    s
}
