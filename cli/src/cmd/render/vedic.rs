//! Vedic chart builders — split from render.rs.

use super::{
    fmt_lon_dms, jd_to_date_str,
    render_south_indian_svg, sarvashtakavarga, BODIES, NI_CELLS, RASI_GLYPHS,
    RASI_NAMES,
};

use celestial_core::body::{Body, CalcFlags, HouseSystem};
use celestial_core::{calc_ut, houses_ex};
use celestial_core::{
    long_to_nakshatra, long_to_navamsa, long_to_rasi, naisargika_relation, nakshatra_name,
    ochchabala, vimshottari_dasha,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt::Write;

pub fn render_north_indian_svg(ctx: &Value) -> String {
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#ffffff");
    let border = ctx["vars"]["border_color"].as_str().unwrap_or("#5c3a00");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#2a1a00");
    let pcol = ctx["vars"]["planet_color"].as_str().unwrap_or("#1a3a7a");
    let retro = ctx["vars"]["retro_color"].as_str().unwrap_or("#a01030");
    let _asc_c = ctx["vars"]["asc_color"].as_str().unwrap_or("#006030");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("North Indian Chart");
    let date = ctx["date"].as_str().unwrap_or("");

    // North Indian: house 1 = ASC sign; house numbers rotate from ASC
    // We need the ASC rasi — use the first planet as proxy; real impl
    // requires sidereal house calc. For now use Moon's rasi as lagna hint,
    // or store asc_rasi in context if present.
    let lagna_rasi = ctx.get("asc_rasi").and_then(serde_json::Value::as_i64).unwrap_or(0) as usize % 12;

    let planets = super::json_array(&ctx["planets"]);
    // Group planets by rasi
    let mut rasi_planets: Vec<Vec<String>> = vec![Vec::new(); 12];
    for p in planets {
        let rasi = p["rasi"].as_i64().unwrap_or(0) as usize % 12;
        let g = p["glyph"].as_str().unwrap_or("?");
        let ret = p["retro"].as_bool().unwrap_or(false);
        rasi_planets[rasi].push(format!("{g}{}", if ret { "℞" } else { "" }));
    }

    let mut s = String::with_capacity(16 * 1024);
    let total_w = 600.0_f64;
    let total_h = 640.0_f64;
    let ox = 30.0_f64; // origin offset
    let oy = 65.0_f64;

    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w} {total_h}" width="{total_w}" height="{total_h}">
  <rect width="{total_w}" height="{total_h}" fill="{bg}"/>
  <text x="300" y="26" text-anchor="middle" font-size="15" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="300" y="44" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"##
    );

    // Outer diamond outline
    let (dx, dy) = (ox + 270.0, oy + 270.0); // diamond centre
    let r = 240.0_f64;
    let pts = [
        (dx, dy - r), // top
        (dx + r, dy), // right
        (dx, dy + r), // bottom
        (dx - r, dy), // left
    ];
    let _ = writeln!(
        s,
        r##"  <polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" fill="none" stroke="{border}" stroke-width="2.0"/>"##,
        pts[0].0, pts[0].1, pts[1].0, pts[1].1, pts[2].0, pts[2].1, pts[3].0, pts[3].1
    );

    // Inner diamond (mid-lines)
    let r2 = r / 2.0;
    let ipts = [(dx, dy - r2), (dx + r2, dy), (dx, dy + r2), (dx - r2, dy)];
    let _ = writeln!(
        s,
        r##"  <polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" fill="none" stroke="{border}" stroke-width="1.0" opacity=".5"/>"##,
        ipts[0].0, ipts[0].1, ipts[1].0, ipts[1].1, ipts[2].0, ipts[2].1, ipts[3].0, ipts[3].1
    );

    // Diagonal dividers
    for (a, b) in [(pts[0], pts[2]), (pts[1], pts[3])] {
        let _ = writeln!(
            s,
            r##"  <line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{border}" stroke-width="1.2" opacity=".6"/>"##,
            a.0, a.1, b.0, b.1
        );
    }

    // Each of the 12 cells
    for (house0, &(cx, cy)) in NI_CELLS.iter().enumerate() {
        // house0 is absolute sign (0=Aries), map to house number relative to ASC
        let house_num = (house0 + 12 - lagna_rasi) % 12 + 1;
        let sign_idx = house0;
        let cx = ox + cx;
        let cy = oy + cy;
        let is_lagna = house0 == lagna_rasi;

        // House number label
        let col = if is_lagna { _asc_c } else { border };
        let _ = writeln!(
            s,
            r##"  <text x="{cx:.1}" y="{:.1}" font-size="9" font-weight="600" text-anchor="middle" fill="{col}" opacity=".8">{house_num}</text>"##,
            cy - 8.0
        );

        // Rasi glyph
        let sg = RASI_GLYPHS[sign_idx % 12];
        let _ = writeln!(
            s,
            r##"  <text x="{cx:.1}" y="{:.1}" font-size="10" text-anchor="middle" font-family="serif" fill="{txt}" opacity=".45">{sg}</text>"##,
            cy + 4.0
        );

        // Planets
        let planets_here = &rasi_planets[sign_idx % 12];
        for (pi, plbl) in planets_here.iter().enumerate() {
            let py = cy + 16.0 + pi as f64 * 12.0;
            let is_r = plbl.contains('℞');
            let c = if is_r { retro } else { pcol };
            let _ = writeln!(
                s,
                r##"  <text x="{cx:.1}" y="{py:.1}" font-size="9" text-anchor="middle" font-family="serif" fill="{c}">{plbl}</text>"##
            );
        }
    }

    let _ = writeln!(s, "</svg>");
    s
}

