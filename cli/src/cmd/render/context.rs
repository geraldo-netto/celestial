//! `build_context` — converts CLI args + calculated astronomy data into
//! the `serde_json::Value` that drives SVG template rendering.
//!
//! Extracted from `mod.rs` to reduce that file's size.

#![allow(clippy::too_many_arguments)]

use crate::error::CliError;
use std::collections::BTreeMap;

use celestial_core::body::{CalcFlags, HouseSystem};
use celestial_core::solar::{solar_cycle, SolarCycleInfo};
use celestial_core::{
    arabic_parts_seven, calc_ut, diff_deg_signed, fixstar_mag, fixstar_ut, houses_ex, is_applying,
    lon_to_sign, midpoint_deg, moon_illumination, zodiac_sign_name,
};
use serde_json::{json, Value};

use super::{
    antiscion_lon, body_color, contra_antiscion_lon, fmt_lon_dms, jd_to_date_str, moon_phase_str,
    planet_dignity, wx, wy, ASPECT_DEFS, BODIES, CX, CY, RC, RH, RI, RM, RO, RP,
};

pub(crate) fn build_context(
    jd: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<Value, CliError> {
    let h = houses_ex(jd, CalcFlags::BUILTIN, lat, lon, HouseSystem(hsys as u8))
        ?;
    let asc = h.ascmc[0];
    let mc = h.ascmc[1];
    let ic = (mc + 180.0).rem_euclid(360.0);
    let dsc = (asc + 180.0).rem_euclid(360.0);

    let planets = build_planets(jd, asc);
    let signs = build_signs(asc);
    let houses = build_houses(&h, asc);
    let aspects = compute_aspects(&planets, asc, mc);
    let arabic_parts = build_arabic_parts(&planets, &h, asc);
    let fixed_stars = build_fixed_stars(jd, asc);
    let angles = build_angles(asc, mc, ic, dsc);
    let illum_pct = (moon_illumination(jd).unwrap_or(0.0) * 1000.0).round() / 10.0;
    let solar_cycle_json = build_solar_cycle(jd);
    let vars = build_vars(&user_vars);

    // ARCH-9/DP-5: typed natal/core context. Field names == JSON keys;
    // `serde_json::to_value` reproduces the former object byte-for-byte,
    // so every build_context-reusing builder (natal/derived/hellenistic/
    // composite/triwheel) and the renderers are unaffected.
    let ctx = NatalContext {
        // Derive canonical date+time from JD; fall back to caller label
        // for composite/synastry/test strings that aren't plain dates.
        date: jd_to_date_str(jd),
        date_label: date_str.to_string(),
        jd: (jd * 1e4).round() / 1e4,
        lat,
        lon,
        asc: (asc * 1e4).round() / 1e4,
        mc: (mc * 1e4).round() / 1e4,
        ic: (ic * 1e4).round() / 1e4,
        dsc: (dsc * 1e4).round() / 1e4,
        asc_dms: fmt_lon_dms(asc),
        mc_dms: fmt_lon_dms(mc),
        ic_dms: fmt_lon_dms(ic),
        dsc_dms: fmt_lon_dms(dsc),
        moon_phase_name: moon_phase_str(jd),
        moon_illumination: illum_pct,
        cx: CX,
        cy: CY,
        r_outer: RO,
        r_sign_outer: RM,
        r_sign_inner: RI,
        r_house: RH,
        r_planet: RP,
        r_inner: RC,
        planets,
        signs,
        houses,
        angles,
        aspects,
        arabic_parts,
        fixed_stars,
        solar_cycle: solar_cycle_json,
        vars: Value::Object(vars),
    };
    serde_json::to_value(&ctx).map_err(|e| CliError::Msg(e.to_string()))
}

/// Typed natal/core chart context (ARCH-9/DP-5). Produced by
/// `build_context` and reused (then mutated as a `Value`) by the
/// derived/hellenistic/composite/triwheel builders. `#[derive(Serialize)]`
/// field names are the exact JSON keys the renderers + templates read.
#[derive(serde::Serialize)]
struct NatalContext {
    date: String,
    date_label: String,
    jd: f64,
    lat: f64,
    lon: f64,
    asc: f64,
    mc: f64,
    ic: f64,
    dsc: f64,
    asc_dms: String,
    mc_dms: String,
    ic_dms: String,
    dsc_dms: String,
    moon_phase_name: &'static str,
    moon_illumination: f64,
    cx: f64,
    cy: f64,
    r_outer: f64,
    r_sign_outer: f64,
    r_sign_inner: f64,
    r_house: f64,
    r_planet: f64,
    r_inner: f64,
    planets: Vec<Value>,
    signs: Vec<Value>,
    houses: Vec<Value>,
    angles: Vec<Value>,
    aspects: Vec<Value>,
    arabic_parts: Vec<Value>,
    fixed_stars: Vec<Value>,
    solar_cycle: Value,
    vars: Value,
}

fn build_planets(jd: f64, asc: f64) -> Vec<Value> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let mut planets = Vec::with_capacity(BODIES.len() + 1);
    for &(body, key, name, glyph) in BODIES {
        if let Ok(pos) = calc_ut(jd, body, flags) {
            let (sign_idx, deg_in_sign) = lon_to_sign(pos.lon);
            let sign_full = zodiac_sign_name(sign_idx);
            let sign_short = &sign_full[..sign_full
                .char_indices()
                .nth(3)
                .map_or(sign_full.len(), |(i, _)| i)];
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
                "color":      body_color(key),
                "lon":        (pos.lon   * 1e4).round() / 1e4,
                "lat":        (pos.lat   * 1e4).round() / 1e4,
                "dist":       (pos.dist  * 1e4).round() / 1e4,
                "speed":      (pos.speed_lon * 1e4).round() / 1e4,
                "retro":      pos.speed_lon < 0.0,
                "near_station": pos.speed_lon.abs() < 0.05,
                "dignity":    planet_dignity(body, sign_idx),
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
                "x":        (wx(CX, RP, pos.lon, asc) * 100.0).round() / 100.0,
                "y":        (wy(CY, RP, pos.lon, asc) * 100.0).round() / 100.0,
                "label_x":  (wx(CX, RP + 20.0, pos.lon, asc) * 100.0).round() / 100.0,
                "label_y":  (wy(CY, RP + 20.0, pos.lon, asc) * 100.0).round() / 100.0,
                "tick_x1":  (wx(CX, RH + 2.0,  pos.lon, asc) * 100.0).round() / 100.0,
                "tick_y1":  (wy(CY, RH + 2.0,  pos.lon, asc) * 100.0).round() / 100.0,
                "tick_x2":  (wx(CX, RP - 12.0, pos.lon, asc) * 100.0).round() / 100.0,
                "tick_y2":  (wy(CY, RP - 12.0, pos.lon, asc) * 100.0).round() / 100.0,
                "asp_x":    (wx(CX, RP - 18.0, pos.lon, asc) * 100.0).round() / 100.0,
                "asp_y":    (wy(CY, RP - 18.0, pos.lon, asc) * 100.0).round() / 100.0}));
        }
    }
    if let Some(south) = synthesize_south_node(&planets, asc) {
        planets.push(south);
    }
    planets
}

