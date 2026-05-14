//! `render_builtin_svg` — renders the built-in Western chart SVG from the
//! JSON context produced by [`super::build_context`].
//!
//! Extracted from `mod.rs` to reduce that file's size.

use std::fmt::Write as FmtWrite;

use serde_json::Value;

use super::{spread_labels, wx, wy, CX, CY, RH, RI, RO, RP};

const LABEL_R: f64 = RP + 26.0;
const RH2: f64 = 16.0;

const DIG_COLORS: [(&str, &str); 5] = [
    ("domicile", "#1a7a1a"),
    ("exaltation", "#0d5ca8"),
    ("detriment", "#b01020"),
    ("fall", "#8b4000"),
    ("peregrine", "#888888"),
];

struct Palette<'a> {
    bg: &'a str,
    ring: &'a str,
    pfg: &'a str,
    retro_c: &'a str,
    hard_c: &'a str,
    soft_c: &'a str,
    txt: &'a str,
    title: &'a str,
}

impl<'a> Palette<'a> {
    fn from(vars: &'a Value) -> Self {
        Self {
            bg: vars["bg_color"].as_str().unwrap_or("#ffffff"),
            ring: vars["ring_color"].as_str().unwrap_or("#1a1a2e"),
            pfg: vars["planet_color"].as_str().unwrap_or("#0d0d1e"),
            retro_c: vars["retro_color"].as_str().unwrap_or("#b01020"),
            hard_c: vars["hard_color"].as_str().unwrap_or("#b01020"),
            soft_c: vars["soft_color"].as_str().unwrap_or("#1a50b0"),
            txt: vars["text_color"].as_str().unwrap_or("#0d0d1e"),
            title: vars["title"].as_str().unwrap_or("Celestial Chart"),
        }
    }
}

struct Angles {
    asc: f64,
    mc: f64,
    ic: f64,
    dsc: f64,
}

/// Header metadata threaded through `write_header` / `write_loc_line`.
/// Bundles the values that historically lived in the centre moon-phase
/// disc so they can be folded into the chart subtitle row instead of
/// obstructing the inner aspect cavity.
struct ChartHeader<'a> {
    date: &'a str,
    jd: f64,
    lat: f64,
    lon: f64,
    phase: &'a str,
    illum: f64,
}

pub(crate) fn render_builtin_svg(ctx: &Value) -> String {
    let pal = Palette::from(&ctx["vars"]);
    let date = ctx["date"].as_str().unwrap_or("");
    let jd = ctx["jd"].as_f64().unwrap_or(0.0);
    let lat = ctx["lat"].as_f64().unwrap_or(0.0);
    let lon = ctx["lon"].as_f64().unwrap_or(0.0);
    let ang = Angles {
        asc: ctx["asc"].as_f64().unwrap_or(0.0),
        mc: ctx["mc"].as_f64().unwrap_or(0.0),
        ic: ctx["ic"].as_f64().unwrap_or(0.0),
        dsc: ctx["dsc"].as_f64().unwrap_or(0.0),
    };
    let header = ChartHeader {
        date,
        jd,
        lat,
        lon,
        phase: ctx["moon_phase_name"].as_str().unwrap_or(""),
        illum: ctx["moon_illumination"].as_f64().unwrap_or(0.0),
    };

    let planets = super::json_array(&ctx["planets"]);
    let signs = super::json_array(&ctx["signs"]);
    let houses = super::json_array(&ctx["houses"]);
    let aspects = super::json_array(&ctx["aspects"]);

    let mut s = String::with_capacity(64 * 1024);

    write_header(&mut s, &pal, &header);
    write_signs(&mut s, &pal, signs);
    write_houses(&mut s, &pal, houses);
    write_angle_labels(&mut s, &pal, &ang);
    write_aspects(&mut s, &pal, aspects);
    write_planets(&mut s, &pal, planets, ang.asc);

    let ly = CY + RO + 24.0;
    let c1x = 24.0_f64;
    let c2x = 314.0_f64;
    let c3x = 584.0_f64;

    write_planet_legend(&mut s, &pal, planets, c1x, ly);
    write_angles_legend(&mut s, &pal, ctx, &ang, c2x, ly);
    write_houses_legend(&mut s, &pal, houses, c2x, ly);
    write_aspects_legend(&mut s, &pal, aspects, c3x, ly);

    let dig_y = ly + 16.0 + planets.len() as f64 * RH2 + 12.0;
    write_dignities(&mut s, &pal, planets, c1x, dig_y);

    let ap_y = dig_y + 26.0 + planets.len() as f64 * RH2 + 8.0;
    write_arabic_parts(&mut s, &pal, ctx, c2x, ap_y);

    // Solar cycle box: sits in the centre column (c2x) below the Arabic
    // Parts block. The right column (c3x) holds the aspects legend which
    // grows with chart busy-ness — keeping solar cycle out of c3x avoids
    // overlap on charts with many active aspects. 7 Arabic Parts rows +
    // a 14-px header + 16-px row spacing → arabic table ends at
    // ap_y + 14 + 7·RH2 ≈ ap_y + 126; pad 20 px before solar block.
    let arabic_rows = 7.0;
    let sc_y = ap_y + 14.0 + arabic_rows * RH2 + 20.0;
    write_solar_cycle(&mut s, &pal, ctx, c2x, sc_y);

    write_footer(&mut s, &pal, date);
    s
}

