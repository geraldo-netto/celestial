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

const MW_CX: f64 = 350.0;
const MW_CY: f64 = 300.0;
const MW_R: f64 = 200.0;

const CARDINALS: &[(f64, &str, &str)] = &[
    (90.0, "N", "#ffffff"),
    (0.0, "E", "#ffff40"),
    (270.0, "S", "#c06020"),
    (180.0, "W", "#404040"),
];

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

fn element_color(elem: &str) -> &'static str {
    match elem {
        "Fire" => "#c04030",
        "Water" => "#3060c0",
        "Earth" => "#806020",
        _ => "#406040",
    }
}

fn write_mw_header(s: &mut String, bg: &str, green: &str, txt: &str, title: &str, date: &str) {
    use std::fmt::Write;
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
    Sun Bear Medicine Wheel synthesis (1980) — modern system, not traditional indigenous</text>
  <circle cx="{MW_CX}" cy="{MW_CY}" r="{MW_R}" fill="none" stroke="{green}" stroke-width="2.0" opacity=".6"/>"##
    );
}

fn write_mw_cardinals(s: &mut String) {
    use std::fmt::Write;
    for &(ang, dir, col) in CARDINALS {
        let (sin_a, cos_a) = ang.to_radians().sin_cos();
        let x = (r_outer()).mul_add(cos_a, MW_CX);
        let y = MW_CY - (r_outer()) * sin_a;
        let _ = writeln!(
            s,
            r##"  <text x="{x:.1}" y="{y:.1}" font-size="14" font-weight="700" text-anchor="middle"
          dominant-baseline="central" fill="{col}">{dir}</text>"##
        );
    }
}

#[inline]
fn r_outer() -> f64 {
    MW_R + 18.0
}

fn write_mw_totems(s: &mut String) {
    use std::fmt::Write;
    for (i, &(totem_i, elem_i)) in TOTEMS_12.iter().enumerate() {
        let (sin_a, cos_a) = (i as f64 * 30.0 + 90.0).to_radians().sin_cos();
        let r = MW_R - 28.0;
        let tx = r.mul_add(cos_a, MW_CX);
        let ty = MW_CY - r * sin_a;
        let col = element_color(elem_i);
        let _ = writeln!(
            s,
            r##"  <text x="{tx:.1}" y="{ty:.1}" font-size="8" text-anchor="middle"
          dominant-baseline="central" fill="{col}" opacity=".7">{totem_i}</text>"##
        );
    }
}

fn write_mw_sun(s: &mut String, sun_lon: f64) {
    use std::fmt::Write;
    let (sin_a, cos_a) = (sun_lon + 90.0).to_radians().sin_cos();
    let sx = MW_R.mul_add(cos_a, MW_CX);
    let sy = MW_CY - MW_R * sin_a;
    let _ = writeln!(
        s,
        r##"  <circle cx="{sx:.2}" cy="{sy:.2}" r="8" fill="#ffd040" opacity=".9"/>
  <text x="{sx:.2}" y="{sy:.2}" font-size="10" text-anchor="middle" dominant-baseline="central" fill="#1a1a00">☉</text>"##
    );
}

fn write_mw_centre(s: &mut String, green: &str, txt: &str, totem: &str, element: &str, clan: &str, season: &str) {
    use std::fmt::Write;
    let _ = writeln!(
        s,
        r##"  <text x="{MW_CX}" y="{:.1}" text-anchor="middle" font-size="20" font-weight="700"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{green}">{totem}</text>
  <text x="{MW_CX}" y="{:.1}" text-anchor="middle" font-size="11" fill="{txt}">{element} · {clan} Clan</text>
  <text x="{MW_CX}" y="{:.1}" text-anchor="middle" font-size="10" fill="{txt}" opacity=".7">{season}</text>"##,
        MW_CY - 10.0,
        MW_CY + 12.0,
        MW_CY + 28.0
    );
}

fn write_mw_decan(s: &mut String, txt: &str, decan_idx: u64, decan_name: &str, decan_star: &str) {
    use std::fmt::Write;
    let _ = writeln!(
        s,
        "  <text x=\"350\" y=\"520\" text-anchor=\"middle\" font-size=\"12\" font-weight=\"600\" fill=\"#c8a030\">Egyptian Decan {} — {decan_name}</text>",
        decan_idx + 1
    );
    let _ = writeln!(
        s,
        "  <text x=\"350\" y=\"538\" text-anchor=\"middle\" font-size=\"10\" fill=\"{txt}\" opacity=\".7\">Rising star: {decan_star}</text>"
    );
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

    let mut s = String::with_capacity(8 * 1024);
    write_mw_header(&mut s, bg, green, txt, title, date);
    write_mw_cardinals(&mut s);
    write_mw_totems(&mut s);
    write_mw_sun(&mut s, sun_lon);
    write_mw_centre(&mut s, green, txt, totem, element, clan, season);
    write_mw_decan(&mut s, txt, decan_idx, decan_name, decan_star);
    let _ = writeln!(s, "</svg>");
    s
}
