//! `celestial` — command-line interface for the celestial astronomical engine.
//!
//! ```text
//! celestial <COMMAND> [OPTIONS]
//!
//! Commands:
//!   calc      Planetary positions
//!   houses    House cusps and special angles
//!   sabbats   Celtic Wheel of the Year
//!   esbats    Named full moons
//!   jd        Julian day ↔ calendar date conversion
//!   crossing  Next ecliptic longitude crossing
//!   eclipse   Solar and lunar eclipses
//!   chart     Full astrological chart (JSON + SVG wheel)
//!   moon      Moon phase, illumination, and phase timing
//!   omer      Sefirat HaOmer — 49-day count
//!   calendar  Religious/spiritual calendars (jewish|easter|islamic|panchanga|vesak|nowruz)
//! ```

mod cmd;
mod format;
mod parse;

use clap::{Parser, Subcommand};

// ─── Top-level CLI ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name    = "celestial",
    about   = "Astronomical calculations — celestial engine",
    version,
    propagate_version = true,
    color   = clap::ColorChoice::Auto,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Calculate geocentric planetary positions
    Calc(cmd::calc::CalcArgs),
    /// Calculate house cusps and special angles
    Houses(cmd::houses::HousesArgs),
    /// Celtic Wheel of the Year (8 sabbats)
    Sabbats(cmd::sabbats::SabbatsArgs),
    /// Named full moons (esbats)
    Esbats(cmd::sabbats::EsbatsArgs),
    /// Convert between Julian day numbers and calendar dates
    Jd(cmd::jd::JdArgs),
    /// Find the next ecliptic longitude crossing
    Crossing(cmd::crossing::CrossingArgs),
    /// Find solar or lunar eclipses
    Eclipse(cmd::eclipse::EclipseArgs),
    /// Full astrological chart with aspects and SVG wheel
    Chart(cmd::chart::ChartArgs),
    /// Moon phase, illumination, and next principal phases
    Moon(cmd::moon::MoonArgs),
    /// Sefirat HaOmer — 49-day Omer count
    Omer(cmd::omer::OmerArgs),
    /// Multi-tradition religious calendars (jewish|easter|islamic|panchanga|vesak|nowruz)
    Calendar(cmd::calendar::CalendarArgs),
}

// ─── Entry point ──────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Calc(a) => cmd::calc::run(a),
        Command::Houses(a) => cmd::houses::run(a),
        Command::Sabbats(a) => cmd::sabbats::run_sabbats(a),
        Command::Esbats(a) => cmd::sabbats::run_esbats(a),
        Command::Jd(a) => cmd::jd::run(a),
        Command::Crossing(a) => cmd::crossing::run(a),
        Command::Eclipse(a) => cmd::eclipse::run(a),
        Command::Chart(a) => cmd::chart::run(a),
        Command::Moon(a) => cmd::moon::run(a),
        Command::Omer(a) => cmd::omer::run(a),
        Command::Calendar(a) => cmd::calendar::run(a),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