pub fn build_ashtakavarga_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN | CalcFlags(64); // sidereal
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Ashtakavarga".to_string());

    let h = houses_ex(jd, CalcFlags::BUILTIN, lat, lon, HouseSystem(b'P'))
        .map_err(|e| e.to_string())?;
    let asc_lon = h.ascmc[0];
    let asc_rasi = (asc_lon % 360.0 / 30.0) as usize % 12;

    let planet_bodies = [
        Body::SUN,
        Body::MOON,
        Body::MARS,
        Body::MERCURY,
        Body::JUPITER,
        Body::VENUS,
        Body::SATURN,
    ];
    let planet_names = [
        "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
    ];

    let mut planet_rasis = [0usize; 7];
    for (i, &b) in planet_bodies.iter().enumerate() {
        if let Ok(pos) = calc_ut(jd, b, flags) {
            planet_rasis[i] = (pos.lon / 30.0) as usize % 12;
        }
    }

    let (totals, rows) = sarvashtakavarga(&planet_rasis, asc_rasi);

    let row_vals: Vec<Value> = rows.iter().zip(planet_names.iter()).map(|(row, &name)| {
        json!({ "planet": name, "bindus": row.iter().map(|&b| json!(b)).collect::<Vec<_>>() })
    }).collect();

    let palette = super::palette_with_defaults(
        &[
        ("bg_color", "#ffffff"),
        ("border_color", "#5c3a00"),
        ("text_color", "#2a1a00"),
        ("planet_color", "#1a3a7a"),
        ("title", "Ashtakavarga"),
        ],
        &vars,
    );

    Ok(json!({
        "date": jd_to_date_str(jd), "date_label": date_str, "jd": jd, "lat": lat, "lon": lon,
        "ashtakavarga_rows": row_vals,
        "sarvashtakavarga": totals.iter().map(|&b| json!(b)).collect::<Vec<_>>(),
        "vars": Value::Object(palette.into_iter().collect())}))
}

const AV_LM: f64 = 80.0;
const AV_TM: f64 = 65.0;
const AV_CW: f64 = 52.0;
const AV_RH: f64 = 26.0;

fn av_bindu_color(bv: u64, txt: &str) -> &str {
    if bv >= 5 {
        "#1a6030"
    } else if bv <= 2 {
        "#901020"
    } else {
        txt
    }
}

fn av_total_color(bv: u64, txt: &str) -> &str {
    if bv >= 28 {
        "#1a6030"
    } else if bv <= 18 {
        "#901020"
    } else {
        txt
    }
}