/// The South Lunar Node (☋) is the point diametrically opposite the
/// North Node. The astronomy layer doesn't expose it as a body, so we
/// synthesise it from the North-Node entry by adding 180° to the
/// longitude and recomputing the wheel-position fields. This means
/// the south node automatically tracks whichever node (mean or true)
/// the renderer chose for the north.
fn synthesize_south_node(planets: &[Value], asc: f64) -> Option<Value> {
    let north = planets.iter().find(|p| {
        let k = p["key"].as_str().unwrap_or("");
        k == "mean_node" || k == "true_node"
    })?;
    let north_lon = north["lon"].as_f64()?;
    let south_lon = (north_lon + 180.0).rem_euclid(360.0);
    let (sign_idx, deg_in_sign) = lon_to_sign(south_lon);
    let sign_full = zodiac_sign_name(sign_idx);
    let sign_short = &sign_full[..sign_full
        .char_indices()
        .nth(3)
        .map_or(sign_full.len(), |(i, _)| i)];
    let speed = north["speed"].as_f64().unwrap_or(0.0);
    let retro = speed < 0.0;
    let deg_label = format!(
        "{:.0}\u{00B0}{}{}",
        deg_in_sign.floor(),
        sign_short,
        if retro { "\u{211E}" } else { "" }
    );
    Some(json!({
        "name":          "Node (South)",
        "key":           "south_node",
        "glyph":         "\u{260B}\u{FE0E}",
        "color":         body_color("south_node"),
        "lon":           (south_lon * 1e4).round() / 1e4,
        "lat":           0.0,
        "dist":          north["dist"].as_f64().unwrap_or(0.0),
        "speed":         (speed * 1e4).round() / 1e4,
        "retro":         retro,
        "near_station":  speed.abs() < 0.05,
        "dignity":       "peregrine",
        "antiscia_lon":  (antiscion_lon(south_lon) * 1e4).round() / 1e4,
        "contra_lon":    (contra_antiscion_lon(south_lon) * 1e4).round() / 1e4,
        "antiscia_x":    (wx(CX, RH - 4.0, antiscion_lon(south_lon), asc) * 100.0).round() / 100.0,
        "antiscia_y":    (wy(CY, RH - 4.0, antiscion_lon(south_lon), asc) * 100.0).round() / 100.0,
        "sign":          sign_idx,
        "sign_name":     zodiac_sign_name(sign_idx),
        "dms":           fmt_lon_dms(south_lon),
        "deg_label":     deg_label,
        "speed_str":     format!("{}{:.2}\u{00B0}/d",
                                if retro { "\u{211E} " } else { "" },
                                speed.abs()),
        "x":             (wx(CX, RP, south_lon, asc) * 100.0).round() / 100.0,
        "y":             (wy(CY, RP, south_lon, asc) * 100.0).round() / 100.0,
        "label_x":       (wx(CX, RP + 20.0, south_lon, asc) * 100.0).round() / 100.0,
        "label_y":       (wy(CY, RP + 20.0, south_lon, asc) * 100.0).round() / 100.0,
        "tick_x1":       (wx(CX, RH + 2.0, south_lon, asc) * 100.0).round() / 100.0,
        "tick_y1":       (wy(CY, RH + 2.0, south_lon, asc) * 100.0).round() / 100.0,
        "tick_x2":       (wx(CX, RP - 12.0, south_lon, asc) * 100.0).round() / 100.0,
        "tick_y2":       (wy(CY, RP - 12.0, south_lon, asc) * 100.0).round() / 100.0,
        "asp_x":         (wx(CX, RP - 18.0, south_lon, asc) * 100.0).round() / 100.0,
        "asp_y":         (wy(CY, RP - 18.0, south_lon, asc) * 100.0).round() / 100.0
    }))
}

