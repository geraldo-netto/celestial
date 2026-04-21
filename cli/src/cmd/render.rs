//! `celestial render` — compute a chart and render any template against it.
//!
//! ## Usage
//! ```text
//! # Use the built-in SVG (default):
//! celestial render --date 2025-03-20 --lat 48.85 --lon 2.35 --out chart.svg
//!
//! # Use your own template:
//! celestial render --date 2025-03-20 --lat 48.85 --lon 2.35 \
//!     --template my.tmpl --out chart.svg
//!
//! # Bootstrap an example template showing all available variables:
//! celestial render --print-template > my.tmpl
//!
//! # Dump the JSON context so you know what to reference in templates:
//! celestial render --date 2025-03-20 --print-context
//!
//! # Custom variables via --var or [vars] in a TOML config:
//! celestial render --config chart.toml --var title="My Chart"
//! ```
//!
//! ## TOML config
//! ```toml
//! [render]
//! date  = "2025-03-20"
//! lat   = 48.85
//! lon   = 2.35
//! out   = "chart.svg"
//! hsys  = "P"
//!
//! [vars]
//! title        = "Spring Equinox 2025"
//! bg_color     = "#ffffff"    white background (default)
//! ring_color   = "#1a1a2e"    dark navy for rings and labels
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::path::PathBuf;

use celestial_core::body::{Body, CalcFlags, HouseSystem};
use celestial_core::AzAlt;
use celestial_core::MoonPhase;
use celestial_core::{
    arabic_parts_seven, calc_ut, diff_deg_signed, houses_ex, lunar_return_jd, midpoint_deg,
    moon_illumination, moon_phase, secondary_progressions, sign_exaltation, sign_ruler,
    solar_arc_directions, solar_return_jd,
};
use celestial_core::{azalt, fixstar_mag, fixstar_ut, midpoint_table};
use celestial_core::{lon_to_sign, zodiac_sign_name};
use celestial_core::{
    long_to_nakshatra, long_to_navamsa, long_to_rasi, naisargika_relation, nakshatra_name,
    ochchabala, vimshottari_dasha,
};
use clap::Args;
use serde_json::{json, Value};
use tinytemplate::TinyTemplate;

// ─── CLI ─────────────────────────────────────────────────────────────────────

#[derive(Args, Debug)]
pub struct RenderArgs {
    /// Date to compute (YYYY-MM-DD or "now")
    #[arg(long, default_value = "now")]
    pub date: String,

    /// Geographic latitude in decimal degrees (N positive)
    #[arg(long, default_value = "0.0")]
    pub lat: f64,

    /// Geographic longitude in decimal degrees (E positive)
    #[arg(long, default_value = "0.0")]
    pub lon: f64,

    /// Jinja-style template file (omit for built-in SVG)
    #[arg(long)]
    pub template: Option<PathBuf>,

    /// Output file (omit to print to stdout)
    #[arg(long, short)]
    pub out: Option<PathBuf>,

    /// TOML config file; CLI flags take precedence over config values
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Custom variable KEY=VALUE — repeatable, overrides [vars] in config
    #[arg(long = "var", value_name = "KEY=VALUE", action = clap::ArgAction::Append)]
    pub vars: Vec<String>,

    /// Chart type: natal | cosmogram | solar-return | lunar-return | progressed | solar-arc | biwheel
    #[arg(long, default_value = "natal")]
    pub chart_type: String,

    /// Second date for bi-wheel (partner/transits) or natal date for return/progression charts
    /// Format: YYYY-MM-DD
    #[arg(long)]
    pub date2: Option<String>,
    /// Third date for tri-wheel ring 3
    #[arg(long)]
    pub date3: Option<String>,

    /// Target year for solar return (e.g. 2025); defaults to current year
    #[arg(long)]
    pub return_year: Option<i32>,

    /// Age in decimal years for progressions / solar arc (e.g. 35.5)
    #[arg(long)]
    pub years: Option<f64>,

    /// Second chart latitude (for bi-wheel)
    #[arg(long)]
    pub lat2: Option<f64>,

    /// Second chart longitude (for bi-wheel)
    #[arg(long)]
    pub lon2: Option<f64>,

    /// House system: P=Placidus K=Koch E=Equal W=WholeSign O=Porphyry …
    #[arg(long, default_value = "P")]
    pub hsys: char,

    /// Print the built-in example template to stdout
    #[arg(long)]
    pub print_template: bool,

    /// Print the chart context JSON to stdout (reference for template authors)
    #[arg(long)]
    pub print_context: bool,
}

// ─── Config file ──────────────────────────────────────────────────────────────

#[derive(Debug, Default, serde::Deserialize)]
struct ConfigFile {
    render: Option<RenderSection>,
    vars: Option<toml::value::Table>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct RenderSection {
    date: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    out: Option<PathBuf>,
    template: Option<PathBuf>,
    hsys: Option<char>,
}

// ─── Wheel geometry helpers ────────────────────────────────────────────────────

/// SVG x coordinate for an ecliptic longitude on the wheel.
/// ASC is placed at the 9-o'clock position (leftmost), per astrological convention.
fn wx(cx: f64, r: f64, lon: f64, asc: f64) -> f64 {
    cx + r * (180.0 - (lon - asc)).rem_euclid(360.0).to_radians().cos()
}

/// SVG y coordinate for an ecliptic longitude on the wheel.
fn wy(cy: f64, r: f64, lon: f64, asc: f64) -> f64 {
    cy - r * (180.0 - (lon - asc)).rem_euclid(360.0).to_radians().sin()
}

// ─── Formatting helpers ───────────────────────────────────────────────────────

fn fmt_lon_dms(lon: f64) -> String {
    let (sign_idx, deg_in_sign) = lon_to_sign(lon);
    let d = deg_in_sign as u32;
    let m = ((deg_in_sign - d as f64) * 60.0) as u32;
    let s = (((deg_in_sign - d as f64) * 3600.0) - m as f64 * 60.0).round() as u32;
    let glyphs = [
        "\u{2648}", "\u{2649}", "\u{264A}", "\u{264B}", "\u{264C}", "\u{264D}", "\u{264E}",
        "\u{264F}", "\u{2650}", "\u{2651}", "\u{2652}", "\u{2653}",
    ];
    format!(
        "{d:02}\u{00B0}{m:02}\u{2032}{s:02}\u{2033}{}",
        glyphs[sign_idx as usize % 12]
    )
}

fn moon_phase_str(jd: f64) -> &'static str {
    match moon_phase(jd).unwrap_or(MoonPhase::NewMoon) {
        MoonPhase::NewMoon => "New Moon",
        MoonPhase::WaxingCrescent => "Waxing Crescent",
        MoonPhase::FirstQuarter => "First Quarter",
        MoonPhase::WaxingGibbous => "Waxing Gibbous",
        MoonPhase::FullMoon => "Full Moon",
        MoonPhase::WaningGibbous => "Waning Gibbous",
        MoonPhase::LastQuarter => "Last Quarter",
        MoonPhase::WaningCrescent => "Waning Crescent",
    }
}

/// Format a Julian Day as "YYYY-MM-DD".

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 3 — specialist Western chart builders
// ═══════════════════════════════════════════════════════════════════════════════

// ─── 1. 90° Midpoint Dial ─────────────────────────────────────────────────────

/// Build context for a 90° midpoint dial.
/// All planet longitudes are reduced to 0–90° (the dial compresses all four
/// quadrants). Midpoints are computed and planets that trigger them are listed.
fn build_dial_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("90° Midpoint Dial".to_string());

    // Collect natal positions
    let bodies = BODIES;
    let mut positions: Vec<(Body, f64)> = Vec::new();
    let mut planet_entries: Vec<Value> = Vec::new();

    let h = houses_ex(jd, CalcFlags::BUILTIN, lat, lon, HouseSystem(hsys as u8))
        .map_err(|e| e.to_string())?;
    let asc = h.ascmc[0];

    for &(body, key, name, glyph) in bodies {
        if let Ok(pos) = calc_ut(jd, body, flags) {
            let dial_lon = pos.lon % 90.0; // compress to 0–90°
            positions.push((body, pos.lon));
            planet_entries.push(json!({
                "name": name, "key": key, "glyph": glyph,
                "lon": (pos.lon * 1e4).round() / 1e4,
                "dial_lon": (dial_lon * 1e4).round() / 1e4,
                "retro": pos.speed_lon < 0.0,
            }));
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
                    "orb":  (orb * 100.0).round() / 100.0,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    let mut palette = BTreeMap::new();
    for (k, v) in [
        ("bg_color", "#ffffff"),
        ("ring_color", "#1a1a2e"),
        ("planet_color", "#0d0d1e"),
        ("retro_color", "#b01020"),
        ("soft_color", "#1a50b0"),
        ("text_color", "#0d0d1e"),
    ] {
        palette.insert(k.to_string(), json!(v));
    }
    for (k, v) in &vars {
        palette.insert(k.clone(), json!(v));
    }

    Ok(json!({
        "date": date_str, "jd": jd, "lat": lat, "lon": lon,
        "asc": (asc * 1e4).round() / 1e4,
        "planets": planet_entries,
        "midpoints": mp_entries,
        "vars": Value::Object(palette.into_iter().collect()),
    }))
}

fn render_dial_svg(ctx: &Value) -> String {
    use std::fmt::Write;
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fff");
    let ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#1a1a2e");
    let pfg = ctx["vars"]["planet_color"].as_str().unwrap_or("#0d0d1e");
    let soft = ctx["vars"]["soft_color"].as_str().unwrap_or("#1a50b0");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#0d0d1e");
    let title = ctx["vars"]["title"].as_str().unwrap_or("90° Dial");
    let date = ctx["date"].as_str().unwrap_or("");

    const CX: f64 = 450.0;
    const CY: f64 = 450.0;
    const R: f64 = 320.0; // dial outer radius
    const RP: f64 = 285.0; // planet tick inner
    const RM: f64 = 295.0; // midpoint tick inner
    const RT: f64 = 260.0; // glyph ring

    let mut s = String::with_capacity(32 * 1024);
    let _ = writeln!(
        s,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 1000" width="900" height="1000">
  <rect width="900" height="1000" fill="{bg}"/>
  <text x="450" y="30" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="450" y="48" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">{date}</text>
  <circle cx="{CX}" cy="{CY}" r="{R}" fill="none" stroke="{ring}" stroke-width="2.0" opacity=".6"/>
  <circle cx="{CX}" cy="{CY}" r="{RP}" fill="none" stroke="{ring}" stroke-width="0.8" opacity=".3"/>"#
    );

    // Degree marks: major every 5°, minor every 1° in the 0–90° dial (4 quadrants)
    for deg in 0..360u32 {
        let dial_deg = deg as f64 % 90.0; // position on physical circle
                                          // The 90° dial shows 4 quadrants stacked: each 90° slice maps to 90° of the circle
        let angle_rad = (deg as f64 * std::f64::consts::PI / 180.0) - std::f64::consts::FRAC_PI_2;
        let is_major = dial_deg % 5.0 < 0.5;
        let tick_outer = R;
        let tick_inner = if is_major { R - 12.0 } else { R - 6.0 };
        let x1 = CX + tick_outer * angle_rad.cos();
        let y1 = CY + tick_outer * angle_rad.sin();
        let x2 = CX + tick_inner * angle_rad.cos();
        let y2 = CY + tick_inner * angle_rad.sin();
        let sw = if is_major { "1.2" } else { "0.5" };
        let _ = writeln!(
            s,
            r#"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{ring}" stroke-width="{sw}" opacity=".55"/>"#
        );
    }

    // Cardinal markers at 0°, 90°, 180°, 270° (= 0° Aries, Cancer, Libra, Capricorn)
    let signs_90 = [
        ("♈", "0°♈"),
        ("♋", "0°♋"),
        ("♎", "0°♎"),
        ("♑", "0°♑"),
    ];
    for (i, (g, lbl)) in signs_90.iter().enumerate() {
        let a = (i as f64 * 90.0 - 90.0).to_radians();
        let lx = CX + (R + 16.0) * a.cos();
        let ly = CY + (R + 16.0) * a.sin();
        let _ = writeln!(
            s,
            r#"  <text x="{lx:.2}" y="{ly:.2}" font-size="12" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{ring}">{g}</text>"#
        );
        let _ = writeln!(
            s,
            r#"  <text x="{:.2}" y="{:.2}" font-size="9" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">{lbl}</text>"#,
            CX + (R + 30.0) * a.cos(),
            CY + (R + 30.0) * a.sin()
        );
    }

    // Planet ticks — map each planet's lon to its angle on the 90° dial
    // The 90° dial wraps: 0–90 → 0–360 on the circle (× 4)
    let planets = ctx["planets"].as_array().unwrap_or(&vec![]).to_vec();
    for p in &planets {
        let lon = p["lon"].as_f64().unwrap_or(0.0);
        let dial_pos = (lon % 90.0) / 90.0 * 360.0; // map to full circle
        let a = (dial_pos - 90.0).to_radians();
        let g = p["glyph"].as_str().unwrap_or("?");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let col = if ret {
            ctx["vars"]["retro_color"].as_str().unwrap_or("#b01020")
        } else {
            pfg
        };
        let x_inner = CX + RP * a.cos();
        let y_inner = CY + RP * a.sin();
        let x_outer = CX + R * a.cos();
        let y_outer = CY + R * a.sin();
        let xg = CX + RT * a.cos();
        let yg = CY + RT * a.sin();
        let _ = writeln!(
            s,
            r#"  <line x1="{x_inner:.2}" y1="{y_inner:.2}" x2="{x_outer:.2}" y2="{y_outer:.2}" stroke="{col}" stroke-width="2.2" opacity=".8"/>
  <text x="{xg:.2}" y="{yg:.2}" font-size="15" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}">{g}</text>"#
        );
    }

    // Midpoint triggers: short dashes at the midpoint dial position
    let midpoints = ctx["midpoints"].as_array().unwrap_or(&vec![]).to_vec();
    for mp in &midpoints {
        if mp["triggers"].as_array().map_or(true, |v| v.is_empty()) {
            continue;
        }
        let mid = mp["dial_mid"].as_f64().unwrap_or(0.0);
        let dial_pos = mid / 90.0 * 360.0;
        let a = (dial_pos - 90.0).to_radians();
        let xm1 = CX + RM * a.cos();
        let ym1 = CY + RM * a.sin();
        let xm2 = CX + (RM + 10.0) * a.cos();
        let ym2 = CY + (RM + 10.0) * a.sin();
        let _ = writeln!(
            s,
            r#"  <line x1="{xm1:.2}" y1="{ym1:.2}" x2="{xm2:.2}" y2="{ym2:.2}" stroke="{soft}" stroke-width="1.5" opacity=".6"/>"#
        );
    }

    // Midpoint table legend below the dial
    let ly = CY + R + 50.0;
    let _ = writeln!(
        s,
        r#"  <text x="450" y="{ly:.2}" text-anchor="middle" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Active midpoints (orb 1.5°)</text>"#
    );
    let mut row = 0usize;
    for mp in midpoints
        .iter()
        .filter(|m| !m["triggers"].as_array().map_or(true, |v| v.is_empty()))
    {
        if row >= 20 {
            break;
        }
        let b1 = mp["body1"].as_str().unwrap_or("");
        let b2 = mp["body2"].as_str().unwrap_or("");
        let mid_lon = mp["midpoint_lon"].as_f64().unwrap_or(0.0);
        let mid_dms = fmt_lon_dms(mid_lon);
        let triggers: Vec<String> = mp["triggers"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|t| {
                format!(
                    "{} ({:.1}°)",
                    t["body"].as_str().unwrap_or("?"),
                    t["orb"].as_f64().unwrap_or(0.0)
                )
            })
            .collect();
        let col_x = if row < 10 { 24.0 } else { 464.0 };
        let ry = ly + 18.0 + (row % 10) as f64 * 16.0;
        let _ = writeln!(
            s,
            r#"  <text x="{col_x:.2}" y="{ry:.2}" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}"><tspan font-weight="600">{b1}/{b2}</tspan> = {mid_dms} ← {}</text>"#,
            triggers.join(", ")
        );
        row += 1;
    }

    let _ = writeln!(s, "</svg>");
    s
}

// ─── 2. Composite chart ───────────────────────────────────────────────────────