fn write_header(s: &mut String, pal: &Palette, h: &ChartHeader) {
    let (bg, txt, ring, title) = (pal.bg, pal.txt, pal.ring, pal.title);
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 1720" width="900" height="1720">"##
    );
    super::glyph_paths::emit_defs(s);
    let _ = writeln!(
        s,
        r##"  <rect width="900" height="1720" fill="{bg}"/>
  <text x="450" y="34" text-anchor="middle" font-size="18" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="450" y="54" text-anchor="middle" font-size="10"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".7">"##
    );
    write_loc_line(s, h);
    let _ = writeln!(
        s,
        r##"</text>

  <!-- Three wheels separated by three concentric rings:
       · outer wheel  (signs band):  RO → RI
       · middle wheel (house band):  RI → RH
       · inner wheel  (aspect area): RH → centre
       Cusps converge at the wheel centre so no inner disc is
       needed — aspect lines and angular axes meet at (CX, CY). -->
  <circle cx="{CX}" cy="{CY}" r="{RO}" fill="none" stroke="{ring}" stroke-width="2.5" opacity=".80"/>
  <circle cx="{CX}" cy="{CY}" r="{RI}" fill="none" stroke="{ring}" stroke-width="2.0" opacity=".70"/>
  <circle cx="{CX}" cy="{CY}" r="{RH}" fill="none" stroke="{ring}" stroke-width="1.6" opacity=".55"/>"##
    );
}

fn write_loc_line(s: &mut String, h: &ChartHeader) {
    let (date, jd, lat, lon) = (h.date, h.jd, h.lat, h.lon);
    if lat == 0.0 && lon == 0.0 {
        let _ = write!(s, "{date} · JD {jd:.4}");
    } else {
        let ns = if lat >= 0.0 { "N" } else { "S" };
        let ew = if lon >= 0.0 { "E" } else { "W" };
        let _ = write!(
            s,
            "{date} · {:.4}°{ns} {:.4}°{ew} · JD {jd:.4}",
            lat.abs(),
            lon.abs()
        );
    }
    if !h.phase.is_empty() {
        let _ = write!(s, " · ☽ {} {:.1}%", h.phase, h.illum);
    }
}

fn write_signs(s: &mut String, pal: &Palette, signs: &[Value]) {
    for sign in signs {
        write_sign(s, pal, sign);
    }
}

/// Font stack for astrological glyphs — both zodiac signs
/// (U+2648..U+2653) and planet symbols (U+2609, U+263D, U+263F,
/// U+2640..U+2647, U+260A, U+26B7). We prefer fonts that ship
/// high-quality dedicated outlines for these blocks: `Segoe UI
/// Symbol` (Windows), `Apple Symbols` and `STIX Two Text` (macOS),
/// `Noto Sans Symbols2` (Linux). Falling back to generic `serif`
/// last produces the chunky bitmap-style glyph that earlier
/// revisions of the wheel suffered from.
const GLYPH_FONT_FAMILY: &str =
    "'Segoe UI Symbol','Apple Symbols','STIX Two Text','Noto Sans Symbols2','DejaVu Sans',serif";

