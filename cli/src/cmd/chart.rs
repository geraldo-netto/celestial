//! `celestial chart` — full astrological chart with JSON and SVG output.
#![allow(clippy::needless_range_loop)]

use crate::error::CliError;
use crate::{format as fmt, parse};
use celestial_core::body::{Body, CalcFlags, HouseSystem};
use celestial_core::{
    calc_chart_aspects, calc_ut, houses_ex, lon_to_sign, zodiac_sign_name, MAJOR_ASPECTS,
};
use clap::Args;
use std::f64::consts::PI;
use std::fmt::Write as _;

// ─── CLI args ─────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct ChartArgs {
    /// Date (YYYY-MM-DD [HH:MM[:SS]]) or Julian day, default: now
    #[arg(short, long, default_value = "now")]
    pub date: String,

    /// Time of day UT (HH:MM or HH:MM:SS) — merged with --date if --date has no time
    #[arg(long)]
    pub time: Option<String>,

    /// Geographic latitude in decimal degrees (N positive)
    #[arg(long, allow_hyphen_values = true)]
    pub lat: f64,

    /// Geographic longitude in decimal degrees (E positive)
    #[arg(long, allow_hyphen_values = true)]
    pub lon: f64,

    /// House system: placidus, koch, equal, whole, porphyry (default: placidus)
    #[arg(short, long, default_value = "placidus")]
    pub system: String,

    /// Write SVG wheel chart to this file (e.g. chart.svg)
    #[arg(long)]
    pub svg: Option<String>,

    /// Chart name / title for display
    #[arg(long, default_value = "")]
    pub name: String,
}

// ─── Data model ───────────────────────────────────────────────────────────────

pub struct Planet {
    pub body: Body,
    pub name: &'static str,
    pub lon: f64,
    pub lat: f64,
    pub speed: f64,
    pub retro: bool,
    pub glyph: &'static str,
}

pub struct ChartData {
    pub jd: f64,
    pub lat: f64,
    pub lon: f64,
    pub hsys: u8,
    pub asc: f64,
    pub mc: f64,
    pub ic: f64,
    pub dsc: f64,
    pub vertex: f64,
    pub cusps: [f64; 13],
    pub planets: Vec<Planet>,
}

const CHART_BODIES: &[Body] = &[
    Body::SUN,
    Body::MOON,
    Body::MERCURY,
    Body::VENUS,
    Body::MARS,
    Body::JUPITER,
    Body::SATURN,
    Body::URANUS,
    Body::NEPTUNE,
    Body::PLUTO,
    Body::MEAN_NODE,
    Body::CHIRON,
];

fn body_glyph(body: Body) -> &'static str {
    match body.as_raw() {
        0 => "☉",       // Sun
        1 => "☽",       // Moon
        2 => "☿",       // Mercury
        3 => "♀",       // Venus
        4 => "♂",       // Mars
        5 => "♃",       // Jupiter
        6 => "♄",       // Saturn
        7 => "♅",       // Uranus
        8 => "♆",       // Neptune
        9 => "♇",       // Pluto
        10 | 11 => "☊", // Node
        15 => "⚷",      // Chiron
        _ => "●",
    }
}

fn sign_glyph(sign: u8) -> &'static str {
    const GLYPHS: [&str; 12] = [
        "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓",
    ];
    GLYPHS[(sign % 12) as usize]
}

// ─── Chart computation ────────────────────────────────────────────────────────

pub fn compute_chart(jd: f64, lat: f64, lon: f64, hsys: u8) -> Result<ChartData, CliError> {
    let h = houses_ex(jd, CalcFlags::BUILTIN, lat, lon, HouseSystem(hsys))
        ?;

    let asc = h.ascmc[0];
    let mc = h.ascmc[1];
    let ic = (mc + 180.0).rem_euclid(360.0);
    let dsc = (asc + 180.0).rem_euclid(360.0);
    let vertex = h.ascmc[3];

    let mut planets = Vec::new();
    for &body in CHART_BODIES {
        let pos = calc_ut(jd, body, CalcFlags::BUILTIN | CalcFlags::SPEED)
            .map_err(|e| format!("{}: {e}", parse::body_name(body)))?;
        planets.push(Planet {
            body,
            name: parse::body_name(body),
            lon: pos.lon,
            lat: pos.lat,
            speed: pos.speed_lon,
            retro: pos.speed_lon < 0.0,
            glyph: body_glyph(body),
        });
    }

    Ok(ChartData {
        jd,
        lat,
        lon,
        hsys,
        asc,
        mc,
        ic,
        dsc,
        vertex,
        cusps: h.cusps,
        planets,
    })
}

