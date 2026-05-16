//! Specialist chart builders — split from render.rs.

use crate::error::CliError;
use super::{
    build_context, fmt_lon_dms, jd_to_date_str, render_builtin_svg, wx, wy, BODIES, CX, CY, RH, RI,
    RO,
};
use celestial_core::AzAlt;
use celestial_core::{lon_to_sign, zodiac_sign_name};

use celestial_core::body::{Body, CalcFlags, HouseSystem};
use celestial_core::{azalt, calc_ut, houses_ex, midpoint_deg, midpoint_table};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt::Write;

/// Build context for a 90° midpoint dial.
/// All planet longitudes are reduced to 0–90° (the dial compresses all four
/// quadrants). Midpoints are computed and planets that trigger them are listed.
pub fn build_dial_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, CliError> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert_with(|| "90° Midpoint Dial".to_string());

    // Collect natal positions
    let bodies = BODIES;
    let mut positions: Vec<(Body, f64)> = Vec::with_capacity(bodies.len());
    let mut planet_entries: Vec<Value> = Vec::with_capacity(bodies.len());

    let h = houses_ex(jd, CalcFlags::BUILTIN, lat, lon, HouseSystem(hsys as u8))
        ?;
    let asc = h.ascmc[0];

    for &(body, key, name, glyph) in bodies {
        if let Ok(pos) = calc_ut(jd, body, flags) {
            let dial_lon = pos.lon % 90.0; // compress to 0–90°
            positions.push((body, pos.lon));
            planet_entries.push(json!({
                "name": name, "key": key, "glyph": glyph,
                "lon": (pos.lon * 1e4).round() / 1e4,
                "dial_lon": (dial_lon * 1e4).round() / 1e4,
                "retro": pos.speed_lon < 0.0}));
        }
    }

    // Compute midpoint table with 1.5° orb
    let midpoints = midpoint_table(&positions, 1.5);
    let mp_entries: Vec<Value> = midpoints
        .iter()
        .map(|(b1, b2, mid, triggers)| {
            let dial_mid = mid % 90.0;
            json!({
                "body1": format!("{b1:?}"),
                "body2": format!("{b2:?}"),
                "midpoint_lon": (mid   * 1e4).round() / 1e4,
                "dial_mid":     (dial_mid * 1e4).round() / 1e4,
                "triggers": triggers.iter().map(|(b, orb)| json!({
                    "body": format!("{b:?}"),
                    "orb":  (orb * 100.0).round() / 100.0})).collect::<Vec<_>>()})
        })
        .collect();

    let palette = super::palette_with_defaults(
        &[
            ("bg_color", "#ffffff"),
            ("ring_color", "#1a1a2e"),
            ("planet_color", "#0d0d1e"),
            ("retro_color", "#b01020"),
            ("soft_color", "#1a50b0"),
            ("text_color", "#0d0d1e"),
        ],
        &vars,
    );

    Ok(json!({
        "date": jd_to_date_str(jd), "date_label": date_str, "jd": jd, "lat": lat, "lon": lon,
        "asc": (asc * 1e4).round() / 1e4,
        "planets": planet_entries,
        "midpoints": mp_entries,
        "vars": Value::Object(palette.into_iter().collect())}))
}