/// Resolve the element colour for a zodiac sign glyph (`♈`..`♓`,
/// U+2648..U+2653). Returns `None` when `g` is not a sign glyph — the
/// caller then keeps the surrounding text colour.
fn sign_glyph_color(g: char) -> Option<&'static str> {
    let idx = (g as u32).checked_sub(0x2648)?;
    Some(match idx % 4 {
        0 => "#c1272d", // Fire — Aries, Leo, Sagittarius
        1 => "#5a7a30", // Earth — Taurus, Virgo, Capricorn
        2 => "#c4a017", // Air — Gemini, Libra, Aquarius
        _ => "#1a5fb4", // Water — Cancer, Scorpio, Pisces
    })
}

/// Split a degree-minute-second string (e.g. `"09°51'58\"♑︎"`) into its
/// numeric prefix and trailing sign glyph (plus VS15 text-presentation
/// selector). Returns `None` when no zodiac sign glyph is present.
fn split_dms_sign(dms: &str) -> Option<(&str, &str, &'static str)> {
    // Walk back over any trailing `U+FE0E` (text-presentation selector)
    // so the actual sign character lights up the colour lookup.
    let trimmed = dms.trim_end_matches('\u{FE0E}');
    let last = trimmed.chars().last()?;
    let col = sign_glyph_color(last)?;
    let glyph_start = trimmed.len() - last.len_utf8();
    let prefix = &dms[..glyph_start];
    let glyph = &dms[glyph_start..];
    Some((prefix, glyph, col))
}

/// Compose a `<text>` element that renders a DMS string with the
/// numeric prefix in the row's text colour + base font, and the
/// trailing sign glyph in its element colour + the symbol font stack.
/// Falls back to a single-font emission when no sign glyph is present.
fn write_dms_text(s: &mut String, x: f64, y: f64, dms: &str, txt: &str) {
    if let Some((pfx, g, gc)) = split_dms_sign(dms) {
        let _ = writeln!(
            s,
            r##"  <text x="{x:.2}" y="{y:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{pfx}<tspan font-family="{GLYPH_FONT_FAMILY}" font-size="13" fill="{gc}">{g}</tspan></text>"##
        );
    } else {
        let _ = writeln!(
            s,
            r##"  <text x="{x:.2}" y="{y:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>"##
        );
    }
}

fn write_sign(s: &mut String, pal: &Palette, sign: &Value) {
    let ring = pal.ring;
    let sx1 = sign["spoke_x1"].as_f64().unwrap_or(0.0);
    let sy1 = sign["spoke_y1"].as_f64().unwrap_or(0.0);
    let sx2 = sign["spoke_x2"].as_f64().unwrap_or(0.0);
    let sy2 = sign["spoke_y2"].as_f64().unwrap_or(0.0);
    let gx = sign["glyph_x"].as_f64().unwrap_or(0.0);
    let gy = sign["glyph_y"].as_f64().unwrap_or(0.0);
    let g = sign["glyph"].as_str().unwrap_or("");
    let col = sign["color"].as_str().unwrap_or(ring);
    let _ = writeln!(
        s,
        r##"  <line x1="{sx1:.2}" y1="{sy1:.2}" x2="{sx2:.2}" y2="{sy2:.2}" stroke="{ring}" stroke-width="1.5" opacity=".55"/>"##
    );
    emit_glyph(s, g, gx, gy, 26.0, col);
}

/// Emit a glyph at (cx, cy). Prefers the embedded `<symbol>` from
/// `glyph_paths::GLYPH_PATHS` (font-independent, identical across
/// renderers); falls back to a centred `<text>` element when the
/// code-point has no path table — keeps unusual glyphs (e.g. fixed
/// stars) renderable via the host symbol-font stack. The text-fallback
/// font-size is derived from `size` so callers don't have to pass it
/// (and `emit_glyph` stays under the clippy 7-arg ceiling).
fn emit_glyph(s: &mut String, glyph: &str, cx: f64, cy: f64, size: f64, fill: &str) {
    let cp = super::glyph_paths::lead_cp(glyph);
    if super::glyph_paths::write_use(s, cp, cx, cy, size, fill) {
        return;
    }
    // Text fallback: empirically a text font-size ≈ 0.75·size produces
    // a glyph whose ink box matches the <use> box at the same `size`.
    let fs = (size * 0.75).round() as u32;
    let _ = writeln!(
        s,
        r##"  <text x="{cx:.2}" y="{cy:.2}" font-size="{fs}" font-weight="500" text-anchor="middle" dominant-baseline="central" font-family="{GLYPH_FONT_FAMILY}" fill="{fill}">{glyph}</text>"##
    );
}