// ─── JSON output ──────────────────────────────────────────────────────────────

pub fn print_json(chart: &ChartData) {
    let date_str = parse::jd_to_str(chart.jd);

    // Planets array
    let planet_items: Vec<String> = chart
        .planets
        .iter()
        .map(|p| {
            let (sign, deg) = lon_to_sign(p.lon);
            fmt::json_obj(&[
                ("body", p.name.to_string()),
                ("glyph", p.glyph.to_string()),
                ("longitude", format!("{:.6}", p.lon)),
                ("sign", zodiac_sign_name(sign).to_string()),
                ("sign_deg", format!("{:.4}", deg)),
                ("latitude", format!("{:.6}", p.lat)),
                ("speed", format!("{:.6}", p.speed)),
                ("retrograde", p.retro.to_string()),
            ])
        })
        .collect();

    // Houses array
    let house_items: Vec<String> = (1..=12)
        .map(|i| {
            let cusp = chart.cusps[i];
            let (sign, deg) = lon_to_sign(cusp);
            fmt::json_obj(&[
                ("house", i.to_string()),
                ("cusp", format!("{:.6}", cusp)),
                ("sign", zodiac_sign_name(sign).to_string()),
                ("sign_deg", format!("{:.4}", deg)),
            ])
        })
        .collect();

    // Aspects array
    let positions: Vec<(Body, f64, f64)> = chart
        .planets
        .iter()
        .map(|p| (p.body, p.lon, p.speed))
        .collect();
    let aspects = calc_chart_aspects(&positions, MAJOR_ASPECTS, 8.0);
    let aspect_name = |deg: f64| match deg as i32 {
        0 => "Conjunction",
        60 => "Sextile",
        90 => "Square",
        120 => "Trine",
        180 => "Opposition",
        _ => "Aspect",
    };
    let aspect_items: Vec<String> = aspects
        .iter()
        .map(|a| {
            fmt::json_obj(&[
                ("body1", parse::body_name(a.body1).to_string()),
                ("body2", parse::body_name(a.body2).to_string()),
                ("aspect", aspect_name(a.aspect).to_string()),
                ("angle", format!("{:.1}", a.aspect)),
                ("orb", format!("{:.4}", a.orb)),
                ("applying", a.applying.to_string()),
            ])
        })
        .collect();

    let (asc_sign, asc_deg) = lon_to_sign(chart.asc);
    let (mc_sign, mc_deg) = lon_to_sign(chart.mc);

    // Top-level object
    println!("{{");
    println!("  \"date\": \"{date_str}\",");
    println!("  \"jd\": {:.4},", chart.jd);
    println!("  \"lat\": {:.4},", chart.lat);
    println!("  \"lon\": {:.4},", chart.lon);
    println!("  \"house_system\": \"{}\",", parse::hsys_name(chart.hsys));
    println!("  \"angles\": {{");
    println!(
        "    \"asc\":    {{ \"lon\": {:.6}, \"sign\": \"{}\", \"deg\": {:.4} }},",
        chart.asc,
        zodiac_sign_name(asc_sign),
        asc_deg
    );
    println!(
        "    \"mc\":     {{ \"lon\": {:.6}, \"sign\": \"{}\", \"deg\": {:.4} }},",
        chart.mc,
        zodiac_sign_name(mc_sign),
        mc_deg
    );
    println!("    \"ic\":     {{ \"lon\": {:.6} }},", chart.ic);
    println!("    \"dsc\":    {{ \"lon\": {:.6} }},", chart.dsc);
    println!("    \"vertex\": {{ \"lon\": {:.6} }}", chart.vertex);
    println!("  }},");
    println!("  \"planets\": {},", fmt::json_array(planet_items));
    println!("  \"houses\": {},", fmt::json_array(house_items));
    println!("  \"aspects\": {}", fmt::json_array(aspect_items));
    println!("}}");
}

