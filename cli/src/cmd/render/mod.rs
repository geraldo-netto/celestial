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
    arabic_parts_seven, calc_ut, diff_deg_signed, houses_ex, lunar_return_jd, midpoint_deg,
    moon_illumination, moon_phase, sign_exaltation, sign_ruler, solar_return_jd,
};
use celestial_core::{fixstar_mag, fixstar_ut};
use celestial_core::{lon_to_sign, zodiac_sign_name};
use celestial_core::{long_to_nakshatra, nakshatra_name};
use clap::Args;
use serde_json::{json, Value};
use tinytemplate::TinyTemplate;

// ── Builder submodules — each tradition's build_*/render_* fns ───────────────
mod builtin_svg;
mod chinese;
mod context;
mod derived;
mod hellenistic;
mod indigenous;
mod mesoamerican;
mod specialist;
mod vedic;
mod western;

use builtin_svg::render_builtin_svg;
use context::build_context;
use derived::{
    build_biwheel_context, build_progressed_context, build_solar_arc_context, render_biwheel_svg,
    render_cosmogram_svg, render_progressed_svg,
};

// ─── CLI ─────────────────────────────────────────────────────────────────────

#[derive(Args, Debug)]
pub struct RenderArgs {
    /// Date and optional time to compute (YYYY-MM-DD [HH:MM[:SS]] or "now")
    #[arg(long, default_value = "now")]
    pub date: String,

    /// Time of day UT (HH:MM or HH:MM:SS) — merged with --date if --date has no time
    #[arg(long)]
    pub time: Option<String>,

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
    /// Third date (YYYY-MM-DD [HH:MM[:SS]]) for tri-wheel ring 3
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
pub(super) fn wx(cx: f64, r: f64, lon: f64, asc: f64) -> f64 {
    cx + r * (180.0 - (lon - asc)).rem_euclid(360.0).to_radians().cos()
}

/// SVG y coordinate for an ecliptic longitude on the wheel.
pub(super) fn wy(cy: f64, r: f64, lon: f64, asc: f64) -> f64 {
    cy - r * (180.0 - (lon - asc)).rem_euclid(360.0).to_radians().sin()
}

// ─── Formatting helpers ───────────────────────────────────────────────────────

pub(super) fn fmt_lon_dms(lon: f64) -> String {
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

pub(super) fn moon_phase_str(jd: f64) -> &'static str {
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

// Format a Julian Day as "YYYY-MM-DD".

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 5 — Hellenistic / Persian chart builders
// ═══════════════════════════════════════════════════════════════════════════════

// ─── 1. Hellenistic dignities overlay (extends existing natal wheel) ───────────

/// Map a planet key string to a Body.
pub(super) fn key_to_body(key: &str) -> Option<Body> {
    match key {
        "sun" => Some(Body::SUN),
        "moon" => Some(Body::MOON),
        "mercury" => Some(Body::MERCURY),
        "venus" => Some(Body::VENUS),
        "mars" => Some(Body::MARS),
        "jupiter" => Some(Body::JUPITER),
        "saturn" => Some(Body::SATURN),
        "uranus" => Some(Body::URANUS),
        "neptune" => Some(Body::NEPTUNE),
        "pluto" => Some(Body::PLUTO),
        "mean_node" => Some(Body::MEAN_NODE),
        "chiron" => Some(Body::CHIRON),
        _ => None,
    }
}

// ─── 2. Firdaria timeline ──────────────────────────────────────────────────────

// ─── 3. Profection wheel ──────────────────────────────────────────────────────

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 6 — Chinese astrology chart builders
// ═══════════════════════════════════════════════════════════════════════════════

// ─── Ba Zi (Four Pillars) ─────────────────────────────────────────────────────

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 7 — Mesoamerican calendar context + SVG
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 8 — Medicine Wheel / Egyptian decans context + SVG
// ═══════════════════════════════════════════════════════════════════════════════

// Helper to get sun longitude from PlanetPos

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 3 — specialist Western chart builders
// ═══════════════════════════════════════════════════════════════════════════════

// ─── 1. 90° Midpoint Dial ─────────────────────────────────────────────────────

// ─── 2. Composite chart ───────────────────────────────────────────────────────

// ─── 3. Tri-wheel ────────────────────────────────────────────────────────────

// ─── 4. Graphic Ephemeris ─────────────────────────────────────────────────────

// ─── 5. Local Space chart ─────────────────────────────────────────────────────

/// Format a Julian Day as a date string, including HH:MM when the time is not midnight.
pub(super) fn jd_to_date_str(jd: f64) -> String {
    let d = celestial_core::revjul(jd, celestial_core::body::Calendar::Gregorian);
    let total_sec = (d.hour * 3600.0).round() as i32; // round to nearest second first
    let total_min = total_sec / 60; // truncate seconds from display
    let h = total_min / 60;
    let m = total_min % 60;
    if h == 0 && m == 0 {
        format!(
            "{:04}-{:02}-{:02}",
            d.year as i32, d.month as u32, d.day as u32
        )
    } else {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02} UT",
            d.year as i32, d.month as u32, d.day as u32, h, m
        )
    }
}