fn write_av_header(s: &mut String, bg: &str, txt: &str, title: &str, date: &str, total_w: f64, total_h: f64) {
    let cx = total_w / 2.0;
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w:.0} {total_h:.0}" width="{total_w:.0}" height="{total_h:.0}">
  <rect width="{total_w:.0}" height="{total_h:.0}" fill="{bg}"/>
  <text x="{cx:.1}" y="22" text-anchor="middle" font-size="15" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="{cx:.1}" y="40" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"##
    );
}

fn write_av_column_headers(s: &mut String, txt: &str) {
    for (si, glyph) in RASI_GLYPHS.iter().enumerate() {
        let x = AV_LM + si as f64 * AV_CW + AV_CW / 2.0;
        let _ = writeln!(
            s,
            r##"  <text x="{x:.1}" y="{:.1}" font-size="13" text-anchor="middle" font-family="serif" fill="{txt}">{}</text>
  <text x="{x:.1}" y="{:.1}" font-size="8" text-anchor="middle" fill="{txt}" opacity=".5">{}</text>"##,
            AV_TM - 16.0,
            glyph,
            AV_TM - 5.0,
            si + 1
        );
    }
}

fn write_av_grid_lines(s: &mut String, border: &str, n_rows: usize) {
    for si in 0..=12usize {
        let x = AV_LM + si as f64 * AV_CW;
        let _ = writeln!(
            s,
            r##"  <line x1="{x:.1}" y1="{AV_TM:.1}" x2="{x:.1}" y2="{:.1}" stroke="{border}" stroke-width="0.6" opacity=".4"/>"##,
            AV_TM + n_rows as f64 * AV_RH
        );
    }
}

fn write_av_planet_row(s: &mut String, row: &Value, ri: usize, border: &str, pcol: &str, txt: &str) {
    let ry = AV_TM + ri as f64 * AV_RH;
    let name = row["planet"].as_str().unwrap_or("?");
    let bindus = super::json_array(&row["bindus"]);
    if ri.is_multiple_of(2) {
        let _ = writeln!(
            s,
            r##"  <rect x="{AV_LM:.1}" y="{ry:.1}" width="{:.1}" height="{AV_RH}" fill="{border}" opacity=".04"/>"##,
            12.0 * AV_CW
        );
    }
    let _ = writeln!(
        s,
        r##"  <line x1="{AV_LM:.1}" y1="{ry:.1}" x2="{:.1}" y2="{ry:.1}" stroke="{border}" stroke-width="0.4" opacity=".3"/>"##,
        AV_LM + 12.0 * AV_CW
    );
    let _ = writeln!(
        s,
        r##"  <text x="{:.1}" y="{:.1}" font-size="10" text-anchor="end" dominant-baseline="central" fill="{pcol}" font-weight="500">{name}</text>"##,
        AV_LM - 4.0,
        ry + AV_RH / 2.0
    );
    for (si, b) in bindus.iter().enumerate() {
        let bv = b.as_u64().unwrap_or(0);
        let x = AV_LM + si as f64 * AV_CW + AV_CW / 2.0;
        let col = av_bindu_color(bv, txt);
        let _ = writeln!(
            s,
            r##"  <text x="{x:.1}" y="{:.1}" font-size="11" text-anchor="middle" dominant-baseline="central" font-weight="500" fill="{col}">{bv}</text>"##,
            ry + AV_RH / 2.0
        );
    }
}

fn write_av_totals_row(s: &mut String, totals: &[Value], ty: f64, border: &str, txt: &str) {
    let _ = writeln!(
        s,
        r##"  <rect x="{AV_LM:.1}" y="{ty:.1}" width="{:.1}" height="{AV_RH}" fill="{border}" opacity=".1"/>
  <line x1="{AV_LM:.1}" y1="{ty:.1}" x2="{:.1}" y2="{ty:.1}" stroke="{border}" stroke-width="1.5" opacity=".6"/>
  <text x="{:.1}" y="{:.1}" font-size="10" text-anchor="end" dominant-baseline="central" fill="{txt}" font-weight="700">Total</text>"##,
        12.0 * AV_CW,
        AV_LM + 12.0 * AV_CW,
        AV_LM - 4.0,
        ty + AV_RH / 2.0
    );
    for (si, b) in totals.iter().enumerate() {
        let bv = b.as_u64().unwrap_or(0);
        let x = AV_LM + si as f64 * AV_CW + AV_CW / 2.0;
        let col = av_total_color(bv, txt);
        let _ = writeln!(
            s,
            r##"  <text x="{x:.1}" y="{:.1}" font-size="12" text-anchor="middle" dominant-baseline="central" font-weight="700" fill="{col}">{bv}</text>"##,
            ty + AV_RH / 2.0
        );
    }
}

