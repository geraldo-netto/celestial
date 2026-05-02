//! `celestial phenomena` — apparent planetary phenomena
//! (magnitude, phase, elongation, illumination, apparent diameter).
//!
//! Wraps `celestial_core::pheno_ut` in a user-friendly CLI form. The
//! programmatic equivalent (Python/JS/PHP bindings) returns a 20-element
//! array; this command formats it as a labelled table (or JSON).

use crate::{format as fmt, parse};
use celestial_core::{pheno_ut, Body, CalcFlags};
use clap::Args;

#[derive(Args)]
pub struct PhenomenaArgs {
    /// Body to query: sun moon mercury venus mars jupiter saturn uranus neptune pluto
    #[arg(short, long)]
    pub body: String,

    /// Date (YYYY-MM-DD [HH:MM]), `now`, or a Julian Day number. Default: now.
    #[arg(short, long, default_value = "now")]
    pub date: String,

    /// Optional time component (HH:MM[:SS]) — merged into `--date` when the
    /// date is a bare calendar date.
    #[arg(short, long)]
    pub time: Option<String>,

    /// Output as JSON instead of a table
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: PhenomenaArgs) -> Result<(), String> {
    let date_str = match (&args.time, args.date.contains(' ')) {
        (Some(t), false) if args.date != "now" && args.date.parse::<f64>().is_err() => {
            format!("{} {}", args.date, t)
        }
        _ => args.date.clone(),
    };
    let jd = parse::parse_date(&date_str)?;

    let body_id = parse::parse_body(&args.body)?;
    let body = Body(body_id);

    let attr = pheno_ut(jd, body, CalcFlags::BUILTIN)
        .map_err(|e| format!("phenomena calculation failed: {e}"))?;

    // attr[0] = phase angle (deg), [1] = illuminated fraction (0..1),
    // [2] = elongation (deg), [3] = apparent diameter (arcseconds), [4] = magnitude
    let phase_angle = attr[0];
    let illuminated = attr[1];
    let elongation = attr[2];
    let ang_diam_arcsec = attr[3];
    let magnitude = attr[4];

    if args.json {
        println!(
            "{}",
            fmt::json_obj(&[
                ("body", parse::body_name(body).to_string()),
                ("jd", format!("{jd:.4}")),
                ("date", parse::jd_to_str(jd)),
                ("phase_angle", format!("{phase_angle:.4}")),
                ("illuminated", format!("{illuminated:.4}")),
                ("illuminated_pct", format!("{:.1}", illuminated * 100.0)),
                ("elongation", format!("{elongation:.4}")),
                ("apparent_diameter_arcsec", format!("{ang_diam_arcsec:.2}")),
                ("magnitude", format!("{magnitude:.3}")),
            ])
        );
        return Ok(());
    }

    let date_str = parse::jd_to_str(jd);
    println!();
    println!("  {} — {}", parse::body_name(body), date_str);
    println!("  {}", fmt::rule(46));
    println!("  Magnitude          {magnitude:.2}");
    println!("  Phase angle        {phase_angle:.2}°");
    println!("  Illuminated        {:.1}%", illuminated * 100.0);
    println!("  Apparent diameter  {ang_diam_arcsec:.2}\"");
    println!("  Elongation         {elongation:.2}°");
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phenomena_for_venus_at_j2000_runs() {
        // J2000.0 — known reference point. We don't check exact magnitude
        // (depends on phase) but ensure the call succeeds and gives finite values.
        let attr = pheno_ut(2_451_545.0, Body::VENUS, CalcFlags::BUILTIN).unwrap();
        assert!(attr[0].is_finite(), "phase angle should be finite");
        assert!(
            (0.0..=1.0).contains(&attr[1]),
            "illuminated fraction must be in [0,1], got {}",
            attr[1]
        );
        assert!(attr[3] > 0.0, "apparent diameter must be positive");
    }

    #[test]
    fn phenomena_args_parse() {
        let args = PhenomenaArgs {
            body: "Venus".to_string(),
            date: "2024-04-08".to_string(),
            time: None,
            json: false,
        };
        // Just ensure the body name parses; full run() requires stdout
        let body_id = parse::parse_body(&args.body).unwrap();
        assert_eq!(body_id, 3, "Venus body id should be 3");
    }
}
