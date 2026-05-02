//! Derived-chart builders: progressed, solar-arc, biwheel, cosmogram.
//!
//! These produce a chart from either a natal chart plus a later moment
//! (progressions, solar arc) or from two charts overlaid (biwheel).
//! Extracted from `mod.rs` to group related logic.

use std::collections::BTreeMap;

use celestial_core::body::{Body, CalcFlags};
use celestial_core::calc_ut;

use super::builtin_svg::render_builtin_svg;
use super::context::build_context;
use super::jd_to_date_str;

pub(super) fn build_progressed_context(
    jd: f64,
    years: f64,
    lat: f64,
    lon: f64,
    _date_str: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<serde_json::Value, String> {
    let prog_jd = jd + years * 365.25;
    let prog_date = jd_to_date_str(prog_jd);
    let mut vars = user_vars;
    vars.entry("chart_type_label".to_string())
        .or_insert_with(|| format!("Progressed ({years:.1}y)"));
    vars.insert("secondary_jd".to_string(), format!("{prog_jd:.4}"));
    vars.insert("secondary_date".to_string(), prog_date.clone());
    let mut ctx = build_context(prog_jd, lat, lon, &prog_date, hsys, vars)?;
    // Tests expect "progressed_planets" = same as "planets" at prog_jd
    let prog_planets = ctx["planets"].clone();
    ctx["progressed_planets"] = prog_planets;
    Ok(ctx)
}

pub(super) fn render_progressed_svg(ctx: &serde_json::Value) -> String {
    render_builtin_svg(ctx)
}

pub(super) fn build_solar_arc_context(
    jd: f64,
    years: f64,
    lat: f64,
    lon: f64,
    date_str: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<serde_json::Value, String> {
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let mut ctx = build_context(jd, lat, lon, date_str, hsys, user_vars)?;
    // Compute solar arc delta
    let sun_natal = calc_ut(jd, Body::SUN, flags).map_err(|e| e.to_string())?;
    // Solar arc direction: 1 day = 1 year (secondary progression rate)
    // Progressed Sun is at birth + years days; arc = difference from natal Sun.
    let progressed_jd = jd + years; // 1 day per year
    let sun_progressed = calc_ut(progressed_jd, Body::SUN, flags).map_err(|e| e.to_string())?;
    let arc = (sun_progressed.lon - sun_natal.lon + 360.0) % 360.0;
    ctx["solar_arc_deg"] = serde_json::json!(arc);
    ctx["solar_arc_degrees"] = serde_json::json!(arc); // alias for test compat
                                                       // Build directed planet list
    let directed: Vec<serde_json::Value> = ctx["planets"]
        .as_array()
        .map(|ps| {
            ps.iter()
                .map(|p| {
                    let mut dp = p.clone();
                    if let Some(lon_val) = p["lon"].as_f64() {
                        dp["lon"] = serde_json::json!((lon_val + arc) % 360.0);
                    }
                    dp
                })
                .collect()
        })
        .unwrap_or_default();
    ctx["directed_planets"] = serde_json::json!(directed);
    Ok(ctx)
}

pub(super) fn render_cosmogram_svg(ctx: &serde_json::Value) -> String {
    render_builtin_svg(ctx)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_biwheel_context(
    jd1: f64,
    jd2: f64,
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
    date1: &str,
    date2: &str,
    hsys: char,
    user_vars: BTreeMap<String, String>,
) -> Result<serde_json::Value, String> {
    let mut vars1 = user_vars.clone();
    vars1
        .entry("title".to_string())
        .or_insert_with(|| "Bi-wheel".to_string());
    vars1.insert("ring2_date".to_string(), date2.to_string());
    let inner = build_context(jd1, lat1, lon1, date1, hsys, vars1)?;
    let vars2 = user_vars;
    let outer = build_context(jd2, lat2, lon2, date2, hsys, vars2)?;
    let mut ctx = inner;
    let op = outer["planets"].clone();
    ctx["ring2_planets"] = op.clone();
    ctx["outer_planets"] = op;
    ctx["ring2_date"] = serde_json::json!(date2);
    let inner_ps = ctx["planets"].as_array().cloned().unwrap_or_default();
    let outer_ps = ctx["outer_planets"].as_array().cloned().unwrap_or_default();
    let cross: Vec<serde_json::Value> = inner_ps
        .iter()
        .flat_map(|ip| {
            outer_ps
                .iter()
                .filter_map(|op| {
                    let il = ip["lon"].as_f64()?;
                    let ol = op["lon"].as_f64()?;
                    let diff = (il - ol).abs().min(360.0 - (il - ol).abs());
                    if diff < 8.0 {
                        Some(serde_json::json!({"inner": ip["key"].clone(),
                        "outer": op["key"].clone(), "orb": diff}))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    ctx["cross_aspects"] = serde_json::json!(cross);
    Ok(ctx)
}

pub(super) fn render_biwheel_svg(ctx: &serde_json::Value) -> String {
    render_builtin_svg(ctx)
}
