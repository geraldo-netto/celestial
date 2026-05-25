//! `celestial jd` — Julian day ↔ calendar date conversion.

use crate::error::CliError;
use crate::{format as fmt, parse};
use celestial_core::body::Calendar;
use celestial_core::{deltat, revjul, sidtime};
use clap::Args;

#[derive(Args)]
pub struct JdArgs {
    /// Date to convert to JD (`YYYY-MM-DD [HH:MM[:SS]]`)
    #[arg(group = "input")]
    pub date: Option<String>,

    /// Julian day to convert to calendar date
    #[arg(long, group = "input", allow_hyphen_values = true)]
    pub from_jd: Option<f64>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: JdArgs) -> Result<(), CliError> {
    let jd = if let Some(jd) = args.from_jd {
        jd
    } else if let Some(ref s) = args.date {
        parse::parse_date(s)?
    } else {
        celestial_core::jdnow()
    };

    let d = revjul(jd, Calendar::Gregorian);
    let dt = deltat(jd) * 86_400.0; // days → seconds
    let st = sidtime(jd); // hours

    let total_min = (d.hour * 60.0).round() as i32;
    let h = total_min / 60;
    let m = total_min % 60;

    let st_h = st as u32;
    let st_m = ((st - st_h as f64) * 60.0) as u32;
    let st_s = (((st - st_h as f64) * 60.0 - st_m as f64) * 60.0).round() as u32;

    // Day of week
    let dow = celestial_core::day_of_week(jd);
    let dow_name = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][dow as usize % 7];

    if args.json {
        println!(
            "{}",
            fmt::json_obj(&[
                ("jd", format!("{jd:.6}")),
                ("year", format!("{}", d.year)),
                ("month", format!("{}", d.month)),
                ("day", format!("{}", d.day)),
                ("hour", format!("{:.6}", d.hour)),
                ("day_of_week", dow_name.to_string()),
                ("delta_t_sec", format!("{dt:.3}")),
                ("sidereal_time", format!("{st:.6}")),
            ])
        );
        return Ok(());
    }

    println!();
    println!("  Julian day     {jd:.6}");
    println!(
        "  Calendar       {:04}-{:02}-{:02} {:02}:{:02} UT  ({})",
        d.year, d.month, d.day, h, m, dow_name
    );
    println!("  Delta T        {dt:.2} s");
    println!("  Sidereal time  {st_h:02}h {st_m:02}m {st_s:02}s (GAST)");
    println!();
    Ok(())
}
