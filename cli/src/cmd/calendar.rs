//! `celestial calendar` — multi-tradition religious and spiritual calendars.
//!
//! Subcommand dispatches to jewish, easter, islamic, panchanga, vesak, or nowruz.

use crate::error::CliError;
use crate::{format as fmt, parse};
use celestial_core::body::Calendar;
use celestial_core::{
    bahai_holy_days, christian_feasts, christian_fixed_feasts, easter_gregorian, easter_orthodox,
    gregorian_to_hijri_years, gregorian_to_solar_hijri, hijri_from_jd, hijri_month_name,
    hindu_festivals, islamic_observances, jd_to_bahai, jdnow, jewish_holidays, naw_ruz_jd,
    nowruz_jd, panchanga, revjul, uposatha_days, vesak_jd, Paksha, UposathaPhase,
};
use celestial_core::JulianDay;
use clap::{Args, Subcommand};

// ─── Top-level args ────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct CalendarArgs {
    #[command(subcommand)]
    pub tradition: Tradition,
}

#[derive(Subcommand)]
pub enum Tradition {
    /// Jewish holidays for a Hebrew year
    Jewish(JewishArgs),
    /// Easter and the Christian liturgical calendar
    Easter(EasterArgs),
    /// Islamic (Hijri) calendar and observances
    Islamic(IslamicArgs),
    /// Hindu Panchānga (five limbs) for a date
    Panchanga(PanchangaArgs),
    /// Buddhist observances — Vesak and Uposatha days
    Vesak(VesakArgs),
    /// Nowruz (Persian New Year) and the Bahá'í calendar
    Nowruz(NowruzArgs),
}

pub fn run(args: CalendarArgs) -> Result<(), CliError> {
    match args.tradition {
        Tradition::Jewish(a) => run_jewish(a),
        Tradition::Easter(a) => run_easter(a),
        Tradition::Islamic(a) => run_islamic(a),
        Tradition::Panchanga(a) => run_panchanga(a),
        Tradition::Vesak(a) => run_vesak(a),
        Tradition::Nowruz(a) => run_nowruz(a),
    }
}

// ─── Jewish ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct JewishArgs {
    /// Gregorian year (default: current year)
    #[arg(short, long)]
    pub year: Option<i32>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

fn run_jewish(args: JewishArgs) -> Result<(), CliError> {
    let greg_year = args
        .year
        .unwrap_or_else(|| revjul(JulianDay::new(jdnow()), Calendar::Gregorian).year);
    // Use both Hebrew years that overlap this Gregorian year
    let jd_jan1 = celestial_core::julday(greg_year, 1, 1, 0.0, Calendar::Gregorian);
    let jd_dec31 = celestial_core::julday(greg_year, 12, 31, 0.0, Calendar::Gregorian);
    let hy1 = celestial_core::hebrew_year_from_jd(JulianDay::new(jd_jan1));
    let hy2 = celestial_core::hebrew_year_from_jd(JulianDay::new(jd_dec31));

    let mut holidays: Vec<_> = jewish_holidays(hy1)
        .into_iter()
        .chain(if hy2 != hy1 {
            jewish_holidays(hy2)
        } else {
            vec![]
        })
        .filter(|h| {
            let d = revjul(JulianDay::new(h.jd), Calendar::Gregorian);
            d.year == greg_year
        })
        .collect();
    // total_cmp is NaN-safe; avoids panic if a calendar calc yields a NaN JD.
    holidays.sort_by(|a, b| a.jd.total_cmp(&b.jd));

    if args.json {
        let items: Vec<String> = holidays
            .iter()
            .map(|h| {
                fmt::json_obj(&[
                    ("name", h.name.to_string()),
                    ("hebrew_name", h.hebrew_name.to_string()),
                    ("jd", format!("{:.4}", h.jd)),
                    ("date", parse::jd_to_str(h.jd)),
                    ("days", h.days.to_string()),
                    ("category", format!("{:?}", h.category)),
                ])
            })
            .collect();
        println!("{}", fmt::json_array(items));
    } else {
        println!();
        println!("  Jewish holidays — {greg_year}");
        println!("  {}", fmt::rule(58));
        for h in &holidays {
            let d = revjul(JulianDay::new(h.jd), Calendar::Gregorian);
            let dur = if h.days > 1 {
                format!(" ({} days)", h.days)
            } else {
                String::new()
            };
            println!(
                "  {:<32}  {:04}-{:02}-{:02}{}",
                h.name, d.year, d.month, d.day, dur
            );
        }
        println!();
    }
    Ok(())
}

