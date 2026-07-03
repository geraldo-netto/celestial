//! `celestial moon` — Moon phase, illumination, and principal phase timing.

use crate::error::CliError;
use crate::{format as fmt, parse};
use celestial_core::JulianDay;
use celestial_core::{
    moon_phase_info, moon_phases_for_month, next_first_quarter, next_full_moon_phase,
    next_last_quarter, next_new_moon,
};
use clap::Args;

#[derive(Args)]
pub struct MoonArgs {
    /// Date for current phase info (default: now). Formats: now, YYYY-MM-DD, JD float.
    #[arg(short, long, default_value = "now")]
    pub date: String,

    /// Show next new moon from the given date
    #[arg(long)]
    pub new: bool,

    /// Show next first-quarter moon from the given date
    #[arg(long, name = "first-quarter")]
    pub first_quarter: bool,

    /// Show next full moon from the given date
    #[arg(long)]
    pub full: bool,

    /// Show next last-quarter moon from the given date
    #[arg(long, name = "last-quarter")]
    pub last_quarter: bool,

    /// Show all 4 phases in a calendar month (YYYY-MM)
    #[arg(long, value_name = "YYYY-MM")]
    pub month: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Dispatch `moon` to the right mode based on CLI flags.
pub fn run(args: MoonArgs) -> Result<(), CliError> {
    if args.month.is_some() && has_phase_request(&args) {
        return Err(CliError::Parse(
            "--month cannot be combined with --new, --first-quarter, --full, or --last-quarter"
                .to_owned(),
        ));
    }

    let jd = parse::parse_date(&args.date)?;

    if let Some(ref ym) = args.month {
        return run_month_mode(ym, args.json);
    }
    if has_phase_request(&args) {
        return run_phase_mode(&args, jd);
    }
    run_info_mode(&args, jd)
}

fn has_phase_request(args: &MoonArgs) -> bool {
    args.new || args.first_quarter || args.full || args.last_quarter
}

/// Print all Moon phases for a calendar month (`--month YYYY-MM`).
fn run_month_mode(ym: &str, json: bool) -> Result<(), CliError> {
    let (year, month) = parse_year_month(ym)?;
    let events = moon_phases_for_month(year, month)?;

    if json {
        let items: Vec<String> = events
            .iter()
            .map(|e| {
                fmt::json_obj(&[
                    ("phase", e.phase.name().to_string()),
                    ("jd", format!("{:.4}", e.jd)),
                    ("date", parse::jd_to_str(e.jd)),
                    ("elongation", format!("{:.2}", e.elongation)),
                ])
            })
            .collect();
        println!("{}", fmt::json_array(items));
    } else {
        println!();
        println!("  Moon phases — {year}-{month:02}");
        println!("  {}", fmt::rule(46));
        for e in &events {
            println!("  {:<16}  {}", e.phase.name(), parse::jd_to_str(e.jd));
        }
        println!();
    }
    Ok(())
}

/// Print the next occurrence of each phase the user asked for.
/// Any combination of `--new`, `--first-quarter`, `--full`, `--last-quarter`.
fn run_phase_mode(args: &MoonArgs, jd: f64) -> Result<(), CliError> {
    let mut results: Vec<(&str, f64)> = Vec::new();
    if args.new {
        results.push(("New Moon", next_new_moon(JulianDay::new(jd))?));
    }
    if args.first_quarter {
        results.push(("First Quarter", next_first_quarter(JulianDay::new(jd))?));
    }
    if args.full {
        results.push(("Full Moon", next_full_moon_phase(JulianDay::new(jd))?));
    }
    if args.last_quarter {
        results.push(("Last Quarter", next_last_quarter(JulianDay::new(jd))?));
    }

    if args.json {
        let items: Vec<String> = results
            .iter()
            .map(|(name, jd_e)| {
                fmt::json_obj(&[
                    ("phase", name.to_string()),
                    ("jd", format!("{jd_e:.4}")),
                    ("date", parse::jd_to_str(*jd_e)),
                ])
            })
            .collect();
        if items.len() == 1 {
            println!("{}", items[0]);
        } else {
            println!("{}", fmt::json_array(items));
        }
    } else {
        println!();
        for (name, jd_e) in &results {
            println!("  {:<16}  {}", name, parse::jd_to_str(*jd_e));
        }
        println!();
    }
    Ok(())
}

/// Print the current Moon phase + surrounding context (default mode).
fn run_info_mode(args: &MoonArgs, jd: f64) -> Result<(), CliError> {
    let info = moon_phase_info(JulianDay::new(jd))?;

    if args.json {
        println!(
            "{}",
            fmt::json_obj(&[
                ("phase", info.phase_name.to_string()),
                ("elongation", format!("{:.4}", info.elongation)),
                ("illumination", format!("{:.4}", info.illumination)),
                (
                    "illumination_pct",
                    format!("{:.1}", info.illumination * 100.0)
                ),
                ("age_days", format!("{:.2}", info.age_days)),
                ("prev_phase", info.prev_phase_name.to_string()),
                ("prev_phase_jd", format!("{:.4}", info.prev_phase_jd)),
                ("prev_phase_date", parse::jd_to_str(info.prev_phase_jd)),
                ("next_phase", info.next_phase_name.to_string()),
                ("next_phase_jd", format!("{:.4}", info.next_phase_jd)),
                ("next_phase_date", parse::jd_to_str(info.next_phase_jd)),
            ])
        );
        return Ok(());
    }

    let date_str = parse::jd_to_str(jd);
    println!();
    println!("  Moon phase — {date_str}");
    println!("  {}", fmt::rule(46));
    println!("  Phase:        {}", info.phase_name);
    println!("  Elongation:   {:.1}°", info.elongation);
    println!("  Illuminated:  {:.0}%", info.illumination * 100.0);
    println!("  Age:          {:.1} days", info.age_days);
    println!();
    println!(
        "  Previous:     {}  ({})",
        info.prev_phase_name,
        parse::jd_to_str(info.prev_phase_jd)
    );
    let days_to_next = info.next_phase_jd - jd;
    println!(
        "  Next:         {}  ({})  in {:.1} days",
        info.next_phase_name,
        parse::jd_to_str(info.next_phase_jd),
        days_to_next
    );
    println!();
    Ok(())
}

fn parse_year_month(s: &str) -> Result<(i32, u8), CliError> {
    let parts: Vec<&str> = s.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(CliError::Parse(format!("expected YYYY-MM, got: {s}")));
    }
    let year: i32 = parts[0]
        .parse()
        .map_err(|_| format!("bad year: {}", parts[0]))?;
    let month: u8 = parts[1]
        .parse()
        .map_err(|_| format!("bad month: {}", parts[1]))?;
    if !(1..=12).contains(&month) {
        return Err(CliError::Parse(format!("month must be 1–12, got: {month}")));
    }
    Ok((year, month))
}