/// Zodiac glyphs with the Unicode text-presentation variation selector
/// (`U+FE0E`) appended. The selector forces SVG renderers to pick the
/// outline / "text" form of the sign from the symbol-font stack rather
/// than the chunky colour-emoji form most systems ship by default
/// (Noto Color Emoji, Apple Color Emoji, Segoe UI Emoji). Without
/// this selector, ♈ would render as a tiny red bubble emoji instead
/// of a crisp vector glyph.
const SIGN_GLYPHS: [&str; 12] = [
    "\u{2648}\u{FE0E}",
    "\u{2649}\u{FE0E}",
    "\u{264A}\u{FE0E}",
    "\u{264B}\u{FE0E}",
    "\u{264C}\u{FE0E}",
    "\u{264D}\u{FE0E}",
    "\u{264E}\u{FE0E}",
    "\u{264F}\u{FE0E}",
    "\u{2650}\u{FE0E}",
    "\u{2651}\u{FE0E}",
    "\u{2652}\u{FE0E}",
    "\u{2653}\u{FE0E}",
];

/// Per-sign colour, indexed by zodiac position (0=Aries .. 11=Pisces).
/// Coloured by classical element — fire/earth/air/water — so the wheel
/// is readable at a glance without losing the traditional astrological
/// language.
///
/// * Fire (Aries, Leo, Sagittarius)         → red    `#c1272d`
/// * Earth (Taurus, Virgo, Capricorn)       → green  `#5a7a30`
/// * Air (Gemini, Libra, Aquarius)          → gold   `#c4a017`
/// * Water (Cancer, Scorpio, Pisces)        → blue   `#1a5fb4`
const SIGN_COLORS: [&str; 12] = [
    "#c1272d", "#5a7a30", "#c4a017", "#1a5fb4", "#c1272d", "#5a7a30", "#c4a017", "#1a5fb4",
    "#c1272d", "#5a7a30", "#c4a017", "#1a5fb4",
];