fn write_houses(s: &mut String, pal: &Palette, houses: &[Value]) {
    s.push('\n');
    for h in houses {
        write_house(s, pal, h);
    }
}

/// Per-cusp visual style. Angular cusps (1/4/7/10) get a thicker, more
/// opaque spoke and a bolder number to mark the ASC/IC/DSC/MC axes —
/// matches the World-of-Wisdom natal-chart layout.
fn house_line_style(is_angle: bool) -> (&'static str, &'static str) {
    if is_angle {
        ("3.0", ".90")
    } else {
        ("1.2", ".50")
    }
}

fn house_number_style(is_angle: bool) -> (&'static str, &'static str, &'static str) {
    if is_angle {
        ("13", "800", ".95")
    } else {
        ("12", "700", ".80")
    }
}

fn write_house(s: &mut String, pal: &Palette, h: &Value) {
    let ring = pal.ring;
    let x1 = h["x1"].as_f64().unwrap_or(0.0);
    let y1 = h["y1"].as_f64().unwrap_or(0.0);
    let x2 = h["x2"].as_f64().unwrap_or(0.0);
    let y2 = h["y2"].as_f64().unwrap_or(0.0);
    let nx = h["num_x"].as_f64().unwrap_or(0.0);
    let ny = h["num_y"].as_f64().unwrap_or(0.0);
    let n = h["num"].as_u64().unwrap_or(0);
    let is_angle = h["is_angle"].as_bool().unwrap_or(false);
    let (sw, op) = house_line_style(is_angle);
    let (nfs, nfw, nop) = house_number_style(is_angle);
    let _ = writeln!(
        s,
        r##"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{ring}" stroke-width="{sw}" opacity="{op}"/>
  <text x="{nx:.2}" y="{ny:.2}" font-size="{nfs}" font-weight="{nfw}" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity="{nop}">{n}</text>"##
    );
}

fn write_angle_labels(s: &mut String, pal: &Palette, ang: &Angles) {
    let ring = pal.ring;
    let entries = [
        (ang.asc, "ASC", "end", 0.0),
        (ang.dsc, "DSC", "start", 0.0),
        (ang.mc, "MC", "middle", -7.0),
        (ang.ic, "IC", "middle", 11.0),
    ];
    for (lon2, name, anchor, dy) in entries {
        let lx = wx(CX, RI + 18.0, lon2, ang.asc);
        let ly = wy(CY, RI + 18.0, lon2, ang.asc) + dy;
        let _ = writeln!(
            s,
            r##"  <text x="{lx:.2}" y="{ly:.2}" text-anchor="{anchor}" font-size="12" font-weight="800" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">{name}</text>"##
        );
    }
}

fn write_aspects(s: &mut String, pal: &Palette, aspects: &[Value]) {
    s.push('\n');
    for asp in aspects {
        write_aspect(s, pal, asp);
    }
}

