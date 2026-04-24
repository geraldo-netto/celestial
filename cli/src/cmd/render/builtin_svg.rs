//! `render_builtin_svg` — renders the built-in Western chart SVG from the
//! JSON context produced by [`super::build_context`].
//!
//! Extracted from `mod.rs` to reduce that file's size.

use super::*;

pub(super) fn render_builtin_svg(ctx: &Value) -> String {
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

    let _empty: Vec<serde_json::Value> = vec![];
    let planets = ctx["planets"]
        .as_array()
        .map_or(&_empty[..], |v| v.as_slice());
    let signs = ctx["signs"]
        .as_array()
        .map_or(&_empty[..], |v| v.as_slice());
    let houses = ctx["houses"]
        .as_array()
        .map_or(&_empty[..], |v| v.as_slice());
    let aspects = ctx["aspects"]
        .as_array()
        .map_or(&_empty[..], |v| v.as_slice());

    let mut s = String::with_capacity(64 * 1024);

    // ── header ────────────────────────────────────────────────────────────────
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
        font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".7">"##
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
        r##"</text>

  <!-- rings -->
  <circle cx="{CX}" cy="{CY}" r="{RO}" fill="none" stroke="{ring}" stroke-width="2.5" opacity=".6"/>
  <circle cx="{CX}" cy="{CY}" r="{RM}" fill="none" stroke="{ring}" stroke-width="1.2" opacity=".35"/>
  <circle cx="{CX}" cy="{CY}" r="{RI}" fill="none" stroke="{ring}" stroke-width="2.0" opacity=".55"/>
  <circle cx="{CX}" cy="{CY}" r="{RH}" fill="none" stroke="{ring}" stroke-width="1.2" opacity=".35"/>
  <circle cx="{CX}" cy="{CY}" r="{RC}" fill="{bg}"  stroke="{ring}" stroke-width="2.0" opacity=".4"/>"##
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
            r##"  <line x1="{sx1:.2}" y1="{sy1:.2}" x2="{sx2:.2}" y2="{sy2:.2}" stroke="{ring}" stroke-width="1.5" opacity=".55"/>
  <text x="{gx:.2}" y="{gy:.2}" font-size="15" font-weight="600" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{ring}">{g}</text>"##
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
            r##"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{ring}" stroke-width="{sw}" opacity="{op}"/>
  <text x="{nx:.2}" y="{ny:.2}" font-size="10" font-weight="500" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".6">{n}</text>"##
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
            r##"  <text x="{lx:.2}" y="{ly:.2}" text-anchor="{anchor}" font-size="12" font-weight="800" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">{name}</text>"##
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
            ("0.9", o, r##" stroke-dasharray="4,3""##)
        } else {
            let o: &str = if orb < 2.0 { ".55" } else { ".22" };
            ("1.8", o, "")
        };
        let _ = writeln!(
            s,
            r##"  <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{col}" stroke-width="{sw}" opacity="{op}"{dash}/>"##
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
                r##"  <line x1="{ax:.2}" y1="{ay:.2}" x2="{lx:.2}" y2="{ly:.2}" stroke="{pfg}" stroke-width="0.9" opacity=".45" stroke-dasharray="3,2"/>"##
            );
        }

        let _ = writeln!(
            s,
            r##"  <line x1="{tx1:.2}" y1="{ty1:.2}" x2="{tx2:.2}" y2="{ty2:.2}" stroke="{pfg}" stroke-width="1.0" opacity=".45"/>
  <text x="{px:.2}" y="{py:.2}" font-size="18" font-weight="bold" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}" filter="url(#glow)">{g}</text>
  <text x="{lx:.2}" y="{ly:.2}" font-size="10" font-weight="600" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}">{dl}</text>"##
        );
        // Station marker: small 'S' if planet speed ≈ 0
        if p["near_station"].as_bool().unwrap_or(false) {
            let sx = px + 9.0;
            let sy = py - 9.0;
            let _ = writeln!(
                s,
                r##"  <text x="{sx:.2}" y="{sy:.2}" font-size="7" font-weight="700" text-anchor="middle" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}" opacity=".9">S</text>"##
            );
        }
    }

    // ── moon phase in centre ──────────────────────────────────────────────────
    let _ = writeln!(
        s,
        r##"  <text x="{CX}" y="{:.2}" text-anchor="middle" font-size="11" font-weight="500" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".9">{phase}</text>
  <text x="{CX}" y="{:.2}" text-anchor="middle" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".65">{illum:.1}%</text>
"##,
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
        r##"  <text x="{c1x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Planets</text>
  <line x1="{c1x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"##,
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
            r##"  <text x="{:.2}" y="{ry:.2}" font-size="14" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}">{g}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="11" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".75">{name}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{scol}" opacity="{sop}">{spd}</text>"##,
            c1x + 2.0,
            c1x + 20.0,
            c1x + 120.0,
            c1x + 222.0
        );
    }

    // ── col 2: angles + houses ────────────────────────────────────────────────
    let _ = writeln!(
        s,
        r##"
  <text x="{c2x}" y="{ly:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Angles &amp; Houses</text>
  <line x1="{c2x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"##,
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
            r##"  <text x="{:.2}" y="{ry:.2}" font-size="10" font-weight="700" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">{name}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{lon2:.4}&#176;</text>"##,
            c2x + 2.0,
            c2x + 38.0,
            c2x + 150.0
        );
    }
    let sep_y = ly + 82.0;
    let _ = writeln!(
        s,
        r##"  <line x1="{c2x}" y1="{sep_y:.2}" x2="{:.2}" y2="{sep_y:.2}" stroke="{ring}" stroke-width=".3" opacity=".2"/>"##,
        c2x + 250.0
    );
    for (i, h) in houses.iter().enumerate() {
        let ry = ly + 94.0 + i as f64 * rh2;
        let dms = h["dms"].as_str().unwrap_or("");
        let hlon = h["lon"].as_f64().unwrap_or(0.0);
        let _ = writeln!(
            s,
            r##"  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".55">H{}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{hlon:.4}&#176;</text>"##,
            c2x + 2.0,
            i + 1,
            c2x + 28.0,
            c2x + 138.0
        );
    }

    // ── col 3: aspects ────────────────────────────────────────────────────────
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
            r##"  <text x="{:.2}" y="{ry:.2}" font-size="13" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}">{g1}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="13" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{col}">{g2}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}">{an4}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".7">{orb:.2}&#176;</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".5">{aind}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{b1s}&#8211;{b2s}</text>"##,
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
        r##"  <text x="{c1x}" y="{dig_y:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Essential Dignities</text>
  <line x1="{c1x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"##,
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
            r##"  <text x="{:.2}" y="{ry:.2}" font-size="13" text-anchor="middle" dominant-baseline="central" font-family="serif" fill="{ring}">{g}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{dcol}" font-weight="500">{dig}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{sign_nm}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".45">{name}</text>"##,
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
        r##"  <text x="{c2x}" y="{ap_y:.2}" font-size="12" font-weight="600" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}">Arabic Parts</text>
  <line x1="{c2x}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{ring}" stroke-width=".5" opacity=".35"/>"##,
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
            r##"  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{soft_c}" opacity=".8">{name}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="10" dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{dms}</text>
  <text x="{:.2}" y="{ry:.2}" font-size="9"  dominant-baseline="central" font-family="'Segoe UI',system-ui,sans-serif" fill="{ring}" opacity=".45">{sign_nm}</text>"##,
            c2x + 2.0,
            c2x + 125.0,
            c2x + 210.0,
        );
    }

    // ── footer ────────────────────────────────────────────────────────────────
    let _ = writeln!(
        s,
        r##"
  <text x="450" y="1090" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif"
        fill="{ring}" opacity=".35">Generated by celestial render · {date}</text>
</svg>"##
    );

    s
}