fn build_signs(asc: f64) -> Vec<Value> {
    (0..12)
        .map(|i| {
            let sl = i as f64 * 30.0;
            let mid = sl + 15.0;
            // Sign-band midpoint: now that the wheel has only three
            // visible rings the glyph sits halfway between the outer
            // (RO) and inner (RI) edges of the sign band.
            let sgr = (RO + RI) / 2.0;
            json!({
                "idx":       i,
                "glyph":     SIGN_GLYPHS[i],
                "color":     SIGN_COLORS[i],
                "spoke_x1":  (wx(CX, RI, sl,  asc) * 100.0).round() / 100.0,
                "spoke_y1":  (wy(CY, RI, sl,  asc) * 100.0).round() / 100.0,
                "spoke_x2":  (wx(CX, RO, sl,  asc) * 100.0).round() / 100.0,
                "spoke_y2":  (wy(CY, RO, sl,  asc) * 100.0).round() / 100.0,
                "glyph_x":   (wx(CX, sgr, mid, asc) * 100.0).round() / 100.0,
                "glyph_y":   (wy(CY, sgr, mid, asc) * 100.0).round() / 100.0})
        })
        .collect()
}

/// House-cusp geometry helpers — picked once per cusp.
///
/// `R2`: outer endpoint of the cusp spoke. Angular cusps (1/4/7/10) extend
/// all the way to `RO` so the ASC/IC/DSC/MC axes visibly cross the sign band
/// (matching the World-of-Wisdom natal-chart layout). Intermediate cusps stop
/// at `RI` so they don't overlap sign glyphs.
fn house_outer_radius(is_angle: bool) -> f64 {
    if is_angle {
        RO
    } else {
        RI
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn build_house_value(house_lons: &[f64], i: usize, asc: f64) -> Value {
    let lon2 = house_lons[i];
    let next_lon = house_lons[(i + 1) % 12];
    let mid_lon = midpoint_deg(lon2, next_lon);
    let is_angle = matches!(i, 0 | 3 | 6 | 9);
    let r_out = house_outer_radius(is_angle);
    let r_num = (RH + RI) / 2.0;
    // All cusps spring from the wheel centre — no inner disc bounds
    // them. Angular cusps continue to `RO` for the full-axis cross;
    // intermediate cusps stop at `RI` so they don't intrude into the
    // sign band.
    json!({
        "num":      i + 1,
        "lon":      (lon2 * 1e4).round() / 1e4,
        "dms":      fmt_lon_dms(lon2),
        "is_angle": is_angle,
        "x1":       round2(CX),
        "y1":       round2(CY),
        "x2":       round2(wx(CX, r_out, lon2, asc)),
        "y2":       round2(wy(CY, r_out, lon2, asc)),
        "num_x":    round2(wx(CX, r_num, mid_lon, asc)),
        "num_y":    round2(wy(CY, r_num, mid_lon, asc))
    })
}

fn build_houses(h: &celestial_core::HouseResult, asc: f64) -> Vec<Value> {
    let house_lons: Vec<f64> = h.cusps[1..=12].to_vec();
    (0..12)
        .map(|i| build_house_value(&house_lons, i, asc))
        .collect()
}

fn sun_house_index(cusps: &[f64], sun_lon: f64) -> usize {
    cusps[1..=12]
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
        .map_or(1, |i| i + 1)
}

fn build_arabic_parts(planets: &[Value], h: &celestial_core::HouseResult, asc: f64) -> Vec<Value> {
    let b = collect_body_longitudes(planets);
    let is_day = sun_house_index(&h.cusps, b.sun) >= 7;
    let raw = arabic_parts_seven(
        asc, b.sun, b.moon, b.saturn, b.mars, b.jupiter, b.mercury, b.venus, is_day,
    );
    raw.iter()
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
                "is_day":   is_day})
        })
        .collect()
}

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

fn build_fixed_stars(jd: f64, asc: f64) -> Vec<Value> {
    TOP_STARS
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
                "y":  (wy(CY, RI + 8.0, lon_s, asc) * 100.0).round() / 100.0}))
        })
        .collect()
}