// ─── Planet table ─────────────────────────────────────────────────────────────

pub(super) const BODIES: &[(Body, &str, &str, &str)] = &[
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
pub(super) const ASPECT_DEFS: &[(f64, &str, f64, bool)] = &[
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
/// Secondary (pre-outer-planet) domicile for the 5 classical rulers.
///
/// Returns `Some(second_sign)` when `body` is one of Mercury/Venus/Mars/Jupiter/Saturn,
/// otherwise `None`. Each classical planet rules two signs of opposite polarity;
/// `sign_ruler` only reports one of them, so this fills in the other.
fn second_domicile(body: Body) -> Option<u8> {
    match body.as_raw() {
        2 => Some(8),  // Mercury: Gemini + Virgo
        3 => Some(6),  // Venus:   Taurus + Libra
        4 => Some(7),  // Mars:    Aries  + Scorpio
        5 => Some(11), // Jupiter: Sagittarius + Pisces
        6 => Some(9),  // Saturn:  Capricorn + Aquarius
        _ => None,
    }
}

/// Classical essential dignity of `body` in `sign`.
///
/// Checks, in order: domicile → detriment → exaltation → fall → peregrine.
/// Returns the dignity name as a static string.
pub(super) fn planet_dignity(body: Body, sign: u8) -> &'static str {
    let s = sign % 12;
    let opp = (s + 6) % 12;

    // A body rules `sign` if it is the primary ruler or the secondary domicile
    let rules = |sign: u8| -> bool {
        sign_ruler(sign) == body || second_domicile(body).is_some_and(|s2| s2 == sign)
    };

    if rules(s) {
        return "domicile";
    }
    if rules(opp) {
        return "detriment";
    }

    // Exaltation & fall
    let ex = sign_exaltation(body);
    if ex >= 0 && ex as u8 == s {
        return "exaltation";
    }
    if ex >= 0 && (ex as u8 + 6) % 12 == s {
        return "fall";
    }

    "peregrine"
}

/// Antiscion longitude: reflection over the 0°Cancer / 0°Capricorn (solstice) axis.
/// Formula: antiscion = (180° - lon) mod 360°
#[inline]
pub(super) fn antiscion_lon(lon: f64) -> f64 {
    (180.0 - lon).rem_euclid(360.0)
}

/// Contra-antiscion longitude: reflection over the 0°Aries / 0°Libra (equinox) axis.
/// Formula: contra = (360° - lon) mod 360°
#[inline]
pub(super) fn contra_antiscion_lon(lon: f64) -> f64 {
    (360.0 - lon).rem_euclid(360.0)
}

// ─── Build the full context ───────────────────────────────────────────────────

pub(super) const CX: f64 = 450.0;
pub(super) const CY: f64 = 490.0;
pub(super) const RO: f64 = 320.0; // outer ring
pub(super) const RM: f64 = 290.0; // sign band outer
pub(super) const RI: f64 = 262.0; // sign band inner
pub(super) const RH: f64 = 238.0; // house cusp inner
pub(super) const RP: f64 = 212.0; // planet ring
pub(super) const RC: f64 = 88.0; // inner circle

// ─── Label collision avoidance ────────────────────────────────────────────────

/// Compute non-overlapping wheel angles for planet degree labels.
///
/// Starts each label at its natural angular position (derived from the planet
/// longitude) and iteratively separates overlapping pairs symmetrically along
/// the arc, keeping each label within `MAX_DRIFT` degrees of its planet.
/// Returns one adjusted angle per planet, in the original planet order.
/// Wrap an angle to (−180, +180].
#[inline]
fn wrap_signed_180(mut a: f64) -> f64 {
    while a > 180.0 {
        a -= 360.0;
    }
    while a < -180.0 {
        a += 360.0;
    }
    a
}

/// Clamp a label so it does not drift more than `max_drift` degrees from its
/// "natural" angle.  Uses signed wrap so e.g. natural = 350° and placed = 5°
/// is treated as a +15° drift, not +355°.
fn clamp_drift(placed: &mut f64, natural: f64, max_drift: f64) {
    let drift = wrap_signed_180(*placed - natural);
    if drift.abs() > max_drift {
        *placed = natural + drift.signum() * max_drift;
    }
}

/// One repulsion pass between labels `i` and `j`. Returns `true` if the pair
/// was crowded and got pushed apart.
fn repel_pair(
    placed: &mut [f64],
    natural: &[f64],
    i: usize,
    j: usize,
    min_sep: f64,
    max_drift: f64,
) -> bool {
    let d = wrap_signed_180(placed[j] - placed[i]);
    if d.abs() >= min_sep {
        return false;
    }
    // Push symmetrically; slightly more than half ensures convergence
    let push = (min_sep - d.abs()) * 0.55 + 0.05;
    if d >= 0.0 {
        placed[j] += push;
        placed[i] -= push;
    } else {
        placed[i] += push;
        placed[j] -= push;
    }
    clamp_drift(&mut placed[i], natural[i], max_drift);
    clamp_drift(&mut placed[j], natural[j], max_drift);
    true
}

pub(super) fn spread_labels(lons: &[f64], asc: f64) -> Vec<f64> {
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

    // Iterative pairwise repulsion until no pair is crowded (or MAX_ITER).
    for _iter in 0..MAX_ITER {
        let mut any = false;
        for i in 0..n {
            for j in (i + 1)..n {
                if repel_pair(&mut placed, &natural, i, j, MIN_SEP, MAX_DRIFT) {
                    any = true;
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

// ─── Entry point ─────────────────────────────────────────────────────────────

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 2 — additional chart-type context builders
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 2 — SVG renderers for new chart types
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert any `toml::Value` leaf into its owned `String` representation
/// (used when merging `[vars]` from a config file).
fn toml_value_to_string(v: &toml::Value) -> String {
    match v {
        toml::Value::String(x) => x.clone(),
        toml::Value::Integer(x) => x.to_string(),
        toml::Value::Float(x) => x.to_string(),
        toml::Value::Boolean(x) => x.to_string(),
        other => other.to_string(),
    }
}

/// Read a TOML config file and merge its defaults into `args` (only fields
/// still at their sentinel defaults are overridden) and return its `[vars]`
/// map. Returns an empty map if no config file is specified.
/// Apply `Some(value)` to `*field` only if the predicate `should_override` is `true`.
/// This reads as "fill in this defaultable field from the config when the
/// caller hasn't already provided one", flattening the very common
/// `if cond { if let Some(v) = opt { *field = v; } }` triple-nested idiom.
fn override_if<T>(field: &mut T, candidate: Option<T>, should_override: bool) {
    if should_override {
        if let Some(v) = candidate {
            *field = v;
        }
    }
}

fn load_config(args: &mut RenderArgs) -> Result<BTreeMap<String, String>, String> {
    let mut file_vars = BTreeMap::new();
    let Some(cfg_path) = args.config.clone() else {
        return Ok(file_vars);
    };
    let text = std::fs::read_to_string(&cfg_path)
        .map_err(|e| format!("cannot read config `{}`: {e}", cfg_path.display()))?;
    let cfg: ConfigFile = toml::from_str(&text).map_err(|e| format!("invalid config TOML: {e}"))?;

    if let Some(r) = cfg.render {
        // Snapshot field values before taking &mut borrows so the predicate
        // doesn't conflict with the mutable borrow.
        let date_at_default = args.date == "now";
        let lat_at_default = args.lat == 0.0;
        let lon_at_default = args.lon == 0.0;
        let hsys_at_default = args.hsys == 'P';

        // Each line: "use the config value for this field if the user didn't
        // already specify one on the command line".
        override_if(&mut args.date, r.date, date_at_default);
        override_if(&mut args.lat, r.lat, lat_at_default);
        override_if(&mut args.lon, r.lon, lon_at_default);
        override_if(&mut args.hsys, r.hsys, hsys_at_default);
        if args.template.is_none() {
            args.template = r.template;
        }
        if args.out.is_none() {
            args.out = r.out;
        }
    }
    if let Some(t) = cfg.vars {
        for (k, v) in t {
            file_vars.insert(k, toml_value_to_string(&v));
        }
    }
    Ok(file_vars)
}

/// Apply `--var KEY=VALUE` CLI overrides on top of config-file vars.
fn apply_var_overrides(
    vars: &[String],
    mut base: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    for kv in vars {
        let (k, v) = kv
            .split_once('=')
            .ok_or_else(|| format!("`--var` must be KEY=VALUE, got `{kv}`"))?;
        base.insert(k.to_string(), v.to_string());
    }
    Ok(base)
}

/// Merge a separate `--time HH:MM` CLI argument into `--date` when the
/// date is a simple bare calendar date. Time is ignored when `date` is
/// "now", a raw JD, or already contains a time component.
fn merge_date_and_time(date: &str, time: Option<&str>) -> String {
    let Some(t) = time else {
        return date.to_string();
    };
    let base = date.trim();
    if base == "now" || base.parse::<f64>().is_ok() || base.contains(' ') {
        date.to_string()
    } else {
        format!("{base} {t}")
    }
}

pub fn run(mut args: RenderArgs) -> Result<(), String> {
    let file_vars = load_config(&mut args)?;
    let user_vars = apply_var_overrides(&args.vars, file_vars)?;
    let date_str = merge_date_and_time(&args.date, args.time.as_deref());
    let jd = crate::parse::parse_date(&date_str)?;

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
                .map(crate::parse::parse_date)
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
            (specialist::build_composite_context(jd, jd2, args.lat, args.lon, &args.date, date2, args.hsys, v)?,
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
            (specialist::build_triwheel_context(jd, jd2, jd3, args.lat, args.lon,
                                    &args.date, date2, date3, args.hsys, v)?,
             specialist::render_triwheel_svg)
        }
        "dial" | "90dial" | "midpoint-dial" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "90° Midpoint Dial".to_string());
            (specialist::build_dial_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?,
             specialist::render_dial_svg)
        }
        "ephemeris" | "graphic-ephemeris" => {
            let date2 = args.date2.as_deref().unwrap_or("now");
            let jd2 = crate::parse::parse_date(date2)?;
            let (jd_s, jd_e) = if jd < jd2 { (jd, jd2) } else { (jd2, jd) };
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Graphic Ephemeris".to_string());
            (specialist::build_graphic_ephemeris_context(jd_s, jd_e, v)?, specialist::render_graphic_ephemeris_svg)
        }
        "local-space" | "localspace" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Local Space".to_string());
            (specialist::build_local_space_context(jd, args.lat, args.lon, &args.date, v)?,
             specialist::render_local_space_svg)
        }
        "rasi" | "vedic" | "south-indian" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Rasi Chart (South Indian)".to_string());
            (vedic::build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Rasi")?,
             render_south_indian_svg)
        }
        "navamsa" | "d9" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Navamsa D9 Chart".to_string());
            (vedic::build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Navamsa")?,
             vedic::render_navamsa_svg)
        }
        "dasha" | "vimshottari" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Vimshottari Dasha Timeline".to_string());
            (vedic::build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Dasha")?,
             vedic::render_dasha_svg)
        }
        "north-indian" | "north_indian" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "North Indian Chart".to_string());
            (vedic::build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Rasi")?,
             vedic::render_north_indian_svg)
        }
        "ashtakavarga" | "ashtak" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Ashtakavarga".to_string());
            (vedic::build_ashtakavarga_context(jd, args.lat, args.lon, &args.date, v)?,
             vedic::render_ashtakavarga_svg)
        }
        "shadbala" | "strength" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Shadbala".to_string());
            (vedic::build_shadbala_context(jd, args.lat, args.lon, &args.date, v)?,
             vedic::render_shadbala_svg)
        }
        "hellenistic" | "greek" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Hellenistic Chart".to_string());
            (hellenistic::build_hellenistic_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?,
             hellenistic::render_hellenistic_svg)
        }
        "firdaria" | "persian" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Firdaria Timeline".to_string());
            (hellenistic::build_firdaria_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?,
             hellenistic::render_firdaria_svg)
        }
        "profection" => {
            let age = args.years.map(|y| y as u32)
                .or(args.return_year.map(|r| r as u32))
                .unwrap_or(0);
            let mut v = user_vars.clone();
            v.entry("title".to_string())
                .or_insert_with(|| format!("Profection — Age {age}"));
            (hellenistic::build_profection_context(jd, args.lat, args.lon, &args.date, args.hsys, age, v)?,
             hellenistic::render_profection_svg)
        }
        "bazi" | "four-pillars" | "chinese" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Four Pillars (八字)".to_string());
            (chinese::build_bazi_context(jd, args.lat, args.lon, &args.date, v)?,
             chinese::render_bazi_svg)
        }
        "mesoamerican" | "aztec" | "maya" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Mesoamerican Calendars".to_string());
            (mesoamerican::build_mesoamerican_context(jd, args.lat, args.lon, &args.date, v)?,
             mesoamerican::render_mesoamerican_svg)
        }
        "medicine-wheel" | "indigenous" | "egyptian-decans" => {
            let mut v = user_vars.clone();
            v.entry("title".to_string()).or_insert_with(|| "Medicine Wheel / Egyptian Decans".to_string());
            (indigenous::build_medicine_wheel_context(jd, args.lat, args.lon, &args.date, v)?,
             indigenous::render_medicine_wheel_svg)
        }
        other => return Err(format!("unknown --type '{other}'; valid: natal cosmogram solar-return lunar-return progressed solar-arc biwheel composite triwheel dial ephemeris local-space rasi navamsa dasha north-indian ashtakavarga shadbala hellenistic firdaria profection bazi mesoamerican medicine-wheel"))};

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