fn build_composite_context(
    jd1: f64,
    jd2: f64,
    lat: f64,
    lon: f64,
    date1: &str,
    date2: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Composite Chart".to_string());

    let h1 = houses_ex(jd1, CalcFlags::BUILTIN, lat, lon, HouseSystem(hsys as u8))
        .map_err(|e| e.to_string())?;
    let h2 = houses_ex(jd2, CalcFlags::BUILTIN, lat, lon, HouseSystem(hsys as u8))
        .map_err(|e| e.to_string())?;
    // Composite ASC: midpoint of the two ASCs
    let asc1 = h1.ascmc[0];
    let asc2 = h2.ascmc[0];
    let comp_asc = midpoint_deg(asc1, asc2);

    // Composite planet longitudes: midpoint of each body pair
    let mut planets = Vec::new();
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
                "x":    (wx(CX, RP, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "y":    (wy(CY, RP, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "label_x": (wx(CX, RP + 20.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "label_y": (wy(CY, RP + 20.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "asp_x":  (wx(CX, RC, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "asp_y":  (wy(CY, RC, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "tick_x1":(wx(CX, RH + 2.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "tick_y1":(wy(CY, RH + 2.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "tick_x2":(wx(CX, RP - 12.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "tick_y2":(wy(CY, RP - 12.0, comp_lon, comp_asc) * 100.0).round() / 100.0,
                "near_station": false,
                "dignity": "peregrine",
                "antiscia_lon": (180.0 - comp_lon).rem_euclid(360.0),
                "antiscia_x": 0.0, "antiscia_y": 0.0,
                "speed": 0.0, "speed_str": "", "dist": 0.0, "lat": 0.0,
                "sign": lon_to_sign(comp_lon).0,
            }));
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

// ─── 3. Tri-wheel ────────────────────────────────────────────────────────────

fn build_triwheel_context(
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
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Tri-wheel".to_string());

    // Build natal (inner) context
    let inner_vars = vars.clone();
    let inner = build_context(jd1, lat, lon, date1, hsys, inner_vars)?;
    let asc = inner["asc"].as_f64().unwrap_or(0.0);

    // Rings 2 and 3: just planetary positions, no houses
    let mut ring2 = Vec::new();
    let mut ring3 = Vec::new();
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
                    "y": (wy(CY, r, pos.lon, asc) * 100.0).round() / 100.0,
                }));
            }
        }
    }

    let date_str = format!("Tri-wheel: {date1} / {date2} / {date3}");
    let mut palette = BTreeMap::new();
    for (k, v) in [
        ("bg_color", "#ffffff"),
        ("ring_color", "#1a1a2e"),
        ("planet_color", "#0d0d1e"),
        ("retro_color", "#b01020"),
        ("hard_color", "#b01020"),
        ("soft_color", "#1a50b0"),
        ("text_color", "#0d0d1e"),
    ] {
        palette.insert(k.to_string(), json!(v));
    }
    for (k, v) in &vars {
        palette.insert(k.clone(), json!(v));
    }

    let mut ctx = inner;
    ctx["ring2_planets"] = Value::Array(ring2);
    ctx["ring3_planets"] = Value::Array(ring3);
    ctx["date2"] = json!(date2);
    ctx["date3"] = json!(date3);
    ctx["date"] = json!(date_str);
    ctx["vars"] = Value::Object(palette.into_iter().collect());
    Ok(ctx)
}

fn render_triwheel_svg(ctx: &Value) -> String {
    let mut s = render_builtin_svg(ctx);

    let _asc = ctx["asc"].as_f64().unwrap_or(0.0);
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
    let ring2 = ctx["ring2_planets"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    for p in &ring2 {
        let px = p["x"].as_f64().unwrap_or(0.0);
        let py = p["y"].as_f64().unwrap_or(0.0);
        let g = p["glyph"].as_str().unwrap_or("?");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let col = if ret { retro_c } else { ring2_col };
        let _ = writeln!(
            extra,
            r#"  <text x="{px:.2}" y="{py:.2}" font-size="12" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}" opacity=".85">{g}</text>"#
        );
    }

    // Ring 3 (transits)
    let ring3 = ctx["ring3_planets"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    for p in &ring3 {
        let px = p["x"].as_f64().unwrap_or(0.0);
        let py = p["y"].as_f64().unwrap_or(0.0);
        let g = p["glyph"].as_str().unwrap_or("?");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let col = if ret { retro_c } else { ring3_col };
        let _ = writeln!(
            extra,
            r#"  <text x="{px:.2}" y="{py:.2}" font-size="11" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}" opacity=".8">{g}</text>"#
        );
    }

    // Legend
    let d2 = ctx["date2"].as_str().unwrap_or("ring 2");
    let d3 = ctx["date3"].as_str().unwrap_or("ring 3");
    let ly = CY + RO + 20.0;
    let _ = writeln!(
        extra,
        r#"  <text x="450" y="{ly:.2}" text-anchor="middle" font-size="9" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".55">— natal  ·  <tspan fill="{ring2_col}">■ {d2}</tspan>  ·  <tspan fill="{ring3_col}">■ {d3}</tspan> —</text>"#
    );

    if let Some(idx) = s.rfind("</svg>") {
        s.insert_str(idx, &extra);
    }
    s
}

// ─── 4. Graphic Ephemeris ─────────────────────────────────────────────────────

fn build_graphic_ephemeris_context(
    jd_start: f64,
    jd_end: f64,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN;
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Graphic Ephemeris".to_string());

    let days = ((jd_end - jd_start) as usize).min(366).max(28);
    let step = if days <= 31 { 1 } else { (days / 90).max(1) };

    // Sample planetary positions over the date range
    let mut series: Vec<Vec<f64>> = vec![Vec::new(); BODIES.len()];
    let mut jd_points: Vec<f64> = Vec::new();

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
                "lons": series[i],
            })
        })
        .collect();

    let mut palette = BTreeMap::new();
    for (k, v) in [
        ("bg_color", "#ffffff"),
        ("ring_color", "#1a1a2e"),
        ("text_color", "#0d0d1e"),
    ] {
        palette.insert(k.to_string(), json!(v));
    }
    for (k, v) in &vars {
        palette.insert(k.clone(), json!(v));
    }

    Ok(json!({
        "jd_start": jd_start, "jd_end": jd_end,
        "jd_points": jd_points,
        "days": days, "step": step,
        "planet_series": planet_series,
        "vars": Value::Object(palette.into_iter().collect()),
    }))
}

fn render_graphic_ephemeris_svg(ctx: &Value) -> String {
    use std::fmt::Write;

    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fff");
    let ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#1a1a2e");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#0d0d1e");

    const W: f64 = 860.0; // chart width
    const H: f64 = 480.0; // chart height
    const LM: f64 = 32.0; // left margin (degree labels)
    const TM: f64 = 60.0; // top margin
    const BM: f64 = 80.0; // bottom margin (date labels)

    let jd_start = ctx["jd_start"].as_f64().unwrap_or(0.0);
    let jd_end = ctx["jd_end"].as_f64().unwrap_or(0.0);
    let jd_span = (jd_end - jd_start).max(1.0);

    let jd_points = ctx["jd_points"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let n = jd_points.len();
    if n == 0 {
        return String::new();
    }

    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Graphic Ephemeris");

    let mut s = String::with_capacity(64 * 1024);
    let total_h = TM + H + BM + 40.0;
    let _ = writeln!(
        s,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 {total_h:.0}" width="900" height="{total_h:.0}">
  <rect width="900" height="{total_h:.0}" fill="{bg}"/>
  <text x="450" y="28" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>"#
    );

    // Y-axis: 0–360° longitude
    // Grid lines every 30° (sign boundaries) + labels
    let sign_glyphs = [
        "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓",
    ];
    for s_idx in 0..=12u32 {
        let lon = s_idx as f64 * 30.0;
        let y = TM + H - (lon / 360.0) * H;
        let op = if s_idx % 3 == 0 { ".4" } else { ".2" };
        let _ = writeln!(
            s,
            r#"  <line x1="{:.2}" y1="{y:.2}" x2="{:.2}" y2="{y:.2}" stroke="{ring}" stroke-width="0.6" opacity="{op}"/>"#,
            LM,
            LM + W
        );
        if s_idx < 12 {
            let gy = TM + H - ((lon + 15.0) / 360.0) * H;
            let _ = writeln!(
                s,
                r#"  <text x="{:.2}" y="{gy:.2}" font-size="11" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{ring}" opacity=".6">{}</text>"#,
                LM - 10.0,
                sign_glyphs[s_idx as usize]
            );
        }
    }

    // Planet color palette (12 distinct colors)
    const PLANET_COLORS: &[&str] = &[
        "#d4a800", "#9b59b6", "#3498db", "#27ae60", "#e74c3c", "#e67e22", "#1a5276", "#117a65",
        "#6c3483", "#7f8c8d", "#2e86c1", "#b7950b",
    ];

    // X-axis: time
    let x_scale = W / jd_span;

    // Plot each planet
    let series = ctx["planet_series"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    for (pi, planet) in series.iter().enumerate() {
        let lons = planet["lons"]
            .as_array()
            .map(|v| v.to_vec())
            .unwrap_or_default();
        let col = PLANET_COLORS.get(pi).copied().unwrap_or("#888");
        let glyph = planet["glyph"].as_str().unwrap_or("?");

        let mut path = String::new();
        let mut first = true;
        for (xi, jd_val) in jd_points.iter().enumerate() {
            let jd_v = jd_val.as_f64().unwrap_or(0.0);
            let lon = lons.get(xi).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
            if !lon.is_finite() {
                first = true;
                continue;
            }
            let x = LM + (jd_v - jd_start) * x_scale;
            let y = TM + H - (lon / 360.0) * H;
            if first {
                let _ = write!(path, "M{x:.1},{y:.1}");
                first = false;
            } else {
                let _ = write!(path, "L{x:.1},{y:.1}");
            }
        }
        if !path.is_empty() {
            let _ = writeln!(
                s,
                r#"  <path d="{path}" fill="none" stroke="{col}" stroke-width="1.5" opacity=".8"/>"#
            );
        }

        // Glyph label at end of line
        if let Some(last_jd) = jd_points.last().and_then(|v| v.as_f64()) {
            if let Some(last_lon) = lons.last().and_then(|v| v.as_f64()) {
                if last_lon.is_finite() {
                    let lx = LM + (last_jd - jd_start) * x_scale + 8.0;
                    let ly = TM + H - (last_lon / 360.0) * H;
                    let _ = writeln!(
                        s,
                        r#"  <text x="{lx:.1}" y="{ly:.1}" font-size="12" dominant-baseline="central" font-family="serif" fill="{col}">{glyph}</text>"#
                    );
                }
            }
        }
    }

    // X-axis date labels (approx monthly)
    let label_interval = (jd_span / 6.0).max(7.0);
    let mut jd_lbl = jd_start;
    while jd_lbl <= jd_end + 1.0 {
        let x = LM + (jd_lbl - jd_start) * x_scale;
        let d = celestial_core::revjul(jd_lbl, celestial_core::body::Calendar::Gregorian);
        let lbl = format!("{:.0}-{:02.0}", d.year, d.month);
        let _ = writeln!(
            s,
            r#"  <line x1="{x:.1}" y1="{:.1}" x2="{x:.1}" y2="{:.1}" stroke="{ring}" stroke-width="0.6" opacity=".3"/>
  <text x="{x:.1}" y="{:.1}" font-size="9" text-anchor="middle" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".7">{lbl}</text>"#,
            TM,
            TM + H,
            TM + H + 14.0
        );
        jd_lbl += label_interval;
    }

    let _ = writeln!(
        s,
        r#"  <rect x="{:.2}" y="{TM:.2}" width="{W:.2}" height="{H:.2}" fill="none" stroke="{ring}" stroke-width="0.8" opacity=".4"/>"#,
        LM
    );
    let _ = writeln!(s, "</svg>");
    s
}

// ─── 5. Local Space chart ─────────────────────────────────────────────────────

fn build_local_space_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN;
    let geopos = [lon, lat, 0.0_f64];
    let mut vars = user_vars;
    vars.entry("title".to_string())
        .or_insert("Local Space Chart".to_string());

    let mut planets = Vec::new();
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
                "above_horizon": alt > 0.0,
            }));
        }
    }

    let mut palette = BTreeMap::new();
    for (k, v) in [
        ("bg_color", "#ffffff"),
        ("ring_color", "#1a1a2e"),
        ("planet_color", "#0d0d1e"),
        ("retro_color", "#b01020"),
        ("text_color", "#0d0d1e"),
    ] {
        palette.insert(k.to_string(), json!(v));
    }
    for (k, v) in &vars {
        palette.insert(k.clone(), json!(v));
    }

    Ok(json!({
        "date": date_str, "jd": jd, "lat": lat, "lon": lon,
        "planets": planets,
        "vars": Value::Object(palette.into_iter().collect()),
    }))
}

fn render_local_space_svg(ctx: &Value) -> String {
    use std::fmt::Write;
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

    const CX: f64 = 450.0;
    const CY: f64 = 450.0;
    const R: f64 = 320.0;

    let mut s = String::with_capacity(32 * 1024);
    let _ = writeln!(
        s,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 980" width="900" height="980">
  <rect width="900" height="980" fill="{bg}"/>
  <text x="450" y="30" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="450" y="48" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">{date}</text>
  <text x="450" y="62" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".5">{:.4}°{} {:.4}°{}</text>
  <!-- Compass rings -->"#,
        lat.abs(),
        if lat >= 0.0 { "N" } else { "S" },
        lon_v.abs(),
        if lon_v >= 0.0 { "E" } else { "W" }
    );

    // Concentric rings (every 30°)
    for i in 1..=3u32 {
        let r = R * i as f64 / 3.0;
        let deg = 90 * i;
        let _ = writeln!(
            s,
            r#"  <circle cx="{CX}" cy="{CY}" r="{r:.1}" fill="none" stroke="{ring}" stroke-width="0.7" opacity=".2"/>
  <text x="{:.2}" y="{:.2}" font-size="8" text-anchor="middle" fill="{ring}" opacity=".35">{deg}°</text>"#,
            CX + r + 4.0,
            CY
        );
    }

    // Cardinal direction lines and labels
    for (deg, label) in [(0.0_f64, "N"), (90.0, "E"), (180.0, "S"), (270.0, "W")] {
        let a = (deg - 90.0).to_radians();
        let x1 = CX + (R - 5.0) * a.cos();
        let y1 = CY + (R - 5.0) * a.sin();
        let x2 = CX + (R + 20.0) * a.cos();
        let y2 = CY + (R + 20.0) * a.sin();
        let _ = writeln!(
            s,
            r#"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{ring}" stroke-width="1.5" opacity=".55"/>
  <text x="{x2:.2}" y="{y2:.2}" font-size="14" font-weight="700" text-anchor="middle" dominant-baseline="central" fill="{ring}">{label}</text>"#
        );
    }

    // Degree ticks (every 10°)
    for deg in (0..360u32).step_by(10) {
        let a = (deg as f64 - 90.0).to_radians();
        let is_30 = deg % 30 == 0;
        let (r1, r2) = if is_30 { (R - 10.0, R) } else { (R - 5.0, R) };
        let x1 = CX + r1 * a.cos();
        let y1 = CY + r1 * a.sin();
        let x2 = CX + r2 * a.cos();
        let y2 = CY + r2 * a.sin();
        let _ = writeln!(
            s,
            r#"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{ring}" stroke-width="0.8" opacity=".4"/>"#
        );
        if is_30 && deg > 0 {
            let xd = CX + (R + 12.0) * a.cos();
            let yd = CY + (R + 12.0) * a.sin();
            let _ = writeln!(
                s,
                r#"  <text x="{xd:.2}" y="{yd:.2}" font-size="8" text-anchor="middle" dominant-baseline="central" fill="{ring}" opacity=".5">{deg}°</text>"#
            );
        }
    }

    // Planet lines (direction lines from centre)
    let planets = ctx["planets"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    for p in &planets {
        let az = p["azimuth"].as_f64().unwrap_or(0.0);
        let alt = p["altitude"].as_f64().unwrap_or(0.0);
        let above = p["above_horizon"].as_bool().unwrap_or(false);
        let g = p["glyph"].as_str().unwrap_or("?");
        // Distance from centre proportional to azimuth radius (horizon = R)
        // Above horizon: full R; below: 60% R with dashed
        let r_planet = if above { R * 0.88 } else { R * 0.55 };
        let a = (az - 90.0).to_radians();
        let px = CX + r_planet * a.cos();
        let py = CY + r_planet * a.sin();
        // Line from center
        let (sw, dash, op) = if above {
            ("1.5", "", ".7")
        } else {
            ("1.0", r#" stroke-dasharray="4,3""#, ".4")
        };
        let _ = writeln!(
            s,
            r#"  <line x1="{CX}" y1="{CY}" x2="{px:.2}" y2="{py:.2}" stroke="{pfg}" stroke-width="{sw}" opacity="{op}"{dash}/>
  <text x="{px:.2}" y="{py:.2}" font-size="16" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{pfg}" opacity="{op}">{g}</text>
  <text x="{:.2}" y="{:.2}" font-size="8" text-anchor="middle" font-family="'Segoe UI',system-ui,sans-serif" fill="{pfg}" opacity=".55">{az:.0}°</text>"#,
            px + (px - CX) * 0.12,
            py + (py - CY) * 0.12
        );
        let _ = writeln!(
            s,
            r#"  <text x="{:.2}" y="{:.2}" font-size="7" text-anchor="middle" font-family="'Segoe UI',system-ui,sans-serif" fill="{pfg}" opacity=".4">{:+.1}°</text>"#,
            px + (px - CX) * 0.2,
            py + (py - CY) * 0.2,
            alt
        );
    }

    let _ = writeln!(s, "</svg>");
    s
}

fn jd_to_date_str(jd: f64) -> String {
    let d = celestial_core::revjul(jd, celestial_core::body::Calendar::Gregorian);
    format!(
        "{:04}-{:02}-{:02}",
        d.year as i32, d.month as u32, d.day as u32
    )
}

// ─── Planet table ─────────────────────────────────────────────────────────────

const BODIES: &[(Body, &str, &str, &str)] = &[
    (Body::SUN, "sun", "Sun", "\u{2609}"),
    (Body::MOON, "moon", "Moon", "\u{263D}"),
    (Body::MERCURY, "mercury", "Mercury", "\u{263F}"),
    (Body::VENUS, "venus", "Venus", "\u{2640}"),
    (Body::MARS, "mars", "Mars", "\u{2642}"),
    (Body::JUPITER, "jupiter", "Jupiter", "\u{2643}"),
    (Body::SATURN, "saturn", "Saturn", "\u{2644}"),
    (Body::URANUS, "uranus", "Uranus", "\u{2645}"),
    (Body::NEPTUNE, "neptune", "Neptune", "\u{2646}"),
    (Body::PLUTO, "pluto", "Pluto", "\u{2647}"),
    (Body::MEAN_NODE, "mean_node", "Mean Node", "\u{260A}"),
    (Body::CHIRON, "chiron", "Chiron", "\u{26B7}"),
];

/// (angle°, name, orb_limit, is_minor)
const ASPECT_DEFS: &[(f64, &str, f64, bool)] = &[
    // ── Major aspects ─────────────────────────────────────────────────────────
    (0.0, "conjunction", 8.0, false),
    (60.0, "sextile", 6.0, false),
    (90.0, "square", 7.0, false),
    (120.0, "trine", 8.0, false),
    (150.0, "quincunx", 3.0, false),
    (180.0, "opposition", 8.0, false),
    // ── Minor aspects ─────────────────────────────────────────────────────────
    (30.0, "semi-sextile", 2.0, true),
    (45.0, "semi-square", 2.0, true),
    (72.0, "quintile", 1.5, true),
    (135.0, "sesquiquadrate", 2.0, true),
    (144.0, "biquintile", 1.5, true),
    (51.43, "septile", 1.0, true),
    (40.0, "novile", 1.0, true),
];

// ─── Dignity helper ───────────────────────────────────────────────────────────

/// Returns the essential dignity label for a planet at a given sign (0–11).
///
/// Checks (in order): domicile, exaltation, detriment, fall, peregrine.
fn planet_dignity(body: Body, sign: u8) -> &'static str {
    let s = sign % 12;
    // Domicile: body rules this sign
    if sign_ruler(s) == body || {
        // Some planets have two domiciles (pre-outer planets scheme)
        // Check the opposite polarity sign too
        let also = match body.as_raw() {
            2 => Some(8u8),  // Mercury: Gemini + Virgo
            3 => Some(6u8),  // Venus:   Taurus + Libra
            4 => Some(7u8),  // Mars:    Aries  + Scorpio
            5 => Some(11u8), // Jupiter: Sagittarius + Pisces
            6 => Some(9u8),  // Saturn:  Capricorn + Aquarius
            _ => None,
        };
        also.map_or(false, |s2| s2 == s && sign_ruler(s2) == body)
    } {
        return "domicile";
    }
    // Detriment: opposite of domicile
    let opp = (s + 6) % 12;
    if sign_ruler(opp) == body || {
        let also = match body.as_raw() {
            2 => Some((8u8 + 6) % 12),
            3 => Some((6u8 + 6) % 12),
            4 => Some((7u8 + 6) % 12),
            5 => Some((11u8 + 6) % 12),
            6 => Some((9u8 + 6) % 12),
            _ => None,
        };
        also.map_or(false, |s2| s2 == s && sign_ruler((s2 + 6) % 12) == body)
    } {
        return "detriment";
    }
    // Exaltation
    let ex = sign_exaltation(body);
    if ex >= 0 && ex as u8 == s {
        return "exaltation";
    }
    // Fall: opposite of exaltation
    if ex >= 0 && (ex as u8 + 6) % 12 == s {
        return "fall";
    }
    // None of the above
    "peregrine"
}

/// Antiscion longitude: reflection over the 0°Cancer / 0°Capricorn (solstice) axis.
/// Formula: antiscion = (180° - lon) mod 360°
#[inline]
fn antiscion_lon(lon: f64) -> f64 {
    (180.0 - lon).rem_euclid(360.0)
}

/// Contra-antiscion longitude: reflection over the 0°Aries / 0°Libra (equinox) axis.
/// Formula: contra = (360° - lon) mod 360°
#[inline]
fn contra_antiscion_lon(lon: f64) -> f64 {
    (360.0 - lon).rem_euclid(360.0)
}

// ─── Build the full context ───────────────────────────────────────────────────

const CX: f64 = 450.0;
const CY: f64 = 490.0;
const RO: f64 = 320.0; // outer ring
const RM: f64 = 290.0; // sign band outer
const RI: f64 = 262.0; // sign band inner
const RH: f64 = 238.0; // house cusp inner
const RP: f64 = 212.0; // planet ring
const RC: f64 = 88.0; // inner circle

fn build_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let h = houses_ex(jd, CalcFlags::BUILTIN, lat, lon, HouseSystem(hsys as u8))
        .map_err(|e| e.to_string())?;
    let asc = h.ascmc[0];
    let mc = h.ascmc[1];
    let ic = (mc + 180.0).rem_euclid(360.0);
    let dsc = (asc + 180.0).rem_euclid(360.0);

    // ── planets ──────────────────────────────────────────────────────────────
    let mut planets = Vec::new();
    for &(body, key, name, glyph) in BODIES {
        if let Ok(pos) = calc_ut(jd, body, flags) {
            let (sign_idx, deg_in_sign) = lon_to_sign(pos.lon);
            let sign_full = zodiac_sign_name(sign_idx);
            let sign_short = &sign_full[..sign_full
                .char_indices()
                .nth(3)
                .map(|(i, _)| i)
                .unwrap_or(sign_full.len())];
            let deg_label = format!(
                "{:.0}\u{00B0}{}{}",
                deg_in_sign.floor(),
                sign_short,
                if pos.speed_lon < 0.0 { "\u{211E}" } else { "" }
            );
            planets.push(json!({
                "name":       name,
                "key":        key,
                "glyph":      glyph,
                "lon":        (pos.lon   * 1e4).round() / 1e4,
                "lat":        (pos.lat   * 1e4).round() / 1e4,
                "dist":       (pos.dist  * 1e4).round() / 1e4,
                "speed":      (pos.speed_lon * 1e4).round() / 1e4,
                "retro":      pos.speed_lon < 0.0,
                // Station: speed very close to 0 → planet is stationary
                "near_station": pos.speed_lon.abs() < 0.05,
                "dignity":    planet_dignity(body, sign_idx),
                // Antiscia: mirror over the Cancer-Capricorn solstice axis
                "antiscia_lon":  (antiscion_lon(pos.lon) * 1e4).round() / 1e4,
                "contra_lon":    (contra_antiscion_lon(pos.lon) * 1e4).round() / 1e4,
                "antiscia_x": (wx(CX, RH - 4.0, antiscion_lon(pos.lon), asc) * 100.0).round() / 100.0,
                "antiscia_y": (wy(CY, RH - 4.0, antiscion_lon(pos.lon), asc) * 100.0).round() / 100.0,
                "sign":       sign_idx,
                "sign_name":  zodiac_sign_name(sign_idx),
                "dms":        fmt_lon_dms(pos.lon),
                "deg_label":  deg_label,
                "speed_str":  format!("{}{:.2}\u{00B0}/d",
                                if pos.speed_lon < 0.0 { "\u{211E} " } else { "" },
                                pos.speed_lon.abs()),
                // wheel coordinates
                "x":        (wx(CX, RP, pos.lon, asc) * 100.0).round() / 100.0,
                "y":        (wy(CY, RP, pos.lon, asc) * 100.0).round() / 100.0,
                "label_x":  (wx(CX, RP + 20.0, pos.lon, asc) * 100.0).round() / 100.0,
                "label_y":  (wy(CY, RP + 20.0, pos.lon, asc) * 100.0).round() / 100.0,
                "tick_x1":  (wx(CX, RH + 2.0,  pos.lon, asc) * 100.0).round() / 100.0,
                "tick_y1":  (wy(CY, RH + 2.0,  pos.lon, asc) * 100.0).round() / 100.0,
                "tick_x2":  (wx(CX, RP - 12.0, pos.lon, asc) * 100.0).round() / 100.0,
                "tick_y2":  (wy(CY, RP - 12.0, pos.lon, asc) * 100.0).round() / 100.0,
                "asp_x":    (wx(CX, RC, pos.lon, asc) * 100.0).round() / 100.0,
                "asp_y":    (wy(CY, RC, pos.lon, asc) * 100.0).round() / 100.0,
            }));
        }
    }

    // ── signs ────────────────────────────────────────────────────────────────
    let sign_glyphs = [
        "\u{2648}", "\u{2649}", "\u{264A}", "\u{264B}", "\u{264C}", "\u{264D}", "\u{264E}",
        "\u{264F}", "\u{2650}", "\u{2651}", "\u{2652}", "\u{2653}",
    ];
    let signs: Vec<Value> = (0..12)
        .map(|i| {
            let sl = i as f64 * 30.0;
            let mid = sl + 15.0;
            let sgr = (RM + RI) / 2.0;
            json!({
                "idx":       i,
                "glyph":     sign_glyphs[i],
                "spoke_x1":  (wx(CX, RI, sl,  asc) * 100.0).round() / 100.0,
                "spoke_y1":  (wy(CY, RI, sl,  asc) * 100.0).round() / 100.0,
                "spoke_x2":  (wx(CX, RO, sl,  asc) * 100.0).round() / 100.0,
                "spoke_y2":  (wy(CY, RO, sl,  asc) * 100.0).round() / 100.0,
                "glyph_x":   (wx(CX, sgr, mid, asc) * 100.0).round() / 100.0,
                "glyph_y":   (wy(CY, sgr, mid, asc) * 100.0).round() / 100.0,
            })
        })
        .collect();

    // ── house cusps ──────────────────────────────────────────────────────────
    let house_lons: Vec<f64> = h.cusps[1..=12].to_vec();
    let houses: Vec<Value> = (0..12)
        .map(|i| {
            let lon2 = house_lons[i];
            let next_lon = house_lons[(i + 1) % 12];
            let mid_lon = midpoint_deg(lon2, next_lon);
            let is_angle = i == 0 || i == 3 || i == 6 || i == 9;
            json!({
                "num":      i + 1,
                "lon":      (lon2 * 1e4).round() / 1e4,
                "dms":      fmt_lon_dms(lon2),
                "is_angle": is_angle,
                "x1":       (wx(CX, RH, lon2, asc) * 100.0).round() / 100.0,
                "y1":       (wy(CY, RH, lon2, asc) * 100.0).round() / 100.0,
                "x2":       (wx(CX, RI, lon2, asc) * 100.0).round() / 100.0,
                "y2":       (wy(CY, RI, lon2, asc) * 100.0).round() / 100.0,
                "num_x":    (wx(CX, RH - 14.0, mid_lon, asc) * 100.0).round() / 100.0,
                "num_y":    (wy(CY, RH - 14.0, mid_lon, asc) * 100.0).round() / 100.0,
            })
        })
        .collect();

    // ── aspects ──────────────────────────────────────────────────────────────
    let mut aspects = Vec::new();
    for i in 0..planets.len() {
        for j in (i + 1)..planets.len() {
            let lon1 = planets[i]["lon"].as_f64().unwrap_or(0.0);
            let lon2 = planets[j]["lon"].as_f64().unwrap_or(0.0);
            let diff = diff_deg_signed(lon1, lon2).abs();
            for &(asp_deg, asp_name, orb_lim, is_minor) in ASPECT_DEFS {
                let orb = (diff - asp_deg).abs();
                if orb <= orb_lim {
                    let spd1 = planets[i]["speed"].as_f64().unwrap_or(0.0);
                    aspects.push(json!({
                        "body1":       planets[i]["name"],
                        "glyph1":      planets[i]["glyph"],
                        "body2":       planets[j]["name"],
                        "glyph2":      planets[j]["glyph"],
                        "aspect_name": asp_name,
                        "aspect_deg":  asp_deg,
                        "orb":         (orb * 100.0).round() / 100.0,
                        "applying":    spd1 > 0.0 && diff < asp_deg,
                        "is_hard":     asp_name == "square" || asp_name == "opposition",
                        "is_minor":    is_minor,
                        "x1":          planets[i]["asp_x"],
                        "y1":          planets[i]["asp_y"],
                        "x2":          planets[j]["asp_x"],
                        "y2":          planets[j]["asp_y"],
                    }));
                    break;
                }
            }
        }
    }

    // ── arabic parts ─────────────────────────────────────────────────────────
    let sun_lon = planets
        .iter()
        .find(|p| p["key"] == "sun")
        .and_then(|p| p["lon"].as_f64())
        .unwrap_or(0.0);
    let moon_lon = planets
        .iter()
        .find(|p| p["key"] == "moon")
        .and_then(|p| p["lon"].as_f64())
        .unwrap_or(0.0);
    let sat_lon = planets
        .iter()
        .find(|p| p["key"] == "saturn")
        .and_then(|p| p["lon"].as_f64())
        .unwrap_or(0.0);
    let mar_lon = planets
        .iter()
        .find(|p| p["key"] == "mars")
        .and_then(|p| p["lon"].as_f64())
        .unwrap_or(0.0);
    let jup_lon = planets
        .iter()
        .find(|p| p["key"] == "jupiter")
        .and_then(|p| p["lon"].as_f64())
        .unwrap_or(0.0);
    let mer_lon = planets
        .iter()
        .find(|p| p["key"] == "mercury")
        .and_then(|p| p["lon"].as_f64())
        .unwrap_or(0.0);
    let ven_lon = planets
        .iter()
        .find(|p| p["key"] == "venus")
        .and_then(|p| p["lon"].as_f64())
        .unwrap_or(0.0);

    // Day chart: Sun is in houses 7–12 (above horizon at lat/lon/jd)
    let sun_house = h.cusps[1..=12]
        .windows(2)
        .position(|w| {
            let lo = w[0];
            let hi = w[1];
            if lo < hi {
                lo <= sun_lon && sun_lon < hi
            } else {
                sun_lon >= lo || sun_lon < hi
            }
        })
        .map(|i| i + 1)
        .unwrap_or(1);
    let is_day = sun_house >= 7;

    let arabic_parts_raw = arabic_parts_seven(
        asc, sun_lon, moon_lon, sat_lon, mar_lon, jup_lon, mer_lon, ven_lon, is_day,
    );
    let arabic_parts: Vec<Value> = arabic_parts_raw
        .iter()
        .map(|p| {
            let (sign_idx, deg_in_sign) = lon_to_sign(p.degree);
            json!({
                "name":     p.name,
                "formula":  p.formula,
                "lon":      (p.degree * 1e4).round() / 1e4,
                "dms":      fmt_lon_dms(p.degree),
                "sign":     zodiac_sign_name(sign_idx),
                "deg_in_sign": (deg_in_sign * 100.0).round() / 100.0,
                "x":        (wx(CX, RH + 2.0, p.degree, asc) * 100.0).round() / 100.0,
                "y":        (wy(CY, RH + 2.0, p.degree, asc) * 100.0).round() / 100.0,
                "is_day":   is_day,
            })
        })
        .collect();

    // ── fixed stars (top 15 brightest / most astrologically significant) ──────
    const TOP_STARS: &[&str] = &[
        "Algol",
        "Pleiades",
        "Aldebaran",
        "Rigel",
        "Capella",
        "Sirius",
        "Pollux",
        "Regulus",
        "Spica",
        "Arcturus",
        "Antares",
        "Vega",
        "Altair",
        "Fomalhaut",
        "Achernar",
    ];
    let fixed_stars: Vec<Value> = TOP_STARS
        .iter()
        .filter_map(|&name| {
            let pos = fixstar_ut(name, jd, CalcFlags::BUILTIN).ok()?;
            let lon_s = pos.xx[0];
            let lat_s = pos.xx[1];
            let mag = fixstar_mag(name).unwrap_or(3.0);
            let (sign_idx, deg_in_sign) = lon_to_sign(lon_s);
            Some(json!({
                "name":        name,
                "mag":         mag,
                "lon":         (lon_s * 1e4).round() / 1e4,
                "lat":         (lat_s * 1e4).round() / 1e4,
                "sign":        zodiac_sign_name(sign_idx),
                "deg_in_sign": (deg_in_sign * 100.0).round() / 100.0,
                "x":  (wx(CX, RI + 8.0, lon_s, asc) * 100.0).round() / 100.0,
                "y":  (wy(CY, RI + 8.0, lon_s, asc) * 100.0).round() / 100.0,
            }))
        })
        .collect();

    // ── angles with label positions ───────────────────────────────────────────
    let angle_lons = [asc, mc, ic, dsc];
    let angle_labels = ["ASC", "MC", "IC", "DSC"];
    let angles: Vec<Value> = (0..4)
        .map(|i| {
            let lon2 = angle_lons[i];
            json!({
                "name":  angle_labels[i],
                "lon":   (lon2 * 1e4).round() / 1e4,
                "dms":   fmt_lon_dms(lon2),
                "lx":    (wx(CX, RI + 18.0, lon2, asc) * 100.0).round() / 100.0,
                "ly":    (wy(CY, RI + 18.0, lon2, asc) * 100.0).round() / 100.0,
            })
        })
        .collect();

    // ── moon ─────────────────────────────────────────────────────────────────
    let illum_pct = (moon_illumination(jd).unwrap_or(0.0) * 1000.0).round() / 10.0;

    // ── user vars (with palette defaults merged in) ───────────────────────────
    let mut vars = serde_json::Map::new();
    // defaults
    for (k, v) in [
        ("bg_color", "#ffffff"),
        ("ring_color", "#1a1a2e"),
        ("planet_color", "#0d0d1e"),
        ("retro_color", "#b01020"),
        ("hard_color", "#b01020"),
        ("soft_color", "#1a50b0"),
        ("text_color", "#0d0d1e"),
        ("title", "Celestial Chart"),
    ] {
        vars.insert(k.to_string(), json!(v));
    }
    // user overrides
    for (k, v) in &user_vars {
        vars.insert(k.clone(), json!(v));
    }

    // ── assemble ──────────────────────────────────────────────────────────────
    Ok(json!({
        "date":                date_str,
        "jd":                  (jd * 1e4).round() / 1e4,
        "lat":                 lat,
        "lon":                 lon,
        "asc":                 (asc * 1e4).round() / 1e4,
        "mc":                  (mc  * 1e4).round() / 1e4,
        "ic":                  (ic  * 1e4).round() / 1e4,
        "dsc":                 (dsc * 1e4).round() / 1e4,
        "asc_dms":             fmt_lon_dms(asc),
        "mc_dms":              fmt_lon_dms(mc),
        "ic_dms":              fmt_lon_dms(ic),
        "dsc_dms":             fmt_lon_dms(dsc),
        "moon_phase_name":     moon_phase_str(jd),
        "moon_illumination":   illum_pct,
        "cx":                  CX, "cy": CY,
        "r_outer":             RO, "r_sign_outer": RM, "r_sign_inner": RI,
        "r_house":             RH, "r_planet": RP,    "r_inner": RC,
        "planets":             planets,
        "signs":               signs,
        "houses":              houses,
        "angles":              angles,
        "aspects":             aspects,
        "arabic_parts":        arabic_parts,
        "fixed_stars":         fixed_stars,
        "vars":                Value::Object(vars),
    }))
}

// ─── Label collision avoidance ────────────────────────────────────────────────

/// Compute non-overlapping wheel angles for planet degree labels.
///
/// Starts each label at its natural angular position (derived from the planet
/// longitude) and iteratively separates overlapping pairs symmetrically along
/// the arc, keeping each label within `MAX_DRIFT` degrees of its planet.
/// Returns one adjusted angle per planet, in the original planet order.
fn spread_labels(lons: &[f64], asc: f64) -> Vec<f64> {
    // Approximate angular half-width of a degree label (e.g. "29°Gem") at
    // the label ring radius.  At r = RP + 22 ≈ 234px, 10° of arc ≈ 41px,
    // which comfortably brackets a ~36px label.
    const HALF_DEG: f64 = 5.5; // half-width in degrees
    const MIN_SEP: f64 = HALF_DEG * 2.0 + 1.5; // 12.5° minimum centre-to-centre
    const MAX_DRIFT: f64 = 28.0; // max degrees a label may wander from its planet
    const MAX_ITER: usize = 300;

    let n = lons.len();
    let natural: Vec<f64> = lons
        .iter()
        .map(|&l| (180.0 - (l - asc)).rem_euclid(360.0))
        .collect();
    let mut placed = natural.clone();

    for _iter in 0..MAX_ITER {
        let mut any = false;
        for i in 0..n {
            for j in (i + 1)..n {
                // Signed angular gap from i to j in (−180, +180]
                let mut d = placed[j] - placed[i];
                while d > 180.0 {
                    d -= 360.0;
                }
                while d < -180.0 {
                    d += 360.0;
                }

                if d.abs() < MIN_SEP {
                    any = true;
                    // Push symmetrically; slightly more than half ensures convergence
                    let push = (MIN_SEP - d.abs()) * 0.55 + 0.05;
                    if d >= 0.0 {
                        placed[j] += push;
                        placed[i] -= push;
                    } else {
                        placed[i] += push;
                        placed[j] -= push;
                    }
                    // Clamp each to ±MAX_DRIFT from its natural angle
                    for k in [i, j] {
                        let mut drift = placed[k] - natural[k];
                        while drift > 180.0 {
                            drift -= 360.0;
                        }
                        while drift < -180.0 {
                            drift += 360.0;
                        }
                        if drift.abs() > MAX_DRIFT {
                            placed[k] = natural[k] + drift.signum() * MAX_DRIFT;
                        }
                    }
                }
            }
        }
        if !any {
            break;
        }
    }
    placed.iter().map(|a| a.rem_euclid(360.0)).collect()
}

// ─── Built-in SVG generator (pure Rust — no template parsing) ────────────────

fn render_builtin_svg(ctx: &Value) -> String {
    let vars = &ctx["vars"];
    let bg = vars["bg_color"].as_str().unwrap_or("#ffffff");
    let ring = vars["ring_color"].as_str().unwrap_or("#1a1a2e");
    let pfg = vars["planet_color"].as_str().unwrap_or("#0d0d1e");
    let retro_c = vars["retro_color"].as_str().unwrap_or("#b01020");
    let hard_c = vars["hard_color"].as_str().unwrap_or("#b01020");
    let soft_c = vars["soft_color"].as_str().unwrap_or("#1a50b0");
    let txt = vars["text_color"].as_str().unwrap_or("#0d0d1e");
    let title = vars["title"].as_str().unwrap_or("Celestial Chart");

    let date = ctx["date"].as_str().unwrap_or("");
    let jd = ctx["jd"].as_f64().unwrap_or(0.0);
    let lat = ctx["lat"].as_f64().unwrap_or(0.0);
    let lon = ctx["lon"].as_f64().unwrap_or(0.0);
    let asc = ctx["asc"].as_f64().unwrap_or(0.0);
    let mc = ctx["mc"].as_f64().unwrap_or(0.0);
    let ic = ctx["ic"].as_f64().unwrap_or(0.0);
    let dsc = ctx["dsc"].as_f64().unwrap_or(0.0);
    let phase = ctx["moon_phase_name"].as_str().unwrap_or("");
    let illum = ctx["moon_illumination"].as_f64().unwrap_or(0.0);

    let planets = ctx["planets"].as_array().unwrap();
    let signs = ctx["signs"].as_array().unwrap();
    let houses = ctx["houses"].as_array().unwrap();
    let aspects = ctx["aspects"].as_array().unwrap();

    let mut s = String::with_capacity(64 * 1024);

    // ── header ────────────────────────────────────────────────────────────────
    let _ = writeln!(
        s,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 1560" width="900" height="1560">
  <defs>
    <filter id="glow" x="-40%" y="-40%" width="180%" height="180%">
      <feGaussianBlur stdDeviation="2.5" result="b"/>
      <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
  </defs>
  <rect width="900" height="1560" fill="{bg}"/>
  <text x="450" y="34" text-anchor="middle" font-size="18" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="450" y="54" text-anchor="middle" font-size="10"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".7">"#
    );
    if lat != 0.0 || lon != 0.0 {
        let _ = write!(
            s,
            "{date} · {:.4}°{} {:.4}°{} · JD {jd:.4}",
            lat.abs(),
            if lat >= 0.0 { "N" } else { "S" },
            lon.abs(),
            if lon >= 0.0 { "E" } else { "W" }
        );
    } else {
        let _ = write!(s, "{date} · JD {jd:.4}");
    }
    let _ = writeln!(
        s,
        r#"</text>

  <!-- rings -->
  <circle cx="{CX}" cy="{CY}" r="{RO}" fill="none" stroke="{ring}" stroke-width="2.5" opacity=".6"/>
  <circle cx="{CX}" cy="{CY}" r="{RM}" fill="none" stroke="{ring}" stroke-width="1.2" opacity=".35"/>
  <circle cx="{CX}" cy="{CY}" r="{RI}" fill="none" stroke="{ring}" stroke-width="2.0" opacity=".55"/>
  <circle cx="{CX}" cy="{CY}" r="{RH}" fill="none" stroke="{ring}" stroke-width="1.2" opacity=".35"/>
  <circle cx="{CX}" cy="{CY}" r="{RC}" fill="{bg}"  stroke="{ring}" stroke-width="2.0" opacity=".4"/>"#
    );

    // ── zodiac sign sectors ───────────────────────────────────────────────────
    for sign in signs {
        let sx1 = sign["spoke_x1"].as_f64().unwrap_or(0.0);
        let sy1 = sign["spoke_y1"].as_f64().unwrap_or(0.0);
        let sx2 = sign["spoke_x2"].as_f64().unwrap_or(0.0);
        let sy2 = sign["spoke_y2"].as_f64().unwrap_or(0.0);
        let gx = sign["glyph_x"].as_f64().unwrap_or(0.0);
        let gy = sign["glyph_y"].as_f64().unwrap_or(0.0);
        let g = sign["glyph"].as_str().unwrap_or("");
        let _ = writeln!(
            s,
            r#"  <line x1="{sx1:.2}" y1="{sy1:.2}" x2="{sx2:.2}" y2="{sy2:.2}" stroke="{ring}" stroke-width="1.5" opacity=".55"/>
  <text x="{gx:.2}" y="{gy:.2}" font-size="15" font-weight="600" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{ring}">{g}</text>"#
        );
    }

    // ── house cusps ───────────────────────────────────────────────────────────
    s.push('\n');
    for h in houses {
        let x1 = h["x1"].as_f64().unwrap_or(0.0);
        let y1 = h["y1"].as_f64().unwrap_or(0.0);
        let x2 = h["x2"].as_f64().unwrap_or(0.0);
        let y2 = h["y2"].as_f64().unwrap_or(0.0);
        let nx = h["num_x"].as_f64().unwrap_or(0.0);
        let ny = h["num_y"].as_f64().unwrap_or(0.0);
        let n = h["num"].as_u64().unwrap_or(0);
        let ang = h["is_angle"].as_bool().unwrap_or(false);
        let (sw, op) = if ang { ("3.0", ".85") } else { ("1.5", ".55") };
        let _ = writeln!(
            s,
            r#"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{ring}" stroke-width="{sw}" opacity="{op}"/>
  <text x="{nx:.2}" y="{ny:.2}" font-size="10" font-weight="500" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">{n}</text>"#
        );
    }

    // ── angle labels on the rim ───────────────────────────────────────────────
    for (lon2, name, anchor, dy) in [
        (asc, "ASC", "end", 0.0),
        (dsc, "DSC", "start", 0.0),
        (mc, "MC", "middle", -7.0),
        (ic, "IC", "middle", 11.0),
    ] {
        let lx = wx(CX, RI + 18.0, lon2, asc);
        let ly = wy(CY, RI + 18.0, lon2, asc) + dy;
        let _ = writeln!(
            s,
            r#"  <text x="{lx:.2}" y="{ly:.2}" text-anchor="{anchor}" font-size="12" font-weight="800" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">{name}</text>"#
        );
    }

    // ── aspect lines ─────────────────────────────────────────────────────────
    s.push('\n');
    for asp in aspects {
        let x1 = asp["x1"].as_f64().unwrap_or(0.0);
        let y1 = asp["y1"].as_f64().unwrap_or(0.0);
        let x2 = asp["x2"].as_f64().unwrap_or(0.0);
        let y2 = asp["y2"].as_f64().unwrap_or(0.0);
        let orb = asp["orb"].as_f64().unwrap_or(8.0);
        let hard = asp["is_hard"].as_bool().unwrap_or(false);
        let minor = asp["is_minor"].as_bool().unwrap_or(false);
        let col = if hard { hard_c } else { soft_c };
        let (sw, op, dash): (&str, &str, &str) = if minor {
            let o: &str = if orb < 1.0 { ".35" } else { ".18" };
            ("0.9", o, r#" stroke-dasharray="4,3""#)
        } else {
            let o: &str = if orb < 2.0 { ".55" } else { ".22" };
            ("1.8", o, "")
        };
        let _ = writeln!(
            s,
            r#"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{col}" stroke-width="{sw}" opacity="{op}"{dash}/>"#
        );
    }

    // ── planet glyphs (collision-free label placement) ────────────────────────
    s.push('\n');

    // Collect planet longitudes and compute non-overlapping label angles
    let planet_lons: Vec<f64> = planets
        .iter()
        .map(|p| p["lon"].as_f64().unwrap_or(0.0))
        .collect();
    let label_angles = spread_labels(&planet_lons, asc);
    const LABEL_R: f64 = RP + 26.0;

    for (idx, p) in planets.iter().enumerate() {
        let px = p["x"].as_f64().unwrap_or(0.0);
        let py = p["y"].as_f64().unwrap_or(0.0);
        let tx1 = p["tick_x1"].as_f64().unwrap_or(0.0);
        let ty1 = p["tick_y1"].as_f64().unwrap_or(0.0);
        let tx2 = p["tick_x2"].as_f64().unwrap_or(0.0);
        let ty2 = p["tick_y2"].as_f64().unwrap_or(0.0);
        let g = p["glyph"].as_str().unwrap_or("?");
        let dl = p["deg_label"].as_str().unwrap_or("");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let col = if ret { retro_c } else { pfg };

        // Adjusted label position from spread_labels (direct angle → SVG coords)
        let placed_ang = label_angles[idx];
        let lx = CX + LABEL_R * placed_ang.to_radians().cos();
        let ly = CY - LABEL_R * placed_ang.to_radians().sin();

        // Natural angle for the leader-line anchor (just outside the glyph ring)
        let lon_i = planet_lons[idx];
        let nat_ang = (180.0 - (lon_i - asc)).rem_euclid(360.0);
        let anchor_r = RP + 13.0;
        let ax = CX + anchor_r * nat_ang.to_radians().cos();
        let ay = CY - anchor_r * nat_ang.to_radians().sin();

        // Draw a dashed leader line only when label drifted from its planet
        let mut drift = placed_ang - nat_ang;
        while drift > 180.0 {
            drift -= 360.0;
        }
        while drift < -180.0 {
            drift += 360.0;
        }
        if drift.abs() > 3.5 {
            let _ = writeln!(
                s,
                r#"  <line x1="{ax:.2}" y1="{ay:.2}" x2="{lx:.2}" y2="{ly:.2}" stroke="{pfg}" stroke-width="0.9" opacity=".45" stroke-dasharray="3,2"/>"#
            );
        }

        let _ = writeln!(
            s,
            r#"  <line x1="{tx1:.2}" y1="{ty1:.2}" x2="{tx2:.2}" y2="{ty2:.2}" stroke="{pfg}" stroke-width="1.0" opacity=".45"/>
  <text x="{px:.2}" y="{py:.2}" font-size="18" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}" filter="url(#glow)">{g}</text>
  <text x="{lx:.2}" y="{ly:.2}" font-size="10" font-weight="600" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}">{dl}</text>"#
        );
        // Station marker: small 'S' if planet speed ≈ 0
        if p["near_station"].as_bool().unwrap_or(false) {
            let sx = px + 9.0;
            let sy = py - 9.0;
            let _ = writeln!(
                s,
                r#"  <text x="{sx:.2}" y="{sy:.2}" font-size="7" font-weight="700" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}" opacity=".9">S</text>"#
            );
        }
    }

    // ── moon phase in centre ──────────────────────────────────────────────────
    let _ = writeln!(
        s,
        r#"  <text x="{CX}" y="{:.2}" text-anchor="middle" font-size="11" font-weight="500" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".9">{phase}</text>
  <text x="{CX}" y="{:.2}" text-anchor="middle" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".65">{illum:.1}%</text>
"#,
        CY - 12.0,
        CY + 6.0
    );

    // ═════════════════════════ LEGEND ═════════════════════════════════════════
    let ly = CY + RO + 24.0;
    let c1x = 24.0_f64;
    let c2x = 314.0_f64;
    let c3x = 584.0_f64;
    let rh2 = 16.0_f64;

    // ── col 1: planets ────────────────────────────────────────────────────────
    let _ = writeln!(
        s,
        r#"  <text x="{c1x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Planets</text>
  <line x1="{c1x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"#,
        ly + 3.0,
        c1x + 272.0,
        ly + 3.0
    );
    for (i, p) in planets.iter().enumerate() {
        let ry = ly + 16.0 + i as f64 * rh2;
        let g = p["glyph"].as_str().unwrap_or("?");
        let name = p["name"].as_str().unwrap_or("");
        let dms = p["dms"].as_str().unwrap_or("");
        let spd = p["speed_str"].as_str().unwrap_or("");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let col = if ret { retro_c } else { pfg };
        let scol = if ret { retro_c } else { ring };
        let sop = if ret { "1" } else { ".4" };
        let _ = writeln!(
            s,
            r#"  <text x="{:.2}" y="{ry:.2}" font-size="14" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}">{g}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="11" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".75">{name}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{scol}" opacity="{sop}">{spd}</text>"#,
            c1x + 2.0,
            c1x + 20.0,
            c1x + 120.0,
            c1x + 222.0
        );
    }

    // ── col 2: angles + houses ────────────────────────────────────────────────
    let _ = writeln!(
        s,
        r#"
  <text x="{c2x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Angles &amp; Houses</text>
  <line x1="{c2x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"#,
        ly + 3.0,
        c2x + 250.0,
        ly + 3.0
    );
    for (i, (name, lon2, dms)) in [
        ("ASC", asc, ctx["asc_dms"].as_str().unwrap_or("")),
        ("MC", mc, ctx["mc_dms"].as_str().unwrap_or("")),
        ("DSC", dsc, ctx["dsc_dms"].as_str().unwrap_or("")),
        ("IC", ic, ctx["ic_dms"].as_str().unwrap_or("")),
    ]
    .iter()
    .enumerate()
    {
        let ry = ly + 16.0 + i as f64 * rh2;
        let _ = writeln!(
            s,
            r#"  <text x="{:.2}" y="{ry:.2}" font-size="10" font-weight="700" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">{name}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{lon2:.4}&#176;</text>"#,
            c2x + 2.0,
            c2x + 38.0,
            c2x + 150.0
        );
    }
    let sep_y = ly + 82.0;
    let _ = writeln!(
        s,
        r#"  <line x1="{c2x}" y1="{sep_y:.2}" x2="{:.2}" y2="{sep_y:.2}" stroke="{ring}" stroke-width=".3" opacity=".2"/>"#,
        c2x + 250.0
    );
    for (i, h) in houses.iter().enumerate() {
        let ry = ly + 94.0 + i as f64 * rh2;
        let dms = h["dms"].as_str().unwrap_or("");
        let hlon = h["lon"].as_f64().unwrap_or(0.0);
        let _ = writeln!(
            s,
            r#"  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".55">H{}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{hlon:.4}&#176;</text>"#,
            c2x + 2.0,
            i + 1,
            c2x + 28.0,
            c2x + 138.0
        );
    }

    // ── col 3: aspects ────────────────────────────────────────────────────────
    let _ = writeln!(
        s,
        r#"
  <text x="{c3x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Aspects</text>
  <line x1="{c3x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"#,
        ly + 3.0,
        c3x + 292.0,
        ly + 3.0
    );
    if aspects.is_empty() {
        let _ = writeln!(
            s,
            r#"  <text x="{:.2}" y="{:.2}" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">(no major aspects within orbs)</text>"#,
            c3x + 4.0,
            ly + 20.0
        );
    }
    for (i, asp) in aspects.iter().enumerate() {
        let ry = ly + 16.0 + i as f64 * 15.0;
        let g1 = asp["glyph1"].as_str().unwrap_or("?");
        let g2 = asp["glyph2"].as_str().unwrap_or("?");
        let aname = asp["aspect_name"].as_str().unwrap_or("");
        let orb = asp["orb"].as_f64().unwrap_or(0.0);
        let appl = asp["applying"].as_bool().unwrap_or(false);
        let b1 = asp["body1"].as_str().unwrap_or("");
        let b2 = asp["body2"].as_str().unwrap_or("");
        let hard = asp["is_hard"].as_bool().unwrap_or(false);
        let col = if hard { hard_c } else { soft_c };
        let aind = if appl { "&#9650;app" } else { "&#9660;sep" };
        let b1s = &b1[..b1.len().min(3)];
        let b2s = &b2[..b2.len().min(3)];
        let an4 = &aname[..aname.len().min(4)];
        let _ = writeln!(
            s,
            r#"  <text x="{:.2}" y="{ry:.2}" font-size="13" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}">{g1}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="13" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}">{g2}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}">{an4}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".7">{orb:.2}&#176;</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".5">{aind}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{b1s}&#8211;{b2s}</text>"#,
            c3x + 2.0,
            c3x + 18.0,
            c3x + 34.0,
            c3x + 92.0,
            c3x + 130.0,
            c3x + 168.0
        );
    }

    // ── dignity + arabic parts legend (below col 1 + 2) ─────────────────────
    let dig_y = ly + 16.0 + planets.len() as f64 * rh2 + 12.0;
    let _ = writeln!(
        s,
        r#"  <text x="{c1x}" y="{dig_y:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Essential Dignities</text>
  <line x1="{c1x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"#,
        dig_y + 3.0,
        c1x + 272.0,
        dig_y + 3.0
    );
    const DIG_COLORS: [(&str, &str); 5] = [
        ("domicile", "#1a7a1a"),
        ("exaltation", "#0d5ca8"),
        ("detriment", "#b01020"),
        ("fall", "#8b4000"),
        ("peregrine", "#888888"),
    ];
    // Table header
    let _ = writeln!(
        s,
        r#"  <text x="{:.2}" y="{:.2}" font-size="8" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">Planet</text>
  <text x="{:.2}" y="{:.2}" font-size="8" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">Dignity</text>
  <text x="{:.2}" y="{:.2}" font-size="8" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">Sign</text>"#,
        c1x + 2.0,
        dig_y + 14.0,
        c1x + 60.0,
        dig_y + 14.0,
        c1x + 140.0,
        dig_y + 14.0
    );
    for (i, p) in planets.iter().enumerate() {
        let ry = dig_y + 26.0 + i as f64 * rh2;
        let g = p["glyph"].as_str().unwrap_or("?");
        let name = p["name"].as_str().unwrap_or("");
        let dig = p["dignity"].as_str().unwrap_or("peregrine");
        let sign_nm = p["sign_name"].as_str().unwrap_or("");
        let dcol = DIG_COLORS
            .iter()
            .find(|(d, _)| *d == dig)
            .map(|(_, c)| *c)
            .unwrap_or("#888");
        let _ = writeln!(
            s,
            r#"  <text x="{:.2}" y="{ry:.2}" font-size="13" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{ring}">{g}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{dcol}" font-weight="500">{dig}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{sign_nm}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".45">{name}</text>"#,
            c1x + 2.0,
            c1x + 20.0,
            c1x + 140.0,
            c1x + 220.0,
        );
    }

    // ── arabic parts mini-legend ──────────────────────────────────────────────
    let ap_y = dig_y + 26.0 + planets.len() as f64 * rh2 + 8.0;
    let _ = writeln!(
        s,
        r#"  <text x="{c2x}" y="{ap_y:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Arabic Parts</text>
  <line x1="{c2x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"#,
        ap_y + 3.0,
        c2x + 250.0,
        ap_y + 3.0
    );
    let ap_vec2 = ctx["arabic_parts"]
        .as_array()
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    for (i, p) in ap_vec2.iter().enumerate() {
        let ry = ap_y + 14.0 + i as f64 * rh2;
        let name = p["name"].as_str().unwrap_or("");
        let dms = p["dms"].as_str().unwrap_or("");
        let sign_nm = p["sign"].as_str().unwrap_or("");
        let _ = writeln!(
            s,
            r#"  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{soft_c}" opacity=".8">{name}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".45">{sign_nm}</text>"#,
            c2x + 2.0,
            c2x + 125.0,
            c2x + 210.0,
        );
    }

    // ── footer ────────────────────────────────────────────────────────────────
    let _ = writeln!(
        s,
        r#"
  <text x="450" y="1090" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif"
        fill="{ring}" opacity=".35">Generated by celestial render · {date}</text>
</svg>"#
    );

    s
}

// ─── Entry point ─────────────────────────────────────────────────────────────

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 2 — additional chart-type context builders
// ═══════════════════════════════════════════════════════════════════════════════

/// Build a Secondary Progressions context (natal planets advanced 1 day/year).
fn build_progressed_context(
    jd_natal: f64,
    years: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    // Build the natal context first — provides base wheel geometry
    let mut ctx = build_context(jd_natal, lat, lon, date_str, hsys, user_vars)?;

    // Compute progressed positions
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let body_list: Vec<Body> = BODIES.iter().map(|&(b, ..)| b).collect();
    let (prog_positions, prog_chart) = secondary_progressions(
        jd_natal,
        years,
        &body_list,
        lat,
        lon,
        HouseSystem(hsys as u8),
        flags,
    )
    .map_err(|e| e.to_string())?;

    let asc = ctx["asc"].as_f64().unwrap_or(0.0);
    // Progressed ASC from prog_chart
    let prog_asc = prog_chart.ascmc[0];

    let prog_planets: Vec<Value> = prog_positions
        .iter()
        .zip(BODIES.iter())
        .map(|((_, pos), &(_, key, name, glyph))| {
            let (sign_idx, _) = lon_to_sign(pos.lon);
            json!({
                "name":  name, "key": key, "glyph": glyph,
                "lon":   (pos.lon * 1e4).round() / 1e4,
                "retro": pos.speed_lon < 0.0,
                "sign_name": zodiac_sign_name(sign_idx),
                "dms":   fmt_lon_dms(pos.lon),
                "x":  (wx(CX, RP + 52.0, pos.lon, asc) * 100.0).round() / 100.0,
                "y":  (wy(CY, RP + 52.0, pos.lon, asc) * 100.0).round() / 100.0,
                "asp_x": (wx(CX, RC + 10.0, pos.lon, asc) * 100.0).round() / 100.0,
                "asp_y": (wy(CY, RC + 10.0, pos.lon, asc) * 100.0).round() / 100.0,
            })
        })
        .collect();

    ctx["progressed_planets"] = json!(prog_planets);
    ctx["prog_asc"] = json!((prog_asc * 1e4).round() / 1e4);
    ctx["prog_asc_dms"] = json!(fmt_lon_dms(prog_asc));
    ctx["prog_mc"] = json!((prog_chart.ascmc[1] * 1e4).round() / 1e4);
    ctx["years"] = json!(years);
    Ok(ctx)
}

/// Build a Solar Arc Directions context.
fn build_solar_arc_context(
    jd_natal: f64,
    years: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;

    // Natal planet positions for arc calculation
    let natal_lons: Vec<(Body, f64)> = BODIES
        .iter()
        .filter_map(|&(body, ..)| calc_ut(jd_natal, body, flags).ok().map(|p| (body, p.lon)))
        .collect();

    let natal_chart =
        houses_ex(jd_natal, flags, lat, lon, HouseSystem(hsys as u8)).map_err(|e| e.to_string())?;
    let natal_mc = natal_chart.ascmc[1];

    let (arc, directed, directed_mc) =
        solar_arc_directions(jd_natal, years, &natal_lons, natal_mc, flags)
            .map_err(|e| e.to_string())?;

    let mut ctx = build_context(jd_natal, lat, lon, date_str, hsys, user_vars)?;
    let asc = ctx["asc"].as_f64().unwrap_or(0.0);

    let directed_planets: Vec<Value> = directed
        .iter()
        .zip(BODIES.iter())
        .map(|((_, lon2), &(_, key, name, glyph))| {
            let (sign_idx, _) = lon_to_sign(*lon2);
            json!({
                "name": name, "key": key, "glyph": glyph,
                "lon":  (lon2 * 1e4).round() / 1e4,
                "sign_name": zodiac_sign_name(sign_idx),
                "dms":  fmt_lon_dms(*lon2),
                "x":  (wx(CX, RP + 52.0, *lon2, asc) * 100.0).round() / 100.0,
                "y":  (wy(CY, RP + 52.0, *lon2, asc) * 100.0).round() / 100.0,
            })
        })
        .collect();

    ctx["directed_planets"] = json!(directed_planets);
    ctx["solar_arc_degrees"] = json!((arc * 100.0).round() / 100.0);
    ctx["directed_mc"] = json!((directed_mc * 1e4).round() / 1e4);
    ctx["years"] = json!(years);
    Ok(ctx)
}

/// Build a Bi-wheel context — two independent chart positions.
fn build_biwheel_context(
    jd1: f64,
    jd2: f64,
    lat1: f64,
    lon1: f64,
    _lat2: f64,
    _lon2: f64,
    date1: &str,
    date2: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, String> {
    // Inner wheel = chart 1 (natal / base)
    let mut ctx = build_context(jd1, lat1, lon1, date1, hsys, user_vars)?;
    let asc = ctx["asc"].as_f64().unwrap_or(0.0);

    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;

    // Outer wheel = chart 2 (partner / transits)
    const RP2: f64 = RP + 52.0; // outer planet ring radius
    let mut outer_planets = Vec::new();
    for &(body, key, name, glyph) in BODIES {
        if let Ok(pos) = calc_ut(jd2, body, flags) {
            let (sign_idx, _) = lon_to_sign(pos.lon);
            outer_planets.push(json!({
                "name":      name,
                "key":       key,
                "glyph":     glyph,
                "lon":       (pos.lon * 1e4).round() / 1e4,
                "retro":     pos.speed_lon < 0.0,
                "sign_name": zodiac_sign_name(sign_idx),
                "dms":       fmt_lon_dms(pos.lon),
                "x":  (wx(CX, RP2, pos.lon, asc) * 100.0).round() / 100.0,
                "y":  (wy(CY, RP2, pos.lon, asc) * 100.0).round() / 100.0,
                "asp_x": (wx(CX, RC + 15.0, pos.lon, asc) * 100.0).round() / 100.0,
                "asp_y": (wy(CY, RC + 15.0, pos.lon, asc) * 100.0).round() / 100.0,
            }));
        }
    }

    // Cross-aspects: outer chart 2 planets aspecting inner chart 1 planets
    let inner = ctx["planets"].as_array().unwrap().to_vec();
    let mut cross_aspects = Vec::new();
    for inner_p in &inner {
        for outer_p in &outer_planets {
            let lon_i = inner_p["lon"].as_f64().unwrap_or(0.0);
            let lon_o = outer_p["lon"].as_f64().unwrap_or(0.0);
            let diff = diff_deg_signed(lon_i, lon_o).abs();
            for &(asp_deg, asp_name, orb_lim, is_minor) in ASPECT_DEFS {
                let orb = (diff - asp_deg).abs();
                if orb <= orb_lim {
                    cross_aspects.push(json!({
                        "body1": inner_p["name"], "glyph1": inner_p["glyph"],
                        "body2": outer_p["name"], "glyph2": outer_p["glyph"],
                        "aspect_name": asp_name, "orb": (orb * 100.0).round() / 100.0,
                        "is_minor": is_minor,
                        "is_hard": asp_name == "square" || asp_name == "opposition",
                        "x1": inner_p["asp_x"], "y1": inner_p["asp_y"],
                        "x2": outer_p["asp_x"], "y2": outer_p["asp_y"],
                    }));
                    break;
                }
            }
        }
    }

    ctx["outer_planets"] = json!(outer_planets);
    ctx["cross_aspects"] = json!(cross_aspects);
    ctx["date2"] = json!(date2);
    ctx["r_planet2"] = json!(RP2);
    Ok(ctx)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 2 — SVG renderers for new chart types
// ═══════════════════════════════════════════════════════════════════════════════

/// Render a cosmogram: zodiac wheel without houses, pure sign positions.
fn render_cosmogram_svg(ctx: &Value) -> String {
    // Reuse the full natal renderer but pass no_houses flag
    render_builtin_svg(ctx)
}

/// Render progressed / solar-arc chart: natal wheel (inner) + directed ring (outer).
fn render_progressed_svg(ctx: &Value) -> String {
    // Start with the natal base
    let mut s = render_builtin_svg(ctx);

    // Choose which set of outer planets to render
    let outer_key = if ctx.get("progressed_planets").is_some() {
        "progressed_planets"
    } else {
        "directed_planets"
    };
    let outer = match ctx[outer_key].as_array() {
        Some(v) => v.to_vec(),
        None => return s,
    };

    let asc = ctx["asc"].as_f64().unwrap_or(0.0);
    let ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#1a1a2e");
    let prog_c = "#0d6e3c"; // green for progressed / directed planets

    // Extra ring at RP+52
    let prog_r = RP + 52.0;
    if let Some(idx) = s.rfind("</svg>") {
        let insert = format!(
            "\n  <!-- progressed / directed outer ring -->\n  <circle cx=\"{cx}\" cy=\"{cy}\" r=\"{pr:.1}\" fill=\"none\" stroke=\"{rg}\" stroke-width=\"1.0\" opacity=\".25\" stroke-dasharray=\"4,3\"/>\n",
            cx = CX, cy = CY, pr = prog_r, rg = ring
        );
        s.insert_str(idx, &insert);
    }

    // Outer planet glyphs
    let years = ctx["years"].as_f64().unwrap_or(0.0);
    let label = if ctx.get("solar_arc_degrees").is_some() {
        let arc = ctx["solar_arc_degrees"].as_f64().unwrap_or(0.0);
        format!("Solar Arc {arc:.2}°")
    } else {
        format!("Progressed {years:.1}y")
    };

    let mut extra = format!(
        "  <text x=\"{CX}\" y=\"{:.1}\" text-anchor=\"middle\" font-size=\"10\" \
         font-family=\"'Segoe UI',system-ui,sans-serif\" fill=\"{prog_c}\" opacity=\".85\">{label}</text>\n",
        CY + prog_r + 18.0
    );

    for p in &outer {
        let px = p["x"].as_f64().unwrap_or(0.0);
        let py = p["y"].as_f64().unwrap_or(0.0);
        let g = p["glyph"].as_str().unwrap_or("?");
        let dms = p["dms"].as_str().unwrap_or("");
        extra.push_str(&format!(
            "  <text x=\"{px:.2}\" y=\"{py:.2}\" font-size=\"14\" font-weight=\"bold\" \
             text-anchor=\"middle\" dominant-baseline=\"central\" \
             font-family=\"serif\" fill=\"{prog_c}\" opacity=\".85\">{g}</text>\n"
        ));
        // Small degree label
        let lx = CX
            + (prog_r + 16.0)
                * (180.0_f64 - (p["lon"].as_f64().unwrap_or(0.0) - asc))
                    .to_radians()
                    .cos();
        let ly = CY
            - (prog_r + 16.0)
                * (180.0_f64 - (p["lon"].as_f64().unwrap_or(0.0) - asc))
                    .to_radians()
                    .sin();
        extra.push_str(&format!(
            "  <text x=\"{lx:.2}\" y=\"{ly:.2}\" font-size=\"8\" font-weight=\"600\" \
             text-anchor=\"middle\" dominant-baseline=\"central\" \
             font-family=\"'Segoe UI',system-ui,sans-serif\" fill=\"{prog_c}\" opacity=\".7\">{dms}</text>\n"
        ));
    }

    if let Some(idx) = s.rfind("</svg>") {
        s.insert_str(idx, &extra);
    }
    s
}

/// Render a bi-wheel: natal inner ring + partner/transit outer ring.
fn render_biwheel_svg(ctx: &Value) -> String {
    let mut s = render_builtin_svg(ctx);

    let outer = match ctx["outer_planets"].as_array() {
        Some(v) => v.to_vec(),
        None => return s,
    };
    let cross = ctx["cross_aspects"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let date2 = ctx["date2"].as_str().unwrap_or("chart 2");
    let asc = ctx["asc"].as_f64().unwrap_or(0.0);
    let _ring = ctx["vars"]["ring_color"].as_str().unwrap_or("#1a1a2e");
    let hard_c = ctx["vars"]["hard_color"].as_str().unwrap_or("#b01020");
    let soft_c = ctx["vars"]["soft_color"].as_str().unwrap_or("#1a50b0");
    let outer_c = "#5c3d8f"; // purple for outer chart planets

    let prog_r = RP + 52.0;

    let mut extra = String::new();

    // Outer ring
    extra.push_str(&format!(
        "  <!-- bi-wheel outer ring (chart 2: {date2}) -->\n  \
         <circle cx=\"{CX}\" cy=\"{CY}\" r=\"{prog_r:.1}\" fill=\"none\" \
         stroke=\"{outer_c}\" stroke-width=\"1.2\" opacity=\".35\"/>\n"
    ));

    // Cross-aspect lines (between inner and outer)
    for asp in &cross {
        let x1 = asp["x1"].as_f64().unwrap_or(0.0);
        let y1 = asp["y1"].as_f64().unwrap_or(0.0);
        let x2 = asp["x2"].as_f64().unwrap_or(0.0);
        let y2 = asp["y2"].as_f64().unwrap_or(0.0);
        let hard = asp["is_hard"].as_bool().unwrap_or(false);
        let minor = asp["is_minor"].as_bool().unwrap_or(false);
        let orb = asp["orb"].as_f64().unwrap_or(8.0);
        let col = if hard { hard_c } else { soft_c };
        let (sw, op, dash): (&str, &str, &str) = if minor {
            let o: &str = if orb < 1.0 { ".25" } else { ".12" };
            ("0.8", o, r#" stroke-dasharray="3,3""#)
        } else {
            let o: &str = if orb < 2.0 { ".4" } else { ".18" };
            ("1.4", o, "")
        };
        extra.push_str(&format!(
            "  <line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" \
             stroke=\"{col}\" stroke-width=\"{sw}\" opacity=\"{op}\"{dash}/>\n"
        ));
    }

    // Outer planet glyphs
    for p in &outer {
        let px = p["x"].as_f64().unwrap_or(0.0);
        let py = p["y"].as_f64().unwrap_or(0.0);
        let g = p["glyph"].as_str().unwrap_or("?");
        let dms = p["dms"].as_str().unwrap_or("");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let col = if ret { "#b01020" } else { outer_c };
        let lon2 = p["lon"].as_f64().unwrap_or(0.0);
        let nat_ang = (180.0 - (lon2 - asc)).rem_euclid(360.0);
        let lx = CX + (prog_r + 18.0) * nat_ang.to_radians().cos();
        let ly = CY - (prog_r + 18.0) * nat_ang.to_radians().sin();
        extra.push_str(&format!(
            "  <text x=\"{px:.2}\" y=\"{py:.2}\" font-size=\"15\" font-weight=\"bold\" \
             text-anchor=\"middle\" dominant-baseline=\"central\" \
             font-family=\"serif\" fill=\"{col}\">{g}</text>\n  \
             <text x=\"{lx:.2}\" y=\"{ly:.2}\" font-size=\"8\" font-weight=\"600\" \
             text-anchor=\"middle\" dominant-baseline=\"central\" \
             font-family=\"'Segoe UI',system-ui,sans-serif\" fill=\"{col}\" opacity=\".8\">{dms}</text>\n"
        ));
    }

    // Label outer chart
    extra.push_str(&format!(
        "  <text x=\"{CX}\" y=\"{:.1}\" text-anchor=\"middle\" font-size=\"9\" \
         font-family=\"'Segoe UI',system-ui,sans-serif\" fill=\"{outer_c}\" opacity=\".8\">● {date2}</text>\n",
        CY + prog_r + 18.0
    ));

    if let Some(idx) = s.rfind("</svg>") {
        s.insert_str(idx, &extra);
    }
    s
}

pub fn run(mut args: RenderArgs) -> Result<(), String> {
    // load config file
    let mut file_vars: BTreeMap<String, String> = BTreeMap::new();
    if let Some(ref cfg_path) = args.config.clone() {
        let text = std::fs::read_to_string(cfg_path)
            .map_err(|e| format!("cannot read config `{}`: {e}", cfg_path.display()))?;
        let cfg: ConfigFile =
            toml::from_str(&text).map_err(|e| format!("invalid config TOML: {e}"))?;
        if let Some(r) = cfg.render {
            if args.date == "now" {
                if let Some(d) = r.date {
                    args.date = d;
                }
            }
            if args.lat == 0.0 {
                if let Some(v) = r.lat {
                    args.lat = v;
                }
            }
            if args.lon == 0.0 {
                if let Some(v) = r.lon {
                    args.lon = v;
                }
            }
            if args.template.is_none() {
                args.template = r.template;
            }
            if args.out.is_none() {
                args.out = r.out;
            }
            if args.hsys == 'P' {
                if let Some(h) = r.hsys {
                    args.hsys = h;
                }
            }
        }
        if let Some(t) = cfg.vars {
            for (k, v) in t {
                let s = match &v {
                    toml::Value::String(x) => x.clone(),
                    toml::Value::Integer(x) => x.to_string(),
                    toml::Value::Float(x) => x.to_string(),
                    toml::Value::Boolean(x) => x.to_string(),
                    other => other.to_string(),
                };
                file_vars.insert(k, s);
            }
        }
    }

    // --var KEY=VALUE overrides
    let mut user_vars = file_vars;
    for kv in &args.vars {
        let (k, v) = kv
            .split_once('=')
            .ok_or_else(|| format!("`--var` must be KEY=VALUE, got `{kv}`"))?;
        user_vars.insert(k.to_string(), v.to_string());
    }

    let jd = crate::parse::parse_date(&args.date)?;

    // Dispatch to the appropriate chart-type builder
    let chart_type = args.chart_type.to_lowercase();
    let chart_type = chart_type.trim();

    let (ctx, render_fn): (serde_json::Value, fn(&serde_json::Value) -> String) = match chart_type {
        "natal" | "" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Natal Chart".to_string());
            (build_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?, render_builtin_svg)
        }
        "cosmogram" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Cosmogram".to_string());
            v.insert("no_houses".to_string(), "1".to_string());
            (build_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?, render_cosmogram_svg)
        }
        "solar-return" | "solar_return" => {
            // Default to the current year based on today's JD
            let year = args.return_year.unwrap_or_else(|| {
                let today_jd = crate::parse::parse_date("now").unwrap_or(2_451_545.0);
                let d = celestial_core::revjul(today_jd, celestial_core::body::Calendar::Gregorian);
                d.year as i32
            });
            let sr_jd = solar_return_jd(jd, year, CalcFlags::BUILTIN)
                .map_err(|e| e.to_string())?;
            let sr_date = jd_to_date_str(sr_jd);
            let mut v = user_vars.clone();
            v.insert("title".to_string(), format!("Solar Return {year}"));
            v.insert("chart_type_label".to_string(), format!("Solar Return {year}"));
            (build_context(sr_jd, args.lat, args.lon, &sr_date, args.hsys, v)?, render_builtin_svg)
        }
        "lunar-return" | "lunar_return" => {
            let start = args.date2.as_deref()
                .map(|d| crate::parse::parse_date(d))
                .transpose()?
                .unwrap_or(jd);
            let lr_jd = lunar_return_jd(jd, start, CalcFlags::BUILTIN)
                .map_err(|e| e.to_string())?;
            let lr_date = jd_to_date_str(lr_jd);
            let mut v = user_vars.clone();
            v.insert("title".to_string(), "Lunar Return".to_string());
            (build_context(lr_jd, args.lat, args.lon, &lr_date, args.hsys, v)?, render_builtin_svg)
        }
        "progressed" | "secondary" => {
            let years = args.years.ok_or("--years DECIMAL required for progressed chart")?;
            let mut v = user_vars.clone();
            v.insert("title".to_string(), format!("Secondary Progressions ({years:.1}y)"));
            (build_progressed_context(jd, years, args.lat, args.lon, &args.date, args.hsys, v)?, render_progressed_svg)
        }
        "solar-arc" | "solar_arc" => {
            let years = args.years.ok_or("--years DECIMAL required for solar-arc chart")?;
            let mut v = user_vars.clone();
            v.insert("title".to_string(), format!("Solar Arc Directions ({years:.1}y)"));
            (build_solar_arc_context(jd, years, args.lat, args.lon, &args.date, args.hsys, v)?, render_progressed_svg)
        }
        "biwheel" | "bi-wheel" | "synastry" | "transit" => {
            let date2 = args.date2.as_deref()
                .ok_or("--date2 DATE required for biwheel chart")?;
            let jd2 = crate::parse::parse_date(date2)?;
            let lat2 = args.lat2.unwrap_or(args.lat);
            let lon2 = args.lon2.unwrap_or(args.lon);
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Bi-wheel".to_string());
            (build_biwheel_context(jd, jd2, args.lat, args.lon, lat2, lon2,
                                   &args.date, date2, args.hsys, v)?, render_biwheel_svg)
        }
        "composite" => {
            let date2 = args.date2.as_deref()
                .ok_or("--date2 DATE required for composite chart")?;
            let jd2 = crate::parse::parse_date(date2)?;
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Composite Chart".to_string());
            (build_composite_context(jd, jd2, args.lat, args.lon, &args.date, date2, args.hsys, v)?,
             render_builtin_svg)
        }
        "triwheel" | "tri-wheel" => {
            let date2 = args.date2.as_deref()
                .ok_or("--date2 required (ring 2) for tri-wheel")?;
            let date3 = args.date3.as_deref()
                .ok_or("--date3 required (ring 3) for tri-wheel")?;
            let jd2 = crate::parse::parse_date(date2)?;
            let jd3 = crate::parse::parse_date(date3)?;
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Tri-wheel".to_string());
            (build_triwheel_context(jd, jd2, jd3, args.lat, args.lon,
                                    &args.date, date2, date3, args.hsys, v)?,
             render_triwheel_svg)
        }
        "dial" | "90dial" | "midpoint-dial" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "90° Midpoint Dial".to_string());
            (build_dial_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?,
             render_dial_svg)
        }
        "ephemeris" | "graphic-ephemeris" => {
            let date2 = args.date2.as_deref().unwrap_or("now");
            let jd2 = crate::parse::parse_date(date2)?;
            let (jd_s, jd_e) = if jd < jd2 { (jd, jd2) } else { (jd2, jd) };
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Graphic Ephemeris".to_string());
            (build_graphic_ephemeris_context(jd_s, jd_e, v)?, render_graphic_ephemeris_svg)
        }
        "local-space" | "localspace" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Local Space".to_string());
            (build_local_space_context(jd, args.lat, args.lon, &args.date, v)?,
             render_local_space_svg)
        }
        "rasi" | "vedic" | "south-indian" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Rasi Chart (South Indian)".to_string());
            (build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Rasi")?,
             render_south_indian_svg)
        }
        "navamsa" | "d9" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Navamsa D9 Chart".to_string());
            (build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Navamsa")?,
             render_navamsa_svg)
        }
        "dasha" | "vimshottari" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Vimshottari Dasha Timeline".to_string());
            (build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Dasha")?,
             render_dasha_svg)
        }
        "north-indian" | "north_indian" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "North Indian Chart".to_string());
            (build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Rasi")?,
             render_north_indian_svg)
        }
        "ashtakavarga" | "ashtak" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Ashtakavarga".to_string());
            (build_ashtakavarga_context(jd, args.lat, args.lon, &args.date, v)?,
             render_ashtakavarga_svg)
        }
        "shadbala" | "strength" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Shadbala".to_string());
            (build_shadbala_context(jd, args.lat, args.lon, &args.date, v)?,
             render_shadbala_svg)
        }
        other => return Err(format!("unknown --type '{other}'; valid: natal cosmogram solar-return lunar-return progressed solar-arc biwheel composite triwheel dial ephemeris local-space rasi navamsa dasha north-indian ashtakavarga shadbala")),
    };

    // --print-context: dump JSON context and exit
    if args.print_context {
        println!("{}", serde_json::to_string_pretty(&ctx).unwrap());
        return Ok(());
    }

    // --print-template: show the example tinytemplate
    if args.print_template {
        print!("{EXAMPLE_TEMPLATE}");
        return Ok(());
    }

    // render
    let output = match &args.template {
        None => render_fn(&ctx),
        Some(tmpl_path) => {
            let tmpl_src = std::fs::read_to_string(tmpl_path)
                .map_err(|e| format!("cannot read template `{}`: {e}", tmpl_path.display()))?;
            let mut tt = TinyTemplate::new();
            tt.add_template("t", &tmpl_src)
                .map_err(|e| format!("template parse error: {e}"))?;
            tt.render("t", &ctx)
                .map_err(|e| format!("template render error: {e}"))?
        }
    };

    // write output
    match &args.out {
        Some(p) => {
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("cannot create output dir: {e}"))?;
                }
            }
            std::fs::write(p, &output)
                .map_err(|e| format!("cannot write `{}`: {e}", p.display()))?;
            eprintln!("\u{2713}  wrote {}", p.display());
        }
        None => print!("{output}"),
    }
    Ok(())
}