fn build_angles(asc: f64, mc: f64, ic: f64, dsc: f64) -> Vec<Value> {
    let angle_lons = [asc, mc, ic, dsc];
    let angle_labels = ["ASC", "MC", "IC", "DSC"];
    (0..4)
        .map(|i| {
            let lon2 = angle_lons[i];
            json!({
                "name":  angle_labels[i],
                "lon":   (lon2 * 1e4).round() / 1e4,
                "dms":   fmt_lon_dms(lon2),
                "lx":    (wx(CX, RI + 18.0, lon2, asc) * 100.0).round() / 100.0,
                "ly":    (wy(CY, RI + 18.0, lon2, asc) * 100.0).round() / 100.0})
        })
        .collect()
}

const VAR_DEFAULTS: &[(&str, &str)] = &[
    ("bg_color", "#ffffff"),
    ("ring_color", "#1a1a2e"),
    ("planet_color", "#0d0d1e"),
    ("retro_color", "#b01020"),
    ("hard_color", "#b01020"),
    ("soft_color", "#1a50b0"),
    ("text_color", "#0d0d1e"),
    ("title", "Celestial Chart"),
];

fn build_vars(user_vars: &BTreeMap<String, String>) -> serde_json::Map<String, Value> {
    let mut vars = serde_json::Map::new();
    for &(k, v) in VAR_DEFAULTS {
        vars.insert(k.to_string(), json!(v));
    }
    for (k, v) in user_vars {
        vars.insert(k.clone(), json!(v));
    }
    vars
}

/// Solar (Schwabe) cycle context for the chart's date. Always present in the
/// template namespace: when the date falls outside numbered cycles (1755 →
/// ~2030) the object only contains a `grand_epoch` field (or is fully empty).
fn build_solar_cycle(jd: f64) -> Value {
    if let Some(info) = solar_cycle(jd) {
        return solar_cycle_to_json(&info);
    }
    // No numbered cycle — emit just the grand-epoch label if one applies.
    match celestial_core::solar::grand_solar_epoch(jd) {
        Some(g) => json!({ "grand_epoch": g.name() }),
        None => json!({}),
    }
}

fn solar_cycle_to_json(info: &SolarCycleInfo) -> Value {
    json!({
        "cycle_num":       info.cycle_num,
        // phase is a 0..1 display fraction — 3 decimals is plenty.
        "phase":           (info.phase * 1e3).round() / 1e3,
        "phase_name":      info.phase_name.name(),
        "min_jd":          (info.min_jd * 1e2).round() / 1e2,
        "max_jd":          (info.max_jd * 1e2).round() / 1e2,
        "next_min_jd":     (info.next_min_jd * 1e2).round() / 1e2,
        "years_since_min": (info.years_since_min * 1e2).round() / 1e2,
        "nickname":        info.nickname,
        "grand_epoch":     info.grand_epoch.map(|g| g.name()),
    })
}

/// The 7 body longitudes used by the Arabic-Parts calculation, plus a
/// fast lookup pattern (single O(n) pass instead of 7 separate `find()`s).
struct BodyLongitudes {
    sun: f64,
    moon: f64,
    saturn: f64,
    mars: f64,
    jupiter: f64,
    mercury: f64,
    venus: f64,
}

/// Single pass over the planets list to extract the 7 longitudes the
/// Arabic-Parts calculation needs. Replaces 7 sequential `iter().find()`
/// linear scans (~84 hash lookups for a 12-planet chart) with one pass
/// (~12 hash lookups).
fn collect_body_longitudes(planets: &[Value]) -> BodyLongitudes {
    let mut out = BodyLongitudes {
        sun: 0.0,
        moon: 0.0,
        saturn: 0.0,
        mars: 0.0,
        jupiter: 0.0,
        mercury: 0.0,
        venus: 0.0,
    };
    for p in planets {
        let Some(key) = p["key"].as_str() else {
            continue;
        };
        let Some(lon) = p["lon"].as_f64() else {
            continue;
        };
        match key {
            "sun" => out.sun = lon,
            "moon" => out.moon = lon,
            "saturn" => out.saturn = lon,
            "mars" => out.mars = lon,
            "jupiter" => out.jupiter = lon,
            "mercury" => out.mercury = lon,
            "venus" => out.venus = lon,
            _ => {}
        }
    }
    out
}