fn aspect_style(orb: f64, minor: bool) -> (&'static str, &'static str, &'static str) {
    if minor {
        let o = if orb < 1.0 { ".35" } else { ".18" };
        ("0.9", o, r##" stroke-dasharray="4,3""##)
    } else {
        let o = if orb < 2.0 { ".55" } else { ".22" };
        ("1.8", o, "")
    }
}

fn write_aspect(s: &mut String, pal: &Palette, asp: &Value) {
    let x1 = asp["x1"].as_f64().unwrap_or(0.0);
    let y1 = asp["y1"].as_f64().unwrap_or(0.0);
    let x2 = asp["x2"].as_f64().unwrap_or(0.0);
    let y2 = asp["y2"].as_f64().unwrap_or(0.0);
    let orb = asp["orb"].as_f64().unwrap_or(8.0);
    let hard = asp["is_hard"].as_bool().unwrap_or(false);
    let minor = asp["is_minor"].as_bool().unwrap_or(false);
    let col = if hard { pal.hard_c } else { pal.soft_c };
    let (sw, op, dash) = aspect_style(orb, minor);
    let _ = writeln!(
        s,
        r##"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{col}" stroke-width="{sw}" opacity="{op}"{dash}/>"##
    );
}

fn write_planets(s: &mut String, pal: &Palette, planets: &[Value], asc: f64) {
    s.push('\n');
    let planet_lons: Vec<f64> = planets
        .iter()
        .map(|p| p["lon"].as_f64().unwrap_or(0.0))
        .collect();
    let label_angles = spread_labels(&planet_lons, asc);
    for (idx, p) in planets.iter().enumerate() {
        write_planet(s, pal, p, planet_lons[idx], label_angles[idx], asc);
    }
}

fn normalize_drift(mut drift: f64) -> f64 {
    while drift > 180.0 {
        drift -= 360.0;
    }
    while drift < -180.0 {
        drift += 360.0;
    }
    drift
}

/// Pick the foreground colour for a planet glyph. Defaults to the
/// per-body palette from `BODY_COLORS`; falls back to the chart's
/// generic planet colour if the body has no entry. Retrograde state is
/// signalled via the trailing ℞ in `deg_label`, not via colour
/// substitution — so each planet keeps its traditional colour even when
/// going retrograde (matches the PDF reference style).
fn planet_color<'a>(p: &'a Value, fallback: &'a str) -> &'a str {
    p["color"].as_str().unwrap_or(fallback)
}

fn write_planet(s: &mut String, pal: &Palette, p: &Value, lon_i: f64, placed_ang: f64, asc: f64) {
    let pfg = pal.pfg;
    let px = p["x"].as_f64().unwrap_or(0.0);
    let py = p["y"].as_f64().unwrap_or(0.0);
    let tx1 = p["tick_x1"].as_f64().unwrap_or(0.0);
    let ty1 = p["tick_y1"].as_f64().unwrap_or(0.0);
    let tx2 = p["tick_x2"].as_f64().unwrap_or(0.0);
    let ty2 = p["tick_y2"].as_f64().unwrap_or(0.0);
    let g = p["glyph"].as_str().unwrap_or("?");
    let dl = p["deg_label"].as_str().unwrap_or("");
    let col = planet_color(p, pfg);

    let lx = CX + LABEL_R * placed_ang.to_radians().cos();
    let ly = CY - LABEL_R * placed_ang.to_radians().sin();

    let nat_ang = super::wheel_angle(lon_i, asc);
    let anchor_r = RP + 13.0;
    let ax = CX + anchor_r * nat_ang.to_radians().cos();
    let ay = CY - anchor_r * nat_ang.to_radians().sin();

    let drift = normalize_drift(placed_ang - nat_ang);
    if drift.abs() > 3.5 {
        let _ = writeln!(
            s,
            r##"  <line x1="{ax:.2}" y1="{ay:.2}" x2="{lx:.2}" y2="{ly:.2}" stroke="{col}" stroke-width="0.9" opacity=".45" stroke-dasharray="3,2"/>"##
        );
    }

    let _ = writeln!(
        s,
        r##"  <line x1="{tx1:.2}" y1="{ty1:.2}" x2="{tx2:.2}" y2="{ty2:.2}" stroke="{col}" stroke-width="1.0" opacity=".55"/>"##
    );
    emit_glyph(s, g, px, py, 24.0, col);
    let _ = writeln!(
        s,
        r##"  <text x="{lx:.2}" y="{ly:.2}" font-size="10" font-weight="600" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}">{dl}</text>"##
    );
    if p["near_station"].as_bool().unwrap_or(false) {
        let sx = px + 9.0;
        let sy = py - 9.0;
        let _ = writeln!(
            s,
            r##"  <text x="{sx:.2}" y="{sy:.2}" font-size="7" font-weight="700" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}" opacity=".9">S</text>"##
        );
    }
}

fn write_planet_legend(s: &mut String, pal: &Palette, planets: &[Value], c1x: f64, ly: f64) {
    let ring = pal.ring;
    let _ = writeln!(
        s,
        r##"  <text x="{c1x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Planets</text>
  <line x1="{c1x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"##,
        ly + 3.0,
        c1x + 272.0,
        ly + 3.0
    );
    for (i, p) in planets.iter().enumerate() {
        write_planet_legend_row(s, pal, p, c1x, ly + 16.0 + i as f64 * RH2);
    }
}

fn write_planet_legend_row(s: &mut String, pal: &Palette, p: &Value, c1x: f64, ry: f64) {
    let (ring, pfg, retro_c, txt) = (pal.ring, pal.pfg, pal.retro_c, pal.txt);
    let g = p["glyph"].as_str().unwrap_or("?");
    let name = p["name"].as_str().unwrap_or("");
    let dms = p["dms"].as_str().unwrap_or("");
    let spd = p["speed_str"].as_str().unwrap_or("");
    let ret = p["retro"].as_bool().unwrap_or(false);
    let col = if ret { retro_c } else { pfg };
    let scol = if ret { retro_c } else { ring };
    let sop = if ret { "1" } else { ".4" };
    emit_glyph(s, g, c1x + 2.0, ry, 18.0, col);
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{ry:.2}" font-size="11" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".75">{name}</text>"##,
        c1x + 20.0,
    );
    write_dms_text(s, c1x + 120.0, ry, dms, txt);
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{scol}" opacity="{sop}">{spd}</text>"##,
        c1x + 222.0
    );
}