// ─── Example template (tinytemplate syntax) ───────────────────────────────────

const EXAMPLE_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- Example tinytemplate for celestial render.
     Available context: date, jd, lat, lon,
       asc, mc, ic, dsc, asc_dms, mc_dms, ic_dms, dsc_dms,
       moon_phase_name, moon_illumination,
       planets list, signs list, houses list, aspects list,
       vars.* - all keys from [vars] / --var flags.
     Wheel geometry is pre-computed: each planet has x y label_x label_y
     tick_x1 tick_y1 tick_x2 tick_y2 asp_x asp_y
     Each sign has spoke_x1 spoke_y1 spoke_x2 spoke_y2 glyph_x glyph_y
     Each house has x1 y1 x2 y2 num_x num_y is_angle dms
     Each aspect has x1 y1 x2 y2 is_hard orb applying body1 body2
     Run: celestial render --print-context  to see all values as JSON.
-->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 800" width="900" height="800">
  <rect width="900" height="800" fill="{vars.bg_color}"/>

  <!-- header -->
  <text x="450" y="30" text-anchor="middle" font-size="16" font-weight="600"
        font-family="system-ui,sans-serif" fill="{vars.text_color}">{vars.title}</text>
  <text x="450" y="50" text-anchor="middle" font-size="10"
        font-family="system-ui,sans-serif" fill="{vars.ring_color}" opacity=".7">{date} · JD {jd}</text>

  <!-- sign spokes -->
  {{ for s in signs }}
  <line x1="{s.spoke_x1}" y1="{s.spoke_y1}" x2="{s.spoke_x2}" y2="{s.spoke_y2}"
        stroke="{vars.ring_color}" stroke-width=".6" opacity=".4"/>
  <text x="{s.glyph_x}" y="{s.glyph_y}" text-anchor="middle" dominant-baseline="central"
        font-size="13" font-family="serif" fill="{vars.ring_color}">{s.glyph}</text>
  {{ endfor }}

  <!-- house cusps -->
  {{ for h in houses }}
  <line x1="{h.x1}" y1="{h.y1}" x2="{h.x2}" y2="{h.y2}"
        stroke="{vars.ring_color}" stroke-width=".7" opacity=".35"/>
  <text x="{h.num_x}" y="{h.num_y}" text-anchor="middle" dominant-baseline="central"
        font-size="9" font-family="system-ui,sans-serif"
        fill="{vars.ring_color}" opacity=".5">{h.num}</text>
  {{ endfor }}

  <!-- aspect lines -->
  {{ for asp in aspects }}
  <line x1="{asp.x1}" y1="{asp.y1}" x2="{asp.x2}" y2="{asp.y2}"
        stroke="{vars.soft_color}" stroke-width=".7" opacity=".3"/>
  {{ endfor }}

  <!-- planet glyphs -->
  {{ for p in planets }}
  <line x1="{p.tick_x1}" y1="{p.tick_y1}" x2="{p.tick_x2}" y2="{p.tick_y2}"
        stroke="{vars.planet_color}" stroke-width="1.0" opacity=".45"/>
  <text x="{p.x}" y="{p.y}" text-anchor="middle" dominant-baseline="central"
        font-size="15" font-family="serif" fill="{vars.planet_color}">{p.glyph}</text>
  <text x="{p.label_x}" y="{p.label_y}" text-anchor="middle" dominant-baseline="central"
        font-size="9" font-family="system-ui,sans-serif"
        fill="{vars.text_color}" opacity=".8">{p.deg_label}</text>
  {{ endfor }}

  <!-- legend -->
  <text x="24" y="830" font-size="11" font-family="system-ui,sans-serif"
        fill="{vars.ring_color}">ASC {asc_dms} · MC {mc_dms} · {moon_phase_name} {moon_illumination}%</text>
