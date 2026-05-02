//! `celestial eclipse` — next solar or lunar eclipse.

use crate::{format as fmt, parse};
use celestial_core::{lun_eclipse_when, sol_eclipse_when_glob, CalcFlags};
use clap::Args;

#[derive(Args)]
pub struct EclipseArgs {
    /// Eclipse type: solar or lunar (default: solar)
    #[arg(short, long, default_value = "solar")]
    pub r#type: String,

    /// Search start date (default: now)
    #[arg(short, long, default_value = "now")]
    pub from: String,

    /// Search backwards in time
    #[arg(long)]
    pub backwards: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: EclipseArgs) -> Result<(), String> {
    let jd_start = parse::parse_date(&args.from)?;

    match args.r#type.to_lowercase().as_str() {
        "solar" | "sol" | "s" => {
            let r = sol_eclipse_when_glob(jd_start, CalcFlags::BUILTIN, 0, args.backwards)
                .map_err(|e| e.to_string())?;
            print_eclipse_result("Solar eclipse", &r.tret, args.json)
        }
        "lunar" | "lun" | "l" => {
            let r = lun_eclipse_when(jd_start, CalcFlags::BUILTIN, 0, args.backwards)
                .map_err(|e| e.to_string())?;
            print_eclipse_result("Lunar eclipse", &r.tret, args.json)
        }
        t => Err(format!(
            "unknown eclipse type: {t} (use 'solar' or 'lunar')"
        )),
    }
}

fn print_eclipse_result(label: &str, tret: &[f64], json: bool) -> Result<(), String> {
    // tret[0] = maximum; tret[1] = first contact; tret[4] = last contact
    let max_jd = tret[0];
    let first_jd = tret[1];
    let last_jd = tret[4];

    if json {
        println!(
            "{}",
            fmt::json_obj(&[
                ("type", label.to_string()),
                ("maximum_jd", format!("{max_jd:.4}")),
                ("maximum_date", parse::jd_to_str(max_jd)),
                ("first_contact", parse::jd_to_str(first_jd)),
                ("last_contact", parse::jd_to_str(last_jd)),
            ])
        );
        return Ok(());
    }

    println!();
    println!("  {label}");
    println!("  {}", fmt::rule(40));
    println!("  Maximum       {}", parse::jd_to_str(max_jd));
    if first_jd > 0.0 {
        println!("  First contact {}", parse::jd_to_str(first_jd));
    }
    if last_jd > 0.0 {
        println!("  Last contact  {}", parse::jd_to_str(last_jd));
    }
    println!("  JD            {max_jd:.4}");
    println!();
    Ok(())
}