fn write_angles_legend(
    s: &mut String,
    pal: &Palette,
    ctx: &Value,
    ang: &Angles,
    c2x: f64,
    ly: f64,
) {
    let ring = pal.ring;
    let _ = writeln!(
        s,
        r##"
  <text x="{c2x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Angles &amp; Houses</text>
  <line x1="{c2x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"##,
        ly + 3.0,
        c2x + 250.0,
        ly + 3.0
    );
    let entries = [
        ("ASC", ang.asc, ctx["asc_dms"].as_str().unwrap_or("")),
        ("MC", ang.mc, ctx["mc_dms"].as_str().unwrap_or("")),
        ("DSC", ang.dsc, ctx["dsc_dms"].as_str().unwrap_or("")),
        ("IC", ang.ic, ctx["ic_dms"].as_str().unwrap_or("")),
    ];
    for (i, (name, lon2, dms)) in entries.iter().enumerate() {
        write_angle_legend_row(s, pal, name, *lon2, dms, c2x, ly + 16.0 + i as f64 * RH2);
    }
}

fn write_angle_legend_row(
    s: &mut String,
    pal: &Palette,
    name: &str,
    lon2: f64,
    dms: &str,
    c2x: f64,
    ry: f64,
) {
    let (ring, txt) = (pal.ring, pal.txt);
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{ry:.2}" font-size="10" font-weight="700" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">{name}</text>"##,
        c2x + 2.0,
    );
    write_dms_text(s, c2x + 38.0, ry, dms, txt);
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{lon2:.4}&#176;</text>"##,
        c2x + 150.0
    );
}

fn write_houses_legend(s: &mut String, pal: &Palette, houses: &[Value], c2x: f64, ly: f64) {
    let ring = pal.ring;
    let sep_y = ly + 82.0;
    let _ = writeln!(
        s,
        r##"  <line x1="{c2x}" y1="{sep_y:.2}" x2="{:.2}" y2="{sep_y:.2}" stroke="{ring}" stroke-width=".3" opacity=".2"/>"##,
        c2x + 250.0
    );
    for (i, h) in houses.iter().enumerate() {
        write_house_legend_row(s, pal, h, i, c2x, ly + 94.0 + i as f64 * RH2);
    }
}

fn write_house_legend_row(s: &mut String, pal: &Palette, h: &Value, i: usize, c2x: f64, ry: f64) {
    let (ring, txt) = (pal.ring, pal.txt);
    let dms = h["dms"].as_str().unwrap_or("");
    let hlon = h["lon"].as_f64().unwrap_or(0.0);
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".55">H{}</text>"##,
        c2x + 2.0,
        i + 1,
    );
    write_dms_text(s, c2x + 28.0, ry, dms, txt);
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{hlon:.4}&#176;</text>"##,
        c2x + 138.0
    );
}

