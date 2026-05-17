//! Render pipeline: the `run` entry point, output writing, the
//! calendar/overlay context builders and the built-in `--print-*`
//! payloads.
//!
//! Split out of the former 3.5k-line `mod.rs` god file — pure code
//! movement, no behaviour change (re-exported by the facade).

use super::*;
use std::path::PathBuf;

/// Default colour palette for the calendar chart-type. Returns a JSON Object
/// with `vars` that the calendar template uses (these can be overridden via
/// `--var bg_color=…` or `[vars]` in a TOML config).
pub(crate) fn calendar_default_vars(
    user_vars: &BTreeMap<String, String>,
    title_default: String,
) -> serde_json::Value {
    let mut vars = user_vars.clone();
    vars.entry("title".to_string()).or_insert(title_default);
    vars.entry("bg_color".to_string())
        .or_insert_with(|| "#ffffff".to_string());
    vars.entry("text_color".to_string())
        .or_insert_with(|| "#222".to_string());
    vars.entry("ring_color".to_string())
        .or_insert_with(|| "#888".to_string());
    vars.entry("accent_color".to_string())
        .or_insert_with(|| "#5c4a8a".to_string());
    vars.entry("lag_color".to_string())
        .or_insert_with(|| "#c87f32".to_string());
    vars.entry("sabbat_color".to_string())
        .or_insert_with(|| "#3d6b35".to_string());
    vars.entry("moon_color".to_string())
        .or_insert_with(|| "#3a4a6a".to_string());
    let mut vars_json = serde_json::Map::new();
    for (k, v) in vars {
        vars_json.insert(k, serde_json::json!(v));
    }
    serde_json::Value::Object(vars_json)
}

/// Build the seed context for `--chart-type calendar` (single-month). Always
/// includes the `gregorian` spine and any requested overlays as null-or-set
/// fields; `apply_universal_overlays` will populate / annotate them.
pub(crate) fn build_calendar_context(
    jd: f64,
    args: &RenderArgs,
    user_vars: &BTreeMap<String, String>,
) -> Result<serde_json::Value, CliError> {
    let date = revjul(jd, Calendar::Gregorian);
    let year = date.year;
    let month = args.month.unwrap_or(date.month as u32);

    // Default to all overlays if none requested explicitly.
    let calendars: Vec<String> = if args.calendars.is_empty() {
        vec![
            "gregorian".to_string(),
            "omer".to_string(),
            "sabbats".to_string(),
            "moon".to_string(),
        ]
    } else {
        let mut v: Vec<String> = args
            .calendars
            .iter()
            .map(|c| c.as_str().to_string())
            .collect();
        if !v.iter().any(|c| c == "gregorian") {
            v.insert(0, "gregorian".to_string());
        }
        v
    };

    let gregorian = calendar_overlays::gregorian_overlay(year, month);
    let title_default = format!("{} {year}", gregorian["month_name"].as_str().unwrap_or(""));

    Ok(serde_json::json!({
        "jd":        jd,
        "year":      year,
        "month":     month,
        "calendars": calendars,
        "gregorian": gregorian,
        // Empty placeholders — apply_universal_overlays will fill these in
        // when `--calendar X` is requested.
        "omer":      serde_json::Value::Null,
        "sabbats":   serde_json::Value::Null,
        "moon":      serde_json::Value::Null,
        "hebrew":    serde_json::Value::Null,
        "vars":      calendar_default_vars(user_vars, title_default),
    }))
}