// ─── Easter ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct EasterArgs {
    /// Gregorian year (default: current year)
    #[arg(short, long)]
    pub year: Option<i32>,
    /// Show all moveable and fixed feasts
    #[arg(long)]
    pub all: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

fn run_easter(args: EasterArgs) -> Result<(), CliError> {
    let year = args
        .year
        .unwrap_or_else(|| revjul(JulianDay::new(jdnow()), Calendar::Gregorian).year);
    let (ey, em, ed) = easter_gregorian(year);
    let (oy, om, od) = easter_orthodox(year);

    if !args.all {
        if args.json {
            println!(
                "{}",
                fmt::json_obj(&[
                    ("year", year.to_string()),
                    ("easter_gregorian", format!("{ey:04}-{em:02}-{ed:02}")),
                    ("easter_orthodox", format!("{oy:04}-{om:02}-{od:02}")),
                ])
            );
        } else {
            println!();
            println!("  Easter {year}");
            println!("  {}", fmt::rule(40));
            println!("  Gregorian (Western):  {ey:04}-{em:02}-{ed:02}");
            println!("  Julian (Orthodox):    {oy:04}-{om:02}-{od:02}");
            println!();
        }
        return Ok(());
    }

    let mut feasts: Vec<_> = christian_feasts(year)
        .into_iter()
        .chain(christian_fixed_feasts(year))
        .collect();
    feasts.sort_by(|a, b| a.jd.total_cmp(&b.jd));

    if args.json {
        let items: Vec<String> = feasts
            .iter()
            .map(|f| {
                fmt::json_obj(&[
                    ("name", f.name.to_string()),
                    ("jd", format!("{:.4}", f.jd)),
                    ("date", format!("{:04}-{:02}-{:02}", f.year, f.month, f.day)),
                    ("easter_offset", f.easter_offset.to_string()),
                ])
            })
            .collect();
        println!("{}", fmt::json_array(items));
    } else {
        println!();
        println!("  Christian Calendar {year}");
        println!("  {}", fmt::rule(52));
        for f in &feasts {
            println!(
                "  {:<36}  {:04}-{:02}-{:02}",
                f.name, f.year, f.month, f.day
            );
        }
        println!();
    }
    Ok(())
}

// ─── Islamic ─────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct IslamicArgs {
    /// Gregorian year (default: current year)
    #[arg(short, long)]
    pub year: Option<i32>,
    /// Convert a date to Hijri (YYYY-MM-DD or JD)
    #[arg(long, value_name = "DATE")]
    pub convert: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

fn run_islamic(args: IslamicArgs) -> Result<(), CliError> {
    if let Some(ref date_str) = args.convert {
        let jd = parse::parse_date(date_str)?;
        let (hy, hm, hd) = hijri_from_jd(JulianDay::new(jd));
        let greg = revjul(JulianDay::new(jd), Calendar::Gregorian);
        if args.json {
            println!(
                "{}",
                fmt::json_obj(&[
                    ("hijri_year", hy.to_string()),
                    ("hijri_month", hm.to_string()),
                    ("hijri_day", hd.to_string()),
                    ("month_name", hijri_month_name(hm).to_string()),
                    (
                        "gregorian",
                        format!("{:04}-{:02}-{:02}", greg.year, greg.month, greg.day)
                    ),
                ])
            );
        } else {
            println!(
                "\n  {} {} {} AH  =  {:04}-{:02}-{:02} CE\n",
                hd,
                hijri_month_name(hm),
                hy,
                greg.year,
                greg.month,
                greg.day
            );
        }
        return Ok(());
    }

    let greg_year = args
        .year
        .unwrap_or_else(|| revjul(JulianDay::new(jdnow()), Calendar::Gregorian).year);
    let (hy1, hy2) = gregorian_to_hijri_years(greg_year);
    let solar_hijri = gregorian_to_solar_hijri(greg_year);

    let mut obs: Vec<_> = islamic_observances(hy1)
        .into_iter()
        .chain(if hy2 != hy1 {
            islamic_observances(hy2)
        } else {
            vec![]
        })
        .filter(|o| {
            let d = revjul(JulianDay::new(o.jd), Calendar::Gregorian);
            d.year == greg_year
        })
        .collect();
    obs.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    obs.dedup_by(|a, b| a.name == b.name);

    if args.json {
        let items: Vec<String> = obs
            .iter()
            .map(|o| {
                fmt::json_obj(&[
                    ("name", o.name.to_string()),
                    ("arabic", o.arabic_name.to_string()),
                    ("jd", format!("{:.4}", o.jd)),
                    ("date", parse::jd_to_str(o.jd)),
                    ("days", o.days.to_string()),
                ])
            })
            .collect();
        println!("{}", fmt::json_array(items));
    } else {
        println!();
        println!("  Islamic observances — {greg_year} CE");
        println!("  (Hijri {hy1}/{hy2} AH · Solar Hijri {solar_hijri})");
        println!("  {}", fmt::rule(56));
        for o in &obs {
            let d = revjul(JulianDay::new(o.jd), Calendar::Gregorian);
            let dur = if o.days > 1 {
                format!(" ({} days)", o.days)
            } else {
                String::new()
            };
            println!(
                "  {:<36}  {:04}-{:02}-{:02}{}",
                o.name, d.year, d.month, d.day, dur
            );
        }
        println!();
    }
    Ok(())
}

