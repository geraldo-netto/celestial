//! Hellenistic chart builders — split from render.rs.

use super::{build_context, jd_to_date_str, key_to_body, render_builtin_svg, wx, wy, CX, CY, RO};

use celestial_core::monthly_profection;

use celestial_core::body::{Body, CalcFlags, HouseSystem};
use celestial_core::{
    almuten, decan_ruler, egyptian_terms_ruler, firdaria, full_dignity, is_day_chart, same_sect,
    triplicity_rulers,
};
use celestial_core::{annual_profection, calc_ut, houses_ex, sign_ruler};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn build_hellenistic_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Hellenistic Chart".to_string());

    // Start from the standard context
    let mut ctx = build_context(jd, lat, lon, date_str, hsys, vars)?;

    let h = houses_ex(jd, CalcFlags::BUILTIN, lat, lon, HouseSystem(hsys as u8))
        .map_err(|e| e.to_string())?;
    let cusps_arr: [f64; 13] = {
        let mut a = [0.0f64; 13];
        a.copy_from_slice(&h.cusps);
        a
    };

    // Determine sect
    let sun_lon = ctx["planets"]
        .as_array()
        .and_then(|p| p.iter().find(|p| p["key"] == "sun"))
        .and_then(|p| p["lon"].as_f64())
        .unwrap_or(0.0);
    let is_day = is_day_chart(sun_lon, &cusps_arr);

    // Augment each planet with Phase 5 dignity data
    if let Some(planets) = ctx["planets"].as_array_mut() {
        for p in planets.iter_mut() {
            let plon = p["lon"].as_f64().unwrap_or(0.0);
            let body_key = p["key"].as_str().unwrap_or("");
            let body = key_to_body(body_key);

            if let Some(body) = body {
                let (dignity, score) = full_dignity(body, plon, is_day);
                let term_ruler = egyptian_terms_ruler(plon);
                let decan = decan_ruler(plon);
                let (trip_d, trip_n, trip_p) = triplicity_rulers(plon);
                let (alm, alm_score) = almuten(plon, is_day);
                let sect_ok = same_sect(body, is_day);

                p["dignity5"] = json!(dignity.to_string());
                p["dignity_score"] = json!(score);
                p["term_ruler"] = json!(format!("{term_ruler:?}"));
                p["decan_ruler"] = json!(format!("{decan:?}"));
                p["triplicity_day"] = json!(format!("{trip_d:?}"));
                p["triplicity_night"] = json!(format!("{trip_n:?}"));
                p["triplicity_part"] = json!(format!("{trip_p:?}"));
                p["almuten"] = json!(format!("{alm:?}"));
                p["almuten_score"] = json!(alm_score);
                p["same_sect"] = json!(sect_ok);
            }
        }
    }
    ctx["is_day"] = json!(is_day);
    Ok(ctx)
}

const HELL_HEADERS: [&str; 8] = [
    "Glyph",
    "Planet",
    "Dignity",
    "Score",
    "Term lord",
    "Decan lord",
    "Triplicity D/N",
    "Sect",
];
const HELL_COL_X: [f64; 8] = [24.0, 48.0, 110.0, 210.0, 258.0, 358.0, 458.0, 600.0];

fn write_hell_table_header(extra: &mut String, ring: &str, is_day: bool, ly: f64) {
    let chart = if is_day { "Day" } else { "Night" };
    extra.push_str(&format!(
        "  <text x=\"24\" y=\"{ly:.0}\" font-size=\"12\" font-weight=\"600\" \
         font-family=\"'Segoe UI',system-ui,sans-serif\" fill=\"{ring}\">\
         Hellenistic Dignities — {chart} chart</text>\n",
    ));
    extra.push_str(&format!(
        "  <line x1=\"24\" y1=\"{:.0}\" x2=\"876\" y2=\"{:.0}\" \
         stroke=\"{ring}\" stroke-width=\".5\" opacity=\".35\"/>\n",
        ly + 3.0,
        ly + 3.0
    ));
    for (h, &x) in HELL_HEADERS.iter().zip(HELL_COL_X.iter()) {
        extra.push_str(&format!(
            "  <text x=\"{x:.0}\" y=\"{:.0}\" font-size=\"8\" font-weight=\"600\" \
             fill=\"{ring}\" opacity=\".6\">{h}</text>\n",
            ly + 14.0
        ));
    }
}