/// Resolve a single `--calendar X` name to its (key, JSON value) tuple.
/// Returns None and emits a warning if the name is unrecognised.
pub(crate) fn resolve_overlay(
    name: &str,
    jd: f64,
    year: i32,
    month: u32,
) -> Option<(String, serde_json::Value)> {
    match name {
        "gregorian" => Some((
            "gregorian".to_string(),
            calendar_overlays::gregorian_overlay(year, month),
        )),
        "gregorian-year" | "year-calendar" => Some((
            "gregorian_year".to_string(),
            calendar_overlays::gregorian_year_overlay(year),
        )),
        "omer" => Some(("omer".to_string(), calendar_overlays::omer_overlay(jd))),
        "sabbats" | "wheel" => Some((
            "sabbats".to_string(),
            calendar_overlays::sabbats_overlay(year),
        )),
        "moon" | "lunar" => Some((
            "moon".to_string(),
            calendar_overlays::moon_overlay(year, month),
        )),
        "hebrew" | "jewish" => {
            let jd_start = celestial_core::julday(year, 1, 1, 0.0, Calendar::Gregorian);
            let jd_end = celestial_core::julday(year, 12, 31, 23.99, Calendar::Gregorian);
            Some((
                "hebrew".to_string(),
                calendar_overlays::hebrew_overlay(jd_start, jd_end),
            ))
        }
        other => {
            eprintln!("warning: unknown --calendar '{other}' (valid: gregorian gregorian-year omer sabbats moon hebrew)");
            None
        }
    }
}

/// Annotate one Gregorian-month JSON value's `days[]` array with the cross-
/// overlay tags it cares about (omer / sabbat / moon / hebrew).
pub(crate) fn annotate_month(
    month_value: &mut serde_json::Value,
    moon_overlay: &serde_json::Value,
    omer_v: &Option<serde_json::Value>,
    sabbats_v: &Option<serde_json::Value>,
    hebrew_v: &Option<serde_json::Value>,
) {
    if let Some(o) = omer_v {
        calendar_overlays::annotate_gregorian_with_omer(month_value, o);
    }
    if let Some(s) = sabbats_v {
        calendar_overlays::annotate_gregorian_with_sabbats(month_value, s);
    }
    calendar_overlays::annotate_gregorian_with_moon(month_value, moon_overlay);
    if let Some(h) = hebrew_v {
        calendar_overlays::annotate_gregorian_with_hebrew(month_value, h);
    }
}

/// Apply universally-requested calendar overlays (`--calendar X --calendar Y …`)
/// to `ctx`. Each named overlay merges a top-level field into the context;
/// `gregorian` and `gregorian_year` overlays additionally have their `days[]`
/// arrays annotated with cross-overlay tags so a single template loop can
/// tag a day from any tradition.
pub(crate) fn apply_universal_overlays(ctx: &mut serde_json::Value, jd: f64, calendars: &[String]) {
    if calendars.is_empty() {
        return;
    }
    let date_now = revjul(jd, Calendar::Gregorian);
    let year = date_now.year;
    let month = date_now.month as u32;

    // Phase 1: compute every requested overlay's value
    let mut overlay_values: std::collections::BTreeMap<String, serde_json::Value> =
        std::collections::BTreeMap::new();
    for c in calendars {
        if let Some((k, v)) = resolve_overlay(c, jd, year, month) {
            overlay_values.insert(k, v);
        }
    }

    // Phase 2: annotate gregorian / gregorian_year days with cross-overlay
    // tags. Clone all referenced values BEFORE any get_mut borrow.
    let omer_v = overlay_values.get("omer").cloned();
    let sabbats_v = overlay_values.get("sabbats").cloned();
    let hebrew_v = overlay_values.get("hebrew").cloned();
    let moon_for_single_month = overlay_values
        .get("moon")
        .cloned()
        .unwrap_or_else(|| calendar_overlays::moon_overlay(year, month));

    if let Some(g) = overlay_values.get_mut("gregorian") {
        annotate_month(g, &moon_for_single_month, &omer_v, &sabbats_v, &hebrew_v);
    }
    if let Some(gy) = overlay_values.get_mut("gregorian_year") {
        if let Some(months_arr) = gy.get_mut("months").and_then(|v| v.as_array_mut()) {
            for (idx, m_val) in months_arr.iter_mut().enumerate() {
                let month_num = (idx + 1) as u32;
                let moon_for_month = calendar_overlays::moon_overlay(year, month_num);
                annotate_month(m_val, &moon_for_month, &omer_v, &sabbats_v, &hebrew_v);
            }
        }
    }

    // Phase 3: merge all overlays into the context
    if let Some(obj) = ctx.as_object_mut() {
        for (k, v) in overlay_values {
            obj.insert(k, v);
        }
    }
}

