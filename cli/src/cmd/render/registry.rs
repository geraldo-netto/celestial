//! Chart-type registry: `ChartRenderer`/`ChartBuilder`, the static
//! `CHART_REGISTRY`, every `dispatch_*` builder, type resolution, and
//! the `vars_with_title` helper they share.
//!
//! Split out of the former 3.5k-line `mod.rs` god file — pure code
//! movement, no behaviour change (the facade re-exports these).

use super::*;

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

pub(crate) type ChartRenderer = fn(&serde_json::Value) -> String;

pub(crate) fn dispatch_natal(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Natal Chart");
    Ok((
        build_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?,
        render_builtin_svg,
    ))
}

pub(crate) fn dispatch_cosmogram(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let mut v = vars_with_title(user_vars, "Cosmogram");
    v.insert("no_houses".to_string(), "1".to_string());
    Ok((
        build_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?,
        render_cosmogram_svg,
    ))
}

pub(crate) fn dispatch_solar_return(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let year = args.return_year.unwrap_or_else(|| {
        let today_jd = crate::parse::parse_date("now").unwrap_or(2_451_545.0);
        let d = celestial_core::revjul(today_jd, celestial_core::body::Calendar::Gregorian);
        d.year as i32
    });
    let sr_jd = solar_return_jd(jd, year, CalcFlags::BUILTIN)?;
    let sr_date = jd_to_date_str(sr_jd);
    let mut v = user_vars.clone();
    v.insert("title".to_string(), format!("Solar Return {year}"));
    v.insert(
        "chart_type_label".to_string(),
        format!("Solar Return {year}"),
    );
    Ok((
        build_context(sr_jd, args.lat, args.lon, &sr_date, args.hsys, v)?,
        render_builtin_svg,
    ))
}

pub(crate) fn dispatch_lunar_return(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let start = args
        .date2
        .as_deref()
        .map(crate::parse::parse_date)
        .transpose()?
        .unwrap_or(jd);
    let lr_jd = lunar_return_jd(jd, start, CalcFlags::BUILTIN)?;
    let lr_date = jd_to_date_str(lr_jd);
    let mut v = user_vars.clone();
    v.insert("title".to_string(), "Lunar Return".to_string());
    Ok((
        build_context(lr_jd, args.lat, args.lon, &lr_date, args.hsys, v)?,
        render_builtin_svg,
    ))
}

pub(crate) fn dispatch_progressed(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let years = args
        .years
        .ok_or("--years DECIMAL required for progressed chart")?;
    let mut v = user_vars.clone();
    v.insert(
        "title".to_string(),
        format!("Secondary Progressions ({years:.1}y)"),
    );
    Ok((
        build_progressed_context(jd, years, args.lat, args.lon, &args.date, args.hsys, v)?,
        render_progressed_svg,
    ))
}

pub(crate) fn dispatch_solar_arc(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let years = args
        .years
        .ok_or("--years DECIMAL required for solar-arc chart")?;
    let mut v = user_vars.clone();
    v.insert(
        "title".to_string(),
        format!("Solar Arc Directions ({years:.1}y)"),
    );
    Ok((
        build_solar_arc_context(jd, years, args.lat, args.lon, &args.date, args.hsys, v)?,
        render_progressed_svg,
    ))
}

pub(crate) fn dispatch_biwheel(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
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
        )?,
        render_biwheel_svg,
    ))
}

pub(crate) fn dispatch_composite(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let date2 = args
        .date2
        .as_deref()
        .ok_or("--date2 DATE required for composite chart")?;
    let jd2 = crate::parse::parse_date(date2)?;
    let v = vars_with_title(user_vars, "Composite Chart");
    Ok((
        specialist::build_composite_context(
            jd, jd2, args.lat, args.lon, &args.date, date2, args.hsys, v,
        )?,
        render_builtin_svg,
    ))
}

pub(crate) fn dispatch_triwheel(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
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
        )?,
        specialist::render_triwheel_svg,
    ))
}

pub(crate) fn dispatch_ephemeris(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let date2 = args.date2.as_deref().unwrap_or("now");
    let jd2 = crate::parse::parse_date(date2)?;
    let (jd_s, jd_e) = if jd < jd2 { (jd, jd2) } else { (jd2, jd) };
    let v = vars_with_title(user_vars, "Graphic Ephemeris");
    Ok((
        specialist::build_graphic_ephemeris_context(jd_s, jd_e, v)?,
        specialist::render_graphic_ephemeris_svg,
    ))
}