fn hell_score_color(score: i64, txt: &str) -> &str {
    if score >= 4 {
        "#1a6030"
    } else if score < 0 {
        "#901020"
    } else {
        txt
    }
}

fn write_hell_planet_row(extra: &mut String, p: &Value, ry: f64, txt: &str) {
    let score = p["dignity_score"].as_i64().unwrap_or(0);
    let ret = p["retro"].as_bool().unwrap_or(false);
    let sect = p["same_sect"].as_bool().unwrap_or(true);
    let trip_d = p["triplicity_day"].as_str().unwrap_or("—");
    let trip_n = p["triplicity_night"].as_str().unwrap_or("—");

    let score_col = hell_score_color(score, txt);
    let pfg = if ret { "#b01020" } else { txt };
    let sect_lbl = if sect { "in sect" } else { "out of sect" };
    let trip_pair = format!("{trip_d}/{trip_n}");
    let score_str = score.to_string();

    let values: [&str; 8] = [
        p["glyph"].as_str().unwrap_or("?"),
        p["name"].as_str().unwrap_or("?"),
        p["dignity5"].as_str().unwrap_or("—"),
        &score_str,
        p["term_ruler"].as_str().unwrap_or("—"),
        p["decan_ruler"].as_str().unwrap_or("—"),
        &trip_pair,
        sect_lbl,
    ];
    for (&x, val) in HELL_COL_X.iter().zip(values.iter()) {
        let col = if x == 210.0 { score_col } else { pfg };
        extra.push_str(&format!(
            "  <text x=\"{x:.0}\" y=\"{ry:.0}\" font-size=\"9\" \
             font-family=\"'Segoe UI',system-ui,sans-serif\" fill=\"{col}\">{val}</text>\n"
        ));
    }
}

pub fn render_hellenistic_svg(ctx: &Value) -> String {
    // Delegate to the full natal SVG — the dignity5/term/decan fields
    // are in the context and visible via --print-context.
    // We add an extra dignities legend section below the normal wheel.
    let mut s = render_builtin_svg(ctx);

    let ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#1a1a2e");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#0d0d1e");
    let is_day = ctx["is_day"].as_bool().unwrap_or(true);

    let planets = super::json_array(&ctx["planets"]);

    let ly = CY + RO + 260.0;
    let mut extra = String::new();
    write_hell_table_header(&mut extra, ring, is_day, ly);
    for (i, p) in planets.iter().enumerate() {
        write_hell_planet_row(&mut extra, p, ly + 26.0 + i as f64 * 15.0, txt);
    }

    if let Some(idx) = s.rfind("</svg>") {
        s.insert_str(idx, &extra);
    }
    s
}

pub fn build_firdaria_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Firdaria Timeline".to_string());

    let h = houses_ex(jd, flags, lat, lon, HouseSystem(hsys as u8)).map_err(|e| e.to_string())?;
    let cusps_arr: [f64; 13] = {
        let mut a = [0.0f64; 13];
        a.copy_from_slice(&h.cusps);
        a
    };

    let sun_pos = calc_ut(jd, Body::SUN, flags).map_err(|e| e.to_string())?;
    let is_day = is_day_chart(sun_pos.lon, &cusps_arr);

    let periods = firdaria(jd, is_day, 75.0);
    let period_vals: Vec<Value> = periods
        .iter()
        .map(|p| {
            json!({
                "major_lord":  format!("{:?}", p.major_lord),
                "minor_lord":  format!("{:?}", p.minor_lord),
                "start":       jd_to_date_str(p.start),
                "end":         jd_to_date_str(p.end),
                "start_jd":    p.start,
                "end_jd":      p.end,
                "years":       (p.years * 100.0).round() / 100.0})
        })
        .collect();

    let palette = super::palette_with_defaults(
        &[
        ("bg_color", "#ffffff"),
        ("ring_color", "#1a1a2e"),
        ("text_color", "#0d0d1e"),
        ("planet_color", "#0d0d1e"),
        ],
        &vars,
    );

    Ok(json!({
        "date": date_str, "jd": jd, "lat": lat, "lon": lon,
        "is_day": is_day,
        "firdaria": period_vals,
        "vars": Value::Object(palette.into_iter().collect())}))
}