pub fn render_ashtakavarga_svg(ctx: &Value) -> String {
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#ffffff");
    let border = ctx["vars"]["border_color"].as_str().unwrap_or("#5c3a00");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#2a1a00");
    let pcol = ctx["vars"]["planet_color"].as_str().unwrap_or("#1a3a7a");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Ashtakavarga");
    let date = ctx["date"].as_str().unwrap_or("");

    let rows = super::json_array(&ctx["ashtakavarga_rows"]);
    let totals = super::json_array(&ctx["sarvashtakavarga"]);

    let n_rows = rows.len() + 1;
    let total_h = AV_TM + n_rows as f64 * AV_RH + 60.0;
    let total_w = AV_LM + 12.0 * AV_CW + 20.0;

    let mut s = String::with_capacity(8 * 1024);
    write_av_header(&mut s, bg, txt, title, date, total_w, total_h);
    write_av_column_headers(&mut s, txt);
    write_av_grid_lines(&mut s, border, n_rows);
    for (ri, row) in rows.iter().enumerate() {
        write_av_planet_row(&mut s, row, ri, border, pcol, txt);
    }
    let ty = AV_TM + rows.len() as f64 * AV_RH;
    write_av_totals_row(&mut s, totals, ty, border, txt);
    let _ = writeln!(s, "</svg>");
    s
}

pub fn build_shadbala_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED | CalcFlags(64);
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Shadbala — Planetary Strength".to_string());

    let trad_bodies = [
        (Body::SUN, 0i32, "Sun"),
        (Body::MOON, 1, "Moon"),
        (Body::MARS, 2, "Mars"),
        (Body::MERCURY, 3, "Mercury"),
        (Body::JUPITER, 4, "Jupiter"),
        (Body::VENUS, 5, "Venus"),
        (Body::SATURN, 6, "Saturn"),
    ];

    // Mean daily motion (degrees/day) for each planet — used for Chesta bala
    const MEAN_SPEED: [f64; 7] = [0.9856, 13.1764, 0.5240, 1.3831, 0.0831, 0.6152, 0.0334];

    let mut rows: Vec<Value> = Vec::with_capacity(trad_bodies.len());
    for (i, &(body, raw, name)) in trad_bodies.iter().enumerate() {
        if let Ok(pos) = calc_ut(jd, body, flags) {
            // 1. Ochchabala: exaltation strength (0–60 shashtiamsas)
            let ochcha = ochchabala(raw, pos.lon).unwrap_or(0.0);

            // 2. Saptavargaja bala: simplified — based on rasi/navamsa placement
            //    Full Saptavargaja needs D1,D2,D3,D7,D9,D12,D30.
            //    We compute D1 (rasi) + D9 (navamsa) contribution only.
            let rasi = long_to_rasi(pos.lon);
            let navamsa = long_to_navamsa(pos.lon);
            // Relationship: 3=moolatrikona, 2=swakshetra, 1=mitravarga, 0=neutral, -1=shatru
            let d1_rel = naisargika_relation(raw, rasi).unwrap_or(0);
            let d9_rel = naisargika_relation(raw, navamsa).unwrap_or(0);
            let sapta: f64 = (d1_rel + d9_rel + 4) as f64 * 7.5; // 0..60 scale

            // 3. Chesta bala: motional strength
            //    1 if direct + faster than mean, 0.5 if direct slower, 0 if retrograde
            let speed_ratio = pos.speed_lon.abs() / MEAN_SPEED[i].max(0.001);
            let chesta: f64 = if pos.speed_lon < 0.0 {
                0.0
            } else if speed_ratio >= 1.0 {
                60.0
            } else {
                speed_ratio * 60.0
            };

            // 4. Dig bala: directional strength (simplified — based on house position)
            //    Sun/Mars strongest in 10th, Moon/Venus in 4th, Mercury/Jupiter in 1st, Saturn in 7th
            let dig: f64 = 30.0; // placeholder — full calc needs house number

            // Total (out of 240 = 4×60)
            let total = ochcha + sapta + chesta + dig;

            rows.push(json!({
                "name":     name,
                "lon":      (pos.lon * 100.0).round() / 100.0,
                "retro":    pos.speed_lon < 0.0,
                "ochchabala": (ochcha * 10.0).round() / 10.0,
                "sapta_bala": (sapta  * 10.0).round() / 10.0,
                "chesta_bala":(chesta * 10.0).round() / 10.0,
                "dig_bala":   (dig    * 10.0).round() / 10.0,
                "total":      (total  * 10.0).round() / 10.0,
                "strong":     total >= 100.0}));
        }
    }

    let palette = super::palette_with_defaults(
        &[
        ("bg_color", "#ffffff"),
        ("border_color", "#5c3a00"),
        ("text_color", "#2a1a00"),
        ("planet_color", "#1a3a7a"),
        ],
        &vars,
    );

    Ok(json!({
        "date": jd_to_date_str(jd), "date_label": date_str, "jd": jd, "lat": lat, "lon": lon,
        "shadbala": rows,
        "vars": Value::Object(palette.into_iter().collect())}))
}