// ─── Panchānga ────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct PanchangaArgs {
    /// Date/time to query (default: now). Formats: now, YYYY-MM-DD, JD float.
    #[arg(short, long, default_value = "now")]
    pub date: String,
    /// Show major Hindu festivals for the year
    #[arg(long)]
    pub festivals: bool,
    /// Gregorian year for festivals (default: current year)
    #[arg(short, long)]
    pub year: Option<i32>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

fn run_panchanga(args: PanchangaArgs) -> Result<(), CliError> {
    if args.festivals {
        let year = args
            .year
            .unwrap_or_else(|| revjul(JulianDay::new(jdnow()), Calendar::Gregorian).year);
        let festivals = hindu_festivals(year);
        if args.json {
            let items: Vec<String> = festivals
                .iter()
                .map(|f| {
                    fmt::json_obj(&[
                        ("name", f.name.to_string()),
                        ("description", f.description.to_string()),
                        ("jd", format!("{:.4}", f.jd)),
                        ("date", parse::jd_to_str(f.jd)),
                    ])
                })
                .collect();
            println!("{}", fmt::json_array(items));
        } else {
            println!();
            println!("  Hindu festivals — {year}");
            println!("  {}", fmt::rule(52));
            for f in &festivals {
                let d = revjul(JulianDay::new(f.jd), Calendar::Gregorian);
                println!(
                    "  {:<32}  {:04}-{:02}-{:02}",
                    f.name, d.year, d.month, d.day
                );
            }
            println!();
        }
        return Ok(());
    }

    let jd = parse::parse_date(&args.date)?;
    let p = panchanga(JulianDay::new(jd));
    let paksha_str = match p.paksha {
        Paksha::Shukla => "Shukla (waxing)",
        Paksha::Krishna => "Krishna (waning)",
    };

    if args.json {
        println!(
            "{}",
            fmt::json_obj(&[
                ("tithi", p.tithi.to_string()),
                ("tithi_name", p.tithi_name.to_string()),
                ("paksha", paksha_str.to_string()),
                ("vara", p.vara.to_string()),
                ("vara_name", p.vara_name.to_string()),
                ("nakshatra", p.nakshatra.to_string()),
                ("nakshatra_name", p.nakshatra_name.to_string()),
                ("nakshatra_pada", p.nakshatra_pada.to_string()),
                ("yoga", p.yoga.to_string()),
                ("yoga_name", p.yoga_name.to_string()),
                ("karana", p.karana.to_string()),
                ("karana_name", p.karana_name.to_string()),
                ("moon_lon", format!("{:.4}", p.moon_lon)),
                ("sun_lon", format!("{:.4}", p.sun_lon)),
                ("elongation", format!("{:.4}", p.elongation)),
            ])
        );
    } else {
        println!();
        println!("  Panchānga — {}", parse::jd_to_str(jd));
        println!("  {}", fmt::rule(44));
        println!(
            "  Tithi:      {} — {} ({})",
            p.tithi, p.tithi_name, paksha_str
        );
        println!("  Vara:       {}", p.vara_name);
        println!(
            "  Nakshatra:  {} pada {} ({:.1}°)",
            p.nakshatra_name, p.nakshatra_pada, p.moon_lon
        );
        println!("  Yoga:       {}", p.yoga_name);
        println!("  Karana:     {}", p.karana_name);
        println!();
    }
    Ok(())
}

