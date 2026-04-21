//! `celestial houses` — house cusps and special angles.

use crate::{format as fmt, parse};
use celestial_core::body::{CalcFlags, HouseSystem};
use celestial_core::houses_ex;
use clap::Args;

#[derive(Args)]
pub struct HousesArgs {
    /// Date (YYYY-MM-DD [HH:MM[:SS]]) or Julian day, default: now
    #[arg(short, long, default_value = "now")]
    pub date: String,

    /// Geographic latitude in decimal degrees (N positive)
    #[arg(long, allow_hyphen_values = true)]
    pub lat: f64,

    /// Geographic longitude in decimal degrees (E positive)
    #[arg(long, allow_hyphen_values = true)]
    pub lon: f64,

    /// House system: placidus, koch, equal, whole, porphyry, regio, campanus, morinus
    #[arg(short, long, default_value = "placidus")]
    pub system: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: HousesArgs) -> Result<(), String> {
    let jd = parse::parse_date(&args.date)?;
    let hsys = parse::parse_hsys(&args.system)?;

    let h = houses_ex(
        jd,
        CalcFlags::BUILTIN,
        args.lat,
        args.lon,
        HouseSystem(hsys),
    )
    .map_err(|e| e.to_string())?;

    let asc = h.ascmc[0];
    let mc = h.ascmc[1];
    let armc = h.ascmc[2];
    let vertex = h.ascmc[3];

    if args.json {
        let cusps_json: Vec<String> = h.cusps[1..=12].iter().map(|c| format!("{c:.6}")).collect();
        println!(
            "{}",
            fmt::json_obj(&[
                ("jd", format!("{jd:.4}")),
                ("lat", format!("{:.4}", args.lat)),
                ("lon", format!("{:.4}", args.lon)),
                ("system", parse::hsys_name(hsys).to_string()),
                ("asc", format!("{asc:.6}")),
                ("mc", format!("{mc:.6}")),
                ("armc", format!("{armc:.6}")),
                ("vertex", format!("{vertex:.6}")),
                ("cusps", format!("[{}]", cusps_json.join(", "))),
            ])
        );
        return Ok(());
    }

    let lat_dir = if args.lat >= 0.0 { "N" } else { "S" };
    let lon_dir = if args.lon >= 0.0 { "E" } else { "W" };
    let lat_d = args.lat.abs() as u32;
    let lat_m = ((args.lat.abs() - lat_d as f64) * 60.0).round() as u32;
    let lon_d = args.lon.abs() as u32;
    let lon_m = ((args.lon.abs() - lon_d as f64) * 60.0).round() as u32;

    println!();
    println!("  {}  ·  JD {:.4}", parse::jd_to_str(jd), jd);
    println!(
        "  {lat_d}°{lat_m:02}'{lat_dir}  {lon_d}°{lon_m:02}'{lon_dir}  ·  {}",
        parse::hsys_name(hsys)
    );
    println!("  {}", fmt::rule(40));
    println!("  {:<5}  Longitude", "Angle");
    println!("  {}", fmt::rule(40));
    println!("  {:<5}  {}", "ASC", fmt::lon_zodiac(asc));
    println!("  {:<5}  {}", "MC", fmt::lon_zodiac(mc));
    println!("  {:<5}  {}", "ARMC", fmt::lon_zodiac(armc));
    println!("  {:<5}  {}", "Vertex", fmt::lon_zodiac(vertex));
    println!("  {}", fmt::rule(40));
    println!("  {:<5}  Cusp", "House");
    println!("  {}", fmt::rule(40));
    for i in 1..=12 {
        println!(
            "  {:<5}  {}",
            format!("{i:2}th"),
            fmt::lon_zodiac(h.cusps[i])
        );
    }
    println!();
    Ok(())
}
