//! `celestial omer` — Sefirat HaOmer (Counting of the Omer).

use crate::error::CliError;
use crate::{format as fmt, parse};
use celestial_core::{omer_days, omer_declaration, omer_from_jd, omer_period};
use clap::Args;

#[derive(Args)]
pub struct OmerArgs {
    /// Date to query (default: tonight). Formats: now, YYYY-MM-DD, JD float.
    #[arg(short, long, default_value = "now")]
    pub date: String,

    /// Show all 49 days for the Hebrew year containing the given date
    #[arg(long)]
    pub all: bool,

    /// Specific Hebrew year (e.g. 5785)
    #[arg(short = 'y', long)]
    pub year: Option<i32>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Print the full 49-day Omer table (`--all` mode).
fn run_all_mode(hebrew_year: i32, jd: f64, json: bool) {
    let days = omer_days(hebrew_year);
    let period = omer_period(jd);

    if json {
        let items: Vec<String> = days
            .iter()
            .map(|d| {
                fmt::json_obj(&[
                    ("day", d.day.to_string()),
                    ("week", d.week.to_string()),
                    ("day_of_week", d.day_of_week.to_string()),
                    ("week_sefirah", d.week_sefirah.to_string()),
                    ("day_sefirah", d.day_sefirah.to_string()),
                    ("hebrew_text", d.hebrew_text.to_string()),
                    ("is_lag_baomer", d.is_lag_baomer.to_string()),
                    ("jd", format!("{:.4}", d.jd)),
                    ("date", parse::jd_to_str(d.jd)),
                ])
            })
            .collect();
        println!("{}", fmt::json_array(items));
        return;
    }

    println!();
    println!("  Sefirat HaOmer {hebrew_year}");
    println!("  {}", fmt::rule(68));
    println!(
        "  {:<4}  {:<10}  {:<12}  {:<12}  Date",
        "Day", "Week", "Week", "Day"
    );
    println!(
        "  {:<4}  {:<10}  {:<12}  {:<12}",
        "", "Sefirah", "Sefirah", ""
    );
    println!("  {}", fmt::rule(68));
    for d in &days {
        let lag = if d.is_lag_baomer {
            " ← Lag Ba'Omer"
        } else {
            ""
        };
        println!(
            "  {:>3}.  {:>2}/7   {:<12}  {:<12}  {}{}",
            d.day,
            d.day_of_week,
            d.week_sefirah,
            d.day_sefirah,
            parse::jd_to_str(d.jd),
            lag,
        );
    }
    println!();
    println!(
        "  Period: {} — {}",
        parse::jd_to_str(period.start_jd),
        parse::jd_to_str(period.end_jd)
    );
    println!();
}

/// Print today's Omer entry when the date IS within the Omer period.
fn run_in_omer_mode(day: celestial_core::OmerDay, json: bool) {
    let declaration = omer_declaration(day.day);
    if json {
        println!(
            "{}",
            fmt::json_obj(&[
                ("day", day.day.to_string()),
                ("week", day.week.to_string()),
                ("day_of_week", day.day_of_week.to_string()),
                ("week_sefirah", day.week_sefirah.to_string()),
                ("day_sefirah", day.day_sefirah.to_string()),
                ("hebrew_text", day.hebrew_text.to_string()),
                ("declaration", declaration),
                ("is_lag_baomer", day.is_lag_baomer.to_string()),
                ("jd", format!("{:.4}", day.jd)),
                ("date", parse::jd_to_str(day.jd)),
            ])
        );
        return;
    }
    println!();
    println!("  Sefirat HaOmer — Day {}", day.day);
    println!("  {}", fmt::rule(52));
    println!("  Day:       {} of 49", day.day);
    println!(
        "  Week:      {} of 7  ({} sheb'{})",
        day.week, day.week_sefirah, day.day_sefirah
    );
    println!("  Sefirot:   {} sheb'{}", day.week_sefirah, day.day_sefirah);
    if day.is_lag_baomer {
        println!("  ★ Lag Ba'Omer (33rd day)");
    }
    println!();
    println!("  {}", day.hebrew_text);
    println!();
}

/// Print the "next Omer begins ..." message when the date is NOT in the period.
fn run_not_in_omer_mode(jd: f64, json: bool) {
    if json {
        println!("{{\"in_omer\": false}}");
        return;
    }
    let period = omer_period(jd);
    println!();
    println!("  Not currently in the Omer period.");
    println!(
        "  Next Omer begins: {} (Hebrew year {})",
        parse::jd_to_str(period.start_jd),
        period.hebrew_year
    );
    println!();
}

pub fn run(args: OmerArgs) -> Result<(), CliError> {
    let jd = parse::parse_date(&args.date)?;

    // Determine Hebrew year (from --year flag or from date)
    let hebrew_year = args.year.unwrap_or_else(|| {
        let period = omer_period(jd);
        period.hebrew_year
    });

    if args.all {
        run_all_mode(hebrew_year, jd, args.json);
        return Ok(());
    }
    match omer_from_jd(jd) {
        Some(day) => run_in_omer_mode(day, args.json),
        None => run_not_in_omer_mode(jd, args.json),
    }
    Ok(())
}
