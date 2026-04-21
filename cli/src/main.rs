//! `celestial` — command-line interface for the celestial astronomical engine.
//!
//! Built-in commands:  calc  houses  chart  render  moon  crossing  eclipse
//!                     jd  sabbats  esbats  omer  calendar
//!
//! Plugin commands: any `celestial-<n>` executable on $PATH becomes a subcommand.
//! Run `celestial --list-plugins` to see discovered plugins.

mod cmd;
mod format;
mod parse;
mod plugin;

use clap::{Parser, Subcommand};

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

    /// List discovered celestial-* plugin executables on $PATH
    #[arg(long, global = true)]
    list_plugins: bool,
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
    /// Full astrological chart with aspects
    Chart(cmd::chart::ChartArgs),
    /// Moon phase, illumination, and next principal phases
    Moon(cmd::moon::MoonArgs),
    /// Sefirat HaOmer — 49-day Omer count
    Omer(cmd::omer::OmerArgs),
    /// Multi-tradition religious calendars
    Calendar(cmd::calendar::CalendarArgs),
    /// Render a Jinja2 template with celestial chart data (SVG, HTML, …)
    Render(cmd::render::RenderArgs),
}

fn main() {
    let raw: Vec<String> = std::env::args().collect();

    // --list-plugins
    if raw.iter().any(|a| a == "--list-plugins") {
        let plugins = plugin::discover();
        if plugins.is_empty() {
            eprintln!("No celestial-* plugins found on $PATH.");
        } else {
            println!("Discovered plugins:");
            for p in &plugins {
                println!("  celestial-{:<22} {}", p.name, p.path.display());
            }
        }
        return;
    }

    // Plugin dispatch: if first arg is not a known built-in, try PATH lookup.
    let sub = raw.get(1).map(String::as_str).unwrap_or("");
    let is_builtin = matches!(
        sub,
        "calc"
            | "houses"
            | "sabbats"
            | "esbats"
            | "jd"
            | "crossing"
            | "eclipse"
            | "chart"
            | "moon"
            | "omer"
            | "calendar"
            | "render"
            | "--help"
            | "-h"
            | "--version"
            | "-V"
            | "--list-plugins"
            | ""
    );
    if !is_builtin && !sub.starts_with('-') {
        if let Err(msg) = plugin::try_exec(sub, &raw[2..]) {
            eprintln!("error: {msg}");
            std::process::exit(1);
        }
    }

    // Normal clap dispatch.
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
        Command::Render(a) => cmd::render::run(a),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
