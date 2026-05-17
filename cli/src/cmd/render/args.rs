//! `RenderArgs` — the `celestial render` clap argument struct.
//!
//! Split out of the former 3.5k-line `mod.rs` god file (pure code
//! movement; re-exported by the facade as before).

use clap::{Args, ValueEnum};
use std::path::PathBuf;

/// Calendar overlays selectable via `--calendar`. A fixed, clap-validated
/// set (replaces the old free-form `Vec<String>`): an unknown value now
/// fails at parse time with the valid list, instead of being silently
/// ignored deep in the overlay code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[non_exhaustive] // NV-2: forward-stable across new calendar overlays
pub enum CalendarKind {
    #[value(name = "gregorian")]
    Gregorian,
    #[value(name = "gregorian-year", alias = "year-calendar")]
    GregorianYear,
    #[value(name = "omer")]
    Omer,
    #[value(name = "sabbats", alias = "wheel")]
    Sabbats,
    #[value(name = "moon", alias = "lunar")]
    Moon,
    #[value(name = "hebrew", alias = "jewish")]
    Hebrew,
}

impl CalendarKind {
    /// Primary input token (the canonical spelling `resolve_overlay`
    /// matches on; clap maps every accepted alias to one of these).
    pub fn as_str(self) -> &'static str {
        match self {
            CalendarKind::Gregorian => "gregorian",
            CalendarKind::GregorianYear => "gregorian-year",
            CalendarKind::Omer => "omer",
            CalendarKind::Sabbats => "sabbats",
            CalendarKind::Moon => "moon",
            CalendarKind::Hebrew => "hebrew",
        }
    }
}

/// clap value-parser for `--chart-type`: validates against the live
/// `CHART_REGISTRY` aliases at parse time (the registry stays the single
/// source of truth — no duplicated enum), so an unknown type fails
/// immediately with the valid list instead of after argument handling.
fn parse_chart_type(s: &str) -> Result<String, String> {
    let key = s.trim().to_lowercase();
    if super::registered_chart_types()
        .split(' ')
        .any(|a| a == key)
    {
        Ok(key)
    } else {
        Err(format!(
            "unknown chart type `{s}`; valid: {}",
            super::registered_chart_types()
        ))
    }
}

/// clap value-parser for `--hsys`: accepts a house-system name or letter
/// (reusing the canonical CLI parser) and validates at parse time.
fn parse_hsys_char(s: &str) -> Result<char, String> {
    crate::parse::parse_hsys(s)
        .map(|b| b as char)
        .map_err(|e| e.to_string())
}

#[derive(Args, Debug, Default)]
pub struct RenderArgs {
    /// Date and optional time to compute (YYYY-MM-DD [HH:MM[:SS]] or "now")
    #[arg(long, default_value = "now")]
    pub date: String,

    /// Time of day (HH:MM or HH:MM:SS) in the --timezone — merged with --date
    /// if --date has no time
    #[arg(long)]
    pub time: Option<String>,

    /// Timezone of the --date/--time (the birth-place local time). Accepts a
    /// numeric offset (`-03:00`, `+05:30`, `+0530`), `UTC`, or an unambiguous
    /// abbreviation (`BRT`, `EST`, …). REQUIRED for natal/derived charts so
    /// the local birth time is converted to UT correctly; omit only for
    /// `--date now`, a raw Julian day, or `--chart-type calendar`.
    #[arg(long = "timezone", visible_alias = "tz")]
    pub timezone: Option<String>,

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

    /// Calendar/tradition overlay to merge into the context (repeatable).
    /// Available: gregorian, omer, sabbats, moon, hebrew. Each adds a
    /// top-level field of the same name to the context, accessible from
    /// `--template` files and `--print-context` output. Works with any
    /// `--chart-type` (the `calendar` chart-type also uses these as its
    /// primary content). Example: `--chart-type natal --calendar omer
    /// --calendar moon` produces a natal context that ALSO carries
    /// `omer.today` and `moon.phases[]`, so a custom template can tag a
    /// natal wheel with current Omer day and full-moon dates.
    #[arg(long = "calendar", value_name = "NAME", value_enum, action = clap::ArgAction::Append)]
    pub calendars: Vec<CalendarKind>,

    /// Month (1–12). Used by `--chart-type calendar`. Defaults to the month
    /// from `--date` (or the current month if `--date` is "now").
    #[arg(long)]
    pub month: Option<u32>,

    /// Chart type: natal | cosmogram | solar-return | lunar-return | progressed | solar-arc | biwheel
    #[arg(long, default_value = "natal", value_parser = parse_chart_type)]
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
    /// (a full name like `placidus` is also accepted)
    #[arg(long, default_value = "P", value_parser = parse_hsys_char)]
    pub hsys: char,

    /// Print the built-in example template to stdout. Pipe to a `.svg`
    /// file to start customizing your own theme.
    #[arg(long)]
    pub print_template: bool,

    /// Print the chart context JSON for the requested chart to stdout.
    /// Useful while authoring a `--template`: shows every field's actual value.
    #[arg(long)]
    pub print_context: bool,

    /// Print the schema of the chart context (field names + types) without
    /// computing values. Lighter-weight than `--print-context` for template
    /// authors who only need to know what's available.
    #[arg(long)]
    pub print_schema: bool,
}

impl RenderArgs {
    /// Range-check geographic inputs before any chart computation so a
    /// bad `--lat`/`--lon` (or the bi/tri-wheel `--lat2`/`--lon2`)
    /// fails early with a clear message instead of reaching core.
    pub fn validate(&self) -> Result<(), crate::error::CliError> {
        fn chk(name: &str, v: f64, lo: f64, hi: f64) -> Result<(), crate::error::CliError> {
            if v.is_finite() && (lo..=hi).contains(&v) {
                Ok(())
            } else {
                Err(crate::error::CliError::Parse(format!(
                    "{name} out of range: {v} (expected {lo}..={hi})"
                )))
            }
        }
        chk("--lat", self.lat, -90.0, 90.0)?;
        chk("--lon", self.lon, -180.0, 180.0)?;
        if let Some(v) = self.lat2 {
            chk("--lat2", v, -90.0, 90.0)?;
        }
        if let Some(v) = self.lon2 {
            chk("--lon2", v, -180.0, 180.0)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> RenderArgs {
        RenderArgs::default()
    }

    #[test]
    fn validate_accepts_in_range() {
        let mut a = args();
        a.lat = -23.5;
        a.lon = -46.6;
        a.lat2 = Some(51.5);
        a.lon2 = Some(-0.12);
        assert!(a.validate().is_ok());
    }

    #[test]
    fn validate_accepts_zero_default() {
        // calendar chart-type leaves lat/lon at the 0.0 default
        assert!(args().validate().is_ok());
    }

    #[test]
    fn validate_rejects_bad_lat() {
        let mut a = args();
        a.lat = 91.0;
        let e = a.validate().unwrap_err().to_string();
        assert!(e.contains("--lat"), "got: {e}");
    }

    #[test]
    fn validate_rejects_bad_lon() {
        let mut a = args();
        a.lon = -200.0;
        assert!(a.validate().is_err());
    }

    #[test]
    fn validate_rejects_bad_lon2() {
        let mut a = args();
        a.lon2 = Some(360.0);
        assert!(a.validate().is_err());
    }

    #[test]
    fn validate_rejects_nan() {
        let mut a = args();
        a.lat = f64::NAN;
        assert!(a.validate().is_err());
    }
}