pub(crate) fn dispatch_profection(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
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
        )?,
        hellenistic::render_profection_svg,
    ))
}

// ── Remaining chart-type dispatchers ──────────────────────────────────────────
// One thin wrapper per chart type so every entry in CHART_REGISTRY can be a
// uniform `ChartBuilder` fn pointer. Each wrapper reads `args` for the params
// its underlying builder needs.

pub(crate) fn dispatch_dial(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "90° Midpoint Dial");
    Ok((
        specialist::build_dial_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?,
        specialist::render_dial_svg,
    ))
}

pub(crate) fn dispatch_local_space(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Local Space");
    Ok((
        specialist::build_local_space_context(jd, args.lat, args.lon, &args.date, v)?,
        specialist::render_local_space_svg,
    ))
}

pub(crate) fn dispatch_rasi(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Rasi Chart (South Indian)");
    Ok((
        vedic::build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Rasi")?,
        render_south_indian_svg,
    ))
}

pub(crate) fn dispatch_navamsa(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Navamsa D9 Chart");
    Ok((
        vedic::build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Navamsa")?,
        vedic::render_navamsa_svg,
    ))
}

pub(crate) fn dispatch_dasha(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Vimshottari Dasha Timeline");
    Ok((
        vedic::build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Dasha")?,
        vedic::render_dasha_svg,
    ))
}

pub(crate) fn dispatch_north_indian(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "North Indian Chart");
    Ok((
        vedic::build_vedic_context(jd, args.lat, args.lon, &args.date, v, "Rasi")?,
        vedic::render_north_indian_svg,
    ))
}

pub(crate) fn dispatch_ashtakavarga(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Ashtakavarga");
    Ok((
        vedic::build_ashtakavarga_context(jd, args.lat, args.lon, &args.date, v)?,
        vedic::render_ashtakavarga_svg,
    ))
}

pub(crate) fn dispatch_shadbala(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Shadbala");
    Ok((
        vedic::build_shadbala_context(jd, args.lat, args.lon, &args.date, v)?,
        vedic::render_shadbala_svg,
    ))
}

pub(crate) fn dispatch_hellenistic(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Hellenistic Chart");
    Ok((
        hellenistic::build_hellenistic_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?,
        hellenistic::render_hellenistic_svg,
    ))
}

pub(crate) fn dispatch_firdaria(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Firdaria Timeline");
    Ok((
        hellenistic::build_firdaria_context(jd, args.lat, args.lon, &args.date, args.hsys, v)?,
        hellenistic::render_firdaria_svg,
    ))
}

pub(crate) fn dispatch_bazi(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Four Pillars (八字)");
    Ok((
        chinese::build_bazi_context(jd, args.lat, args.lon, &args.date, v)?,
        chinese::render_bazi_svg,
    ))
}

pub(crate) fn dispatch_mesoamerican(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Mesoamerican Calendars");
    Ok((
        mesoamerican::build_mesoamerican_context(jd, args.lat, args.lon, &args.date, v)?,
        mesoamerican::render_mesoamerican_svg,
    ))
}

pub(crate) fn dispatch_medicine_wheel(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    let v = vars_with_title(user_vars, "Medicine Wheel / Egyptian Decans");
    Ok((
        indigenous::build_medicine_wheel_context(jd, args.lat, args.lon, &args.date, v)?,
        indigenous::render_medicine_wheel_svg,
    ))
}

pub(crate) fn dispatch_wheel_of_year(
    jd: f64,
    _args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    Ok((
        calendar_wheel::build_sabbat_wheel_context(jd, user_vars.clone())?,
        calendar_wheel::render_sabbat_wheel_svg,
    ))
}

pub(crate) fn dispatch_omer_grid(
    jd: f64,
    _args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    Ok((
        omer_grid::build_omer_grid_context(jd, user_vars.clone())?,
        omer_grid::render_omer_grid_svg,
    ))
}

pub(crate) fn dispatch_calendar(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<(Value, ChartRenderer), CliError> {
    Ok((
        build_calendar_context(jd, args, user_vars)?,
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
    fn(f64, &RenderArgs, &BTreeMap<String, String>) -> Result<(Value, ChartRenderer), CliError>;

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
) -> Result<(serde_json::Value, ChartRenderer), CliError> {
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