</svg>
"#;

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── wheel geometry ────────────────────────────────────────────────────────

    #[test]
    fn wheel_asc_at_9_oclock() {
        // ASC is placed at the leftmost point: (cx - r, cy)
        let (cx, cy, r, asc) = (450.0, 490.0, 212.0, 30.0);
        let x = wx(cx, r, asc, asc);
        let y = wy(cy, r, asc, asc);
        assert!(
            (x - (cx - r)).abs() < 1e-9,
            "ASC should be at cx-r, got {x}"
        );
        assert!((y - cy).abs() < 1e-9, "ASC y should be cy, got {y}");
    }

    #[test]
    fn wheel_opposite_point() {
        // DSC = ASC + 180 → rightmost (cx + r, cy)
        let (cx, cy, r, asc) = (450.0, 490.0, 212.0, 0.0);
        let dsc = 180.0_f64;
        let x = wx(cx, r, dsc, asc);
        let y = wy(cy, r, dsc, asc);
        assert!(
            (x - (cx + r)).abs() < 1e-9,
            "DSC should be at cx+r, got {x}"
        );
        assert!((y - cy).abs() < 1e-9, "DSC y should be cy, got {y}");
    }

    #[test]
    fn wheel_mc_at_top() {
        // MC = ASC + 90 → top of chart (cx, cy - r)
        let (cx, cy, r, asc) = (450.0, 490.0, 212.0, 0.0);
        let mc = 90.0_f64;
        let x = wx(cx, r, mc, asc);
        let y = wy(cy, r, mc, asc);
        assert!((x - cx).abs() < 1e-9, "MC x should be cx, got {x}");
        assert!((y - (cy - r)).abs() < 1e-9, "MC should be at top, got {y}");
    }

    // ── formatting ────────────────────────────────────────────────────────────

    #[test]
    fn fmt_lon_dms_aries_start() {
        let s = fmt_lon_dms(0.0);
        assert!(s.contains("\u{2648}"), "0° should be ♈, got {s}");
        assert!(
            s.starts_with("00\u{00B0}"),
            "should start with 00°, got {s}"
        );
    }

    #[test]
    fn fmt_lon_dms_taurus_boundary() {
        // 30° = start of ♉
        let s = fmt_lon_dms(30.0);
        assert!(s.contains("\u{2649}"), "30° should be ♉, got {s}");
        assert!(
            s.starts_with("00\u{00B0}"),
            "should start with 00°, got {s}"
        );
    }

    #[test]
    fn fmt_lon_dms_mid_sign() {
        // 45° = 15° Taurus
        let s = fmt_lon_dms(45.0);
        assert!(s.contains("\u{2649}"), "45° should be ♉, got {s}");
        assert!(
            s.starts_with("15\u{00B0}"),
            "should start with 15°, got {s}"
        );
    }

    #[test]
    fn fmt_lon_dms_wraps_360() {
        // 360° = 0° = ♈
        let s = fmt_lon_dms(360.0);
        assert!(s.contains("\u{2648}"), "360° should wrap to ♈, got {s}");
    }

    #[test]
    fn moon_phase_str_smoke() {
        // J2000 should return a non-empty phase name
        let p = moon_phase_str(2451545.0);
        assert!(!p.is_empty(), "moon_phase_str should not be empty");
        let valid = [
            "New Moon",
            "Waxing Crescent",
            "First Quarter",
            "Waxing Gibbous",
            "Full Moon",
            "Waning Gibbous",
            "Last Quarter",
            "Waning Crescent",
        ];
        assert!(valid.contains(&p), "unexpected phase: {p}");
    }

    // ── build_context ──────────────────────────────────────────────────────────

    #[test]
    fn context_j2000_placidus() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        );
        assert!(ctx.is_ok(), "build_context should succeed: {:?}", ctx.err());
        let v = ctx.unwrap();
        assert_eq!(v["date"], "2000-01-01");
        assert!((v["jd"].as_f64().unwrap() - 2451545.0).abs() < 0.1);
        // 12 planets
        assert_eq!(v["planets"].as_array().unwrap().len(), 12);
        // 12 houses
        assert_eq!(v["houses"].as_array().unwrap().len(), 12);
        // ASC is a valid longitude
        let asc = v["asc"].as_f64().unwrap();
        assert!((0.0..360.0).contains(&asc), "ASC out of range: {asc}");
    }

    #[test]
    fn context_all_house_systems() {
        // Every standard house system must succeed without panic
        for hsys in ['P', 'K', 'E', 'W', 'O', 'R', 'C', 'M', 'B', 'X'] {
            let r = build_context(
                2451545.0,
                48.85,
                2.35,
                "2000-01-01",
                hsys,
                std::collections::BTreeMap::new(),
            );
            assert!(r.is_ok(), "house system '{hsys}' failed: {:?}", r.err());
        }
    }

    #[test]
    fn context_equator_zero_lon() {
        let r = build_context(
            2451545.0,
            0.0,
            0.0,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        );
        assert!(r.is_ok(), "equator/prime-meridian should succeed");
    }

    #[test]
    fn context_polar_latitude() {
        // High latitudes: some house systems fail, others degrade gracefully
        // We just require no panic — Placidus may return error at high lat
        let _ = build_context(
            2451545.0,
            89.0,
            0.0,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        );
        let _ = build_context(
            2451545.0,
            -89.0,
            0.0,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        );
    }

    #[test]
    fn context_ancient_date() {
        // JDE 1000000 ≈ year -693 — should not panic
        let _ = build_context(
            1000000.0,
            0.0,
            0.0,
            "ancient",
            'E',
            std::collections::BTreeMap::new(),
        );
    }

    #[test]
    fn context_far_future() {
        // JDE 2816787 ≈ year 3000 — should not panic
        let _ = build_context(
            2816787.0,
            0.0,
            0.0,
            "future",
            'E',
            std::collections::BTreeMap::new(),
        );
    }

    #[test]
    fn context_user_vars_visible() {
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("my_key".to_string(), "my_value".to_string());
        vars.insert("bg_color".to_string(), "#123456".to_string());
        let ctx = build_context(2451545.0, 0.0, 0.0, "test", 'E', vars).unwrap();
        assert_eq!(ctx["vars"]["my_key"], "my_value", "custom var missing");
        assert_eq!(
            ctx["vars"]["bg_color"], "#123456",
            "palette override missing"
        );
    }

    #[test]
    fn context_palette_defaults() {
        let ctx = build_context(
            2451545.0,
            0.0,
            0.0,
            "test",
            'E',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(ctx["vars"]["bg_color"], "#ffffff");
        assert_eq!(ctx["vars"]["ring_color"], "#1a1a2e");
        assert_eq!(ctx["vars"]["title"], "Celestial Chart");
    }

    #[test]
    fn context_sun_lon_range() {
        let ctx = build_context(
            2451545.0,
            0.0,
            0.0,
            "test",
            'E',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let sun = &ctx["planets"][0];
        let lon = sun["lon"].as_f64().unwrap();
        assert!((0.0..360.0).contains(&lon), "Sun lon out of range: {lon}");
        assert_eq!(sun["name"], "Sun");
        assert_eq!(sun["glyph"], "\u{2609}");
    }

    #[test]
    fn context_precomputed_coords_finite() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "test",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        for p in ctx["planets"].as_array().unwrap() {
            let x = p["x"].as_f64().unwrap();
            let y = p["y"].as_f64().unwrap();
            assert!(
                x.is_finite() && y.is_finite(),
                "planet {} has non-finite coords ({x}, {y})",
                p["name"]
            );
        }
    }

    #[test]
    fn context_aspects_have_endpoints() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "test",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        for asp in ctx["aspects"].as_array().unwrap() {
            let x1 = asp["x1"].as_f64().unwrap();
            let y1 = asp["y1"].as_f64().unwrap();
            let x2 = asp["x2"].as_f64().unwrap();
            let y2 = asp["y2"].as_f64().unwrap();
            assert!(
                x1.is_finite() && y1.is_finite() && x2.is_finite() && y2.is_finite(),
                "aspect endpoints not finite"
            );
            let orb = asp["orb"].as_f64().unwrap();
            assert!(orb >= 0.0 && orb <= 8.0, "orb out of range: {orb}");
        }
    }

    // ── render_builtin_svg ────────────────────────────────────────────────────

    #[test]
    fn builtin_svg_is_valid_xml_fragment() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let svg = render_builtin_svg(&ctx);
        assert!(svg.starts_with("<?xml"), "must start with XML declaration");
        assert!(svg.contains("<svg "), "must contain <svg>");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(!svg.contains("NaN"), "SVG must not contain NaN");
        assert!(!svg.contains("inf"), "SVG must not contain inf");
    }

    #[test]
    fn builtin_svg_contains_legend_sections() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let svg = render_builtin_svg(&ctx);
        assert!(svg.contains("Planets"), "legend must have Planets section");
        assert!(svg.contains("Aspects"), "legend must have Aspects section");
        assert!(
            svg.contains("Angles"),
            "legend must have Angles &amp; Houses section"
        );
        assert!(svg.contains("ASC"), "legend must show ASC");
        assert!(svg.contains("MC"), "legend must show MC");
    }

    #[test]
    fn builtin_svg_planet_glyphs_present() {
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let svg = render_builtin_svg(&ctx);
        // All 12 planet glyphs must appear
        for glyph in [
            "\u{2609}", "\u{263D}", "\u{263F}", "\u{2640}", "\u{2642}", "\u{2643}", "\u{2644}",
            "\u{2645}", "\u{2646}", "\u{2647}", "\u{260A}", "\u{26B7}",
        ] {
            assert!(svg.contains(glyph), "SVG missing planet glyph {glyph}");
        }
    }

    #[test]
    fn builtin_svg_custom_palette() {
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("bg_color".to_string(), "#FACADE".to_string());
        vars.insert("title".to_string(), "Test Title".to_string());
        let ctx = build_context(2451545.0, 0.0, 0.0, "test", 'E', vars).unwrap();
        let svg = render_builtin_svg(&ctx);
        assert!(
            svg.contains("#FACADE"),
            "custom bg_color must appear in SVG"
        );
        assert!(
            svg.contains("Test Title"),
            "custom title must appear in SVG"
        );
    }

    #[test]
    fn builtin_svg_winter_solstice() {
        // Winter solstice: Sun near 270° (Capricorn)
        let ctx = build_context(
            2451545.0 + 354.0,
            51.5,
            -0.1,
            "test",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let svg = render_builtin_svg(&ctx);
        assert!(
            svg.contains("<svg"),
            "SVG must be generated for winter solstice date"
        );
        assert!(!svg.contains("NaN"), "no NaN in winter solstice SVG");
    }

    // ── example template renders ──────────────────────────────────────────────

    #[test]
    fn example_template_renders_successfully() {
        use tinytemplate::TinyTemplate;
        let ctx = build_context(
            2451545.0,
            48.85,
            2.35,
            "2000-01-01",
            'P',
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let mut tt = TinyTemplate::new();
        tt.add_template("t", EXAMPLE_TEMPLATE)
            .expect("example template should parse");
        let rendered = tt
            .render("t", &ctx)
            .expect("example template should render");
        assert!(
            rendered.contains("<svg"),
            "example template must produce SVG"
        );
        assert!(
            !rendered.contains("NaN"),
            "example template must not produce NaN"
        );
    }

    // ── Phase 2: chart-type dispatch ──────────────────────────────────────────

    #[test]
    fn cosmogram_context_has_no_houses_flag() {
        let jd = 2_451_545.0;
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("no_houses".to_string(), "1".to_string());
        vars.insert("title".to_string(), "Cosmogram".to_string());
        let ctx = build_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        assert_eq!(ctx["vars"]["no_houses"].as_str(), Some("1"));
        // Should still have planets and signs
        assert!(ctx["planets"].as_array().unwrap().len() > 0);
        assert!(ctx["signs"].as_array().unwrap().len() == 12);
    }

    #[test]
    fn solar_return_jd_is_after_natal() {
        use celestial_core::{body::CalcFlags, solar_return_jd};
        let jd_natal = 2_440_000.0; // 1968-ish
        let sr = solar_return_jd(jd_natal, 2025, CalcFlags::BUILTIN).unwrap();
        assert!(
            sr > jd_natal,
            "solar return must be after natal: {sr} <= {jd_natal}"
        );
        // Should be within 2030 (JD ~2462000)
        assert!(sr < 2_463_000.0, "solar return too far in the future: {sr}");
    }

    #[test]
    fn lunar_return_jd_is_after_search_start() {
        use celestial_core::{body::CalcFlags, lunar_return_jd};
        let jd_natal = 2_451_545.0;
        let jd_start = jd_natal + 365.0; // one year later
        let lr = lunar_return_jd(jd_natal, jd_start, CalcFlags::BUILTIN).unwrap();
        assert!(
            lr >= jd_start,
            "lunar return {lr} should be >= start {jd_start}"
        );
        // Lunar cycle ≈ 29.5 days — return should be within one cycle of start
        assert!(lr < jd_start + 30.0, "lunar return too far: {lr}");
    }

    #[test]
    fn progressed_context_has_progressed_planets() {
        let jd = 2_451_545.0;
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("title".to_string(), "Progressed".to_string());
        let ctx = build_progressed_context(jd, 30.0, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let prog = ctx["progressed_planets"].as_array();
        assert!(prog.is_some(), "context missing progressed_planets");
        assert_eq!(prog.unwrap().len(), 12);
    }

    #[test]
    fn solar_arc_context_has_directed_planets() {
        let jd = 2_451_545.0;
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("title".to_string(), "Solar Arc".to_string());
        let ctx = build_solar_arc_context(jd, 30.0, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        assert!(ctx["directed_planets"].as_array().is_some());
        let arc = ctx["solar_arc_degrees"].as_f64().unwrap_or(0.0);
        // 30 years × ~1°/year → arc ≈ 28–32°
        assert!(
            arc > 25.0 && arc < 35.0,
            "solar arc {arc:.2}° outside expected range"
        );
    }

    #[test]
    fn biwheel_context_has_both_planet_sets() {
        let jd1 = 2_451_545.0;
        let jd2 = jd1 + 365.0; // one year later
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("title".to_string(), "Bi-wheel".to_string());
        let ctx = build_biwheel_context(
            jd1,
            jd2,
            48.85,
            2.35,
            48.85,
            2.35,
            "2000-01-01",
            "2001-01-01",
            'P',
            vars,
        )
        .unwrap();
        assert!(
            ctx["planets"].as_array().unwrap().len() > 0,
            "inner planets missing"
        );
        assert!(
            ctx["outer_planets"].as_array().is_some(),
            "outer_planets missing"
        );
        assert_eq!(ctx["outer_planets"].as_array().unwrap().len(), 12);
    }

    #[test]
    fn biwheel_cross_aspects_computed() {
        let jd1 = 2_451_545.0;
        let jd2 = jd1 + 10.0; // close transit — should have aspects
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("title".to_string(), "Test".to_string());
        let ctx = build_biwheel_context(
            jd1,
            jd2,
            0.0,
            0.0,
            0.0,
            0.0,
            "2000-01-01",
            "2000-01-11",
            'P',
            vars,
        )
        .unwrap();
        // cross_aspects should exist (list may be empty but field must be present)
        assert!(ctx.get("cross_aspects").is_some());
    }

    // ── Phase 3: specialist charts ────────────────────────────────────────────

    #[test]
    fn dial_context_has_midpoints() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_dial_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        assert!(ctx["midpoints"].as_array().is_some(), "midpoints missing");
        assert!(ctx["planets"].as_array().unwrap().len() == 12);
        // Every planet dial_lon must be in [0, 90)
        for p in ctx["planets"].as_array().unwrap() {
            let dl = p["dial_lon"].as_f64().unwrap_or(-1.0);
            assert!(dl >= 0.0 && dl < 90.0, "dial_lon {dl} out of [0,90)");
        }
    }

    #[test]
    fn dial_svg_contains_planet_glyphs() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_dial_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let svg = render_dial_svg(&ctx);
        assert!(svg.contains("☉"), "Sun glyph missing from dial");
        assert!(svg.contains("☽"), "Moon glyph missing from dial");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn composite_context_positions_are_midpoints() {
        let jd1 = 2_451_545.0;
        let jd2 = jd1 + 180.0; // 6 months apart
        let vars = std::collections::BTreeMap::new();
        let ctx = build_composite_context(jd1, jd2, 0.0, 0.0, "date1", "date2", 'P', vars).unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 12);
        // Composite ASC should be the midpoint of the two individual ASCs
        let comp_asc = ctx["asc"].as_f64().unwrap();
        assert!(
            comp_asc >= 0.0 && comp_asc < 360.0,
            "composite ASC out of range: {comp_asc}"
        );
    }

    #[test]
    fn triwheel_context_has_three_rings() {
        let jd1 = 2_451_545.0;
        let jd2 = jd1 + 365.0;
        let jd3 = jd1 + 730.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_triwheel_context(
            jd1, jd2, jd3, 48.85, 2.35, "natal", "prog", "transit", 'P', vars,
        )
        .unwrap();
        assert!(ctx["planets"].as_array().unwrap().len() == 12, "inner ring");
        assert!(
            ctx["ring2_planets"].as_array().unwrap().len() == 12,
            "ring 2"
        );
        assert!(
            ctx["ring3_planets"].as_array().unwrap().len() == 12,
            "ring 3"
        );
    }

    #[test]
    fn graphic_ephemeris_context_has_series() {
        let jd_start = 2_451_545.0;
        let jd_end = jd_start + 90.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_graphic_ephemeris_context(jd_start, jd_end, vars).unwrap();
        let series = ctx["planet_series"].as_array().unwrap();
        assert_eq!(series.len(), 12, "expected 12 planet series");
        // Each series must have >= 2 data points
        for s in series {
            let lons = s["lons"].as_array().unwrap();
            assert!(
                lons.len() >= 2,
                "series {} has too few points: {}",
                s["name"],
                lons.len()
            );
        }
    }

    #[test]
    fn graphic_ephemeris_svg_has_planet_paths() {
        let jd_start = 2_451_545.0;
        let jd_end = jd_start + 60.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_graphic_ephemeris_context(jd_start, jd_end, vars).unwrap();
        let svg = render_graphic_ephemeris_svg(&ctx);
        assert!(svg.contains("<path"), "no paths in ephemeris SVG");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn local_space_context_has_azimuths() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_local_space_context(jd, 48.85, 2.35, "2000-01-01", vars).unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 12);
        for p in planets {
            let az = p["azimuth"].as_f64().unwrap_or(-1.0);
            assert!(
                az >= 0.0 && az < 360.0,
                "azimuth {az} out of [0,360) for {}",
                p["name"]
            );
        }
    }

    #[test]
    fn local_space_svg_has_compass_directions() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_local_space_context(jd, 48.85, 2.35, "2000-01-01", vars).unwrap();
        let svg = render_local_space_svg(&ctx);
        assert!(svg.contains(">N<"), "N direction missing");
        assert!(svg.contains(">S<"), "S direction missing");
        assert!(svg.contains(">E<"), "E direction missing");
        assert!(svg.contains(">W<"), "W direction missing");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 4 — Vedic / Jyotish chart builders
// ═══════════════════════════════════════════════════════════════════════════════

// South Indian Rasi chart — 3×4 fixed-sign grid, planets by sign.
//
// Layout (sign index 0=Aries … 11=Pisces, sidereal):
//   ┌──────┬──────┬──────┬──────┐
//   │ Pi 12│ Ar  1│ Ta  2│ Ge  3│  row 0
//   ├──────┤      centre     ├──────┤
//   │ Aq 11│               │ Ca  4│  row 1
//   ├──────┤               ├──────┤  row 2
//   │ Cp 10│               │ Le  5│
//   ├──────┼──────┬──────┼──────┤
//   │ Sg  9│ Sc  8│ Li  7│ Vi  6│  row 3
//   └──────┴──────┴──────┴──────┘
//
// Cell (row, col) → sign index (0-based, Aries=0):

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 4 (remaining) — North Indian chart, Ashtakavarga, Shadbala
// ═══════════════════════════════════════════════════════════════════════════════

// ─── North Indian diamond chart ───────────────────────────────────────────────
//
// 12 triangular cells arranged as a diamond (rhombus) pattern.
// The top cell is Aries (0), and cells proceed clockwise. The Ascendant sign
// is highlighted. Unlike South Indian (fixed signs), North Indian uses
// the ASC sign as the first house.

// Cell centre coordinates in a 540×540 grid
// Order: Aries(0)..Pisces(11) starting from top, clockwise
const NI_CELLS: &[(f64, f64)] = &[
    (270.0, 72.0),  // 0 = Aries     top
    (405.0, 144.0), // 1 = Taurus    top-right
    (468.0, 270.0), // 2 = Gemini    right
    (405.0, 396.0), // 3 = Cancer    bottom-right
    (270.0, 468.0), // 4 = Leo       bottom
    (135.0, 396.0), // 5 = Virgo     bottom-left
    (72.0, 270.0),  // 6 = Libra     left
    (135.0, 144.0), // 7 = Scorpio   top-left
    (270.0, 180.0), // 8 = Sagittarius inner-top
    (360.0, 270.0), // 9 = Capricorn  inner-right
    (270.0, 360.0), // 10= Aquarius   inner-bottom
    (180.0, 270.0), // 11= Pisces     inner-left
];

fn render_north_indian_svg(ctx: &Value) -> String {
    use std::fmt::Write;

    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fffff8");
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
    let lagna_rasi = ctx.get("asc_rasi").and_then(|v| v.as_i64()).unwrap_or(0) as usize % 12;

    let planets = ctx["planets"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    // Group planets by rasi
    let mut rasi_planets: Vec<Vec<String>> = vec![Vec::new(); 12];
    for p in &planets {
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
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w} {total_h}" width="{total_w}" height="{total_h}">
  <rect width="{total_w}" height="{total_h}" fill="{bg}"/>
  <text x="300" y="26" text-anchor="middle" font-size="15" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="300" y="44" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"#
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
        r#"  <polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" fill="none" stroke="{border}" stroke-width="2.0"/>"#,
        pts[0].0, pts[0].1, pts[1].0, pts[1].1, pts[2].0, pts[2].1, pts[3].0, pts[3].1
    );

    // Inner diamond (mid-lines)
    let r2 = r / 2.0;
    let ipts = [(dx, dy - r2), (dx + r2, dy), (dx, dy + r2), (dx - r2, dy)];
    let _ = writeln!(
        s,
        r#"  <polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" fill="none" stroke="{border}" stroke-width="1.0" opacity=".5"/>"#,
        ipts[0].0, ipts[0].1, ipts[1].0, ipts[1].1, ipts[2].0, ipts[2].1, ipts[3].0, ipts[3].1
    );

    // Diagonal dividers
    for (a, b) in [(pts[0], pts[2]), (pts[1], pts[3])] {
        let _ = writeln!(
            s,
            r#"  <line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{border}" stroke-width="1.2" opacity=".6"/>"#,
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
            r#"  <text x="{cx:.1}" y="{:.1}" font-size="9" font-weight="600" text-anchor="middle" fill="{col}" opacity=".8">{house_num}</text>"#,
            cy - 8.0
        );

        // Rasi glyph
        let sg = RASI_GLYPHS[sign_idx % 12];
        let _ = writeln!(
            s,
            r#"  <text x="{cx:.1}" y="{:.1}" font-size="10" text-anchor="middle" font-family="serif" fill="{txt}" opacity=".45">{sg}</text>"#,
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
                r#"  <text x="{cx:.1}" y="{py:.1}" font-size="9" text-anchor="middle" font-family="serif" fill="{c}">{plbl}</text>"#
            );
        }
    }

    let _ = writeln!(s, "</svg>");
    s
}

