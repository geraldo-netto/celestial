//! `celestial crossing` — next ecliptic longitude crossing.

use crate::error::CliError;
use crate::{format as fmt, parse};
use celestial_core::body::{Body, CalcFlags};
use celestial_core::{helio_cross_ut, mooncross_ut, solcross_ut};
use clap::Args;

#[derive(Args)]
pub struct CrossingArgs {
    /// Body: sun, moon, mercury, venus, mars, … (default: sun)
    #[arg(short, long, default_value = "sun")]
    pub body: String,

    /// Target ecliptic longitude in degrees
    #[arg(short, long)]
    pub lon: f64,

    /// Start date (default: now)
    #[arg(short, long, default_value = "now")]
    pub from: String,

    /// Use heliocentric positions
    #[arg(long)]
    pub helio: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: CrossingArgs) -> Result<(), CliError> {
    let body = parse::parse_body(&args.body)?;
    let jd = parse::parse_date(&args.from)?;
    let lon = args.lon.rem_euclid(360.0);

    let result_jd = if args.helio {
        helio_cross_ut(Body::from_raw(body), lon, jd, CalcFlags::BUILTIN, 1)
            ?
    } else {
        match body {
            0 => solcross_ut(lon, jd, CalcFlags::BUILTIN)?,
            1 => mooncross_ut(lon, jd, CalcFlags::BUILTIN)?,
            _ => helio_cross_ut(Body::from_raw(body), lon, jd, CalcFlags::BUILTIN, 1)
                ?,
        }
    };

    let body_label = parse::body_name(Body::from_raw(body));
    let mode_label = if args.helio { " (heliocentric)" } else { "" };

    if args.json {
        println!(
            "{}",
            fmt::json_obj(&[
                ("body", body_label.to_string()),
                ("target_lon", format!("{lon:.4}")),
                ("jd", format!("{result_jd:.6}")),
                ("date", parse::jd_to_str(result_jd)),
            ])
        );
        return Ok(());
    }

    println!();
    println!(
        "  {}{} crossing {}",
        body_label,
        mode_label,
        fmt::lon_zodiac(lon)
    );
    println!("  {}", fmt::rule(42));
    println!("  {}", parse::jd_to_str(result_jd));
    println!("  JD {result_jd:.4}");
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> CrossingArgs {
        CrossingArgs {
            body: "sun".into(),
            lon: 0.0,
            from: "2451545.0".into(),
            helio: false,
            json: false,
        }
    }

    #[test]
    fn run_text_ok() {
        assert!(run(base()).is_ok());
    }

    #[test]
    fn run_json_ok() {
        let mut a = base();
        a.json = true;
        assert!(run(a).is_ok());
    }

    #[test]
    fn run_helio_ok() {
        let mut a = base();
        a.helio = true;
        a.body = "mars".into();
        a.lon = 120.0;
        assert!(run(a).is_ok());
    }

    #[test]
    fn run_bad_body_errs() {
        let mut a = base();
        a.body = "notabody".into();
        assert!(run(a).is_err());
    }
}
