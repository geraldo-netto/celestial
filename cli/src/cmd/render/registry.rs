//! Chart-type registry: `ChartRenderer`/`ChartBuilder`, the static
//! `CHART_REGISTRY`, every `dispatch_*` builder, type resolution, and
//! the `vars_with_title` helper they share.
//!
//! Split out of the former 3.5k-line `mod.rs` god file — pure code
//! movement, no behaviour change (the facade re-exports these).

use super::*;
use celestial_core::JulianDay;

/// Clone `user_vars` and ensure a `title` field exists, using `default`
/// only when the user didn't supply one via `--var title=…`.
pub(crate) fn vars_with_title(
    user_vars: &BTreeMap<String, String>,
    default: &str,
) -> BTreeMap<String, String> {
    let mut v = user_vars.clone();
    v.entry("title".to_string())
        .or_insert_with(|| default.to_string());
    v
}

pub(crate) type ChartRenderer = fn(&ChartContext) -> String;

pub(crate) fn dispatch_natal(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Natal Chart");
    Ok((
        build_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?.into(),
        render_builtin_svg,
    ))
}

pub(crate) fn dispatch_cosmogram(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let mut v = vars_with_title(user_vars, "Cosmogram");
    v.insert("no_houses".to_string(), "1".to_string());
    Ok((
        build_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?.into(),
        render_cosmogram_svg,
    ))
}

pub(crate) fn dispatch_solar_return(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let year = args.return_year.unwrap_or_else(|| {
        let today_jd = crate::parse::parse_date("now").unwrap_or(2_451_545.0);
        let d = celestial_core::revjul(JulianDay::new(today_jd), celestial_core::body::Calendar::Gregorian);
        d.year as i32
    });
    let sr_jd = solar_return_jd(JulianDay::new(jd), year, CalcFlags::BUILTIN)?;
    let sr_date = jd_to_date_str(sr_jd);
    let mut v = user_vars.clone();
    v.insert("title".to_string(), format!("Solar Return {year}"));
    v.insert(
        "chart_type_label".to_string(),
        format!("Solar Return {year}"),
    );
    Ok((
        build_context(sr_jd, args.lat, args.lon, &sr_date, args.hsys, v)?.into(),
        render_builtin_svg,
    ))
}

pub(crate) fn dispatch_lunar_return(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let start = args
        .date2
        .as_deref()
        .map(crate::parse::parse_date)
        .transpose()?
        .unwrap_or(jd);
    let lr_jd = lunar_return_jd(JulianDay::new(jd), JulianDay::new(start), CalcFlags::BUILTIN)?;
    let lr_date = jd_to_date_str(lr_jd);
    let mut v = user_vars.clone();
    v.insert("title".to_string(), "Lunar Return".to_string());
    Ok((
        build_context(lr_jd, args.lat, args.lon, &lr_date, args.hsys, v)?.into(),
        render_builtin_svg,
    ))
}

pub(crate) fn dispatch_progressed(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let years = args
        .years
        .ok_or("--years DECIMAL required for progressed chart")?;
    let mut v = user_vars.clone();
    v.insert(
        "title".to_string(),
        format!("Secondary Progressions ({years:.1}y)"),
    );
    Ok((
        build_progressed_context(jd, years, args.lat, args.lon, &args.date, args.hsys, v)?.into(),
        render_progressed_svg,
    ))
}

pub(crate) fn dispatch_solar_arc(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let years = args
        .years
        .ok_or("--years DECIMAL required for solar-arc chart")?;
    let mut v = user_vars.clone();
    v.insert(
        "title".to_string(),
        format!("Solar Arc Directions ({years:.1}y)"),
    );
    Ok((
        build_solar_arc_context(jd, years, args.lat, args.lon, &args.date, args.hsys, v)?.into(),
        render_progressed_svg,
    ))
}

pub(crate) fn dispatch_biwheel(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let date2 = args
        .date2
        .as_deref()
        .ok_or("--date2 DATE required for biwheel chart")?;
    let jd2 = crate::parse::parse_date(date2)?;
    let lat2 = args.lat2.unwrap_or(args.lat);
    let lon2 = args.lon2.unwrap_or(args.lon);
    let v = vars_with_title(user_vars, "Bi-wheel");
    Ok((
        build_biwheel_context(
            jd, jd2, args.lat, args.lon, lat2, lon2, &args.date, date2, args.hsys, v,
        )?.into(),
        render_biwheel_svg,
    ))
}