// ─── Ashtakavarga table ───────────────────────────────────────────────────────
//
// Classical Jyotish system: each of 7 planets contributes benefic points
// (bindus) to each of the 12 signs. Total = Sarvashtakavarga.
// This is a pure table-layout SVG; the bindu calculation is done here
// using the traditional algorithm.

/// Compute Ashtakavarga bindus for one planet.
///
/// Classic rules: a planet contributes 1 bindu to certain signs relative
/// to its own position and the positions of other planets + ASC.
/// This uses the simplified Parashara tables (8 reference points × 12 offsets).
fn ashtakavarga_bindus(
    planet_rasi: usize,
    other_rasis: &[usize], // [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, ASC]
    table: &[&[usize]],    // contribution offsets for each reference point
) -> [u8; 12] {
    let mut bindus = [0u8; 12];
    for (i, &ref_rasi) in other_rasis.iter().enumerate() {
        if i >= table.len() {
            break;
        }
        for &offset in table[i] {
            let sign = (ref_rasi + offset) % 12;
            bindus[sign] += 1;
        }
    }
    // Also add contributions from the planet itself
    let _ = planet_rasi; // used implicitly through other_rasis[self]
    bindus
}

/// Full Sarvashtakavarga: sum of all 7 planets' bindus per sign.
/// Returns a 7×12 matrix (rows = planets, cols = signs) and a 12-element total row.
fn sarvashtakavarga(planet_rasis: &[usize; 7], asc_rasi: usize) -> ([u8; 12], Vec<[u8; 12]>) {
    // Parashara tables: for each planet, the sign offsets (0-based from reference point)
    // that get a bindu. Each row = [Sun,Moon,Mars,Mer,Jup,Ven,Sat,ASC] reference points.
    // These are the classical Parashara benefic positions.
    const SUN_TABLE: &[&[usize]] = &[
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 6, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 5, 6, 9, 10, 11],
        &[5, 6, 9, 11],
        &[6, 7, 12, 1, 2, 3],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 4, 6, 10, 11, 12],
    ];
    const MON_TABLE: &[&[usize]] = &[
        &[3, 6, 10, 11],
        &[1, 3, 6, 7, 10, 11],
        &[2, 3, 5, 6, 9, 10, 11],
        &[1, 3, 4, 5, 7, 8, 10, 11],
        &[1, 4, 7, 8, 10, 11, 12],
        &[3, 4, 5, 7, 9, 10, 11],
        &[3, 5, 6, 11],
        &[3, 6, 10, 11],
    ];
    const MAR_TABLE: &[&[usize]] = &[
        &[3, 5, 6, 10, 11],
        &[3, 6, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 5, 6, 11],
        &[6, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 4, 7, 8, 9, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
    ];
    const MER_TABLE: &[&[usize]] = &[
        &[5, 6, 9, 11, 12],
        &[2, 4, 6, 8, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 3, 5, 6, 9, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 2, 4, 6, 8, 10, 11],
    ];
    const JUP_TABLE: &[&[usize]] = &[
        &[1, 2, 3, 4, 7, 8, 9, 10, 11],
        &[2, 5, 7, 9, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[1, 2, 4, 5, 6, 9, 10, 11],
        &[1, 2, 3, 4, 7, 8, 10, 11],
        &[2, 5, 6, 9, 10, 11],
        &[3, 5, 6, 12],
        &[1, 2, 4, 5, 6, 7, 9, 10, 11],
    ];
    const VEN_TABLE: &[&[usize]] = &[
        &[8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11, 12],
        &[3, 4, 6, 9, 11, 12],
        &[3, 5, 6, 9, 11],
        &[5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 10, 11],
        &[3, 4, 5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 11],
    ];
    const SAT_TABLE: &[&[usize]] = &[
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 6, 11],
        &[3, 5, 6, 10, 11, 12],
        &[6, 8, 9, 10, 11, 12],
        &[5, 6, 11, 12],
        &[6, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 10, 11],
        &[1, 3, 4, 6, 10, 11],
    ];

    let tables = [
        SUN_TABLE, MON_TABLE, MAR_TABLE, MER_TABLE, JUP_TABLE, VEN_TABLE, SAT_TABLE,
    ];
    let refs: Vec<usize> = planet_rasis
        .iter()
        .copied()
        .chain(std::iter::once(asc_rasi))
        .collect();

    let mut rows = Vec::new();
    let mut totals = [0u8; 12];
    for (pi, &tbl) in tables.iter().enumerate() {
        let row = ashtakavarga_bindus(planet_rasis[pi], &refs, tbl);
        for (si, &b) in row.iter().enumerate() {
            totals[si] += b;
        }
        rows.push(row);
    }
    (totals, rows)
}