/// Build a synthetic aspect-grid entry for ASC or MC.
///
/// Angles have no body speed, so `speed=0`; aspect endpoints sit at
/// the same `RP - 18.0` radius the planet entries use so the resulting
/// line lands inside the inner wheel without overshooting the house ring.
fn angle_entry(name: &str, lon: f64, asc: f64) -> Value {
    json!({
        "name":   name,
        "key":    name.to_lowercase(),
        "glyph":  name,
        "lon":    (lon * 1e4).round() / 1e4,
        "speed":  0.0,
        "is_angle": true,
        "asp_x":  (wx(CX, RP - 18.0, lon, asc) * 100.0).round() / 100.0,
        "asp_y":  (wy(CY, RP - 18.0, lon, asc) * 100.0).round() / 100.0,
    })
}

/// Compute pairwise aspects between every body in `planets` plus the
/// ASC and MC angles. PDF natal charts include aspects to the angles
/// (Sat-ASC, Ura-MC, Plu-ASC etc.) — omitting them produces a visibly
/// incomplete aspect grid vs. a commercial reference.
///
/// Aspects are matched in order of `ASPECT_DEFS` (conjunction, opposition,
/// trine, square, sextile, …); first match within orb wins so a body pair
/// satisfying multiple nearby aspects gets a single entry.
///
/// `applying` uses signed orb (which side of exact) and relative speed
/// (`spd1 - spd2`) so it correctly reflects whether the pair is moving
/// toward or away from exact. Static angles (ASC/MC) get `speed = 0` so
/// `applying` collapses to the planet's own approach direction.
fn compute_aspects(planets: &[Value], asc_lon: f64, mc_lon: f64) -> Vec<Value> {
    type PRef<'a> = (
        f64,       // lon
        f64,       // speed
        &'a Value, // name
        &'a Value, // glyph
        &'a Value, // asp_x
        &'a Value, // asp_y
    );
    let asc_entry = angle_entry("ASC", asc_lon, asc_lon);
    let mc_entry = angle_entry("MC", mc_lon, asc_lon);

    let mut p_data: Vec<PRef> = planets
        .iter()
        .map(|p| {
            (
                p["lon"].as_f64().unwrap_or(0.0),
                p["speed"].as_f64().unwrap_or(0.0),
                &p["name"],
                &p["glyph"],
                &p["asp_x"],
                &p["asp_y"],
            )
        })
        .collect();
    for ang in [&asc_entry, &mc_entry] {
        p_data.push((
            ang["lon"].as_f64().unwrap_or(0.0),
            0.0,
            &ang["name"],
            &ang["glyph"],
            &ang["asp_x"],
            &ang["asp_y"],
        ));
    }

    // Upper bound: each pair × ASPECT_DEFS could match, but most don't.
    // n*(n-1)/2 is a safe ceiling; typical chart has < 40 aspects with angles.
    let n = p_data.len();
    let mut aspects = Vec::with_capacity(n * n / 4);
    for i in 0..n {
        let (lon1, spd1, name1, glyph1, asp_x1, asp_y1) = p_data[i];
        for (lon2, spd2, name2, glyph2, asp_x2, asp_y2) in p_data.iter().skip(i + 1).copied() {
            // ASC↔MC angle is a house-system geometry artifact, not an
            // aspect — exclude so the grid matches commercial output.
            let n1 = name1.as_str().unwrap_or("");
            let n2 = name2.as_str().unwrap_or("");
            if (n1 == "ASC" && n2 == "MC") || (n1 == "MC" && n2 == "ASC") {
                continue;
            }
            let signed = diff_deg_signed(lon1, lon2);
            let diff = signed.abs();
            for &(asp_deg, asp_name, orb_lim, is_minor) in ASPECT_DEFS {
                let orb = (diff - asp_deg).abs();
                if orb <= orb_lim {
                    let applying = is_applying(signed, spd1 - spd2, asp_deg);
                    aspects.push(json!({
                        "body1":       name1,
                        "glyph1":      glyph1,
                        "body2":       name2,
                        "glyph2":      glyph2,
                        "aspect_name": asp_name,
                        "aspect_deg":  asp_deg,
                        "orb":         (orb * 100.0).round() / 100.0,
                        "applying":    applying,
                        "is_hard":     asp_name == "square" || asp_name == "opposition",
                        "is_minor":    is_minor,
                        "x1":          asp_x1,
                        "y1":          asp_y1,
                        "x2":          asp_x2,
                        "y2":          asp_y2,
                    }));
                    break;
                }
            }
        }
    }
    aspects
}