pub fn render_shadbala_svg(ctx: &Value) -> String {
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#ffffff");
    let border = ctx["vars"]["border_color"].as_str().unwrap_or("#5c3a00");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#2a1a00");
    let pcol = ctx["vars"]["planet_color"].as_str().unwrap_or("#1a3a7a");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Shadbala");
    let date = ctx["date"].as_str().unwrap_or("");

    const COLS: &[(&str, f64)] = &[
        ("Planet", 70.0),
        ("Lon", 64.0),
        ("Ochcha", 56.0),
        ("Sapta", 56.0),
        ("Chesta", 56.0),
        ("Dig", 56.0),
        ("Total", 60.0),
    ];
    const RH: f64 = 28.0;
    const TM: f64 = 60.0;

    let total_w: f64 = COLS.iter().map(|(_, w)| w).sum::<f64>() + 40.0;
    let rows = super::json_array(&ctx["shadbala"]);
    let total_h = TM + (rows.len() + 1) as f64 * RH + 20.0;

    let mut s = String::with_capacity(6 * 1024);
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w:.0} {total_h:.0}" width="{total_w:.0}" height="{total_h:.0}">
  <rect width="{total_w:.0}" height="{total_h:.0}" fill="{bg}"/>
  <text x="{:.1}" y="22" text-anchor="middle" font-size="14" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="{:.1}" y="38" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"##,
        total_w / 2.0,
        total_w / 2.0
    );

    // Column headers
    let mut cx = 20.0_f64;
    for &(hdr, cw) in COLS {
        let _ = writeln!(
            s,
            r##"  <text x="{:.1}" y="{:.1}" font-size="10" font-weight="600" text-anchor="middle" fill="{txt}" opacity=".75">{hdr}</text>"##,
            cx + cw / 2.0,
            TM - 8.0
        );
        let _ = writeln!(
            s,
            r##"  <line x1="{cx:.1}" y1="{TM:.1}" x2="{:.1}" y2="{:.1}" stroke="{border}" stroke-width="0.5" opacity=".4"/>"##,
            cx,
            TM + rows.len() as f64 * RH
        );
        cx += cw;
    }
    let _ = writeln!(
        s,
        r##"  <line x1="{cx:.1}" y1="{TM:.1}" x2="{cx:.1}" y2="{:.1}" stroke="{border}" stroke-width="0.5" opacity=".4"/>"##,
        TM + rows.len() as f64 * RH
    );

    // Header separator
    let _ = writeln!(
        s,
        r##"  <line x1="20" y1="{TM:.1}" x2="{cx:.1}" y2="{TM:.1}" stroke="{border}" stroke-width="1.2" opacity=".6"/>"##
    );

    // Data rows
    for (ri, row) in rows.iter().enumerate() {
        render_shadbala_row(
            &mut s,
            row,
            ri,
            ShadbalaRowCtx {
                tm: TM,
                rh: RH,
                cx,
                pcol,
                border,
                txt,
                cols: COLS,
            },
        );
    }

    // Bottom border
    let by = TM + rows.len() as f64 * RH;
    let _ = writeln!(
        s,
        r##"  <line x1="20" y1="{by:.1}" x2="{cx:.1}" y2="{by:.1}" stroke="{border}" stroke-width="1.0" opacity=".5"/>"##
    );
    let _ = writeln!(
        s,
        r##"  <text x="{:.1}" y="{:.1}" font-size="7" fill="{txt}" opacity=".4">Ochcha=exaltation · Sapta=sign placement · Chesta=motional · Dig=directional (shashtiamsas, out of 60)</text>"##,
        20.0,
        by + 14.0
    );

    let _ = writeln!(s, "</svg>");
    s
}

