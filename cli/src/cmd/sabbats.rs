//! `celestial sabbats` and `celestial esbats` — Celtic calendar.

use crate::error::CliError;
use crate::{format as fmt, parse};
use celestial_core::body::Calendar;
use celestial_core::{esbats_for_year, jdnow, next_esbat, next_sabbat, revjul, sabbats_for_year};
use clap::Args;

// ─── Sabbats ──────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct SabbatsArgs {
    /// Gregorian year (default: current year)
    #[arg(short, long)]
    pub year: Option<i32>,

    /// Show only the next sabbat from now
    #[arg(long)]
    pub next: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run_sabbats(args: SabbatsArgs) -> Result<(), CliError> {
    if args.next {
        let s = next_sabbat(jdnow())?;
        if args.json {
            println!(
                "{}",
                fmt::json_obj(&[
                    ("name", s.name.to_string()),
                    ("jd", format!("{:.4}", s.jd)),
                    ("date", parse::jd_to_str(s.jd)),
                    (
                        "solar_longitude",
                        format!("{:.0}", s.kind.solar_longitude())
                    ),
                ])
            );
        } else {
            println!(
                "\n  Next sabbat: {}  —  {}\n",
                s.name,
                parse::jd_to_str(s.jd)
            );
        }
        return Ok(());
    }

    let year = args
        .year
        .unwrap_or_else(|| revjul(jdnow(), Calendar::Gregorian).year);
    let sabbats = sabbats_for_year(year)?;

    if args.json {
        let items: Vec<String> = sabbats
            .iter()
            .map(|s| {
                fmt::json_obj(&[
                    ("name", s.name.to_string()),
                    (
                        "type",
                        if s.kind.is_quarter_day() {
                            "quarter".to_string()
                        } else {
                            "cross-quarter".to_string()
                        },
                    ),
                    (
                        "solar_longitude",
                        format!("{:.0}", s.kind.solar_longitude()),
                    ),
                    ("jd", format!("{:.4}", s.jd)),
                    ("date", parse::jd_to_str(s.jd)),
                ])
            })
            .collect();
        println!("{}", fmt::json_array(items));
        return Ok(());
    }

    println!();
    println!("  Wheel of the Year {year}");
    println!("  {}", fmt::rule(52));
    println!("  {:<14} {:<24} Sun", "Sabbat", "Date");
    println!("  {}", fmt::rule(52));
    for s in &sabbats {
        let kind_marker = if s.kind.is_quarter_day() {
            "☀"
        } else {
            "☽"
        };
        println!(
            "  {:<14} {:<24} {} {:.0}°",
            s.name,
            parse::jd_to_str(s.jd),
            kind_marker,
            s.kind.solar_longitude(),
        );
    }
    println!();
    Ok(())
}

// ─── Esbats ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct EsbatsArgs {
    /// Gregorian year (default: current year)
    #[arg(short, long)]
    pub year: Option<i32>,

    /// Show only the next full moon from now
    #[arg(long)]
    pub next: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run_esbats(args: EsbatsArgs) -> Result<(), CliError> {
    if args.next {
        let e = next_esbat(jdnow())?;
        if args.json {
            println!(
                "{}",
                fmt::json_obj(&[
                    ("name", e.display_name.to_string()),
                    ("jd", format!("{:.4}", e.jd)),
                    ("date", parse::jd_to_str(e.jd)),
                ])
            );
        } else {
            println!(
                "\n  Next esbat: {}  —  {}\n",
                e.display_name,
                parse::jd_to_str(e.jd)
            );
        }
        return Ok(());
    }

    let year = args
        .year
        .unwrap_or_else(|| revjul(jdnow(), Calendar::Gregorian).year);
    let esbats = esbats_for_year(year)?;

    if args.json {
        let items: Vec<String> = esbats
            .iter()
            .map(|e| {
                fmt::json_obj(&[
                    ("name", e.display_name.to_string()),
                    ("jd", format!("{:.4}", e.jd)),
                    ("date", parse::jd_to_str(e.jd)),
                ])
            })
            .collect();
        println!("{}", fmt::json_array(items));
        return Ok(());
    }

    println!();
    println!("  Full Moons {year}  ({} moons)", esbats.len());
    println!("  {}", fmt::rule(46));
    for e in &esbats {
        println!("  {:<18}  {}", e.display_name, parse::jd_to_str(e.jd));
    }
    println!();
    Ok(())
}