pub fn render_firdaria_svg(ctx: &Value) -> String {
    use std::fmt::Write;
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fff");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#0d0d1e");
    let ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#1a1a2e");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Firdaria");
    let date = ctx["date"].as_str().unwrap_or("");
    let is_day = ctx["is_day"].as_bool().unwrap_or(true);
    let jd_birth = ctx["jd"].as_f64().unwrap_or(0.0);

    // Planet colours (same as dasha palette)
    const FIRD_COLORS: &[(&str, &str)] = &[
        ("SUN", "#e67e22"),
        ("MOON", "#7f8c8d"),
        ("MERCURY", "#27ae60"),
        ("VENUS", "#3498db"),
        ("MARS", "#e74c3c"),
        ("JUPITER", "#f39c12"),
        ("SATURN", "#2c3e50"),
        ("MEAN_NODE", "#8e44ad"),
        ("TRUE_NODE", "#d35400"),
    ];

    let periods = super::json_array(&ctx["firdaria"]);
    if periods.is_empty() {
        return String::new();
    }

    let jd_start = periods[0]["start_jd"].as_f64().unwrap_or(jd_birth);
    let jd_end = periods
        .last()
        .and_then(|p| p["end_jd"].as_f64())
        .unwrap_or(jd_birth + 75.0 * 365.25);
    let span = (jd_end - jd_start).max(1.0);

    const LM: f64 = 90.0;
    const TM: f64 = 70.0;
    const W: f64 = 720.0;
    const BH: f64 = 18.0;
    const BG: f64 = 2.0;

    let n = periods.len().min(63); // show up to 9 major × 7 minor
    let total_h = TM + n as f64 * (BH + BG) + 50.0;

    let mut s = String::with_capacity(12 * 1024);
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 {total_h:.0}" width="900" height="{total_h:.0}">
  <rect width="900" height="{total_h:.0}" fill="{bg}"/>
  <text x="450" y="28" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="450" y="46" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">{date}  ·  {} chart</text>"##,
        if is_day { "Day" } else { "Night" }
    );

    // Year axis
    let birth_year = {
        let d = celestial_core::revjul(jd_birth, celestial_core::body::Calendar::Gregorian);
        d.year as i32
    };
    let end_year = birth_year + (span / 365.25) as i32 + 1;
    for yr in (birth_year..=end_year).step_by(5) {
        let jd_yr =
            celestial_core::julday(yr, 1, 1, 0.0, celestial_core::body::Calendar::Gregorian);
        let x = LM + (jd_yr - jd_start) / span * W;
        if !(LM - 5.0..=LM + W + 5.0).contains(&x) {
            continue;
        }
        let _ = writeln!(
            s,
            r##"  <line x1="{x:.1}" y1="{TM:.1}" x2="{x:.1}" y2="{:.1}" stroke="{ring}" stroke-width="0.5" opacity=".2"/>
  <text x="{x:.1}" y="{:.1}" text-anchor="middle" font-size="7" fill="{ring}" opacity=".5">{yr}</text>"##,
            TM + n as f64 * (BH + BG),
            TM - 6.0
        );
    }

    let mut prev_major = "";
    for (i, p) in periods.iter().take(n).enumerate() {
        prev_major = render_firdaria_row(
            &mut s,
            p,
            i,
            jd_start,
            jd_end,
            span,
            ring,
            FIRD_COLORS,
            prev_major,
            (LM, TM, W, BH, BG),
        );
    }

    let _ = writeln!(s, "</svg>");
    s
}