// ─── Vesak ────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct VesakArgs {
    /// Gregorian year (default: current year)
    #[arg(short, long)]
    pub year: Option<i32>,
    /// Show all Uposatha days instead of just Vesak
    #[arg(long)]
    pub uposatha: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

fn run_vesak(args: VesakArgs) -> Result<(), CliError> {
    let year = args
        .year
        .unwrap_or_else(|| revjul(JulianDay::new(jdnow()), Calendar::Gregorian).year);

    if args.uposatha {
        let days = uposatha_days(year);
        if args.json {
            let items: Vec<String> = days
                .iter()
                .map(|u| {
                    let phase_str = format!("{:?}", u.phase);
                    fmt::json_obj(&[
                        ("phase", phase_str),
                        ("jd", format!("{:.4}", u.jd)),
                        ("date", parse::jd_to_str(u.jd)),
                        ("elongation", format!("{:.2}", u.elongation)),
                    ])
                })
                .collect();
            println!("{}", fmt::json_array(items));
        } else {
            println!();
            println!("  Uposatha days — {year}  ({} days)", days.len());
            println!("  {}", fmt::rule(46));
            for u in &days {
                let phase = format!("{:?}", u.phase);
                let marker = match u.phase {
                    UposathaPhase::FullMoon => "●",
                    UposathaPhase::NewMoon => "○",
                    UposathaPhase::FirstQuarter => "◑",
                    UposathaPhase::LastQuarter => "◐",
                };
                println!("  {}  {:<14}  {}", marker, phase, parse::jd_to_str(u.jd));
            }
            println!();
        }
        return Ok(());
    }

    let vesak = vesak_jd(year);
    let d = revjul(JulianDay::new(vesak), Calendar::Gregorian);
    if args.json {
        println!(
            "{}",
            fmt::json_obj(&[
                ("year", year.to_string()),
                ("jd", format!("{vesak:.4}")),
                ("date", format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)),
            ])
        );
    } else {
        println!();
        println!("  Vesak (Buddha Day) {year}");
        println!("  {}", fmt::rule(36));
        println!("  Vesak:  {:04}-{:02}-{:02}", d.year, d.month, d.day);
        println!();
    }
    Ok(())
}

// ─── Nowruz ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct NowruzArgs {
    /// Gregorian year (default: current year)
    #[arg(short, long)]
    pub year: Option<i32>,
    /// Show Bahá'í holy days for the year
    #[arg(long)]
    pub bahai: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