#[allow(clippy::too_many_arguments)]
pub fn build_composite_context(
    jd1: f64,
    jd2: f64,
    lat: f64,
    lon: f64,
    date1: &str,
    date2: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, CliError> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert_with(|| "Composite Chart".to_string());

    let h1 = houses_ex(jd1, CalcFlags::BUILTIN, lat, lon, HouseSystem(hsys as u8))
        ?;
    let h2 = houses_ex(jd2, CalcFlags::BUILTIN, lat, lon, HouseSystem(hsys as u8))
        ?;
    // Composite ASC: midpoint of the two ASCs
    let asc1 = h1.ascmc[0];
    let asc2 = h2.ascmc[0];
    let comp_asc = midpoint_deg(asc1, asc2);

    // Composite planet longitudes: midpoint of each body pair
    let mut planets = Vec::with_capacity(BODIES.len());
    for &(body, key, name, glyph) in BODIES {
        let p1 = calc_ut(jd1, body, flags).ok();
        let p2 = calc_ut(jd2, body, flags).ok();
        if let (Some(pos1), Some(pos2)) = (p1, p2) {
            let comp_lon = midpoint_deg(pos1.lon, pos2.lon);
            let (sign_idx, _) = lon_to_sign(comp_lon);
            planets.push(json!({
                "name": name, "key": key, "glyph": glyph,
                "lon":  (comp_lon * 1e4).round() / 1e4,
                "retro": false, // composite positions don't have retrograde
                "sign_name": zodiac_sign_name(sign_idx),
                "dms": fmt_lon_dms(comp_lon),
                "deg_label": fmt_lon_dms(comp_lon),
                "x":    (wx(CX, RO, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "y":    (wy(CY, RO, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "label_x": (wx(CX, RO + 20.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "label_y": (wy(CY, RO + 20.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "asp_x":  (wx(CX, RO, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "asp_y":  (wy(CY, RO, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "tick_x1":(wx(CX, RO + 2.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "tick_y1":(wy(CY, RO + 2.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "tick_x2":(wx(CX, RO - 12.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "tick_y2":(wy(CY, RO - 12.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "near_station": false,
                "dignity": "peregrine",
                "antiscia_lon": (180.0 - comp_lon).rem_euclid(360.0),
                "antiscia_x": 0.0, "antiscia_y": 0.0,
                "speed": 0.0, "speed_str": "", "dist": 0.0, "lat": 0.0,
                "sign": lon_to_sign(comp_lon).0}));
        }
    }

    let date_str = format!("Composite: {date1} / {date2}");
    // Use existing build_context at the midpoint JD for houses/signs geometry
    let comp_jd = (jd1 + jd2) / 2.0;
    let mut ctx = build_context(comp_jd, lat, lon, &date_str, hsys, vars)?;
    // Override planets with composite positions
    ctx["planets"] = Value::Array(planets);
    ctx["asc"] = json!((comp_asc * 1e4).round() / 1e4);
    Ok(ctx)
}

#[allow(clippy::too_many_arguments)]
pub fn build_triwheel_context(
    jd1: f64,
    jd2: f64,
    jd3: f64,
    lat: f64,
    lon: f64,
    date1: &str,
    date2: &str,
    date3: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, CliError> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert_with(|| "Tri-wheel".to_string());

    // Build natal (inner) context
    let inner_vars = vars.clone();
    let inner = build_context(jd1, lat, lon, date1, hsys, inner_vars)?;
    let asc = inner["asc"].as_f64().unwrap_or(0.0);

    // Rings 2 and 3: just planetary positions, no houses
    let mut ring2 = Vec::with_capacity(BODIES.len());
    let mut ring3 = Vec::with_capacity(BODIES.len());
    const R2: f64 = RH + 26.0; // progressed ring
    const R3: f64 = RI - 4.0; // transit ring (inside sign band)

    for &(body, key, name, glyph) in BODIES {
        for (jd_r, ring, r) in [(jd2, &mut ring2, R2), (jd3, &mut ring3, R3)] {
            if let Ok(pos) = calc_ut(jd_r, body, flags) {
                ring.push(json!({
                    "name": name, "key": key, "glyph": glyph,
                    "lon":  (pos.lon * 1e4).round() / 1e4,
                    "retro": pos.speed_lon < 0.0,
                    "dms":  fmt_lon_dms(pos.lon),
                    "x": (wx(CX, r, pos.lon, asc) * 100.0).round() / 100.0,
                    "y": (wy(CY, r, pos.lon, asc) * 100.0).round() / 100.0}));
            }
        }
    }

    let date_str = format!("Tri-wheel: {date1} / {date2} / {date3}");
    let palette = super::palette_with_defaults(
        &[
            ("bg_color", "#ffffff"),
            ("ring_color", "#1a1a2e"),
            ("planet_color", "#0d0d1e"),
            ("retro_color", "#b01020"),
            ("hard_color", "#b01020"),
            ("soft_color", "#1a50b0"),
            ("text_color", "#0d0d1e"),
        ],
        &vars,
    );

    let mut ctx = inner;
    ctx["ring2_planets"] = Value::Array(ring2);
    ctx["ring3_planets"] = Value::Array(ring3);
    ctx["date2"] = json!(date2);
    ctx["date3"] = json!(date3);
    ctx["date"] = json!(date_str);
    ctx["vars"] = Value::Object(palette.into_iter().collect());
    Ok(ctx)
}

pub fn render_triwheel_svg(ctx: &Value) -> String {
    let mut s = render_builtin_svg(ctx);

    let ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#1a1a2e");
    let retro_c = ctx["vars"]["retro_color"].as_str().unwrap_or("#b01020");

    let ring2_col = "#0a6b3c"; // green for ring 2 (progressed)
    let ring3_col = "#6b0a3c"; // magenta for ring 3 (transits)

    let mut extra = String::new();

    // Extra rings
    let r2 = RH + 26.0;
    let r3 = RI - 4.0;
    let _ = write!(extra,
        "  <circle cx=\"{CX}\" cy=\"{CY}\" r=\"{r2:.1}\" fill=\"none\" stroke=\"{ring2_col}\" stroke-width=\"1.2\" stroke-dasharray=\"5,3\" opacity=\".45\"/>\n\
         <circle cx=\"{CX}\" cy=\"{CY}\" r=\"{r3:.1}\" fill=\"none\" stroke=\"{ring3_col}\" stroke-width=\"1.2\" stroke-dasharray=\"3,2\" opacity=\".45\"/>\n"
    );

    // Ring 2 (progressed)
    let ring2 = super::json_array(&ctx["ring2_planets"]);
    for p in ring2 {
        let px = p["x"].as_f64().unwrap_or(0.0);
        let py = p["y"].as_f64().unwrap_or(0.0);
        let g = p["glyph"].as_str().unwrap_or("?");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let col = if ret { retro_c } else { ring2_col };
        let _ = writeln!(
            extra,
            r##"  <text x="{px:.2}" y="{py:.2}" font-size="12" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}" opacity=".85">{g}</text>"##
        );
    }

    // Ring 3 (transits)
    let ring3 = super::json_array(&ctx["ring3_planets"]);
    for p in ring3 {
        let px = p["x"].as_f64().unwrap_or(0.0);
        let py = p["y"].as_f64().unwrap_or(0.0);
        let g = p["glyph"].as_str().unwrap_or("?");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let col = if ret { retro_c } else { ring3_col };
        let _ = writeln!(
            extra,
            r##"  <text x="{px:.2}" y="{py:.2}" font-size="11" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}" opacity=".8">{g}</text>"##
        );
    }

    // Legend
    let d2 = ctx["date2"].as_str().unwrap_or("ring 2");
    let d3 = ctx["date3"].as_str().unwrap_or("ring 3");
    let ly = CY + RO + 20.0;
    let _ = writeln!(
        extra,
        r##"  <text x="450" y="{ly:.2}" text-anchor="middle" font-size="9" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".55">— natal  ·  <tspan fill="{ring2_col}">■ {d2}</tspan>  ·  <tspan fill="{ring3_col}">■ {d3}</tspan> —</text>"##
    );

    if let Some(idx) = s.rfind("</svg>") {
        s.insert_str(idx, &extra);
    }
    s
}

pub fn build_graphic_ephemeris_context(
    jd_start: f64,
    jd_end: f64,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, CliError> {
    let flags = CalcFlags::BUILTIN;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert_with(|| "Graphic Ephemeris".to_string());

    let days = ((jd_end - jd_start) as usize).clamp(28, 366);
    let step = if days <= 31 { 1 } else { (days / 90).max(1) };

    // Sample planetary positions over the date range
    let n_samples = days / step.max(1) + 2;
    let mut series: Vec<Vec<f64>> = (0..BODIES.len())
        .map(|_| Vec::with_capacity(n_samples))
        .collect();
    let mut jd_points: Vec<f64> = Vec::with_capacity(n_samples);

    let mut jd = jd_start;
    while jd <= jd_end + 0.5 {
        jd_points.push((jd * 100.0).round() / 100.0);
        for (i, &(body, ..)) in BODIES.iter().enumerate() {
            if let Ok(pos) = calc_ut(jd, body, flags) {
                series[i].push((pos.lon * 100.0).round() / 100.0);
            } else {
                series[i].push(f64::NAN);
            }
        }
        jd += step as f64;
    }

    let planet_series: Vec<Value> = BODIES
        .iter()
        .enumerate()
        .map(|(i, &(_, key, name, glyph))| {
            json!({
                "name": name, "key": key, "glyph": glyph,
                "lons": series[i]})
        })
        .collect();

    let palette = super::palette_with_defaults(
        &[
            ("bg_color", "#ffffff"),
            ("ring_color", "#1a1a2e"),
            ("text_color", "#0d0d1e"),
        ],
        &vars,
    );

    Ok(json!({
        "jd_start": jd_start, "jd_end": jd_end,
        "jd_points": jd_points,
        "days": days, "step": step,
        "planet_series": planet_series,
        "vars": Value::Object(palette.into_iter().collect())}))
}

// Graphic ephemeris layout constants
const GE_W: f64 = 860.0;
const GE_H: f64 = 480.0;
const GE_LM: f64 = 32.0;
const GE_TM: f64 = 60.0;
const GE_BM: f64 = 80.0;

const GE_PLANET_COLORS: &[&str] = &[
    "#d4a800", "#9b59b6", "#3498db", "#27ae60", "#e74c3c", "#e67e22", "#1a5276", "#117a65",
    "#6c3483", "#7f8c8d", "#2e86c1", "#b7950b",
];

const GE_SIGN_GLYPHS: [&str; 12] = [
    "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓",
];

pub fn render_graphic_ephemeris_svg(ctx: &Value) -> String {
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fff");
    let ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#1a1a2e");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#0d0d1e");

    let jd_start = ctx["jd_start"].as_f64().unwrap_or(0.0);
    let jd_end = ctx["jd_end"].as_f64().unwrap_or(0.0);
    let jd_span = (jd_end - jd_start).max(1.0);

    let jd_points = super::json_array(&ctx["jd_points"]);
    if jd_points.is_empty() {
        return String::new();
    }

    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Graphic Ephemeris");

    let mut s = String::with_capacity(64 * 1024);
    let total_h = GE_TM + GE_H + GE_BM + 40.0;
    write_ge_header(&mut s, bg, txt, title, total_h);
    write_ge_y_axis(&mut s, ring);

    let x_scale = GE_W / jd_span;
    let series = super::json_array(&ctx["planet_series"]);
    for (pi, planet) in series.iter().enumerate() {
        write_ge_planet_series(&mut s, planet, pi, jd_points, jd_start, x_scale);
    }

    write_ge_x_axis_labels(&mut s, ring, jd_start, jd_end, jd_span, x_scale);

    let _ = writeln!(
        s,
        r##"  <rect x="{GE_LM:.2}" y="{GE_TM:.2}" width="{GE_W:.2}" height="{GE_H:.2}" fill="none" stroke="{ring}" stroke-width="0.8" opacity=".4"/>"##
    );
    let _ = writeln!(s, "</svg>");
    s
}

fn write_ge_header(s: &mut String, bg: &str, txt: &str, title: &str, total_h: f64) {
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 {total_h:.0}" width="900" height="{total_h:.0}">
  <rect width="900" height="{total_h:.0}" fill="{bg}"/>
  <text x="450" y="28" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>"##
    );
}

fn write_ge_y_axis(s: &mut String, ring: &str) {
    for s_idx in 0..=12u32 {
        let lon = s_idx as f64 * 30.0;
        let y = GE_TM + GE_H - (lon / 360.0) * GE_H;
        let op = if s_idx % 3 == 0 { ".4" } else { ".2" };
        let _ = writeln!(
            s,
            r##"  <line x1="{:.2}" y1="{y:.2}" x2="{:.2}" y2="{y:.2}" stroke="{ring}" stroke-width="0.6" opacity="{op}"/>"##,
            GE_LM,
            GE_LM + GE_W
        );
        if s_idx < 12 {
            let gy = GE_TM + GE_H - ((lon + 15.0) / 360.0) * GE_H;
            let _ = writeln!(
                s,
                r##"  <text x="{:.2}" y="{gy:.2}" font-size="11" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{ring}" opacity=".6">{}</text>"##,
                GE_LM - 10.0,
                GE_SIGN_GLYPHS[s_idx as usize]
            );
        }
    }
}

fn ge_build_path(lons: &[Value], jd_points: &[Value], jd_start: f64, x_scale: f64) -> String {
    let mut path = String::new();
    let mut first = true;
    for (xi, jd_val) in jd_points.iter().enumerate() {
        let jd_v = jd_val.as_f64().unwrap_or(0.0);
        let lon = lons
            .get(xi)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(f64::NAN);
        if !lon.is_finite() {
            first = true;
            continue;
        }
        let x = GE_LM + (jd_v - jd_start) * x_scale;
        let y = GE_TM + GE_H - (lon / 360.0) * GE_H;
        let cmd = if first { 'M' } else { 'L' };
        let _ = write!(path, "{cmd}{x:.1},{y:.1}");
        first = false;
    }
    path
}

fn write_ge_end_glyph(
    s: &mut String,
    lons: &[Value],
    jd_points: &[Value],
    jd_start: f64,
    x_scale: f64,
    col: &str,
    glyph: &str,
) {
    let last_jd = jd_points.last().and_then(serde_json::Value::as_f64);
    let last_lon = lons.last().and_then(serde_json::Value::as_f64);
    let (Some(last_jd), Some(last_lon)) = (last_jd, last_lon) else {
        return;
    };
    if !last_lon.is_finite() {
        return;
    }
    let lx = GE_LM + (last_jd - jd_start) * x_scale + 8.0;
    let ly = GE_TM + GE_H - (last_lon / 360.0) * GE_H;
    let _ = writeln!(
        s,
        r##"  <text x="{lx:.1}" y="{ly:.1}" font-size="12" dominant-baseline="central" font-family="serif" fill="{col}">{glyph}</text>"##
    );
}

fn write_ge_planet_series(
    s: &mut String,
    planet: &Value,
    pi: usize,
    jd_points: &[Value],
    jd_start: f64,
    x_scale: f64,
) {
    let lons = super::json_array(&planet["lons"]);
    let col = GE_PLANET_COLORS.get(pi).copied().unwrap_or("#888");
    let glyph = planet["glyph"].as_str().unwrap_or("?");
    let path = ge_build_path(lons, jd_points, jd_start, x_scale);
    if !path.is_empty() {
        let _ = writeln!(
            s,
            r##"  <path d="{path}" fill="none" stroke="{col}" stroke-width="1.5" opacity=".8"/>"##
        );
    }
    write_ge_end_glyph(s, lons, jd_points, jd_start, x_scale, col, glyph);
}

fn write_ge_x_axis_labels(
    s: &mut String,
    ring: &str,
    jd_start: f64,
    jd_end: f64,
    jd_span: f64,
    x_scale: f64,
) {
    let label_interval = (jd_span / 6.0).max(7.0);
    let mut jd_lbl = jd_start;
    while jd_lbl <= jd_end + 1.0 {
        let x = GE_LM + (jd_lbl - jd_start) * x_scale;
        let d = celestial_core::revjul(jd_lbl, celestial_core::body::Calendar::Gregorian);
        let lbl = format!("{:.0}-{:02.0}", d.year, d.month);
        let _ = writeln!(
            s,
            r##"  <line x1="{x:.1}" y1="{:.1}" x2="{x:.1}" y2="{:.1}" stroke="{ring}" stroke-width="0.6" opacity=".3"/>
  <text x="{x:.1}" y="{:.1}" font-size="9" text-anchor="middle" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".7">{lbl}</text>"##,
            GE_TM,
            GE_TM + GE_H,
            GE_TM + GE_H + 14.0
        );
        jd_lbl += label_interval;
    }
}

pub fn build_local_space_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, CliError> {
    let flags = CalcFlags::BUILTIN;
    let geopos = [lon, lat, 0.0_f64];
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert_with(|| "Local Space Chart".to_string());

    let mut planets = Vec::with_capacity(BODIES.len());
    for &(body, key, name, glyph) in BODIES {
        if let Ok(pos) = calc_ut(jd, body, flags) {
            // Convert to azimuth/altitude using azalt
            let az_result: AzAlt = azalt(jd, 0, geopos, 0.0, 10.0, [pos.lon, pos.lat, pos.dist]);
            let az = az_result.azimuth;
            let alt = az_result.true_alt;
            planets.push(json!({
                "name": name, "key": key, "glyph": glyph,
                "lon": (pos.lon * 1e4).round() / 1e4,
                "azimuth":  (az  * 100.0).round() / 100.0,
                "altitude": (alt * 100.0).round() / 100.0,
                "above_horizon": alt > 0.0}));
        }
    }

    let palette = super::palette_with_defaults(
        &[
            ("bg_color", "#ffffff"),
            ("ring_color", "#1a1a2e"),
            ("planet_color", "#0d0d1e"),
            ("retro_color", "#b01020"),
            ("text_color", "#0d0d1e"),
        ],
        &vars,
    );

    Ok(json!({
        "date": jd_to_date_str(jd), "date_label": date_str, "jd": jd, "lat": lat, "lon": lon,
        "planets": planets,
        "vars": Value::Object(palette.into_iter().collect())}))
}

const LS_CX: f64 = 450.0;
const LS_CY: f64 = 450.0;
const LS_R: f64 = 320.0;

#[allow(clippy::too_many_arguments)]
fn write_ls_header(
    s: &mut String,
    bg: &str,
    ring: &str,
    txt: &str,
    title: &str,
    date: &str,
    lat: f64,
    lon_v: f64,
) {
    let ns = if lat >= 0.0 { "N" } else { "S" };
    let ew = if lon_v >= 0.0 { "E" } else { "W" };
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 980" width="900" height="980">
  <rect width="900" height="980" fill="{bg}"/>
  <text x="450" y="30" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="450" y="48" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">{date}</text>
  <text x="450" y="62" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".5">{:.4}°{ns} {:.4}°{ew}</text>
  <!-- Compass rings -->"##,
        lat.abs(),
        lon_v.abs(),
    );
}

fn write_ls_concentric_rings(s: &mut String, ring: &str) {
    for i in 1..=3u32 {
        let r = LS_R * i as f64 / 3.0;
        let deg = 90 * i;
        let ring_a = (-90.0_f64).to_radians();
        let tx = LS_CX + (r + 4.0) * ring_a.cos();
        let ty = LS_CY - (r + 4.0) * ring_a.sin();
        let _ = writeln!(
            s,
            r##"  <circle cx="{LS_CX}" cy="{LS_CY}" r="{r:.1}" fill="none" stroke="{ring}" stroke-width="0.7" opacity=".2"/>
  <text x="{tx:.2}" y="{ty:.2}" font-size="8" text-anchor="middle" fill="{ring}" opacity=".35">{deg}°</text>"##
        );
    }
}

fn write_ls_cardinals(s: &mut String, ring: &str) {
    for (deg, label) in [(0.0_f64, "N"), (90.0, "E"), (180.0, "S"), (270.0, "W")] {
        let a = (deg - 90.0).to_radians();
        let x1 = LS_CX + (LS_R - 5.0) * a.cos();
        let y1 = LS_CY + (LS_R - 5.0) * a.sin();
        let x2 = LS_CX + (LS_R + 20.0) * a.cos();
        let y2 = LS_CY + (LS_R + 20.0) * a.sin();
        let _ = writeln!(
            s,
            r##"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{ring}" stroke-width="1.5" opacity=".55"/>
  <text x="{x2:.2}" y="{y2:.2}" font-size="14" font-weight="700" text-anchor="middle" dominant-baseline="central" fill="{ring}">{label}</text>"##
        );
    }
}

fn write_ls_degree_tick(s: &mut String, ring: &str, deg: u32) {
    let a = (deg as f64 - 90.0).to_radians();
    let is_30 = deg.is_multiple_of(30);
    let (r1, r2) = if is_30 {
        (LS_R - 10.0, LS_R)
    } else {
        (LS_R - 5.0, LS_R)
    };
    let x1 = LS_CX + r1 * a.cos();
    let y1 = LS_CY + r1 * a.sin();
    let x2 = LS_CX + r2 * a.cos();
    let y2 = LS_CY + r2 * a.sin();
    let _ = writeln!(
        s,
        r##"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{ring}" stroke-width="0.8" opacity=".4"/>"##
    );
    if is_30 && deg > 0 {
        let xd = LS_CX + (LS_R + 12.0) * a.cos();
        let yd = LS_CY + (LS_R + 12.0) * a.sin();
        let _ = writeln!(
            s,
            r##"  <text x="{xd:.2}" y="{yd:.2}" font-size="8" text-anchor="middle" dominant-baseline="central" fill="{ring}" opacity=".5">{deg}°</text>"##
        );
    }
}

fn write_ls_planet(s: &mut String, p: &Value, pfg: &str) {
    let az = p["azimuth"].as_f64().unwrap_or(0.0);
    let alt = p["altitude"].as_f64().unwrap_or(0.0);
    let above = p["above_horizon"].as_bool().unwrap_or(false);
    let g = p["glyph"].as_str().unwrap_or("?");
    let r_planet = if above { LS_R * 0.88 } else { LS_R * 0.55 };
    let a = (az - 90.0).to_radians();
    let px = LS_CX + r_planet * a.cos();
    let py = LS_CY + r_planet * a.sin();
    let (sw, dash, op) = if above {
        ("1.5", "", ".7")
    } else {
        ("1.0", r##" stroke-dasharray="4,3""##, ".4")
    };
    let _ = writeln!(
        s,
        r##"  <line x1="{LS_CX}" y1="{LS_CY}" x2="{px:.2}" y2="{py:.2}" stroke="{pfg}" stroke-width="{sw}" opacity="{op}"{dash}/>
  <text x="{px:.2}" y="{py:.2}" font-size="16" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{pfg}" opacity="{op}">{g}</text>
  <text x="{:.2}" y="{:.2}" font-size="8" text-anchor="middle" font-family="'Segoe UI',system-ui,sans-serif" fill="{pfg}" opacity=".55">{az:.0}°</text>"##,
        px + (px - LS_CX) * 0.12,
        py + (py - LS_CY) * 0.12
    );
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{:.2}" font-size="7" text-anchor="middle" font-family="'Segoe UI',system-ui,sans-serif" fill="{pfg}" opacity=".4">{:+.1}°</text>"##,
        px + (px - LS_CX) * 0.2,
        py + (py - LS_CY) * 0.2,
        alt
    );
}

pub fn render_local_space_svg(ctx: &Value) -> String {
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fff");
    let ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#1a1a2e");
    let pfg = ctx["vars"]["planet_color"].as_str().unwrap_or("#0d0d1e");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#0d0d1e");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Local Space");
    let date = ctx["date"].as_str().unwrap_or("");
    let lat = ctx["lat"].as_f64().unwrap_or(0.0);
    let lon_v = ctx["lon"].as_f64().unwrap_or(0.0);

    let mut s = String::with_capacity(32 * 1024);
    write_ls_header(&mut s, bg, ring, txt, title, date, lat, lon_v);
    write_ls_concentric_rings(&mut s, ring);
    write_ls_cardinals(&mut s, ring);
    for deg in (0..360u32).step_by(10) {
        write_ls_degree_tick(&mut s, ring, deg);
    }
    let planets = super::json_array(&ctx["planets"]);
    for p in planets {
        write_ls_planet(&mut s, p, pfg);
    }
    let _ = writeln!(s, "</svg>");
    s
}

/// Render a 90° midpoint dial SVG.
pub fn render_dial_svg(ctx: &serde_json::Value) -> String {
    use std::fmt::Write;
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fff");
    let ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#1a1a2e");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#0d0d1e");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("90° Dial");
    let date = ctx["date"].as_str().unwrap_or("");
    let planets = super::json_array(&ctx["planets"]);

    const CR: f64 = 200.0; // dial radius
    let mut s = String::with_capacity(8 * 1024);
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 700 600" width="700" height="600">
  <rect width="700" height="600" fill="{bg}"/>
  <text x="350" y="26" text-anchor="middle" font-size="15" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">{title}</text>
  <text x="350" y="42" text-anchor="middle" font-size="9" fill="{ring}" opacity=".6">{date}</text>
  <circle cx="350" cy="320" r="{CR}" fill="none" stroke="{ring}" stroke-width="1.5" opacity=".5"/>"##
    );

    // Degree marks every 5°
    for deg in (0u32..90).step_by(5) {
        let a = (deg as f64 - 90.0).to_radians();
        let r1 = CR - 8.0;
        let r2 = CR;
        let (x1, y1) = (350.0 + r1 * a.cos(), 320.0 - r1 * a.sin());
        let (x2, y2) = (350.0 + r2 * a.cos(), 320.0 - r2 * a.sin());
        let _ = writeln!(
            s,
            r##"  <line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{ring}" stroke-width="0.8" opacity=".4"/>"##
        );
    }

    // Plot planets
    for p in planets {
        let dial_lon = p["dial_lon"].as_f64().unwrap_or(0.0);
        let glyph = p["glyph"].as_str().unwrap_or("●");
        let a = (dial_lon - 90.0).to_radians();
        let rx = 350.0 + (CR - 18.0) * a.cos();
        let ry = 320.0 - (CR - 18.0) * a.sin();
        let _ = writeln!(
            s,
            r##"  <text x="{rx:.1}" y="{ry:.1}" font-size="11" text-anchor="middle" dominant-baseline="central" fill="{txt}">{glyph}</text>"##
        );
    }

    let _ = writeln!(s, "</svg>");
    s
}