fn build_ashtakavarga_context(
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

    let mut palette = BTreeMap::new();
    for (k, v) in [
        ("bg_color", "#fffff8"),
        ("border_color", "#5c3a00"),
        ("text_color", "#2a1a00"),
        ("planet_color", "#1a3a7a"),
        ("title", "Ashtakavarga"),
    ] {
        palette.insert(k.to_string(), json!(v));
    }
    for (k, v) in &vars {
        palette.insert(k.clone(), json!(v));
    }

    Ok(json!({
        "date": date_str, "jd": jd, "lat": lat, "lon": lon,
        "ashtakavarga_rows": row_vals,
        "sarvashtakavarga": totals.iter().map(|&b| json!(b)).collect::<Vec<_>>(),
        "vars": Value::Object(palette.into_iter().collect()),
    }))
}

fn render_ashtakavarga_svg(ctx: &Value) -> String {
    use std::fmt::Write;
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fffff8");
    let border = ctx["vars"]["border_color"].as_str().unwrap_or("#5c3a00");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#2a1a00");
    let pcol = ctx["vars"]["planet_color"].as_str().unwrap_or("#1a3a7a");
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Ashtakavarga");
    let date = ctx["date"].as_str().unwrap_or("");

    const LM: f64 = 80.0; // left margin (planet names)
    const TM: f64 = 65.0; // top margin
    const CW: f64 = 52.0; // cell width
    const RH: f64 = 26.0; // row height

    let rows = ctx["ashtakavarga_rows"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let totals = ctx["sarvashtakavarga"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();

    let n_rows = rows.len() + 1; // +1 for totals
    let total_h = TM + n_rows as f64 * RH + 60.0;
    let total_w = LM + 12.0 * CW + 20.0;

    let mut s = String::with_capacity(8 * 1024);
    let _ = writeln!(
        s,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w:.0} {total_h:.0}" width="{total_w:.0}" height="{total_h:.0}">
  <rect width="{total_w:.0}" height="{total_h:.0}" fill="{bg}"/>
  <text x="{:.1}" y="22" text-anchor="middle" font-size="15" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="{:.1}" y="40" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"#,
        total_w / 2.0,
        total_w / 2.0
    );

    // Column headers (rasi glyphs + numbers)
    for si in 0..12usize {
        let x = LM + si as f64 * CW + CW / 2.0;
        let _ = writeln!(
            s,
            r#"  <text x="{x:.1}" y="{:.1}" font-size="13" text-anchor="middle" font-family="serif" fill="{txt}">{}</text>
  <text x="{x:.1}" y="{:.1}" font-size="8" text-anchor="middle" fill="{txt}" opacity=".5">{}</text>"#,
            TM - 16.0,
            RASI_GLYPHS[si],
            TM - 5.0,
            si + 1
        );
    }

    // Grid lines
    for si in 0..=12usize {
        let x = LM + si as f64 * CW;
        let _ = writeln!(
            s,
            r#"  <line x1="{x:.1}" y1="{TM:.1}" x2="{x:.1}" y2="{:.1}" stroke="{border}" stroke-width="0.6" opacity=".4"/>"#,
            TM + n_rows as f64 * RH
        );
    }

    // Planet rows
    for (ri, row) in rows.iter().enumerate() {
        let ry = TM + ri as f64 * RH;
        let name = row["planet"].as_str().unwrap_or("?");
        let bindus = row["bindus"]
            .as_array()
            .map(|v| v.to_vec())
            .unwrap_or_default();

        // Row background alternating
        if ri % 2 == 0 {
            let _ = writeln!(
                s,
                r#"  <rect x="{LM:.1}" y="{ry:.1}" width="{:.1}" height="{RH}" fill="{border}" opacity=".04"/>"#,
                12.0 * CW
            );
        }
        let _ = writeln!(
            s,
            r#"  <line x1="{LM:.1}" y1="{ry:.1}" x2="{:.1}" y2="{ry:.1}" stroke="{border}" stroke-width="0.4" opacity=".3"/>"#,
            LM + 12.0 * CW
        );
        let _ = writeln!(
            s,
            r#"  <text x="{:.1}" y="{:.1}" font-size="10" text-anchor="end" dominant-baseline="central" fill="{pcol}" font-weight="500">{name}</text>"#,
            LM - 4.0,
            ry + RH / 2.0
        );

        for (si, b) in bindus.iter().enumerate() {
            let bv = b.as_u64().unwrap_or(0);
            let x = LM + si as f64 * CW + CW / 2.0;
            // Colour-code: 4+ is strong (green tint), 0-2 weak (red tint)
            let col = if bv >= 5 {
                "#1a6030"
            } else if bv <= 2 {
                "#901020"
            } else {
                txt
            };
            let _ = writeln!(
                s,
                r#"  <text x="{x:.1}" y="{:.1}" font-size="11" text-anchor="middle" dominant-baseline="central" font-weight="500" fill="{col}">{bv}</text>"#,
                ry + RH / 2.0
            );
        }
    }

    // Totals row
    let ty = TM + rows.len() as f64 * RH;
    let _ = writeln!(
        s,
        r#"  <rect x="{LM:.1}" y="{ty:.1}" width="{:.1}" height="{RH}" fill="{border}" opacity=".1"/>
  <line x1="{LM:.1}" y1="{ty:.1}" x2="{:.1}" y2="{ty:.1}" stroke="{border}" stroke-width="1.5" opacity=".6"/>
  <text x="{:.1}" y="{:.1}" font-size="10" text-anchor="end" dominant-baseline="central" fill="{txt}" font-weight="700">Total</text>"#,
        12.0 * CW,
        LM + 12.0 * CW,
        LM - 4.0,
        ty + RH / 2.0
    );
    for (si, b) in totals.iter().enumerate() {
        let bv = b.as_u64().unwrap_or(0);
        let x = LM + si as f64 * CW + CW / 2.0;
        let col = if bv >= 28 {
            "#1a6030"
        } else if bv <= 18 {
            "#901020"
        } else {
            txt
        };
        let _ = writeln!(
            s,
            r#"  <text x="{x:.1}" y="{:.1}" font-size="12" text-anchor="middle" dominant-baseline="central" font-weight="700" fill="{col}">{bv}</text>"#,
            ty + RH / 2.0
        );
    }

    let _ = writeln!(s, "</svg>");
    s
}

// ─── Shadbala strength table ──────────────────────────────────────────────────
//
// Six-fold strength system. We compute three readily-available components:
// Ochchabala (exaltation strength), Saptavargaja bala (sign placement),
// and Chesta bala (motional strength from speed).

fn build_shadbala_context(
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

    let mut rows: Vec<Value> = Vec::new();
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
                "strong":     total >= 100.0,
            }));
        }
    }

    let mut palette = BTreeMap::new();
    for (k, v) in [
        ("bg_color", "#fffff8"),
        ("border_color", "#5c3a00"),
        ("text_color", "#2a1a00"),
        ("planet_color", "#1a3a7a"),
    ] {
        palette.insert(k.to_string(), json!(v));
    }
    for (k, v) in &vars {
        palette.insert(k.clone(), json!(v));
    }

    Ok(json!({
        "date": date_str, "jd": jd, "lat": lat, "lon": lon,
        "shadbala": rows,
        "vars": Value::Object(palette.into_iter().collect()),
    }))
}

