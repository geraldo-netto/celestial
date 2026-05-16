//! `RenderArgs` — the `celestial render` clap argument struct.
//!
//! Split out of the former 3.5k-line `mod.rs` god file (pure code
//! movement; re-exported by the facade as before).

use clap::Args;
use std::path::PathBuf;

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
    #[arg(long = "calendar", value_name = "NAME", action = clap::ArgAction::Append)]
    pub calendars: Vec<String>,

    /// Month (1–12). Used by `--chart-type calendar`. Defaults to the month
    /// from `--date` (or the current month if `--date` is "now").
    #[arg(long)]
    pub month: Option<u32>,

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