pub(crate) fn dispatch_composite(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let date2 = args
        .date2
        .as_deref()
        .ok_or("--date2 DATE required for composite chart")?;
    let jd2 = crate::parse::parse_date(date2)?;
    let v = vars_with_title(user_vars, "Composite Chart");
    Ok((
        specialist::build_composite_context(
            jd, jd2, args.lat, args.lon, &args.date, date2, args.hsys, v,
        )?.into(),
        render_builtin_svg,
    ))
}

pub(crate) fn dispatch_triwheel(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let date2 = args
        .date2
        .as_deref()
        .ok_or("--date2 required (ring 2) for tri-wheel")?;
    let date3 = args
        .date3
        .as_deref()
        .ok_or("--date3 required (ring 3) for tri-wheel")?;
    let jd2 = crate::parse::parse_date(date2)?;
    let jd3 = crate::parse::parse_date(date3)?;
    let v = vars_with_title(user_vars, "Tri-wheel");
    Ok((
        specialist::build_triwheel_context(
            jd, jd2, jd3, args.lat, args.lon, &args.date, date2, date3, args.hsys, v,
        )?.into(),
        specialist::render_triwheel_svg,
    ))
}

pub(crate) fn dispatch_ephemeris(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let date2 = args.date2.as_deref().unwrap_or("now");
    let jd2 = crate::parse::parse_date(date2)?;
    let (jd_s, jd_e) = if jd < jd2 { (jd, jd2) } else { (jd2, jd) };
    let v = vars_with_title(user_vars, "Graphic Ephemeris");
    Ok((
        specialist::build_graphic_ephemeris_context(jd_s, jd_e, v)?.into(),
        specialist::render_graphic_ephemeris_svg,
    ))
}

pub(crate) fn dispatch_profection(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    let age = args
        .years
        .map(|y| y as u32)
        .or(args.return_year.map(|r| r as u32))
        .unwrap_or(0);
    let mut v = user_vars.clone();
    v.entry("title".to_string())
        .or_insert_with(|| format!("Profection — Age {age}"));
    Ok((
        hellenistic::build_profection_context(
            jd, args.lat, args.lon, &args.date, args.hsys, age, v,
        )?.into(),
        hellenistic::render_profection_svg,
    ))
}

// ── Remaining chart-type dispatchers ──────────────────────────────────────────
// One thin wrapper per chart type so every entry in CHART_REGISTRY can be a
// uniform `ChartBuilder` fn pointer. Each wrapper reads `args` for the params
// its underlying builder needs.

// ── Uniform specialist dispatchers (DP-1b) ───────────────────────────────────
//
// The 15 dispatchers below were byte-for-byte identical except for
// (title, builder fn, renderer fn) — only the wrapper layer was duplicated;
// the `build_*_context` fns themselves stay heterogeneous. This macro keeps
// each `pub(crate) fn dispatch_*` (so `CHART_REGISTRY` still names them) while
// removing the copy-paste. Expansion is token-identical to the former hand-
// written bodies → SVG output is unchanged (byte-identical precision gate).
//
// Shapes: `hsys` = builder takes `args.hsys`; `vedic <kind>` = the shared
// `build_vedic_context(..,, kind)`; `novars` = no title, raw `user_vars`;
// bare = `(jd, lat, lon, &date, v)`.
macro_rules! specialist_dispatch {
    ($fn:ident, $title:literal, hsys $build:path => $render:path) => {
        pub(crate) fn $fn(
            jd: f64,
            args: &RenderArgs,
            user_vars: &BTreeMap<String, String>,
        ) -> Result<(ChartContext, ChartRenderer), CliError> {
            let v = vars_with_title(user_vars, $title);
            Ok((
                $build(jd, args.lat, args.lon, &args.date, args.hsys, v)?.into(),
                $render,
            ))
        }
    };
    ($fn:ident, $title:literal, vedic $kind:literal => $render:path) => {
        pub(crate) fn $fn(
            jd: f64,
            args: &RenderArgs,
            user_vars: &BTreeMap<String, String>,
        ) -> Result<(ChartContext, ChartRenderer), CliError> {
            let v = vars_with_title(user_vars, $title);
            Ok((
                vedic::build_vedic_context(jd, args.lat, args.lon, &args.date, v, $kind)?.into(),
                $render,
            ))
        }
    };
    ($fn:ident, novars $build:path => $render:path) => {
        pub(crate) fn $fn(
            jd: f64,
            _args: &RenderArgs,
            user_vars: &BTreeMap<String, String>,
        ) -> Result<(ChartContext, ChartRenderer), CliError> {
            Ok(($build(jd, user_vars.clone())?.into(), $render))
        }
    };
    ($fn:ident, $title:literal, $build:path => $render:path) => {
        pub(crate) fn $fn(
            jd: f64,
            args: &RenderArgs,
            user_vars: &BTreeMap<String, String>,
        ) -> Result<(ChartContext, ChartRenderer), CliError> {
            let v = vars_with_title(user_vars, $title);
            Ok((
                $build(jd, args.lat, args.lon, &args.date, v)?.into(),
                $render,
            ))
        }
    };
}

