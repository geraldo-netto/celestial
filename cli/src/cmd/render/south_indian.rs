//! North-Indian / South-Indian Vedic chart tables + Ashtakavarga +
//! the South-Indian Rasi SVG renderer.
//!
//! Pure layout + the classical Parashara bindu algorithm, moved
//! verbatim from the render god-file (ARCH-8) — no formula change.
//! Re-exported by `mod.rs` so `super::sarvashtakavarga` /
//! `super::render_south_indian_svg` call sites are unchanged.

use celestial_core::{long_to_nakshatra, nakshatra_name};
use serde_json::Value;

use super::{json_array, ChartContext};

// ─── North Indian diamond chart ───────────────────────────────────────────────
//
// 12 triangular cells arranged as a diamond (rhombus) pattern.
// The top cell is Aries (0), and cells proceed clockwise. The Ascendant sign
// is highlighted. Unlike South Indian (fixed signs), North Indian uses
// the ASC sign as the first house.

// Cell centre coordinates in a 540×540 grid
// Order: Aries(0)..Pisces(11) starting from top, clockwise
pub(crate) const NI_CELLS: &[(f64, f64)] = &[
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
pub(crate) fn sarvashtakavarga(
    planet_rasis: &[usize; 7],
    asc_rasi: usize,
) -> ([u8; 12], Vec<[u8; 12]>) {
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

// ─── Shadbala strength table ──────────────────────────────────────────────────
//
// Six-fold strength system. We compute three readily-available components:
// Ochchabala (exaltation strength), Saptavargaja bala (sign placement),
// and Chesta bala (motional strength from speed).

pub(crate) const RASI_NAMES: &[&str] = &[
    "Ar", "Ta", "Ge", "Ca", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi",
];

pub(crate) const RASI_GLYPHS: &[&str] = &[
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

const SI_CW: f64 = 140.0;
const SI_CH: f64 = 120.0;
const SI_OX: f64 = 30.0;
const SI_OY: f64 = 70.0;

/// Palette for the South-Indian Rasi chart renderer. Bundles the five
/// colour strings that every `write_si_*` helper needs so each helper's
/// signature stays under the clippy 7-arg ceiling.
struct SiPalette<'a> {
    bg: &'a str,
    txt: &'a str,
    border: &'a str,
    pcol: &'a str,
    retro: &'a str,
}

fn group_planets_by_rasi(planets: &[Value]) -> Vec<Vec<String>> {
    let mut rasi_planets: Vec<Vec<String>> = vec![Vec::new(); 12];
    for p in planets {
        let rasi = p["rasi"].as_i64().unwrap_or(0) as usize % 12;
        let g = p["glyph"].as_str().unwrap_or("?");
        let ret = p["retro"].as_bool().unwrap_or(false);
        let deg = p["deg_in_rasi"].as_f64().unwrap_or(0.0);
        let lbl = format!("{g}{}", if ret { "℞" } else { "" });
        rasi_planets[rasi].push(format!("{lbl} {deg:.0}°"));
    }
    rasi_planets
}

fn write_si_header(
    s: &mut String,
    pal: &SiPalette,
    title: &str,
    date: &str,
    total_w: f64,
    total_h: f64,
) {
    use std::fmt::Write;
    let (bg, txt, border) = (pal.bg, pal.txt, pal.border);
    let half = total_w / 2.0;
    let _ = writeln!(
        s,
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w:.0} {total_h:.0}" width="{total_w:.0}" height="{total_h:.0}">
  <rect width="{total_w:.0}" height="{total_h:.0}" fill="{bg}"/>
  <text x="{half:.2}" y="28" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="{half:.2}" y="48" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>
  <rect x="{SI_OX}" y="{SI_OY}" width="{:.2}" height="{:.2}" fill="none" stroke="{border}" stroke-width="2.0"/>"##,
        4.0 * SI_CW,
        4.0 * SI_CH
    );
}

fn write_si_centre(s: &mut String, ctx: &Value, pal: &SiPalette) {
    use std::fmt::Write;
    let (bg, txt, border) = (pal.bg, pal.txt, pal.border);
    let cx = SI_OX + SI_CW;
    let cy = SI_OY + SI_CH;
    let moon_sid = ctx["moon_sid_lon"].as_f64().unwrap_or(0.0);
    let (moon_nak, _) = long_to_nakshatra(moon_sid);
    let nak_nm = nakshatra_name(moon_nak).unwrap_or("?");
    let _ = writeln!(
        s,
        r##"  <rect x="{cx:.2}" y="{cy:.2}" width="{:.2}" height="{:.2}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>
  <text x="{:.2}" y="{:.2}" text-anchor="middle" font-size="11" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">Vedic Chart</text>
  <text x="{:.2}" y="{:.2}" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".7">☽ {nak_nm}</text>"##,
        2.0 * SI_CW,
        2.0 * SI_CH,
        cx + SI_CW,
        SI_CH.mul_add(0.85, cy),
        cx + SI_CW,
        SI_CH.mul_add(1.1, cy)
    );
}

fn write_si_cells(s: &mut String, rasi_planets: &[Vec<String>], pal: &SiPalette) {
    use std::fmt::Write;
    let (bg, txt, border, pcol, retro) = (pal.bg, pal.txt, pal.border, pal.pcol, pal.retro);
    for &(row, col, sign_idx) in SI_CELLS {
        let x = (col as f64).mul_add(SI_CW, SI_OX);
        let y = (row as f64).mul_add(SI_CH, SI_OY);
        let sg = RASI_GLYPHS[sign_idx as usize % 12];
        let sn = RASI_NAMES[sign_idx as usize % 12];
        let _ = writeln!(
            s,
            r##"  <rect x="{x:.2}" y="{y:.2}" width="{SI_CW:.2}" height="{SI_CH:.2}" fill="{bg}" stroke="{border}" stroke-width="1.0"/>
  <text x="{:.2}" y="{:.2}" font-size="11" font-family="serif" fill="{txt}" opacity=".5">{sg}</text>
  <text x="{:.2}" y="{:.2}" font-size="8" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{sn}</text>"##,
            x + 4.0,
            y + 14.0,
            x + 4.0,
            y + 24.0
        );

        let prasi = &rasi_planets[sign_idx as usize % 12];
        for (pi, plabel) in prasi.iter().enumerate() {
            let py = (pi as f64).mul_add(14.0, y + 36.0);
            let col_s = if plabel.contains('℞') { retro } else { pcol };
            let _ = writeln!(
                s,
                r##"  <text x="{:.2}" y="{py:.2}" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{col_s}">{plabel}</text>"##,
                x + 6.0
            );
        }
    }
}

fn write_si_dashas(s: &mut String, ctx: &Value, dy: f64, total_w: f64, txt: &str, pcol: &str) {
    use std::fmt::Write;
    let _ = writeln!(
        s,
        r##"  <text x="{SI_OX}" y="{dy:.2}" font-size="11" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">Vimshottari Dashas</text>"##
    );
    let dashas = json_array(&ctx["dashas"]);
    let col_w = (total_w - SI_OX * 2.0) / 3.0;
    for (i, d) in dashas.iter().take(9).enumerate() {
        let col_x = ((i % 3) as f64).mul_add(col_w, SI_OX);
        let row_y = ((i / 3) as f64).mul_add(14.0, dy + 14.0);
        let body = d["body"].as_str().unwrap_or("?");
        let yrs = d["years"].as_f64().unwrap_or(0.0);
        let start = d["start"].as_str().unwrap_or("");
        let _ = writeln!(
            s,
            r##"  <text x="{col_x:.2}" y="{row_y:.2}" font-size="9"
            font-family="'Segoe UI',system-ui,sans-serif" fill="{pcol}"><tspan font-weight="600">{body}</tspan> {yrs:.1}y · {start}</text>"##
        );
    }
}

pub(crate) fn render_south_indian_svg(ctx: &ChartContext) -> String {
    use std::fmt::Write;
    let pal = SiPalette {
        bg: ctx["vars"]["bg_color"].as_str().unwrap_or("#ffffff"),
        border: ctx["vars"]["border_color"].as_str().unwrap_or("#5c3a00"),
        txt: ctx["vars"]["text_color"].as_str().unwrap_or("#2a1a00"),
        pcol: ctx["vars"]["planet_color"].as_str().unwrap_or("#1a3a7a"),
        retro: ctx["vars"]["retro_color"].as_str().unwrap_or("#a01030"),
    };
    let title = ctx["vars"]
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Rasi Chart");
    let date = ctx["date"].as_str().unwrap_or("");

    let planets = json_array(&ctx["planets"]);
    let rasi_planets = group_planets_by_rasi(planets);

    let mut s = String::with_capacity(16 * 1024);
    let total_h = 4.0_f64.mul_add(SI_CH, SI_OY) + 40.0;
    let total_w = 4.0_f64.mul_add(SI_CW, SI_OX * 2.0);
    write_si_header(&mut s, &pal, title, date, total_w, total_h);
    write_si_centre(&mut s, ctx, &pal);
    write_si_cells(&mut s, &rasi_planets, &pal);
    let dy = 4.0_f64.mul_add(SI_CH, SI_OY) + 10.0;
    write_si_dashas(&mut s, ctx, dy, total_w, pal.txt, pal.pcol);
    let _ = writeln!(s, "</svg>");
    s
}