struct ShadbalaRowCtx<'a> {
    tm: f64,
    rh: f64,
    cx: f64,
    pcol: &'a str,
    border: &'a str,
    txt: &'a str,
    cols: &'a [(&'a str, f64)],
}

fn render_shadbala_row(s: &mut String, row: &Value, ri: usize, c: ShadbalaRowCtx<'_>) {
    use std::fmt::Write;
    let ry = c.tm + ri as f64 * c.rh;
    let name = row["name"].as_str().unwrap_or("?");
    let lon = row["lon"].as_f64().unwrap_or(0.0);
    let ret = row["retro"].as_bool().unwrap_or(false);
    let och = row["ochchabala"].as_f64().unwrap_or(0.0);
    let sap = row["sapta_bala"].as_f64().unwrap_or(0.0);
    let che = row["chesta_bala"].as_f64().unwrap_or(0.0);
    let dig = row["dig_bala"].as_f64().unwrap_or(0.0);
    let tot = row["total"].as_f64().unwrap_or(0.0);
    let strong = row["strong"].as_bool().unwrap_or(false);
    let rcol = if strong { "#1a5030" } else { c.pcol };
    let rh = c.rh;
    let cx = c.cx;
    let border = c.border;
    let txt = c.txt;

    if ri.is_multiple_of(2) {
        let _ = writeln!(
            s,
            r##"  <rect x="20" y="{ry:.1}" width="{:.1}" height="{rh}" fill="{border}" opacity=".04"/>"##,
            cx - 20.0
        );
    }
    let _ = writeln!(
        s,
        r##"  <line x1="20" y1="{ry:.1}" x2="{cx:.1}" y2="{ry:.1}" stroke="{border}" stroke-width="0.3" opacity=".25"/>"##
    );

    let mut x = 20.0_f64;
    let data: [String; 7] = [
        format!("{}{}", name, if ret { " ℞" } else { "" }),
        format!("{lon:.4}"),
        format!("{och:.1}"),
        format!("{sap:.1}"),
        format!("{che:.1}"),
        format!("{dig:.1}"),
        format!("{tot:.1}"),
    ];
    for (ci, (val, &(_hdr, cw))) in data.iter().zip(c.cols.iter()).enumerate() {
        let fw = if ci == 0 || ci == 6 { "600" } else { "400" };
        let fc = if ci == 6 { rcol } else { txt };
        let _ = writeln!(
            s,
            r##"  <text x="{:.1}" y="{:.1}" font-size="10" font-weight="{fw}" text-anchor="middle" dominant-baseline="central" fill="{fc}">{val}</text>"##,
            x + cw / 2.0,
            ry + rh / 2.0
        );
        x += cw;
    }
}