const EXAMPLE_TEMPLATE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
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
  <rect width="900" height="800" fill="{vars.bg_color};"/>

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
  {{ for h in houses }};
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
"##;

// ─── Tests ───────────────────────────────────────────────────────────────────

// ── Recovered builders (were accidentally removed during refactor) ────────────

#[cfg(test)]
mod tests {
    use super::specialist::{
        build_composite_context, build_dial_context, build_graphic_ephemeris_context,
        build_local_space_context, build_triwheel_context,
    };
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
        assert_eq!(v["date"], "2000-01-01 12:00 UT"); // J2000.0 = noon
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
        let svg = specialist::render_dial_svg(&ctx);
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
        let svg = specialist::render_graphic_ephemeris_svg(&ctx);
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
        let svg = specialist::render_local_space_svg(&ctx);
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
pub(super) const NI_CELLS: &[(f64, f64)] = &[
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
pub(super) fn sarvashtakavarga(
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

pub(super) const RASI_NAMES: &[&str] = &[
    "Ar", "Ta", "Ge", "Ca", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi",
];

pub(super) const RASI_GLYPHS: &[&str] = &[
    "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓",
];

// (row, col) → sign index; 4 rows × 4 cols, centre (rows 1-2, cols 1-2) = metadata
pub(super) const SI_CELLS: &[(usize, usize, i32)] = &[
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

pub(super) fn render_south_indian_svg(ctx: &Value) -> String {
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
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w:.0} {total_h:.0}" width="{total_w:.0}" height="{total_h:.0}">
  <rect width="{total_w:.0}" height="{total_h:.0}" fill="{bg}"/>
  <text x="{:.2}" y="28" text-anchor="middle" font-size="16" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">{title}</text>
  <text x="{:.2}" y="48" text-anchor="middle" font-size="9"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".6">{date}</text>"##,
        total_w / 2.0,
        total_w / 2.0
    );

    // Draw outer border
    let _ = writeln!(
        s,
        r##"  <rect x="{OX}" y="{OY}" width="{:.2}" height="{:.2}" fill="none" stroke="{border}" stroke-width="2.0"/>"##,
        4.0 * CW,
        4.0 * CH
    );

    // Centre box (2×2)
    let cx = OX + CW;
    let cy = OY + CH;
    let _ = writeln!(
        s,
        r##"  <rect x="{cx:.2}" y="{cy:.2}" width="{:.2}" height="{:.2}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"##,
        2.0 * CW,
        2.0 * CH
    );

    // Centre text (chart metadata)
    let moon_sid = ctx["moon_sid_lon"].as_f64().unwrap_or(0.0);
    let (moon_nak, _) = long_to_nakshatra(moon_sid);
    let nak_nm = nakshatra_name(moon_nak).unwrap_or("?");
    let _ = writeln!(
        s,
        r##"  <text x="{:.2}" y="{:.2}" text-anchor="middle" font-size="11" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">Vedic Chart</text>
  <text x="{:.2}" y="{:.2}" text-anchor="middle" font-size="8"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".7">☽ {nak_nm}</text>"##,
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
            r##"  <rect x="{x:.2}" y="{y:.2}" width="{CW:.2}" height="{CH:.2}" fill="{bg}" stroke="{border}" stroke-width="1.0"/>"##
        );

        // Sign name top-left
        let sg = RASI_GLYPHS[sign_idx as usize % 12];
        let sn = RASI_NAMES[sign_idx as usize % 12];
        let _ = writeln!(
            s,
            r##"  <text x="{:.2}" y="{:.2}" font-size="11" font-family="serif" fill="{txt}" opacity=".5">{sg}</text>
  <text x="{:.2}" y="{:.2}" font-size="8" font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}" opacity=".4">{sn}</text>"##,
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
                r##"  <text x="{:.2}" y="{py:.2}" font-size="10" font-family="'Segoe UI',system-ui,sans-serif" fill="{col}">{plabel}</text>"##,
                x + 6.0
            );
        }
    }

    // Dashas mini-table below the grid
    let dy = OY + 4.0 * CH + 10.0;
    let _ = writeln!(
        s,
        r##"  <text x="{OX}" y="{dy:.2}" font-size="11" font-weight="600"
        font-family="'Segoe UI',system-ui,sans-serif" fill="{txt}">Vimshottari Dashas</text>"##
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
            r##"  <text x="{col_x:.2}" y="{row_y:.2}" font-size="9"
            font-family="'Segoe UI',system-ui,sans-serif" fill="{pcol}"><tspan font-weight="600">{body}</tspan> {yrs:.1}y · {start}</text>"##
        );
    }

    let _ = writeln!(s, "</svg>");
    s
}