specialist_dispatch!(dispatch_dial, "90° Midpoint Dial",
    hsys specialist::build_dial_context => specialist::render_dial_svg);
specialist_dispatch!(dispatch_local_space, "Local Space",
    specialist::build_local_space_context => specialist::render_local_space_svg);
specialist_dispatch!(dispatch_rasi, "Rasi Chart (South Indian)",
    vedic "Rasi" => render_south_indian_svg);
specialist_dispatch!(dispatch_navamsa, "Navamsa D9 Chart",
    vedic "Navamsa" => vedic::render_navamsa_svg);
specialist_dispatch!(dispatch_dasha, "Vimshottari Dasha Timeline",
    vedic "Dasha" => vedic::render_dasha_svg);
specialist_dispatch!(dispatch_north_indian, "North Indian Chart",
    vedic "Rasi" => vedic::render_north_indian_svg);
specialist_dispatch!(dispatch_ashtakavarga, "Ashtakavarga",
    vedic::build_ashtakavarga_context => vedic::render_ashtakavarga_svg);
specialist_dispatch!(dispatch_shadbala, "Shadbala",
    vedic::build_shadbala_context => vedic::render_shadbala_svg);
specialist_dispatch!(dispatch_hellenistic, "Hellenistic Chart",
    hsys hellenistic::build_hellenistic_context => hellenistic::render_hellenistic_svg);
specialist_dispatch!(dispatch_firdaria, "Firdaria Timeline",
    hsys hellenistic::build_firdaria_context => hellenistic::render_firdaria_svg);
specialist_dispatch!(dispatch_bazi, "Four Pillars (八字)",
    chinese::build_bazi_context => chinese::render_bazi_svg);
specialist_dispatch!(dispatch_mesoamerican, "Mesoamerican Calendars",
    mesoamerican::build_mesoamerican_context => mesoamerican::render_mesoamerican_svg);
specialist_dispatch!(dispatch_medicine_wheel, "Medicine Wheel / Egyptian Decans",
    indigenous::build_medicine_wheel_context => indigenous::render_medicine_wheel_svg);
specialist_dispatch!(dispatch_wheel_of_year,
    novars calendar_wheel::build_sabbat_wheel_context => calendar_wheel::render_sabbat_wheel_svg);
specialist_dispatch!(dispatch_omer_grid,
    novars omer_grid::build_omer_grid_context => omer_grid::render_omer_grid_svg);

pub(crate) fn dispatch_calendar(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    Ok((
        build_calendar_context(jd, args, user_vars)?.into(),
        calendar_overlays::render_default_calendar_svg,
    ))
}

// ── Chart-type registry ──────────────────────────────────────────────────────
//
// Uniform `(aliases, builder)` table. To add a new chart type, append a
// `ChartEntry` here and write its dispatch wrapper above. The `--type` parser
// and the unknown-type error message both derive from this table, so there's
// nothing else to keep in sync.

pub(crate) type ChartBuilder =
    fn(f64, &RenderArgs, &BTreeMap<String, String>) -> Result<(ChartContext, ChartRenderer), CliError>;

pub(crate) struct ChartEntry {
    aliases: &'static [&'static str],
    builder: ChartBuilder,
}