/// Render the final output: either feeds `ctx` to the built-in renderer
/// for `chart_type`, or runs `ctx` through the user-supplied template.
pub(crate) fn render_to_string(
    ctx: &ChartContext,
    render_fn: ChartRenderer,
    template_path: Option<&PathBuf>,
) -> Result<String, CliError> {
    let Some(tmpl_path) = template_path else {
        return Ok(render_fn(ctx));
    };
    let tmpl_src = std::fs::read_to_string(tmpl_path)
        .map_err(|e| format!("cannot read template `{}`: {e}", tmpl_path.display()))?;
    let mut env = Environment::new();
    // SEC-6: cap the VM instruction budget so a hostile/buggy
    // `--template` (unbounded loop, runaway recursion) fails fast
    // instead of hanging the process. 50M instructions is far above
    // any legitimate chart template (the year-calendar — 365 days ×
    // cells — is the heaviest and stays well under this); recursion is
    // separately bounded by minijinja's default MAX_RECURSION.
    env.set_fuel(Some(50_000_000));
    // SVG output is verbatim — disable HTML auto-escape that would mangle
    // attribute quotes.
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
    env.add_template("t", &tmpl_src)
        .map_err(|e| format!("template parse error: {e}"))?;
    let tpl = env
        .get_template("t")
        .map_err(|e| format!("template lookup error: {e}"))?;
    tpl.render(MjValue::from_serialize(ctx))
        .map_err(|e| CliError::Msg(format!("template render error: {e}")))
}

/// Either write `output` to `path` (creating parent dirs if needed) or
/// print it to stdout if `path` is None.
pub(crate) fn write_or_print(output: &str, path: Option<&PathBuf>) -> Result<(), CliError> {
    let Some(p) = path else {
        print!("{output}");
        return Ok(());
    };
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create output dir: {e}"))?;
        }
    }
    std::fs::write(p, output).map_err(|e| format!("cannot write `{}`: {e}", p.display()))?;
    eprintln!("\u{2713}  wrote {}", p.display());
    Ok(())
}


/// Where a [`compute`] result should go. `Stdout` is the verbatim
/// payload for the `--print-*` modes (always stdout, ignoring `--out`,
/// as before); `Out` honours `--out` via `write_or_print`.
/// Where a [`compute`] result should go. `Stdout` is the verbatim
/// payload for the `--print-*` modes (always stdout, ignoring `--out`,
/// as before); `Out` is the rendered chart, written via
/// `write_or_print` honouring `--out`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RenderOutput {
    Stdout(String),
    Out {
        body: String,
        path: Option<PathBuf>,
    },
}

/// Thin IO wrapper: compute the payload, then emit it.
pub fn run(args: RenderArgs) -> Result<(), CliError> {
    match compute(args)? {
        RenderOutput::Stdout(s) => {
            print!("{s}");
            Ok(())
        }
        RenderOutput::Out { body, path } => write_or_print(&body, path.as_ref()),
    }
}