#[allow(clippy::too_many_arguments)]
fn render_firdaria_row<'a>(
    s: &mut String,
    p: &'a Value,
    i: usize,
    jd_start: f64,
    jd_end: f64,
    span: f64,
    ring: &str,
    colors: &[(&str, &str)],
    prev_major: &'a str,
    layout: (f64, f64, f64, f64, f64),
) -> &'a str {
    use std::fmt::Write;
    let (lm, tm, w, bh, bg_pad) = layout;

    let major = p["major_lord"].as_str().unwrap_or("?");
    let minor = p["minor_lord"].as_str().unwrap_or("?");
    let jd_s = p["start_jd"].as_f64().unwrap_or(jd_start);
    let jd_e = p["end_jd"].as_f64().unwrap_or(jd_end);
    let start = p["start"].as_str().unwrap_or("");

    let bx = lm + (jd_s - jd_start) / span * w;
    let bw = ((jd_e - jd_s) / span * w).max(1.0);
    let by = tm + i as f64 * (bh + bg_pad);

    let col = colors
        .iter()
        .find(|(n, _)| *n == major)
        .map_or("#888", |(_, c)| *c);

    let op = if major == minor { "0.85" } else { "0.55" };
    let _ = writeln!(
        s,
        r##"  <rect x="{bx:.1}" y="{by:.1}" width="{bw:.1}" height="{bh}" rx="3" fill="{col}" opacity="{op}"/>"##
    );

    if bw > 30.0 {
        let lbl = if major == minor {
            major.to_string()
        } else {
            format!("{major}/{minor}")
        };
        let _ = writeln!(
            s,
            r##"  <text x="{:.1}" y="{:.1}" font-size="8" dominant-baseline="central" fill="#fff">{lbl}</text>"##,
            bx + 4.0,
            by + bh * 0.5
        );
    }

    if major != prev_major {
        let _ = writeln!(
            s,
            r##"  <text x="{:.1}" y="{:.1}" text-anchor="end" font-size="9" font-weight="600" dominant-baseline="central" fill="{col}">{major}</text>
  <text x="{:.1}" y="{:.1}" text-anchor="end" font-size="7" dominant-baseline="central" fill="{ring}" opacity=".5">{start}</text>"##,
            lm - 4.0,
            by + bh * 0.5,
            lm - 4.0,
            by + bh * 0.5 + 9.0
        );
        major
    } else {
        prev_major
    }
}

pub fn build_profection_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    hsys: char,
    age: u32,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert(format!("Annual Profection — Age {age}"));

    let h = houses_ex(jd, flags, lat, lon, HouseSystem(hsys as u8)).map_err(|e| e.to_string())?;
    let cusps_arr: [f64; 13] = {
        let mut a = [0.0f64; 13];
        a.copy_from_slice(&h.cusps);
        a
    };

    let (house_num, prof_lon) = annual_profection(&cusps_arr, age);
    let (month_house, month_lon) = monthly_profection(&cusps_arr, age, 0);
    let prof_lord = sign_ruler((prof_lon / 30.0) as u8 % 12);

    let mut ctx = build_context(jd, lat, lon, date_str, hsys, vars)?;
    ctx["profection_house"] = json!(house_num);
    ctx["profection_lon"] = json!((prof_lon * 1e4).round() / 1e4);
    ctx["profection_lord"] = json!(format!("{prof_lord:?}"));
    ctx["month_house"] = json!(month_house);
    ctx["month_lon"] = json!((month_lon * 1e4).round() / 1e4);
    ctx["profection_age"] = json!(age);
    Ok(ctx)
}

pub fn render_profection_svg(ctx: &Value) -> String {
    // Start from the natal wheel, add a profection marker
    let mut s = render_builtin_svg(ctx);

    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#0d0d1e");
    let prof_house = ctx["profection_house"].as_u64().unwrap_or(1);
    let prof_lon = ctx["profection_lon"].as_f64().unwrap_or(0.0);
    let prof_lord = ctx["profection_lord"].as_str().unwrap_or("?");
    let age = ctx["profection_age"].as_u64().unwrap_or(0);
    let asc = ctx["asc"].as_f64().unwrap_or(0.0);

    // Highlight the profected house with an arc on the outer ring
    let arc_col = "#d4a800";
    let arc_x = wx(CX, RO + 10.0, prof_lon, asc);
    let arc_y = wy(CY, RO + 10.0, prof_lon, asc);

    let mut extra = format!(
        "  <!-- Profection marker for age {age}, house {prof_house} ;-->\n\
         <circle cx=\"{arc_x:.2}\" cy=\"{arc_y:.2}\" r=\"8\" fill=\"{arc_col}\" opacity=\".8\"/>\n\
         <text x=\"{arc_x:.2}\" y=\"{arc_y:.2}\" font-size=\"9\" font-weight=\"700\" \
         text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"{txt}\">{prof_house}</text>\n"
    );

    // Legend note
    let ly = CY + RO + 20.0;
    extra.push_str(&format!(
        "  <text x=\"450\" y=\"{ly:.0}\" text-anchor=\"middle\" font-size=\"10\" \
         font-family=\"'Segoe UI',system-ui,sans-serif\" fill=\"{arc_col}\" font-weight=\"600\">\
         Age {age}: House {prof_house} ;profection — Lord: {prof_lord}</text>\n"
    ));

    if let Some(idx) = s.rfind("</svg>") {
        s.insert_str(idx, &extra);
    }
    s
}
