//! `celestial calc` — geocentric planetary positions.

use celestial_core::JulianDay;
use crate::error::CliError;
use crate::{format as fmt, parse};
use celestial_core::body::{Body, CalcFlags, SiderealMode};
use celestial_core::{calc_ut, set_sid_mode};
use clap::Args;

#[derive(Args)]
pub struct CalcArgs {
    /// Date (`YYYY-MM-DD [HH:MM[:SS]]`) or Julian day, default: now
    #[arg(short, long, default_value = "now")]
    pub date: String,

    /// Bodies: sun,moon,mercury,venus,mars,jupiter,saturn,uranus,neptune,chiron,node
    /// (comma-separated, default: all main planets)
    #[arg(short, long)]
    pub body: Option<String>,

    /// Use sidereal positions
    #[arg(long)]
    pub sidereal: bool,

    /// Sidereal mode when --sidereal is set (lahiri, fagan, raman, …)
    #[arg(long, default_value = "lahiri")]
    pub mode: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: CalcArgs) -> Result<(), CliError> {
    let jd = parse::parse_date(&args.date)?;

    let bodies: Vec<i32> = match &args.body {
        Some(s) => s
            .split(',')
            .map(|b| parse::parse_body(b.trim()))
            .collect::<Result<Vec<_>, _>>()?,
        None => parse::default_bodies(),
    };

    let mut flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    if args.sidereal {
        let mode = parse::parse_sid_mode(&args.mode)?;
        set_sid_mode(SiderealMode(mode), 0.0, 0.0);
        flags |= CalcFlags::SIDEREAL;
    }

    // Collect rows
    struct Row {
        name: String,
        lon: f64,
        lat: f64,
        dist: f64,
        speed: f64,
    }
    let mut rows = Vec::new();
    for body in &bodies {
        let pos = calc_ut(JulianDay::new(jd), Body::from_raw(*body), flags)
            .map_err(|e| format!("{}: {e}", parse::body_name(Body::from_raw(*body))))?;
        rows.push(Row {
            name: parse::body_name(Body::from_raw(*body)).to_string(),
            lon: pos.lon,
            lat: pos.lat,
            dist: pos.dist,
            speed: pos.speed_lon,
        });
    }

    if args.json {
        let items: Vec<String> = rows
            .iter()
            .map(|r| {
                fmt::json_obj(&[
                    ("body", r.name.clone()),
                    ("longitude", format!("{:.6}", r.lon)),
                    ("latitude", format!("{:.6}", r.lat)),
                    ("distance", format!("{:.6}", r.dist)),
                    ("speed", format!("{:.6}", r.speed)),
                    ("jd", format!("{jd:.4}")),
                ])
            })
            .collect();
        println!("{}", fmt::json_array(items));
        return Ok(());
    }

    // Header
    let mode_label = if args.sidereal {
        format!("  ·  sidereal ({})", args.mode)
    } else {
        String::new()
    };
    println!();
    println!("  {}  ·  JD {:.4}{}", parse::jd_to_str(jd), jd, mode_label);
    println!("  {}", fmt::rule(68));
    println!(
        "  {:<11} {:<16} {:<12} {:<14} Speed",
        "Body", "Longitude", "Latitude", "Distance"
    );
    println!("  {}", fmt::rule(68));

    for r in &rows {
        println!(
            "  {:<11} {:<16} {:<12} {:<14} {}",
            r.name,
            fmt::lon_zodiac(r.lon),
            fmt::deg_dms(r.lat),
            fmt::dist_au(r.dist),
            fmt::speed_dday(r.speed),
        );
    }
    println!();
    Ok(())
}