fn write_aspects_legend(s: &mut String, pal: &Palette, aspects: &[Value], c3x: f64, ly: f64) {
    let (ring, txt) = (pal.ring, pal.txt);
    let _ = writeln!(
        s,
        r##"
  <text x="{c3x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Aspects</text>
  <line x1="{c3x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"##,
        ly + 3.0,
        c3x + 292.0,
        ly + 3.0
    );
    if aspects.is_empty() {
        let _ = writeln!(
            s,
            r##"  <text x="{:.2}" y="{:.2}" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">(no major aspects within orbs)</text>"##,
            c3x + 4.0,
            ly + 20.0
        );
    }
    for (i, asp) in aspects.iter().enumerate() {
        write_aspect_legend_row(s, pal, asp, c3x, ly + 16.0 + i as f64 * 15.0);
    }
}

fn write_aspect_legend_row(s: &mut String, pal: &Palette, asp: &Value, c3x: f64, ry: f64) {
    let (ring, txt) = (pal.ring, pal.txt);
    let g1 = asp["glyph1"].as_str().unwrap_or("?");
    let g2 = asp["glyph2"].as_str().unwrap_or("?");
    let aname = asp["aspect_name"].as_str().unwrap_or("");
    let orb = asp["orb"].as_f64().unwrap_or(0.0);
    let appl = asp["applying"].as_bool().unwrap_or(false);
    let b1 = asp["body1"].as_str().unwrap_or("");
    let b2 = asp["body2"].as_str().unwrap_or("");
    let hard = asp["is_hard"].as_bool().unwrap_or(false);
    let col = if hard { pal.hard_c } else { pal.soft_c };
    let aind = if appl { "&#9650;app" } else { "&#9660;sep" };
    let b1s = &b1[..b1.len().min(3)];
    let b2s = &b2[..b2.len().min(3)];
    let an4 = &aname[..aname.len().min(4)];
    emit_glyph(s, g1, c3x + 2.0, ry, 16.0, col);
    emit_glyph(s, g2, c3x + 18.0, ry, 16.0, col);
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}">{an4}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".7">{orb:.2}&#176;</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".5">{aind}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{b1s}&#8211;{b2s}</text>"##,
        c3x + 34.0,
        c3x + 92.0,
        c3x + 130.0,
        c3x + 168.0
    );
}

fn write_dignities(s: &mut String, pal: &Palette, planets: &[Value], c1x: f64, dig_y: f64) {
    let ring = pal.ring;
    let _ = writeln!(
        s,
        r##"  <text x="{c1x}" y="{dig_y:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Essential Dignities</text>
  <line x1="{c1x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"##,
        dig_y + 3.0,
        c1x + 272.0,
        dig_y + 3.0
    );
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{:.2}" font-size="8" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">Planet</text>
  <text x="{:.2}" y="{:.2}" font-size="8" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">Dignity</text>
  <text x="{:.2}" y="{:.2}" font-size="8" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">Sign</text>"##,
        c1x + 2.0,
        dig_y + 14.0,
        c1x + 60.0,
        dig_y + 14.0,
        c1x + 140.0,
        dig_y + 14.0
    );
    for (i, p) in planets.iter().enumerate() {
        write_dignity_row(s, pal, p, c1x, dig_y + 26.0 + i as f64 * RH2);
    }
}

fn dignity_color(dig: &str) -> &'static str {
    DIG_COLORS
        .iter()
        .find(|(d, _)| *d == dig)
        .map_or("#888", |(_, c)| *c)
}

fn write_dignity_row(s: &mut String, pal: &Palette, p: &Value, c1x: f64, ry: f64) {
    let (ring, txt) = (pal.ring, pal.txt);
    let g = p["glyph"].as_str().unwrap_or("?");
    let name = p["name"].as_str().unwrap_or("");
    let dig = p["dignity"].as_str().unwrap_or("peregrine");
    let sign_nm = p["sign_name"].as_str().unwrap_or("");
    let dcol = dignity_color(dig);
    emit_glyph(s, g, c1x + 2.0, ry, 16.0, ring);
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{dcol}" font-weight="500">{dig}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{sign_nm}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".45">{name}</text>"##,
        c1x + 20.0,
        c1x + 140.0,
        c1x + 220.0,
    );
}