fn run_nowruz(args: NowruzArgs) -> Result<(), CliError> {
    let year = args
        .year
        .unwrap_or_else(|| revjul(JulianDay::new(jdnow()), Calendar::Gregorian).year);
    let solar_hijri = gregorian_to_solar_hijri(year);

    if args.bahai {
        // Bahá'í year starts at Nowruz; year n CE = BE year (n - 1843)
        let bahai_year = year - 1843;
        let holy_days = bahai_holy_days(bahai_year);
        if args.json {
            let items: Vec<String> = holy_days
                .iter()
                .map(|h| {
                    fmt::json_obj(&[
                        ("name", h.name.to_string()),
                        ("description", h.description.to_string()),
                        ("jd", format!("{:.4}", h.jd)),
                        ("date", parse::jd_to_str(h.jd)),
                    ])
                })
                .collect();
            println!("{}", fmt::json_array(items));
        } else {
            let naw_ruz = naw_ruz_jd(bahai_year);
            let d = revjul(JulianDay::new(naw_ruz), Calendar::Gregorian);
            println!();
            println!("  Bahá'í Calendar — {bahai_year} BE  ({year} CE)");
            println!("  Naw-Rúz: {:04}-{:02}-{:02}", d.year, d.month, d.day);
            println!("  {}", fmt::rule(56));
            for h in &holy_days {
                let d = revjul(JulianDay::new(h.jd), Calendar::Gregorian);
                println!(
                    "  {:<36}  {:04}-{:02}-{:02}",
                    h.name, d.year, d.month, d.day
                );
            }
            println!();
        }
        return Ok(());
    }

    let nowruz = nowruz_jd(year);
    let d = revjul(JulianDay::new(nowruz), Calendar::Gregorian);
    let bahai_year = year - 1843;
    let bd = jd_to_bahai(JulianDay::new(nowruz));

    if args.json {
        println!(
            "{}",
            fmt::json_obj(&[
                ("gregorian_year", year.to_string()),
                ("solar_hijri", solar_hijri.to_string()),
                ("bahai_year", bahai_year.to_string()),
                ("jd", format!("{nowruz:.4}")),
                ("date", format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)),
                ("time_ut", parse::jd_to_str(nowruz)),
            ])
        );
    } else {
        println!();
        println!("  Nowruz {year}");
        println!("  {}", fmt::rule(44));
        println!("  Date:         {:04}-{:02}-{:02}", d.year, d.month, d.day);
        println!("  Time (UT):    {}", parse::jd_to_str(nowruz));
        println!("  Solar Hijri:  {solar_hijri} SH");
        println!(
            "  Bahá'í:       {} BE  (1 {}, {})",
            bd.year, bd.month_name, bd.year
        );
        println!();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jewish_modes() {
        assert!(run(CalendarArgs { tradition: Tradition::Jewish(JewishArgs { year: Some(2024), json: false }) }).is_ok());
        assert!(run(CalendarArgs { tradition: Tradition::Jewish(JewishArgs { year: None, json: true }) }).is_ok());
    }

    #[test]
    fn easter_modes() {
        assert!(run(CalendarArgs { tradition: Tradition::Easter(EasterArgs { year: Some(2025), all: false, json: false }) }).is_ok());
        assert!(run(CalendarArgs { tradition: Tradition::Easter(EasterArgs { year: Some(2025), all: true, json: true }) }).is_ok());
    }

    #[test]
    fn islamic_modes() {
        assert!(run(CalendarArgs { tradition: Tradition::Islamic(IslamicArgs { year: Some(2024), convert: None, json: false }) }).is_ok());
        assert!(run(CalendarArgs { tradition: Tradition::Islamic(IslamicArgs { year: None, convert: Some("2024-01-01".into()), json: true }) }).is_ok());
    }

    #[test]
    fn panchanga_modes() {
        assert!(run(CalendarArgs { tradition: Tradition::Panchanga(PanchangaArgs { date: "2451545.0".into(), festivals: false, year: None, json: false }) }).is_ok());
        assert!(run(CalendarArgs { tradition: Tradition::Panchanga(PanchangaArgs { date: "2451545.0".into(), festivals: true, year: Some(2024), json: true }) }).is_ok());
    }

    #[test]
    fn vesak_modes() {
        assert!(run(CalendarArgs { tradition: Tradition::Vesak(VesakArgs { year: Some(2024), uposatha: false, json: false }) }).is_ok());
        assert!(run(CalendarArgs { tradition: Tradition::Vesak(VesakArgs { year: None, uposatha: true, json: true }) }).is_ok());
    }

    #[test]
    fn nowruz_modes() {
        assert!(run(CalendarArgs { tradition: Tradition::Nowruz(NowruzArgs { year: Some(2024), bahai: false, json: false }) }).is_ok());
        assert!(run(CalendarArgs { tradition: Tradition::Nowruz(NowruzArgs { year: None, bahai: true, json: true }) }).is_ok());
    }

    #[test]
    fn islamic_convert_bad_and_easter_all_text() {
        let _ = run(CalendarArgs { tradition: Tradition::Islamic(IslamicArgs { year: None, convert: Some("not-a-date".into()), json: false }) });
        assert!(run(CalendarArgs { tradition: Tradition::Easter(EasterArgs { year: None, all: true, json: false }) }).is_ok());
        assert!(run(CalendarArgs { tradition: Tradition::Jewish(JewishArgs { year: Some(5785), json: false }) }).is_ok());
        assert!(run(CalendarArgs { tradition: Tradition::Panchanga(PanchangaArgs { date: "1986-05-30 09:00".into(), festivals: true, year: None, json: false }) }).is_ok());
        assert!(run(CalendarArgs { tradition: Tradition::Nowruz(NowruzArgs { year: Some(2025), bahai: true, json: false }) }).is_ok());
        assert!(run(CalendarArgs { tradition: Tradition::Vesak(VesakArgs { year: Some(2025), uposatha: true, json: false }) }).is_ok());
    }
}