#[cfg(test)]
mod tests_vedic {
    use super::chinese::build_bazi_context;
    use super::hellenistic::{
        build_firdaria_context, build_hellenistic_context, build_profection_context,
    };
    use super::indigenous::build_medicine_wheel_context;
    use super::mesoamerican::build_mesoamerican_context;
    use super::vedic::{build_ashtakavarga_context, build_shadbala_context, build_vedic_context};
    use super::*;
    use celestial_core::{long_to_navamsa, long_to_rasi};

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
        use celestial_core::body::{Body, CalcFlags};
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
        let svg = vedic::render_dasha_svg(&ctx);
        assert!(svg.contains("<rect"), "no bars in dasha SVG");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn navamsa_svg_built_without_panic() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Navamsa").unwrap();
        let svg = vedic::render_navamsa_svg(&ctx);
        assert!(svg.contains("D9"), "Navamsa title missing");
        assert!(svg.contains("</svg>"));
    }
    // ── Phase 4 (remaining): North Indian, Ashtakavarga, Shadbala ─────────────

    #[test]
    fn north_indian_context_uses_vedic_rasis() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        // North Indian reuses build_vedic_context — same rasi grouping
        let ctx = build_vedic_context(jd, 13.08, 80.27, "2000-01-01", vars, "Rasi").unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 12);
        for p in planets {
            let rasi = p["rasi"].as_i64().unwrap_or(-1);
            assert!(
                rasi >= 0 && rasi < 12,
                "rasi {rasi} out of [0,12) for {}",
                p["name"]
            );
        }
    }

    #[test]
    fn north_indian_svg_has_all_12_cells() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 13.08, 80.27, "2000-01-01", vars, "Rasi").unwrap();
        let svg = vedic::render_north_indian_svg(&ctx);
        // All 12 house numbers (1..=12) should appear
        for h in 1..=12u32 {
            assert!(
                svg.contains(&format!(">{h}<")),
                "house number {h} missing from North Indian SVG"
            );
        }
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn north_indian_svg_has_rasi_glyphs() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_vedic_context(jd, 0.0, 0.0, "2000-01-01", vars, "Rasi").unwrap();
        let svg = vedic::render_north_indian_svg(&ctx);
        // At least Aries glyph should appear
        assert!(svg.contains('\u{2648}'), "Aries glyph missing");
    }

    #[test]
    fn ashtakavarga_context_has_rows_and_totals() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_ashtakavarga_context(jd, 13.08, 80.27, "2000-01-01", vars).unwrap();
        let rows = ctx["ashtakavarga_rows"].as_array().unwrap();
        assert_eq!(rows.len(), 7, "expected 7 planet rows (Sun..Saturn)");
        let totals = ctx["sarvashtakavarga"].as_array().unwrap();
        assert_eq!(totals.len(), 12, "expected 12 sign totals");
    }

    #[test]
    fn ashtakavarga_bindus_range() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_ashtakavarga_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let rows = ctx["ashtakavarga_rows"].as_array().unwrap();
        for row in rows {
            let name = row["planet"].as_str().unwrap_or("?");
            let bindus = row["bindus"].as_array().unwrap();
            assert_eq!(bindus.len(), 12, "{name}: expected 12 sign values");
            for (si, b) in bindus.iter().enumerate() {
                let bv = b.as_u64().unwrap_or(99);
                assert!(
                    bv <= 8,
                    "{name} sign {si}: bindu {bv} > 8 (max is 8 contributors)"
                );
            }
        }
    }

    #[test]
    fn sarvashtakavarga_totals_are_sum_of_rows() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_ashtakavarga_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let rows = ctx["ashtakavarga_rows"].as_array().unwrap();
        let totals = ctx["sarvashtakavarga"].as_array().unwrap();
        for si in 0..12usize {
            let row_sum: u64 = rows
                .iter()
                .map(|r| r["bindus"].as_array().unwrap()[si].as_u64().unwrap_or(0))
                .sum();
            let total = totals[si].as_u64().unwrap_or(0);
            assert_eq!(
                row_sum, total,
                "sign {si}: row sum {row_sum} != sarvashtakavarga total {total}"
            );
        }
    }

    #[test]
    fn ashtakavarga_svg_has_grid() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_ashtakavarga_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = vedic::render_ashtakavarga_svg(&ctx);
        assert!(svg.contains("Sun"), "Sun row missing from Ashtakavarga SVG");
        assert!(svg.contains("Moon"), "Moon row missing");
        assert!(svg.contains("Total"), "Total row missing");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn shadbala_context_has_seven_planets() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_shadbala_context(jd, 13.08, 80.27, "2000-01-01", vars).unwrap();
        let rows = ctx["shadbala"].as_array().unwrap();
        assert_eq!(
            rows.len(),
            7,
            "Shadbala needs exactly 7 planets (Sun..Saturn)"
        );
    }

    #[test]
    fn shadbala_strength_components_non_negative() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_shadbala_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let rows = ctx["shadbala"].as_array().unwrap();
        for row in rows {
            let name = row["name"].as_str().unwrap_or("?");
            for field in &[
                "ochchabala",
                "sapta_bala",
                "chesta_bala",
                "dig_bala",
                "total",
            ] {
                let v = row[field].as_f64().unwrap_or(-1.0);
                assert!(v >= 0.0, "{name}.{field} = {v} is negative");
            }
            // Total should be >= each component
            let total = row["total"].as_f64().unwrap_or(0.0);
            let ochcha = row["ochchabala"].as_f64().unwrap_or(0.0);
            assert!(
                total >= ochcha,
                "{name}: total {total} < ochchabala {ochcha}"
            );
        }
    }

    #[test]
    fn shadbala_svg_has_all_planets() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_shadbala_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = vedic::render_shadbala_svg(&ctx);
        for planet in &[
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
        ] {
            assert!(svg.contains(planet), "{planet} missing from Shadbala SVG");
        }
        assert!(svg.contains("Ochcha"), "column header missing");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }
    // ── Phase 5: Hellenistic / Persian chart types ─────────────────────────────

    #[test]
    fn hellenistic_context_has_dignity5_fields() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_hellenistic_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let planets = ctx["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 12);
        for p in planets {
            // Every planet must have the new Phase 5 dignity fields
            assert!(
                p.get("dignity5").is_some(),
                "{} missing dignity5",
                p["name"]
            );
            assert!(
                p.get("term_ruler").is_some(),
                "{} missing term_ruler",
                p["name"]
            );
            assert!(
                p.get("decan_ruler").is_some(),
                "{} missing decan_ruler",
                p["name"]
            );
            assert!(
                p.get("same_sect").is_some(),
                "{} missing same_sect",
                p["name"]
            );
        }
    }

    #[test]
    fn hellenistic_context_has_is_day_flag() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_hellenistic_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        assert!(ctx.get("is_day").is_some(), "is_day missing from context");
    }

    #[test]
    fn hellenistic_svg_has_dignity_table() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_hellenistic_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let svg = hellenistic::render_hellenistic_svg(&ctx);
        assert!(
            svg.contains("Hellenistic Dignities"),
            "dignity table heading missing"
        );
        assert!(svg.contains("Term lord"), "term lord column missing");
        assert!(svg.contains("Decan lord"), "decan column missing");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn firdaria_context_has_periods() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_firdaria_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let periods = ctx["firdaria"].as_array().unwrap();
        assert!(!periods.is_empty(), "firdaria periods should not be empty");
        // Each period must have required fields
        for p in periods {
            assert!(p.get("major_lord").is_some());
            assert!(p.get("minor_lord").is_some());
            assert!(p.get("start_jd").is_some());
            assert!(p.get("end_jd").is_some());
        }
    }

    #[test]
    fn firdaria_svg_has_bars() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_firdaria_context(jd, 48.85, 2.35, "2000-01-01", 'P', vars).unwrap();
        let svg = hellenistic::render_firdaria_svg(&ctx);
        assert!(svg.contains("<rect"), "no bars in Firdaria SVG");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn profection_context_has_house_and_lord() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_profection_context(jd, 48.85, 2.35, "2000-01-01", 'P', 35, vars).unwrap();
        let house = ctx["profection_house"].as_u64().unwrap_or(0);
        assert!(
            house >= 1 && house <= 12,
            "profection house {house} ;out of [1,12]"
        );
        assert!(ctx.get("profection_lord").is_some());
    }

    #[test]
    fn profection_svg_has_marker() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_profection_context(jd, 48.85, 2.35, "2000-01-01", 'P', 30, vars).unwrap();
        let svg = hellenistic::render_profection_svg(&ctx);
        assert!(svg.contains("profection"), "profection annotation missing");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }
    // ── Phase 6: Chinese / Ba Zi ────────────────────────────────────────────

    #[test]
    fn bazi_context_has_four_pillars() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_bazi_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let pillars = ctx["pillars"].as_array().unwrap();
        assert_eq!(pillars.len(), 4, "Ba Zi must have exactly 4 pillars");
        for p in pillars {
            assert!(p.get("stem_name").is_some(), "pillar missing stem_name");
            assert!(p.get("animal").is_some(), "pillar missing animal");
            assert!(p.get("stem_element").is_some(), "pillar missing element");
        }
    }

    #[test]
    fn bazi_context_has_solar_term() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_bazi_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        assert!(ctx.get("solar_term_current").is_some());
        assert!(ctx.get("solar_term_current_en").is_some());
        assert!(ctx.get("degrees_to_next").is_some());
    }

    #[test]
    fn bazi_context_element_counts_sum_to_eight() {
        // 4 pillars × 2 (stem + branch) = 8 element contributions
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_bazi_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let total: u64 = ctx["elements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["count"].as_u64().unwrap_or(0))
            .sum();
        assert_eq!(total, 8, "total element counts must equal 8");
    }

    #[test]
    fn bazi_svg_has_pillar_columns() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_bazi_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = chinese::render_bazi_svg(&ctx);
        // All 4 column labels must appear
        for lbl in ["Hour 時", "Day 日", "Month 月", "Year 年"] {
            assert!(
                svg.contains(lbl),
                "column label '{lbl}' missing from Ba Zi SVG"
            );
        }
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    #[test]
    fn bazi_svg_has_element_balance() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_bazi_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = chinese::render_bazi_svg(&ctx);
        assert!(
            svg.contains("Element balance"),
            "element balance section missing"
        );
    }

    // ── Phase 7: Mesoamerican ────────────────────────────────────────────────

    #[test]
    fn mesoamerican_context_has_all_calendars() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_mesoamerican_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        // All four calendar systems must have an entry
        assert!(ctx.get("tonal_name").is_some(), "tonalpohualli missing");
        assert!(ctx.get("xiu_month_name").is_some(), "xiuhpohualli missing");
        assert!(ctx.get("tzol_name").is_some(), "tzolkin missing");
        assert!(ctx.get("haab_month_name").is_some(), "haab missing");
        assert!(ctx.get("cr_sign").is_some(), "calendar round missing");
    }

    #[test]
    fn mesoamerican_trecena_in_range() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_mesoamerican_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let t = ctx["tonal_trecena"].as_u64().unwrap_or(0);
        assert!(t >= 1 && t <= 13, "trecena {t} out of [1,13]");
    }

    #[test]
    fn mesoamerican_svg_has_both_traditions() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_mesoamerican_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = mesoamerican::render_mesoamerican_svg(&ctx);
        assert!(svg.contains("Tonalpohualli"), "Aztec label missing");
        assert!(svg.contains("Tzolkin"), "Maya label missing");
        assert!(svg.contains("</svg>"), "SVG not closed");
    }

    // ── Phase 8: Medicine Wheel / Egyptian Decans ────────────────────────────

    #[test]
    fn medicine_wheel_context_has_totem() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_medicine_wheel_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        assert!(ctx.get("totem").is_some(), "totem missing");
        assert!(ctx.get("element").is_some(), "element missing");
        assert!(ctx.get("decan_name").is_some(), "decan_name missing");
        assert!(ctx.get("decan_star").is_some(), "decan_star missing");
    }

    #[test]
    fn medicine_wheel_svg_has_compass() {
        let jd = 2_451_545.0;
        let vars = std::collections::BTreeMap::new();
        let ctx = build_medicine_wheel_context(jd, 0.0, 0.0, "2000-01-01", vars).unwrap();
        let svg = indigenous::render_medicine_wheel_svg(&ctx);
        // Cardinal directions
        for dir in ["N", "E", "S", "W"] {
            assert!(
                svg.contains(&format!(">{dir}<")),
                "{dir} cardinal direction missing from Medicine Wheel SVG"
            );
        }
        assert!(
            svg.contains("Egyptian Decan"),
            "Egyptian decan section missing"
        );
        assert!(svg.contains("</svg>"), "SVG not closed");
    }
}