pub(crate) static CHART_REGISTRY: &[ChartEntry] = &[
    ChartEntry {
        aliases: &["natal", ""],
        builder: dispatch_natal,
    },
    ChartEntry {
        aliases: &["cosmogram"],
        builder: dispatch_cosmogram,
    },
    ChartEntry {
        aliases: &["solar-return", "solar_return"],
        builder: dispatch_solar_return,
    },
    ChartEntry {
        aliases: &["lunar-return", "lunar_return"],
        builder: dispatch_lunar_return,
    },
    ChartEntry {
        aliases: &["progressed", "secondary"],
        builder: dispatch_progressed,
    },
    ChartEntry {
        aliases: &["solar-arc", "solar_arc"],
        builder: dispatch_solar_arc,
    },
    ChartEntry {
        aliases: &["biwheel", "bi-wheel", "synastry", "transit"],
        builder: dispatch_biwheel,
    },
    ChartEntry {
        aliases: &["composite"],
        builder: dispatch_composite,
    },
    ChartEntry {
        aliases: &["triwheel", "tri-wheel"],
        builder: dispatch_triwheel,
    },
    ChartEntry {
        aliases: &["dial", "90dial", "midpoint-dial"],
        builder: dispatch_dial,
    },
    ChartEntry {
        aliases: &["ephemeris", "graphic-ephemeris"],
        builder: dispatch_ephemeris,
    },
    ChartEntry {
        aliases: &["local-space", "localspace"],
        builder: dispatch_local_space,
    },
    ChartEntry {
        aliases: &["rasi", "vedic", "south-indian"],
        builder: dispatch_rasi,
    },
    ChartEntry {
        aliases: &["navamsa", "d9"],
        builder: dispatch_navamsa,
    },
    ChartEntry {
        aliases: &["dasha", "vimshottari"],
        builder: dispatch_dasha,
    },
    ChartEntry {
        aliases: &["north-indian", "north_indian"],
        builder: dispatch_north_indian,
    },
    ChartEntry {
        aliases: &["ashtakavarga", "ashtak"],
        builder: dispatch_ashtakavarga,
    },
    ChartEntry {
        aliases: &["shadbala", "strength"],
        builder: dispatch_shadbala,
    },
    ChartEntry {
        aliases: &["hellenistic", "greek"],
        builder: dispatch_hellenistic,
    },
    ChartEntry {
        aliases: &["firdaria", "persian"],
        builder: dispatch_firdaria,
    },
    ChartEntry {
        aliases: &["profection"],
        builder: dispatch_profection,
    },
    ChartEntry {
        aliases: &["bazi", "four-pillars", "chinese"],
        builder: dispatch_bazi,
    },
    ChartEntry {
        aliases: &["mesoamerican", "aztec", "maya"],
        builder: dispatch_mesoamerican,
    },
    ChartEntry {
        aliases: &["medicine-wheel", "indigenous", "egyptian-decans"],
        builder: dispatch_medicine_wheel,
    },
    ChartEntry {
        aliases: &["wheel-of-year", "sabbats", "celtic"],
        builder: dispatch_wheel_of_year,
    },
    ChartEntry {
        aliases: &["omer-grid", "omer", "sefirat-haomer"],
        builder: dispatch_omer_grid,
    },
    ChartEntry {
        aliases: &["calendar"],
        builder: dispatch_calendar,
    },
];

/// Comma-separated list of every primary alias in `CHART_REGISTRY`,
/// for use in error messages. Skips the empty-string alias (default).
pub(crate) fn registered_chart_types() -> String {
    CHART_REGISTRY
        .iter()
        .flat_map(|e| e.aliases.iter().copied())
        .filter(|a| !a.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Build the chart context and select the SVG renderer for the requested
/// `--chart-type`.
pub(crate) fn dispatch_chart_type(
    chart_type: &str,
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(ChartContext, ChartRenderer), CliError> {
    CHART_REGISTRY
        .iter()
        .find(|e| e.aliases.contains(&chart_type))
        .map(|e| (e.builder)(jd, args, user_vars))
        .unwrap_or_else(|| {
            Err(CliError::Parse(format!(
                "unknown --type '{chart_type}'; valid: {}",
                registered_chart_types()
            )))
        })
}