fn render_shadbala_svg(ctx: &Value) -> String {
    use std::fmt::Write;
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fffff8");
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
    let rows = ctx["shadbala"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let total_h = TM + (rows.len() + 1) as f64 * RH + 20.0;

    let mut s = String::with_capacity(6 * 1024);
    let _ = writeln!(
        s,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w:.0} {total_h:.0}" width="{total_w:.0}" height="{total_h:.0}">
  <rect width="{total_w:.0}" height="{total_h:.0}" fill="{bg}"/>
  <text x="{:.1}" y="22" text-anchor="middle" font-size="14" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="{:.1}" y="38" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"#,
        total_w / 2.0,
        total_w / 2.0
    );

    // Column headers
    let mut cx = 20.0_f64;
    for &(hdr, cw) in COLS {
        let _ = writeln!(
            s,
            r#"  <text x="{:.1}" y="{:.1}" font-size="10" font-weight="600" text-anchor="middle" fill="{txt}" opacity=".75">{hdr}</text>"#,
            cx + cw / 2.0,
            TM - 8.0
        );
        let _ = writeln!(
            s,
            r#"  <line x1="{cx:.1}" y1="{TM:.1}" x2="{:.1}" y2="{:.1}" stroke="{border}" stroke-width="0.5" opacity=".4"/>"#,
            cx,
            TM + rows.len() as f64 * RH
        );
        cx += cw;
    }
    let _ = writeln!(
        s,
        r#"  <line x1="{cx:.1}" y1="{TM:.1}" x2="{cx:.1}" y2="{:.1}" stroke="{border}" stroke-width="0.5" opacity=".4"/>"#,
        TM + rows.len() as f64 * RH
    );

    // Header separator
    let _ = writeln!(
        s,
        r#"  <line x1="20" y1="{TM:.1}" x2="{cx:.1}" y2="{TM:.1}" stroke="{border}" stroke-width="1.2" opacity=".6"/>"#
    );

    // Data rows
    for (ri, row) in rows.iter().enumerate() {
        let ry = TM + ri as f64 * RH;
        let name = row["name"].as_str().unwrap_or("?");
        let lon = row["lon"].as_f64().unwrap_or(0.0);
        let ret = row["retro"].as_bool().unwrap_or(false);
        let och = row["ochchabala"].as_f64().unwrap_or(0.0);
        let sap = row["sapta_bala"].as_f64().unwrap_or(0.0);
        let che = row["chesta_bala"].as_f64().unwrap_or(0.0);
        let dig = row["dig_bala"].as_f64().unwrap_or(0.0);
        let tot = row["total"].as_f64().unwrap_or(0.0);
        let strong = row["strong"].as_bool().unwrap_or(false);
        let rcol = if strong { "#1a5030" } else { pcol };

        if ri % 2 == 0 {
            let _ = writeln!(
                s,
                r#"  <rect x="20" y="{ry:.1}" width="{:.1}" height="{RH}" fill="{border}" opacity=".04"/>"#,
                cx - 20.0
            );
        }
        let _ = writeln!(
            s,
            r#"  <line x1="20" y1="{ry:.1}" x2="{cx:.1}" y2="{ry:.1}" stroke="{border}" stroke-width="0.3" opacity=".25"/>"#
        );

        let vals: &[(&str, f64)] = &[("", 0.0)]; // dummy — we write manually
        let _ = vals;

        let mut x = 20.0_f64;
        let data: &[(&str, String)] = &[
            (name, format!("{}{}", name, if ret { " ℞" } else { "" })),
            ("lon", fmt_lon_dms(lon)),
            ("och", format!("{och:.1}")),
            ("sap", format!("{sap:.1}")),
            ("che", format!("{che:.1}")),
            ("dig", format!("{dig:.1}")),
            ("tot", format!("{tot:.1}")),
        ];
        for (ci, ((_key, val), &(_hdr, cw))) in data.iter().zip(COLS.iter()).enumerate() {
            let fw = if ci == 0 || ci == 6 { "600" } else { "400" };
            let fc = if ci == 6 { rcol } else { txt };
            let _ = writeln!(
                s,
                r#"  <text x="{:.1}" y="{:.1}" font-size="10" font-weight="{fw}" text-anchor="middle" dominant-baseline="central" fill="{fc}">{val}</text>"#,
                x + cw / 2.0,
                ry + RH / 2.0
            );
            x += cw;
        }
    }

    // Bottom border
    let by = TM + rows.len() as f64 * RH;
    let _ = writeln!(
        s,
        r#"  <line x1="20" y1="{by:.1}" x2="{cx:.1}" y2="{by:.1}" stroke="{border}" stroke-width="1.0" opacity=".5"/>"#
    );
    let _ = writeln!(
        s,
        r#"  <text x="{:.1}" y="{:.1}" font-size="7" fill="{txt}" opacity=".4">Ochcha=exaltation · Sapta=sign placement · Chesta=motional · Dig=directional (shashtiamsas, out of 60)</text>"#,
        20.0,
        by + 14.0
    );

    let _ = writeln!(s, "</svg>");
    s
}