fn write_arabic_parts(s: &mut String, pal: &Palette, ctx: &Value, c2x: f64, ap_y: f64) {
    let ring = pal.ring;
    let _ = writeln!(
        s,
        r##"  <text x="{c2x}" y="{ap_y:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Arabic Parts</text>
  <line x1="{c2x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"##,
        ap_y + 3.0,
        c2x + 250.0,
        ap_y + 3.0
    );
    let ap_vec = super::json_array(&ctx["arabic_parts"]);
    for (i, p) in ap_vec.iter().enumerate() {
        write_arabic_part_row(s, pal, p, c2x, ap_y + 14.0 + i as f64 * RH2);
    }
}

fn write_arabic_part_row(s: &mut String, pal: &Palette, p: &Value, c2x: f64, ry: f64) {
    let (ring, txt, soft_c) = (pal.ring, pal.txt, pal.soft_c);
    let name = p["name"].as_str().unwrap_or("");
    let dms = p["dms"].as_str().unwrap_or("");
    let sign_nm = p["sign"].as_str().unwrap_or("");
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{soft_c}" opacity=".8">{name}</text>"##,
        c2x + 2.0,
    );
    write_dms_text(s, c2x + 125.0, ry, dms, txt);
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".45">{sign_nm}</text>"##,
        c2x + 210.0,
    );
}

/// Draws a compact two-column key/value table summarising the Solar
/// (Schwabe) cycle context for the chart's date. Renders nothing when
/// `ctx["solar_cycle"]` is an empty object (i.e. date outside the
/// numbered cycles AND outside any grand epoch).
fn write_solar_cycle(s: &mut String, pal: &Palette, ctx: &Value, x: f64, y: f64) {
    let sc = &ctx["solar_cycle"];
    if !sc.is_object() || sc.as_object().is_some_and(serde_json::Map::is_empty) {
        return;
    }
    let (ring, txt, soft_c) = (pal.ring, pal.txt, pal.soft_c);

    let _ = writeln!(
        s,
        r##"  <text x="{x}" y="{y:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Solar Cycle</text>
  <line x1="{x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"##,
        y + 3.0,
        x + 250.0,
        y + 3.0
    );

    // Build the row list dynamically: numbered-cycle fields when known,
    // grand-epoch fallback otherwise. Each row is (label, value).
    let mut rows: Vec<(String, String)> = Vec::new();

    if let Some(n) = sc.get("cycle_num").and_then(serde_json::Value::as_u64) {
        let mut head = format!("Cycle {n}");
        if let Some(nick) = sc.get("nickname").and_then(serde_json::Value::as_str) {
            head.push_str(&format!(" — {nick}"));
        }
        rows.push(("Number".into(), head));
    }
    if let Some(phase) = sc.get("phase_name").and_then(serde_json::Value::as_str) {
        rows.push(("Phase".into(), phase.into()));
    }
    if let Some(yrs) = sc.get("years_since_min").and_then(serde_json::Value::as_f64) {
        rows.push(("Years since min".into(), format!("{yrs:.1}")));
    }
    if let Some(p) = sc.get("phase").and_then(serde_json::Value::as_f64) {
        rows.push(("Cycle fraction".into(), format!("{:.2}", p)));
    }
    if let Some(g) = sc.get("grand_epoch").and_then(serde_json::Value::as_str) {
        rows.push(("Grand epoch".into(), g.into()));
    }

    if rows.is_empty() {
        return;
    }

    for (i, (label, value)) in rows.iter().enumerate() {
        let ry = y + 14.0 + i as f64 * RH2;
        let _ = writeln!(
            s,
            r##"  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{soft_c}" opacity=".8">{label}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{value}</text>"##,
            x + 2.0,
            x + 125.0,
        );
    }
}

fn write_footer(s: &mut String, pal: &Palette, date: &str) {
    let ring = pal.ring;
    let _ = writeln!(
        s,
        r##"
  <text x="450" y="1700" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif"
        fill="{ring}" opacity=".35">Generated by celestial render · {date}</text>
</svg>"##
    );
}