pub fn build_vedic_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    user_vars: BTreeMap<String, String>,
    chart_type: &str,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED | CalcFlags(64); // FLG_SIDEREAL
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert_with(|| format!("Vedic {chart_type}"));

    // Collect sidereal positions for all bodies
    let mut planets: Vec<Value> = Vec::with_capacity(BODIES.len());
    let mut moon_sid_lon = 0.0_f64;
    for &(body, key, name, glyph) in BODIES {
        if let Ok(pos) = calc_ut(jd, body, flags) {
            let rasi = long_to_rasi(pos.lon);
            let navamsa = long_to_navamsa(pos.lon);
            let (nak, pada) = long_to_nakshatra(pos.lon);
            let nak_name = nakshatra_name(nak).unwrap_or("?");
            let deg_in_rasi = pos.lon % 30.0;
            if key == "moon" {
                moon_sid_lon = pos.lon;
            }
            planets.push(json!({
                "name": name, "key": key, "glyph": glyph,
                "lon": (pos.lon * 1e4).round() / 1e4,
                "rasi": rasi,
                "rasi_name": RASI_NAMES[rasi as usize % 12],
                "rasi_glyph": RASI_GLYPHS[rasi as usize % 12],
                "deg_in_rasi": (deg_in_rasi * 100.0).round() / 100.0,
                "navamsa": navamsa,
                "nakshatra": nak,
                "nakshatra_name": nak_name,
                "pada": pada + 1,
                "retro": pos.speed_lon < 0.0,
                "dms": fmt_lon_dms(pos.lon)}));
        }
    }

    // Vimshottari dasha (next 120 years)
    let dashas: Vec<Value> = vimshottari_dasha(jd, moon_sid_lon, 120.0)
        .iter()
        .map(|d| {
            let start = jd_to_date_str(d.start);
            let end = jd_to_date_str(d.end);
            json!({
                "body": format!("{:?}", d.body),
                "years": (d.years * 100.0).round() / 100.0,
                "start": start,
                "end":   end,
                "start_jd": d.start,
                "end_jd":   d.end})
        })
        .collect();

    // Ochchabala (exaltation strength 0–60)
    let strengths: Vec<Value> = (0i32..7)
        .filter_map(|raw| {
            let body_lon = planets
                .iter()
                .find(|p| {
                    p["key"].as_str().is_some_and(|k| {
                        k == [
                            "sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn",
                        ][raw as usize]
                    })
                })?
                .get("lon")?
                .as_f64()?;
            let strength = ochchabala(raw, body_lon)?;
            Some(json!({ "body": raw, "ochchabala": (strength * 10.0).round() / 10.0 }))
        })
        .collect();

    let palette = super::palette_with_defaults(
        &[
        ("bg_color", "#ffffff"),
        ("border_color", "#5c3a00"),
        ("text_color", "#2a1a00"),
        ("planet_color", "#1a3a7a"),
        ("retro_color", "#a01030"),
        ("asc_color", "#006030"),
        ],
        &vars,
    );

    Ok(json!({
        "date": jd_to_date_str(jd), "date_label": date_str, "jd": jd, "lat": lat, "lon": lon,
        "planets": planets, "dashas": dashas, "strengths": strengths,
        "moon_sid_lon": (moon_sid_lon * 1e4).round() / 1e4,
        "vars": Value::Object(palette.into_iter().collect())}))
}

pub fn render_navamsa_svg(ctx: &Value) -> String {
    // Navamsa uses the same South Indian grid layout but with navamsa positions
    let planets_orig = super::json_array(&ctx["planets"]);
    // Rebuild rasi_planets using navamsa index instead of rasi
    let mut rasi_planets: Vec<Vec<String>> = vec![Vec::new(); 12];
    for p in planets_orig {
        let nav = p["navamsa"].as_i64().unwrap_or(0) as usize % 12;
        let g = p["glyph"].as_str().unwrap_or("?");
        let ret = p["retro"].as_bool().unwrap_or(false);
        rasi_planets[nav].push(format!("{g}{}", if ret { "℞" } else { "" }));
    }
    // Temporarily patch ctx to use navamsa groupings
    let mut ctx2 = ctx.clone();
    if let Some(planets) = ctx2["planets"].as_array_mut() {
        for p in planets.iter_mut() {
            let nav = p["navamsa"].as_i64().unwrap_or(0);
            p["rasi"] = json!(nav);
        }
    }
    if let Some(v) = ctx2["vars"].as_object_mut() {
        v.insert("title".to_string(), json!("Navamsa (D9) Chart"));
    }
    render_south_indian_svg(&ctx2)
}