/// Pure render pipeline: produces the output payload (no stdout, no
/// file write) so it is unit-testable without a filesystem.
pub(crate) fn compute(mut args: RenderArgs) -> Result<RenderOutput, CliError> {
    // --print-schema: dump the context schema and exit. This runs BEFORE any
    // chart computation so the user gets fast feedback while learning the
    // template format — no need for valid date/lat/lon args.
    if args.print_schema {
        return Ok(RenderOutput::Stdout(CONTEXT_SCHEMA.to_string()));
    }

    args.validate()?;

    let file_vars = load_config(&mut args)?;
    let user_vars = apply_var_overrides(&args.vars, file_vars)?;
    let date_str = merge_date_and_time(&args.date, args.time.as_deref());

    let chart_type = args.chart_type.to_lowercase();
    let chart_type = chart_type.trim();

    // `--date` / `--time` are a *local* civil time at the birth place. Convert
    // to UT using the mandatory `--timezone`. `now` and a bare Julian day are
    // already unambiguous in UT, and `--chart-type calendar` only uses the
    // date for its month, so those skip the requirement.
    let trimmed = date_str.trim();
    let is_now_or_jd =
        trimmed.eq_ignore_ascii_case("now") || trimmed.parse::<f64>().is_ok();
    // `tz_label` / `date_local` describe the *input* civil time so the chart
    // can state it explicitly alongside the derived UT (`ctx["date"]`).
    let (jd, tz_label, date_local) = if chart_type != "calendar" && !is_now_or_jd {
        crate::parse::require_datetime(&date_str)?;
        let tz = args.timezone.as_deref().ok_or_else(|| {
            "missing --timezone: a birth time is a *local* time and must be \
             converted to UT. Pass e.g. `--timezone -03:00` or `--tz BRT` \
             (see the README timezone table)."
                .to_string()
        })?;
        let offset = crate::parse::parse_tz_offset(tz)?;
        let label = crate::parse::fmt_utc_offset(offset);
        let local = format!("{} {label}", date_str.trim());
        (
            crate::parse::parse_date(&date_str)? - offset / 24.0,
            label,
            local,
        )
    } else {
        (
            crate::parse::parse_date(&date_str)?,
            "UTC".to_string(),
            String::new(),
        )
    };

    // Dispatch to the appropriate chart-type builder
    let (ctx, render_fn) = dispatch_chart_type(chart_type, jd, &args, &user_vars)?;
    // The builders compose JSON; mutate it as a `Value` here, then re-wrap
    // into the typed `ChartContext` at the render / serialize boundary.
    let mut ctx = ctx.into_value();

    let overlay_cals: Vec<String> =
        args.calendars.iter().map(|c| c.as_str().to_string()).collect();
    apply_universal_overlays(&mut ctx, jd, &overlay_cals);

    // Expose the input timezone + local civil time to templates and the
    // built-in SVG. `date_local` falls back to the UT date when the input
    // was `now`/JD (already unambiguous in UT).
    if let Some(obj) = ctx.as_object_mut() {
        let ut_date = obj
            .get("date")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let local = if date_local.is_empty() {
            ut_date
        } else {
            date_local
        };
        obj.insert("timezone".into(), serde_json::Value::from(tz_label));
        obj.insert("date_local".into(), serde_json::Value::from(local));
    }

    // --print-context: dump JSON context and exit (println! added a
    // trailing newline — preserve it byte-for-byte).
    if args.print_context {
        let mut s = serde_json::to_string_pretty(&ctx).unwrap();
        s.push('\n');
        return Ok(RenderOutput::Stdout(s));
    }

    // --print-template: show the example MiniJinja template
    if args.print_template {
        return Ok(RenderOutput::Stdout(EXAMPLE_TEMPLATE.to_string()));
    }

    let ctx = ChartContext::from(ctx);
    let output = render_to_string(&ctx, render_fn, args.template.as_ref())?;
    Ok(RenderOutput::Out {
        body: output,
        path: args.out.clone(),
    })
}

// ─── Example template (MiniJinja / Jinja2 syntax) ───────────────────────────────────

pub(crate) const EXAMPLE_TEMPLATE: &str = include_str!("../../../templates/example.svg.tt");

/// Human-readable description of the chart-context fields available to
/// `--template` files. Printed by `--print-schema` for fast template-author
/// onboarding (no chart computation required).
pub(crate) const CONTEXT_SCHEMA: &str = include_str!("../../../templates/context_schema.txt");