// ─── SVG wheel chart ──────────────────────────────────────────────────────────

const CX: f64 = 400.0;
const CY: f64 = 400.0;
const R_OUTER: f64 = 360.0; // zodiac outer ring
const R_ZODIAC: f64 = 320.0; // zodiac inner edge / house outer edge
const R_HOUSE: f64 = 230.0; // house number ring
const R_PLANET: f64 = 185.0; // planet glyph placement
const R_INNER: f64 = 100.0; // inner circle (chart center)

// Astrological colour palette
const FIRE: &str = "#e05252"; // Aries, Leo, Sagittarius
const EARTH: &str = "#7caa5a"; // Taurus, Virgo, Capricorn
const AIR: &str = "#5fa8d3"; // Gemini, Libra, Aquarius
const WATER: &str = "#9b74c4"; // Cancer, Scorpio, Pisces

fn sign_color(sign: u8) -> &'static str {
    match sign % 12 {
        0 | 4 | 8 => FIRE,   // Aries, Leo, Sagittarius
        1 | 5 | 9 => EARTH,  // Taurus, Virgo, Capricorn
        2 | 6 | 10 => AIR,   // Gemini, Libra, Aquarius
        3 | 7 | 11 => WATER, // Cancer, Scorpio, Pisces
        _ => "#888",
    }
}

fn aspect_color(deg: f64) -> &'static str {
    match deg as i32 {
        0 => "#ff6b6b",   // Conjunction — red
        60 => "#4ecdc4",  // Sextile — teal
        90 => "#ff9f43",  // Square — orange
        120 => "#54a0ff", // Trine — blue
        180 => "#a29bfe", // Opposition — purple
        _ => "#aaa",
    }
}

/// Convert ecliptic longitude to SVG angle (degrees, clockwise from right).
/// The chart is drawn with ASC on the left (9 o'clock position).
/// Ecliptic goes counter-clockwise; SVG angles go clockwise.
fn ecl_to_svg_angle(lon: f64, asc: f64) -> f64 {
    // Rotate so ASC is at 180° (left/9 o'clock)
    let rel = (lon - asc + 360.0).rem_euclid(360.0);
    // Flip: ecliptic is CCW, SVG is CW
    (180.0 - rel).rem_euclid(360.0)
}

/// Polar → SVG x,y at radius r from centre.
fn polar(angle_deg: f64, r: f64) -> (f64, f64) {
    let a = angle_deg * PI / 180.0;
    (CX + r * a.cos(), CY + r * a.sin())
}

/// SVG arc path string (large-arc flag computed automatically).
#[allow(dead_code)]
fn arc_path(r: f64, a1: f64, a2: f64) -> String {
    let (x1, y1) = polar(a1, r);
    let (x2, y2) = polar(a2, r);
    let span = (a2 - a1).rem_euclid(360.0);
    let large = if span > 180.0 { 1 } else { 0 };
    format!("M {x1:.2} {y1:.2} A {r:.2} {r:.2} 0 {large} 1 {x2:.2} {y2:.2}")
}

pub fn render_svg(chart: &ChartData, name: &str) -> String {
    let asc = chart.asc;
    let mut s = String::with_capacity(64 * 1024);

    write_chart_header(&mut s);
    write_zodiac_ring(&mut s, asc);
    write_zodiac_ticks(&mut s, asc);
    write_house_cusps(&mut s, chart, asc);
    write_house_numbers(&mut s, chart, asc);
    write_aspects_section(&mut s, chart, asc);
    write_planets_section(&mut s, chart, asc);
    write_angle_labels(&mut s, chart, asc);
    write_ring_borders(&mut s);
    write_centre_metadata(&mut s, chart, name);

    s.push_str("</svg>\n");
    s
}