pub fn render_dasha_svg(ctx: &Value) -> String {
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#ffffff");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#2a1a00");
    let pcol = ctx["vars"]["planet_color"].as_str().unwrap_or("#1a3a7a");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Dasha Timeline");
    let date = ctx["date"].as_str().unwrap_or("");
    let jd_birth = ctx["jd"].as_f64().unwrap_or(0.0);

    // Planet colours for dasha bars
    const DASHA_COLORS: &[(&str, &str)] = &[
        ("SUN", "#e67e22"),
        ("MOON", "#95a5a6"),
        ("MARS", "#e74c3c"),
        ("RAHU", "#8e44ad"),
        ("JUPITER", "#f1c40f"),
        ("SATURN", "#2c3e50"),
        ("MERCURY", "#27ae60"),
        ("KETU", "#d35400"),
        ("VENUS", "#3498db"),
    ];

    let dashas = super::json_array(&ctx["dashas"]);
    if dashas.is_empty() {
        return String::new();
    }

    let jd_start = dashas[0]["start_jd"].as_f64().unwrap_or(jd_birth);
    let jd_end = dashas
        .last()
        .and_then(|d| d["end_jd"].as_f64())
        .unwrap_or(jd_birth + 120.0 * 365.25);
    let span = (jd_end - jd_start).max(1.0);

    const LM: f64 = 80.0; // left margin
    const TM: f64 = 70.0; // top margin
    const BH: f64 = 36.0; // bar height
    const BG: f64 = 4.0; // bar gap
    const W: f64 = 720.0; // bar area width

    let n = dashas.len().min(18);
    let total_h = TM + n as f64 * (BH + BG) + 40.0;

    let mut s = String::with_capacity(8 * 1024);
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 {total_h:.0}" width="900" height="{total_h:.0}">
  <rect width="900" height="{total_h:.0}" fill="{bg}"/>
  <text x="450" y="28" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="450" y="48" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"##
    );

    // Year axis: every 5 years
    let birth_year = {
        let d = celestial_core::revjul(jd_birth, celestial_core::body::Calendar::Gregorian);
        d.year as i32
    };
    let end_year = birth_year + (span / 365.25) as i32 + 1;
    for yr in (birth_year..=end_year).step_by(5) {
        let jd_yr =
            celestial_core::julday(yr, 1, 1, 0.0, celestial_core::body::Calendar::Gregorian);
        let x = LM + (jd_yr - jd_start) / span * W;
        if !(LM..=LM + W).contains(&x) {
            continue;
        }
        let _ = writeln!(
            s,
            r##"  <line x1="{x:.1}" y1="{TM:.1}" x2="{x:.1}" y2="{:.1}" stroke="{txt}" stroke-width="0.5" opacity=".2"/>
  <text x="{x:.1}" y="{:.1}" text-anchor="middle" font-size="8" fill="{txt}" opacity=".5">{yr}</text>"##,
            TM + n as f64 * (BH + BG),
            TM - 6.0
        );
    }

    // Dasha bars
    for (i, d) in dashas.iter().take(n).enumerate() {
        let body = d["body"].as_str().unwrap_or("?");
        let jd_s = d["start_jd"].as_f64().unwrap_or(jd_start);
        let jd_e = d["end_jd"].as_f64().unwrap_or(jd_end);
        let yrs = d["years"].as_f64().unwrap_or(0.0);
        let start = d["start"].as_str().unwrap_or("");

        let bx = LM + (jd_s - jd_start) / span * W;
        let bw = ((jd_e - jd_s) / span * W).max(1.0);
        let by = TM + i as f64 * (BH + BG);

        let col = DASHA_COLORS
            .iter()
            .find(|(n, _)| *n == body)
            .map_or(pcol, |(_, c)| *c);
        let _ = writeln!(
            s,
            r##"  <rect x="{bx:.1}" y="{by:.1}" width="{bw:.1}" height="{BH}" rx="4" fill="{col}" opacity=".75"/>
  <text x="{:.1}" y="{:.1}" font-size="10" font-weight="600" dominant-baseline="central" fill="#fff">{body}</text>
  <text x="{:.1}" y="{:.1}" font-size="8" dominant-baseline="central" fill="#fff" opacity=".85">{yrs:.1}y · {start}</text>"##,
            bx + 6.0,
            by + BH * 0.35,
            bx + 6.0,
            by + BH * 0.7
        );

        // Label on left axis
        let _ = writeln!(
            s,
            r##"  <text x="{:.1}" y="{:.1}" text-anchor="end" font-size="9" dominant-baseline="central" fill="{txt}" opacity=".7">{body}</text>"##,
            LM - 4.0,
            by + BH * 0.5
        );
    }

    let _ = writeln!(s, "</svg>");
    s
}