#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::render::RenderArgs;

    fn natal_args() -> RenderArgs {
        let mut a = RenderArgs::default();
        a.chart_type = "natal".to_string();
        a.date = "1986-05-30 09:00".to_string();
        a.timezone = Some("UTC".to_string());
        a.lat = -23.5;
        a.lon = -46.6;
        a
    }

    #[test]
    fn compute_natal_returns_svg_payload() {
        match compute(natal_args()).expect("compute ok") {
            RenderOutput::Out { body, path } => {
                assert!(path.is_none());
                assert!(body.contains("<svg"), "no svg in body");
                assert!(body.contains("</svg>"), "svg not closed");
            }
            other => panic!("expected Out, got {other:?}"),
        }
    }

    #[test]
    fn compute_print_schema_is_stdout_verbatim() {
        let mut a = natal_args();
        a.print_schema = true;
        assert_eq!(
            compute(a).unwrap(),
            RenderOutput::Stdout(CONTEXT_SCHEMA.to_string())
        );
    }

    #[test]
    fn compute_print_context_json_trailing_newline() {
        let mut a = natal_args();
        a.print_context = true;
        match compute(a).unwrap() {
            RenderOutput::Stdout(s) => {
                assert!(s.ends_with('\n'), "println! newline must be preserved");
                assert!(s.trim_start().starts_with('{'), "not JSON");
            }
            o => panic!("expected Stdout, got {o:?}"),
        }
    }

    #[test]
    fn compute_rejects_out_of_range_lat() {
        let mut a = natal_args();
        a.lat = 999.0;
        assert!(compute(a).is_err());
    }

    #[test]
    fn compute_out_carries_out_path() {
        let mut a = natal_args();
        a.out = Some(std::path::PathBuf::from("/tmp/celestial_test_unused.svg"));
        match compute(a).unwrap() {
            RenderOutput::Out { path, .. } => {
                assert_eq!(path, Some(std::path::PathBuf::from("/tmp/celestial_test_unused.svg")));
            }
            o => panic!("expected Out, got {o:?}"),
        }
    }

    /// Drive `compute()` across one alias of every chart family —
    /// exercises `dispatch_chart_type` + each `dispatch_*` + the
    /// per-tradition builders + render, raising registry/pipeline
    /// coverage without spawning the binary.
    #[test]
    fn compute_covers_all_chart_families() {
        let types = [
            "natal", "cosmogram", "solar-return", "lunar-return", "biwheel",
            "composite", "dial", "graphic-ephemeris", "local-space", "rasi",
            "navamsa", "dasha", "north-indian", "ashtakavarga", "shadbala",
            "hellenistic", "firdaria", "profection", "bazi", "mesoamerican",
            "medicine-wheel", "wheel-of-year", "omer-grid", "calendar",
        ];
        for ct in types {
            let mut a = natal_args();
            a.chart_type = ct.to_string();
            a.date2 = Some("1990-01-01".into());
            a.date3 = Some("2000-06-15".into());
            assert!(
                matches!(compute(a), Ok(RenderOutput::Out { .. })),
                "compute failed for chart-type `{ct}`"
            );
        }
    }

    #[test]
    fn compute_progressed_solar_arc_triwheel_with_years() {
        for ct in ["progressed", "solar-arc", "triwheel"] {
            let mut a = natal_args();
            a.chart_type = ct.to_string();
            a.years = Some(35.5);
            a.date2 = Some("1990-01-01".into());
            a.date3 = Some("2000-06-15".into());
            assert!(compute(a).is_ok(), "compute failed for `{ct}`");
        }
    }

    #[test]
    fn compute_unknown_chart_type_errs() {
        let mut a = natal_args();
        a.chart_type = "definitely-not-a-chart".into();
        assert!(compute(a).is_err());
    }

    #[test]
    fn compute_with_config_file_and_var_override() {
        // config.rs: load_config (TOML [render]+[vars]), apply_var_overrides,
        // merge_date_and_time, SEC-5 out guard.
        let dir = std::env::temp_dir().join(format!("cel_cfgcov_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfg = dir.join("c.toml");
        std::fs::write(
            &cfg,
            b"[render]\ndate = \"1986-05-30 09:00\"\ntimezone = \"UTC\"\nlat = -23.5\nlon = -46.6\nout = \"rel.svg\"\n[vars]\ntitle = \"Cfg\"\nring_color = \"#101010\"\n",
        )
        .unwrap();
        let mut a = RenderArgs::default();
        a.chart_type = "natal".into();
        a.date = "now".into(); // clap-default sentinel so config `date` overrides
        a.config = Some(cfg);
        a.vars = vec!["ring_color=#abcdef".into()];
        match compute(a) {
            Ok(RenderOutput::Out { body, path }) => {
                assert_eq!(path, Some(std::path::PathBuf::from("rel.svg")));
                assert!(body.contains("#abcdef"), "--var override not applied");
            }
            o => panic!("expected Out, got {o:?}"),
        }
        let bad = dir.join("bad.toml");
        std::fs::write(
            &bad,
            b"[render]\ndate = \"1986-05-30 09:00\"\ntimezone = \"UTC\"\nlat = 0\nlon = 0\nout = \"/tmp/evil.svg\"\n",
        )
        .unwrap();
        let mut b = RenderArgs::default();
        b.chart_type = "natal".into();
        b.date = "now".into();
        b.config = Some(bad);
        assert!(compute(b).is_err(), "absolute config out must be rejected");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn compute_calendar_overlays() {
        use super::args::CalendarKind;
        let mut a = natal_args();
        a.calendars = vec![
            CalendarKind::Omer,
            CalendarKind::Sabbats,
            CalendarKind::Moon,
            CalendarKind::Hebrew,
            CalendarKind::GregorianYear,
        ];
        assert!(compute(a).is_ok());
    }

    #[test]
    fn compute_config_error_and_override_paths() {
        let dir = std::env::temp_dir().join(format!("cel_cfgerr_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);

        // invalid TOML → load_config error branch
        let bad = dir.join("broken.toml");
        std::fs::write(&bad, b"[render\ndate =").unwrap();
        let mut a = RenderArgs::default();
        a.date = "now".into();
        a.config = Some(bad);
        assert!(compute(a).is_err(), "invalid TOML must error");

        // config sets template + hsys (override_if template/hsys branches);
        // missing template file → render error branch.
        let cfg = dir.join("t.toml");
        std::fs::write(
            &cfg,
            b"[render]\ndate = \"1986-05-30 09:00\"\ntimezone = \"UTC\"\nlat = 1.0\nlon = 2.0\nhsys = \"K\"\ntemplate = \"no_such.tt\"\n",
        )
        .unwrap();
        let mut b = RenderArgs::default();
        b.date = "now".into();
        b.config = Some(cfg);
        assert!(compute(b).is_err(), "missing template path must error");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn compute_config_vars_non_string_toml_types() {
        // toml_value_to_string arms: Integer / Float / Boolean / other.
        let dir = std::env::temp_dir().join(format!("cel_cfgty_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfg = dir.join("ty.toml");
        std::fs::write(
            &cfg,
            b"[render]\ndate = \"1986-05-30 09:00\"\ntimezone = \"UTC\"\nlat = 0\nlon = 0\n\
              [vars]\ntitle = \"T\"\nint_v = 7\nfloat_v = 1.5\nbool_v = true\narr_v = [1, 2]\n",
        )
        .unwrap();
        let mut a = RenderArgs::default();
        a.date = "now".into();
        a.config = Some(cfg);
        assert!(matches!(compute(a), Ok(RenderOutput::Out { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn compute_time_flag_merge() {
        // merge_date_and_time: separate --date + --time.
        let mut a = natal_args();
        a.date = "1986-05-30".into();
        a.time = Some("09:00".into());
        assert!(compute(a).is_ok());
    }

    #[test]
    fn compute_malformed_var_is_rejected() {
        let mut a = natal_args();
        a.vars = vec!["no_equals_sign".into()];
        // apply_var_overrides should reject a `--var` without `=`.
        assert!(compute(a).is_err());
    }

    #[test]
    fn compute_now_and_jd_date_paths() {
        // merge_date_and_time / require_datetime branches: bare JD + "now".
        let mut a = natal_args();
        a.date = "2451545.0".into();
        a.timezone = None;
        assert!(compute(a).is_ok());
        let mut b = natal_args();
        b.chart_type = "calendar".into();
        b.date = "now".into();
        b.timezone = None;
        assert!(compute(b).is_ok());
    }
}