fn write_chart_header(s: &mut String) {
    s.push_str(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 800" width="800" height="800"
     font-family="Georgia, 'DejaVu Serif', serif">
  <defs>
    <style>
      text { font-family: inherit; }
      .glyph { font-size: 18px; text-anchor: middle; dominant-baseline: central; }
      .sign-glyph { font-size: 20px; text-anchor: middle; dominant-baseline: central; }
      .house-num { font-size: 13px; text-anchor: middle; dominant-baseline: central; fill: #555; }
      .angle-lbl { font-size: 12px; text-anchor: middle; dominant-baseline: central;
                    font-weight: bold; }
      .degree { font-size: 9px; text-anchor: middle; dominant-baseline: central; fill: #666; }
      .title { font-size: 13px; text-anchor: middle; fill: #333; }
      .subtitle { font-size: 11px; text-anchor: middle; fill: #666; }
    </style>
  </defs>
  <!-- Background -->
  <rect width="800" height="800" fill="#fafaf7"/>
"##,
    );
}

fn write_zodiac_ring(s: &mut String, asc: f64) {
    for sign in 0u8..12 {
        let lon_start = sign as f64 * 30.0;
        let lon_end = lon_start + 30.0;
        let a1 = ecl_to_svg_angle(lon_start, asc);
        let a2 = ecl_to_svg_angle(lon_end, asc);

        let (ox1, oy1) = polar(a1, R_OUTER);
        let (ox2, oy2) = polar(a2, R_OUTER);
        let (ix2, iy2) = polar(a2, R_ZODIAC);
        let (ix1, iy1) = polar(a1, R_ZODIAC);
        let span = (a2 - a1).rem_euclid(360.0);
        let large = if span > 180.0 { 1 } else { 0 };
        let color = sign_color(sign);
        let _ = write!(
            s,
            "  <path d=\"M {ox1:.2} {oy1:.2} A {R_OUTER:.0} {R_OUTER:.0} 0 {large} 1 {ox2:.2} {oy2:.2} \
             L {ix2:.2} {iy2:.2} A {R_ZODIAC:.0} {R_ZODIAC:.0} 0 {large} 0 {ix1:.2} {iy1:.2} Z\" \
             fill=\"{color}\" fill-opacity=\"0.18\" stroke=\"{color}\" stroke-width=\"0.5\"/>\n"
        );

        let mid_a = ecl_to_svg_angle(lon_start + 15.0, asc);
        let r_text = (R_OUTER + R_ZODIAC) / 2.0;
        let (tx, ty) = polar(mid_a, r_text);
        let glyph = sign_glyph(sign);
        let _ = write!(
            s,
            "  <text x=\"{tx:.2}\" y=\"{ty:.2}\" class=\"sign-glyph\" fill=\"{color}\">{glyph}</text>\n"
        );
    }
}

fn write_zodiac_ticks(s: &mut String, asc: f64) {
    for deg in 0..360 {
        let a = ecl_to_svg_angle(deg as f64, asc);
        let (r_in, stroke, sw) = if deg % 10 == 0 {
            (R_OUTER - 12.0, "#666", "1.0")
        } else if deg % 5 == 0 {
            (R_OUTER - 8.0, "#999", "0.7")
        } else {
            (R_OUTER - 4.0, "#bbb", "0.5")
        };
        let (x1, y1) = polar(a, R_OUTER);
        let (x2, y2) = polar(a, r_in);
        let _ = write!(
            s,
            "  <line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" \
             stroke=\"{stroke}\" stroke-width=\"{sw}\"/>\n"
        );
    }
}

fn write_house_cusps(s: &mut String, chart: &ChartData, asc: f64) {
    for (i, &cusp_lon) in chart.cusps[1..=12]
        .iter()
        .enumerate()
        .map(|(i, v)| (i + 1, v))
    {
        let a = ecl_to_svg_angle(cusp_lon, asc);
        let (x1, y1) = polar(a, R_ZODIAC);
        let (x2, y2) = polar(a, R_INNER);
        let is_angle = i == 1 || i == 4 || i == 7 || i == 10;
        let (stroke, sw) = if is_angle {
            ("#333", "1.8")
        } else {
            ("#999", "0.8")
        };
        let _ = write!(
            s,
            "  <line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" \
             stroke=\"{stroke}\" stroke-width=\"{sw}\"/>\n"
        );
    }
}

fn write_house_numbers(s: &mut String, chart: &ChartData, asc: f64) {
    for i in 1usize..=12 {
        let next = if i == 12 { 1 } else { i + 1 };
        let c1 = chart.cusps[i];
        let c2 = chart.cusps[next];
        let span = (c2 - c1).rem_euclid(360.0);
        let mid_lon = c1 + span / 2.0;
        let a = ecl_to_svg_angle(mid_lon, asc);
        let (tx, ty) = polar(a, (R_HOUSE + R_INNER) / 2.0);
        let _ = write!(
            s,
            "  <text x=\"{tx:.2}\" y=\"{ty:.2}\" class=\"house-num\">{i}</text>\n"
        );
    }
}

fn write_aspects_section(s: &mut String, chart: &ChartData, asc: f64) {
    let positions: Vec<(Body, f64, f64)> = chart
        .planets
        .iter()
        .map(|p| (p.body, p.lon, p.speed))
        .collect();
    let aspects = calc_chart_aspects(&positions, MAJOR_ASPECTS, 8.0);

    s.push_str("  <g opacity=\"0.35\">\n");
    for asp in &aspects {
        let Some(p1) = chart.planets.iter().find(|p| p.body == asp.body1) else { continue };
        let Some(p2) = chart.planets.iter().find(|p| p.body == asp.body2) else { continue };
        let a1 = ecl_to_svg_angle(p1.lon, asc);
        let a2 = ecl_to_svg_angle(p2.lon, asc);
        let (x1, y1) = polar(a1, R_INNER - 5.0);
        let (x2, y2) = polar(a2, R_INNER - 5.0);
        let color = aspect_color(asp.aspect);
        let opacity = 1.0 - (asp.orb / 8.0).min(0.85);
        let _ = write!(
            s,
            "  <line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" \
             stroke=\"{color}\" stroke-width=\"1.2\" opacity=\"{opacity:.2}\"/>\n"
        );
    }
    s.push_str("  </g>\n");
}

fn nudge_planet_angle(base_a: f64, placed: &[(f64, f64)]) -> f64 {
    let mut a = base_a;
    for _ in 0..8 {
        let conflict = placed.iter().any(|&(pa, _)| {
            let diff = (a - pa).abs().min(360.0 - (a - pa).abs());
            diff < 8.0
        });
        if !conflict {
            return a;
        }
        a = (a + 9.0).rem_euclid(360.0);
    }
    a
}

fn write_planets_section(s: &mut String, chart: &ChartData, asc: f64) {
    let mut placed: Vec<(f64, f64)> = Vec::new();
    for planet in &chart.planets {
        let base_a = ecl_to_svg_angle(planet.lon, asc);
        let a = nudge_planet_angle(base_a, &placed);
        placed.push((a, R_PLANET));

        let (gx, gy) = polar(a, R_PLANET);
        let (dx, dy) = polar(base_a, R_ZODIAC - 6.0);
        let _ = write!(
            s,
            "  <circle cx=\"{dx:.2}\" cy=\"{dy:.2}\" r=\"2.5\" fill=\"#333\"/>\n"
        );

        if (a - base_a).abs() > 1.0 {
            let (lx1, ly1) = polar(base_a, R_ZODIAC - 14.0);
            let (lx2, ly2) = polar(a, R_PLANET + 14.0);
            let _ = write!(
                s,
                "  <line x1=\"{lx1:.2}\" y1=\"{ly1:.2}\" x2=\"{lx2:.2}\" y2=\"{ly2:.2}\" \
                 stroke=\"#aaa\" stroke-width=\"0.6\"/>\n"
            );
        }

        let fill = if planet.retro { "#c0392b" } else { "#1a1a2e" };
        let retro = if planet.retro { " ℞" } else { "" };
        let _ = write!(
            s,
            "  <text x=\"{gx:.2}\" y=\"{gy:.2}\" class=\"glyph\" fill=\"{fill}\">{}{retro}</text>\n",
            planet.glyph
        );

        let (sign, deg) = lon_to_sign(planet.lon);
        let deg_label = format!("{:.0}°{}", deg, sign_glyph(sign));
        let (lx, ly) = polar(a, R_PLANET - 18.0);
        let _ = write!(
            s,
            "  <text x=\"{lx:.2}\" y=\"{ly:.2}\" class=\"degree\">{deg_label}</text>\n"
        );
    }
}

fn write_angle_labels(s: &mut String, chart: &ChartData, asc: f64) {
    for (lon, label) in [
        (chart.asc, "ASC"),
        (chart.mc, "MC"),
        (chart.ic, "IC"),
        (chart.dsc, "DSC"),
    ] {
        let a = ecl_to_svg_angle(lon, asc);
        let (x, y) = polar(a, R_ZODIAC + 22.0);
        let _ = write!(
            s,
            "  <text x=\"{x:.2}\" y=\"{y:.2}\" class=\"angle-lbl\" fill=\"#333\">{label}</text>\n"
        );
    }
}

fn write_ring_borders(s: &mut String) {
    for (r, stroke, sw) in [
        (R_OUTER, "#555", "2.0"),
        (R_ZODIAC, "#777", "1.2"),
        (R_INNER, "#aaa", "1.0"),
    ] {
        let _ = write!(
            s,
            "  <circle cx=\"{CX}\" cy=\"{CY}\" r=\"{r}\" \
             fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{sw}\"/>\n"
        );
    }
}

fn write_centre_metadata(s: &mut String, chart: &ChartData, name: &str) {
    let date_str = parse::jd_to_str(chart.jd);
    let lat_dir = if chart.lat >= 0.0 { "N" } else { "S" };
    let lon_dir = if chart.lon >= 0.0 { "E" } else { "W" };
    let loc_str = format!(
        "{:.2}°{} {:.2}°{}",
        chart.lat.abs(),
        lat_dir,
        chart.lon.abs(),
        lon_dir
    );

    if !name.is_empty() {
        let _ = write!(
            s,
            "  <text x=\"{CX}\" y=\"{:.1}\" class=\"title\">{}</text>\n",
            CY - 18.0,
            name
        );
    }
    let _ = write!(
        s,
        "  <text x=\"{CX}\" y=\"{:.1}\" class=\"subtitle\">{date_str}</text>\n",
        CY
    );
    let _ = write!(
        s,
        "  <text x=\"{CX}\" y=\"{:.1}\" class=\"subtitle\">{loc_str}</text>\n",
        CY + 16.0
    );
    let _ = write!(
        s,
        "  <text x=\"{CX}\" y=\"{:.1}\" class=\"subtitle\">{}</text>\n",
        CY + 32.0,
        parse::hsys_name(chart.hsys)
    );
}

// ─── Text table output ────────────────────────────────────────────────────────

fn print_table(chart: &ChartData, name: &str) {
    print_table_header(chart, name);
    print_table_angles(chart);
    print_table_planets(chart);
    print_table_houses(chart);
    print_table_aspects(chart);
    eprintln!();
}

fn print_table_header(chart: &ChartData, name: &str) {
    let date_str = parse::jd_to_str(chart.jd);
    eprintln!();
    if !name.is_empty() {
        eprintln!("  {name}");
    }
    eprintln!("  {}  ·  JD {:.4}", date_str, chart.jd);
    let lat_dir = if chart.lat >= 0.0 { "N" } else { "S" };
    let lon_dir = if chart.lon >= 0.0 { "E" } else { "W" };
    eprintln!(
        "  {:.4}°{}  {:.4}°{}  ·  {}",
        chart.lat.abs(),
        lat_dir,
        chart.lon.abs(),
        lon_dir,
        parse::hsys_name(chart.hsys)
    );
}

fn print_table_angles(chart: &ChartData) {
    eprintln!("\n  {} Angles {}", fmt::rule(22), fmt::rule(22));
    for (label, lon) in [
        ("ASC", chart.asc),
        ("MC", chart.mc),
        ("IC", chart.ic),
        ("DSC", chart.dsc),
    ] {
        eprintln!("  {:<6}  {}", label, fmt::lon_zodiac(lon));
    }
}

fn print_table_planets(chart: &ChartData) {
    eprintln!("\n  {} Planets {}", fmt::rule(20), fmt::rule(20));
    eprintln!("  {:<11} {:<16} {:<8} Speed", "Body", "Longitude", "Lat");
    eprintln!("  {}", fmt::rule(52));
    for p in &chart.planets {
        let retro = if p.retro { " ℞" } else { "  " };
        eprintln!(
            "  {:<11} {}{:<14} {:<8} {}",
            p.name,
            retro,
            fmt::lon_zodiac(p.lon),
            fmt::deg_dms(p.lat),
            fmt::speed_dday(p.speed),
        );
    }
}

fn print_table_houses(chart: &ChartData) {
    eprintln!(
        "\n  {} Houses ({}) {}",
        fmt::rule(14),
        parse::hsys_name(chart.hsys),
        fmt::rule(14)
    );
    for i in 1..=12 {
        eprintln!("  House {:2}  {}", i, fmt::lon_zodiac(chart.cusps[i]));
    }
}

fn aspect_short_name(d: f64) -> &'static str {
    match d as i32 {
        0 => "Conj",
        60 => "Sext",
        90 => "Sqre",
        120 => "Trin",
        180 => "Oppo",
        _ => "Asp",
    }
}

fn print_table_aspects(chart: &ChartData) {
    let positions: Vec<(Body, f64, f64)> = chart
        .planets
        .iter()
        .map(|p| (p.body, p.lon, p.speed))
        .collect();
    let aspects = calc_chart_aspects(&positions, MAJOR_ASPECTS, 8.0);
    if aspects.is_empty() {
        return;
    }
    eprintln!("\n  {} Aspects {}", fmt::rule(20), fmt::rule(20));
    for a in &aspects {
        let app = if a.applying { "Apl" } else { "Sep" };
        eprintln!(
            "  {:<11} {} {:<5}  orb {:.2}°  {}",
            parse::body_name(a.body1),
            aspect_short_name(a.aspect),
            parse::body_name(a.body2),
            a.orb,
            app,
        );
    }
}

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn run(args: ChartArgs) -> Result<(), CliError> {
    // Merge --time into --date if provided and --date has no time component
    let date_str = if let Some(ref t) = args.time {
        let base = args.date.trim();
        if base == "now" || base.parse::<f64>().is_ok() || base.contains(' ') {
            args.date.clone() // already has time or is JD/now — ignore --time
        } else {
            format!("{base} {t}") // append time to bare date
        }
    } else {
        args.date.clone()
    };
    let jd = parse::parse_date(&date_str)?;
    let hsys = parse::parse_hsys(&args.system)?;
    let chart = compute_chart(jd, args.lat, args.lon, hsys)?;

    // Always print JSON to stdout
    print_json(&chart);

    // Also print the text table to stderr so JSON stays pipe-clean
    eprintln!();
    print_table(&chart, &args.name);

    // Write SVG if requested
    if let Some(svg_path) = &args.svg {
        let svg = render_svg(&chart, &args.name);
        std::fs::write(svg_path, &svg).map_err(|e| format!("failed to write SVG: {e}"))?;
        eprintln!("  SVG chart written to: {svg_path}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn time_flag_merges_with_date() {
        // --date 2000-01-01 --time 14:30 should produce the same JD as
        // --date "2000-01-01 14:30"
        let jd_combined = parse::parse_date("2000-01-01 14:30").unwrap();
        let jd_separate = {
            let base = "2000-01-01";
            let time = "14:30";
            parse::parse_date(&format!("{base} {time}")).unwrap()
        };
        assert!(
            (jd_combined - jd_separate).abs() < 1e-9,
            "combined={jd_combined} separate={jd_separate}"
        );
    }

    #[test]
    fn time_flag_ignored_when_date_has_time() {
        // If --date already has a time component, --time should be ignored
        let args = ChartArgs {
            date: "2000-01-01 12:00".to_string(),
            time: Some("14:30".to_string()),
            lat: 0.0,
            lon: 0.0,
            system: "placidus".to_string(),
            svg: None,
            name: String::new(),
        };
        let date_str = if let Some(ref t) = args.time {
            let base = args.date.trim();
            if base.contains(' ') {
                args.date.clone()
            } else {
                format!("{base} {t}")
            }
        } else {
            args.date.clone()
        };
        // Should keep the original "12:00", not overwrite with "14:30"
        assert!(date_str.contains("12:00"), "date_str={date_str}");
    }

    #[test]
    fn time_flag_ignored_for_now() {
        let args = ChartArgs {
            date: "now".to_string(),
            time: Some("14:30".to_string()),
            lat: 0.0,
            lon: 0.0,
            system: "placidus".to_string(),
            svg: None,
            name: String::new(),
        };
        let date_str = if let Some(ref t) = args.time {
            let base = args.date.trim();
            if base == "now" || base.parse::<f64>().is_ok() || base.contains(' ') {
                args.date.clone()
            } else {
                format!("{base} {t}")
            }
        } else {
            args.date.clone()
        };
        assert_eq!(date_str, "now");
    }
}
