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
use celestial_core::MoonPhase;
use celestial_core::{
    calc_ut, diff_deg_signed, houses_ex, midpoint_deg, moon_illumination, moon_phase,
};
use celestial_core::{lon_to_sign, zodiac_sign_name};
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

const ASPECT_DEFS: &[(f64, &str, f64)] = &[
    (0.0, "conjunction", 8.0),
    (60.0, "sextile", 6.0),
    (90.0, "square", 7.0),
    (120.0, "trine", 8.0),
    (150.0, "quincunx", 3.0),
    (180.0, "opposition", 8.0),
];

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
            for &(asp_deg, asp_name, orb_lim) in ASPECT_DEFS {
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
    let _ = write!(
        s,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 1100" width="900" height="1100">
  <defs>
    <filter id="glow" x="-40%" y="-40%" width="180%" height="180%">
      <feGaussianBlur stdDeviation="2.5" result="b"/>
      <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
  </defs>
  <rect width="900" height="1100" fill="{bg}"/>
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
    let _ = write!(
        s,
        r#"</text>

  <!-- rings -->
  <circle cx="{CX}" cy="{CY}" r="{RO}" fill="none" stroke="{ring}" stroke-width="2.5" opacity=".6"/>
  <circle cx="{CX}" cy="{CY}" r="{RM}" fill="none" stroke="{ring}" stroke-width="1.2" opacity=".35"/>
  <circle cx="{CX}" cy="{CY}" r="{RI}" fill="none" stroke="{ring}" stroke-width="2.0" opacity=".55"/>
  <circle cx="{CX}" cy="{CY}" r="{RH}" fill="none" stroke="{ring}" stroke-width="1.2" opacity=".35"/>
  <circle cx="{CX}" cy="{CY}" r="{RC}" fill="{bg}"  stroke="{ring}" stroke-width="2.0" opacity=".4"/>

"#
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
        let _ = write!(
            s,
            r#"  <line x1="{sx1:.2}" y1="{sy1:.2}" x2="{sx2:.2}" y2="{sy2:.2}" stroke="{ring}" stroke-width="1.5" opacity=".55"/>
  <text x="{gx:.2}" y="{gy:.2}" font-size="15" font-weight="600" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{ring}">{g}</text>
"#
        );
    }

    // ── house cusps ───────────────────────────────────────────────────────────
    s.push_str("\n");
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
        let _ = write!(
            s,
            r#"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{ring}" stroke-width="{sw}" opacity="{op}"/>
  <text x="{nx:.2}" y="{ny:.2}" font-size="10" font-weight="500" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">{n}</text>
"#
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
        let _ = write!(
            s,
            r#"  <text x="{lx:.2}" y="{ly:.2}" text-anchor="{anchor}" font-size="12" font-weight="800" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">{name}</text>
"#
        );
    }

    // ── aspect lines ─────────────────────────────────────────────────────────
    s.push_str("\n");
    for asp in aspects {
        let x1 = asp["x1"].as_f64().unwrap_or(0.0);
        let y1 = asp["y1"].as_f64().unwrap_or(0.0);
        let x2 = asp["x2"].as_f64().unwrap_or(0.0);
        let y2 = asp["y2"].as_f64().unwrap_or(0.0);
        let orb = asp["orb"].as_f64().unwrap_or(8.0);
        let hard = asp["is_hard"].as_bool().unwrap_or(false);
        let col = if hard { hard_c } else { soft_c };
        let op = if orb < 2.0 { ".55" } else { ".22" };
        let _ = write!(
            s,
            r#"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{col}" stroke-width="1.8" opacity="{op}"/>
"#
        );
    }

    // ── planet glyphs (collision-free label placement) ────────────────────────
    s.push_str("\n");

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
            let _ = write!(
                s,
                r#"  <line x1="{ax:.2}" y1="{ay:.2}" x2="{lx:.2}" y2="{ly:.2}" stroke="{pfg}" stroke-width="0.9" opacity=".45" stroke-dasharray="3,2"/>
"#
            );
        }

        let _ = write!(
            s,
            r#"  <line x1="{tx1:.2}" y1="{ty1:.2}" x2="{tx2:.2}" y2="{ty2:.2}" stroke="{pfg}" stroke-width="1.0" opacity=".45"/>
  <text x="{px:.2}" y="{py:.2}" font-size="18" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}" filter="url(#glow)">{g}</text>
  <text x="{lx:.2}" y="{ly:.2}" font-size="10" font-weight="600" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}">{dl}</text>
"#
        );
    }

    // ── moon phase in centre ──────────────────────────────────────────────────
    let _ = write!(
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
    let _ = write!(
        s,
        r#"  <text x="{c1x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Planets</text>
  <line x1="{c1x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>
"#,
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
        let _ = write!(
            s,
            r#"  <text x="{:.2}" y="{ry:.2}" font-size="14" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}">{g}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="11" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".75">{name}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{scol}" opacity="{sop}">{spd}</text>
"#,
            c1x + 2.0,
            c1x + 20.0,
            c1x + 120.0,
            c1x + 222.0
        );
    }

    // ── col 2: angles + houses ────────────────────────────────────────────────
    let _ = write!(
        s,
        r#"
  <text x="{c2x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Angles &amp; Houses</text>
  <line x1="{c2x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>
"#,
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
        let _ = write!(
            s,
            r#"  <text x="{:.2}" y="{ry:.2}" font-size="10" font-weight="700" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">{name}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{lon2:.4}&#176;</text>
"#,
            c2x + 2.0,
            c2x + 38.0,
            c2x + 150.0
        );
    }
    let sep_y = ly + 82.0;
    let _ = write!(
        s,
        r#"  <line x1="{c2x}" y1="{sep_y:.2}" x2="{:.2}" y2="{sep_y:.2}" stroke="{ring}" stroke-width=".3" opacity=".2"/>
"#,
        c2x + 250.0
    );
    for (i, h) in houses.iter().enumerate() {
        let ry = ly + 94.0 + i as f64 * rh2;
        let dms = h["dms"].as_str().unwrap_or("");
        let hlon = h["lon"].as_f64().unwrap_or(0.0);
        let _ = write!(
            s,
            r#"  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".55">H{}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{hlon:.4}&#176;</text>
"#,
            c2x + 2.0,
            i + 1,
            c2x + 28.0,
            c2x + 138.0
        );
    }

    // ── col 3: aspects ────────────────────────────────────────────────────────
    let _ = write!(
        s,
        r#"
  <text x="{c3x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Aspects</text>
  <line x1="{c3x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>
"#,
        ly + 3.0,
        c3x + 292.0,
        ly + 3.0
    );
    if aspects.is_empty() {
        let _ = write!(
            s,
            r#"  <text x="{:.2}" y="{:.2}" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">(no major aspects within orbs)</text>
"#,
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
        let _ = write!(
            s,
            r#"  <text x="{:.2}" y="{ry:.2}" font-size="13" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}">{g1}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="13" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}">{g2}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}">{an4}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".7">{orb:.2}&#176;</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".5">{aind}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{b1s}&#8211;{b2s}</text>
"#,
            c3x + 2.0,
            c3x + 18.0,
            c3x + 34.0,
            c3x + 92.0,
            c3x + 130.0,
            c3x + 168.0
        );
    }

    // ── footer ────────────────────────────────────────────────────────────────
    let _ = write!(
        s,
        r#"
  <text x="450" y="1090" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif"
        fill="{ring}" opacity=".35">Generated by celestial render · {date}</text>
</svg>
"#
    );

    s
}

// ─── Entry point ─────────────────────────────────────────────────────────────

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
    let ctx = build_context(jd, args.lat, args.lon, &args.date, args.hsys, user_vars)?;

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
        None => render_builtin_svg(&ctx),
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
}