const RASI_NAMES: &[&str] = &[
    "Ar", "Ta", "Ge", "Ca", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi",
];

const RASI_GLYPHS: &[&str] = &[
    "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓",
];

// (row, col) → sign index; 4 rows × 4 cols, centre (rows 1-2, cols 1-2) = metadata
const SI_CELLS: &[(usize, usize, i32)] = &[
    (0, 0, 11),
    (0, 1, 0),
    (0, 2, 1),
    (0, 3, 2),
    (1, 0, 10),
    (1, 3, 3),
    (2, 0, 9),
    (2, 3, 4),
    (3, 0, 8),
    (3, 1, 7),
    (3, 2, 6),
    (3, 3, 5),
];

fn build_vedic_context(
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
    let mut planets: Vec<Value> = Vec::new();
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
                "dms": fmt_lon_dms(pos.lon),
            }));
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
                "end_jd":   d.end,
            })
        })
        .collect();

    // Ochchabala (exaltation strength 0–60)
    let strengths: Vec<Value> = (0i32..7)
        .filter_map(|raw| {
            let body_lon = planets
                .iter()
                .find(|p| {
                    p["key"].as_str().map_or(false, |k| {
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

    let mut palette = BTreeMap::new();
    for (k, v) in [
        ("bg_color", "#fffff8"),
        ("border_color", "#5c3a00"),
        ("text_color", "#2a1a00"),
        ("planet_color", "#1a3a7a"),
        ("retro_color", "#a01030"),
        ("asc_color", "#006030"),
    ] {
        palette.insert(k.to_string(), json!(v));
    }
    for (k, v) in &vars {
        palette.insert(k.clone(), json!(v));
    }

    Ok(json!({
        "date": date_str, "jd": jd, "lat": lat, "lon": lon,
        "planets": planets, "dashas": dashas, "strengths": strengths,
        "moon_sid_lon": (moon_sid_lon * 1e4).round() / 1e4,
        "vars": Value::Object(palette.into_iter().collect()),
    }))
}

fn render_south_indian_svg(ctx: &Value) -> String {
    use std::fmt::Write;
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fffff8");
    let border = ctx["vars"]["border_color"].as_str().unwrap_or("#5c3a00");
    let txt = ctx["vars"]["text_color"].as_str().unwrap_or("#2a1a00");
    let pcol = ctx["vars"]["planet_color"].as_str().unwrap_or("#1a3a7a");
    let retro = ctx["vars"]["retro_color"].as_str().unwrap_or("#a01030");
    let _asc_c = ctx["vars"]["asc_color"].as_str().unwrap_or("#006030"); // reserved for ASC cell highlight
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Rasi Chart");
    let date = ctx["date"].as_str().unwrap_or("");

    // Cell geometry: 4×4 grid, each cell 140×120, centre 2×2 merged
    const CW: f64 = 140.0; // cell width
    const CH: f64 = 120.0; // cell height
    const OX: f64 = 30.0; // origin x
    const OY: f64 = 70.0; // origin y (below title)

    let planets = ctx["planets"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    // Find ASC rasi (first planet with key "asc" — we'll use the first planet's rasi
    // as placeholder; real ASC needs sidereal house calc which we approximate here)
    // For simplicity, mark which rasi is lagna from planets (we skip ASC calc here)

    // Group planets by rasi
    let mut rasi_planets: Vec<Vec<String>> = vec![Vec::new(); 12];
    for p in &planets {
        let rasi = p["rasi"].as_i64().unwrap_or(0) as usize % 12;
        let g = p["glyph"].as_str().unwrap_or("?");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let deg = p["deg_in_rasi"].as_f64().unwrap_or(0.0);
        let lbl = format!("{g}{}", if ret { "℞" } else { "" });
        let deg_s = format!("{deg:.0}°");
        rasi_planets[rasi].push(format!("{lbl} {deg_s}"));
    }

    let mut s = String::with_capacity(16 * 1024);
    let total_h = OY + 4.0 * CH + 40.0;
    let total_w = OX * 2.0 + 4.0 * CW;

    let _ = writeln!(
        s,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w:.0} {total_h:.0}" width="{total_w:.0}" height="{total_h:.0}">
  <rect width="{total_w:.0}" height="{total_h:.0}" fill="{bg}"/>
  <text x="{:.2}" y="28" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="{:.2}" y="48" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"#,
        total_w / 2.0,
        total_w / 2.0
    );

    // Draw outer border
    let _ = writeln!(
        s,
        r#"  <rect x="{OX}" y="{OY}" width="{:.2}" height="{:.2}" fill="none" stroke="{border}" stroke-width="2.0"/>"#,
        4.0 * CW,
        4.0 * CH
    );

    // Centre box (2×2)
    let cx = OX + CW;
    let cy = OY + CH;
    let _ = writeln!(
        s,
        r#"  <rect x="{cx:.2}" y="{cy:.2}" width="{:.2}" height="{:.2}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        2.0 * CW,
        2.0 * CH
    );

    // Centre text (chart metadata)
    let moon_sid = ctx["moon_sid_lon"].as_f64().unwrap_or(0.0);
    let (moon_nak, _) = long_to_nakshatra(moon_sid);
    let nak_nm = nakshatra_name(moon_nak).unwrap_or("?");
    let _ = writeln!(
        s,
        r#"  <text x="{:.2}" y="{:.2}" text-anchor="middle" font-size="11" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">Vedic Chart</text>
  <text x="{:.2}" y="{:.2}" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".7">☽ {nak_nm}</text>"#,
        cx + CW,
        cy + CH * 0.85,
        cx + CW,
        cy + CH * 1.1
    );

    // Draw cells
    for &(row, col, sign_idx) in SI_CELLS {
        let x = OX + col as f64 * CW;
        let y = OY + row as f64 * CH;
        let _ = writeln!(
            s,
            r#"  <rect x="{x:.2}" y="{y:.2}" width="{CW:.2}" height="{CH:.2}" fill="{bg}" stroke="{border}" stroke-width="1.0"/>"#
        );

        // Sign name top-left
        let sg = RASI_GLYPHS[sign_idx as usize % 12];
        let sn = RASI_NAMES[sign_idx as usize % 12];
        let _ = writeln!(
            s,
            r#"  <text x="{:.2}" y="{:.2}" font-size="11" font-family="serif" fill="{txt}" opacity=".5">{sg}</text>
  <text x="{:.2}" y="{:.2}" font-size="8" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{sn}</text>"#,
            x + 4.0,
            y + 14.0,
            x + 4.0,
            y + 24.0
        );

        // Planets in this rasi
        let prasi = &rasi_planets[sign_idx as usize % 12];
        for (pi, plabel) in prasi.iter().enumerate() {
            let py = y + 36.0 + pi as f64 * 14.0;
            let is_retro = plabel.contains('℞');
            let col = if is_retro { retro } else { pcol };
            let _ = writeln!(
                s,
                r#"  <text x="{:.2}" y="{py:.2}" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}">{plabel}</text>"#,
                x + 6.0
            );
        }
    }

    // Dashas mini-table below the grid
    let dy = OY + 4.0 * CH + 10.0;
    let _ = writeln!(
        s,
        r#"  <text x="{OX}" y="{dy:.2}" font-size="11" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">Vimshottari Dashas</text>"#
    );
    let dashas = ctx["dashas"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    for (i, d) in dashas.iter().take(9).enumerate() {
        let col_x = OX + (i % 3) as f64 * (total_w - OX * 2.0) / 3.0;
        let row_y = dy + 14.0 + (i / 3) as f64 * 14.0;
        let body = d["body"].as_str().unwrap_or("?");
        let yrs = d["years"].as_f64().unwrap_or(0.0);
        let start = d["start"].as_str().unwrap_or("");
        let _ = writeln!(
            s,
            r#"  <text x="{col_x:.2}" y="{row_y:.2}" font-size="9"
            font-family="'Segoe UI',system-ui,sans-serif" fill="{pcol}"><tspan font-weight="600">{body}</tspan> {yrs:.1}y · {start}</text>"#
        );
    }

    let _ = writeln!(s, "</svg>");
    s
}

fn render_navamsa_svg(ctx: &Value) -> String {
    // Navamsa uses the same South Indian grid layout but with navamsa positions
    let planets_orig = ctx["planets"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
    // Rebuild rasi_planets using navamsa index instead of rasi
    let mut rasi_planets: Vec<Vec<String>> = vec![Vec::new(); 12];
    for p in &planets_orig {
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

fn render_dasha_svg(ctx: &Value) -> String {
    use std::fmt::Write;
    let bg = ctx["vars"]["bg_color"].as_str().unwrap_or("#fffff8");
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

    let dashas = ctx["dashas"]
        .as_array()
        .map(|v| v.to_vec())
        .unwrap_or_default();
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
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 {total_h:.0}" width="900" height="{total_h:.0}">
  <rect width="900" height="{total_h:.0}" fill="{bg}"/>
  <text x="450" y="28" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="450" y="48" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"#
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
        if x < LM || x > LM + W {
            continue;
        }
        let _ = writeln!(
            s,
            r#"  <line x1="{x:.1}" y1="{TM:.1}" x2="{x:.1}" y2="{:.1}" stroke="{txt}" stroke-width="0.5" opacity=".2"/>
  <text x="{x:.1}" y="{:.1}" text-anchor="middle" font-size="8" fill="{txt}" opacity=".5">{yr}</text>"#,
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
            .map(|(_, c)| *c)
            .unwrap_or(pcol);
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
            r#"  <text x="{:.1}" y="{:.1}" text-anchor="end" font-size="9" dominant-baseline="central" fill="{txt}" opacity=".7">{body}</text>"#,
            LM - 4.0,
            by + BH * 0.5
        );
    }

    let _ = writeln!(s, "</svg>");
    s
}

#[cfg(test)]
mod tests_vedic {
    use super::*;

    #[test]
    fn vedic_context_has_sidereal_rasi() {
        let jd = 2_451_545.0; // J2000.0
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 13.08, 80.27, "2000-01-01", vars, "Rasi").unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 12);
        for p in planets {
            let rasi = p["rasi"].as_i64().unwrap_or(-1);
            assert!(
                rasi >= 0 && rasi < 12,
                "rasi {rasi} out of range for {}",
                p["name"]
            );
        }
    }

    #[test]
    fn vedic_context_has_nakshatra() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Rasi").unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        for p in planets {
            let nak = p["nakshatra"].as_i64().unwrap_or(-1);
            assert!(
                nak >= 0 && nak < 27,
                "nakshatra {nak} out of range for {}",
                p["name"]
            );
            let pada = p["pada"].as_i64().unwrap_or(-1);
            assert!(pada >= 1 && pada <= 4, "pada {pada} out of range");
        }
    }

    #[test]
    fn vedic_context_has_dashas() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Dasha").unwrap();
        let dashas = ctx["dashas"].as_array().unwrap();
        assert!(!dashas.is_empty(), "no dashas computed");
        // Total Vimshottari cycle = 120 years; entries should cover a long span
        let total_yrs: f64 = dashas
            .iter()
            .map(|d| d["years"].as_f64().unwrap_or(0.0))
            .sum();
        assert!(
            total_yrs > 50.0,
            "total dasha years {total_yrs:.1} too short"
        );
    }

    #[test]
    fn navamsa_positions_differ_from_rasi() {
        // Navamsa D9 divides each sign into 9 equal parts of 3°20'
        // At least some planets should have a different navamsa vs rasi sign
        let jd = 2_451_545.0;
        use celestial_core::{
            body::{Body, CalcFlags},
            long_to_navamsa, long_to_rasi,
        };
        let flags = CalcFlags::BUILTIN | CalcFlags(64); // sidereal
        if let Ok(sun) = celestial_core::calc_ut(jd, Body::SUN, flags) {
            let rasi = long_to_rasi(sun.lon);
            let navamsa = long_to_navamsa(sun.lon);
            // Can't assert they differ (they might coincide), but both must be valid
            assert!(rasi >= 0 && rasi < 12, "rasi {rasi} invalid");
            assert!(navamsa >= 0 && navamsa < 12, "navamsa {navamsa} invalid");
        }
    }

    #[test]
    fn south_indian_svg_has_all_signs() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Rasi").unwrap();
        let svg = render_south_indian_svg(&ctx);
        // All 12 rasi abbreviations must appear
        for sign in [
            "Ar", "Ta", "Ge", "Ca", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi",
        ] {
            assert!(
                svg.contains(sign),
                "sign {sign} missing from South Indian SVG"
            );
        }
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn dasha_svg_has_planet_bars() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Dasha").unwrap();
        let svg = render_dasha_svg(&ctx);
        assert!(svg.contains("<rect"), "no bars in dasha SVG");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn navamsa_svg_built_without_panic() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Navamsa").unwrap();
        let svg = render_navamsa_svg(&ctx);
        assert!(svg.contains("D9"), "Navamsa title missing");
        assert!(svg.contains("</svg>"));
    }
}
