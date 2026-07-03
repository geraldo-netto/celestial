//! `celestial` — command-line interface for the celestial astronomical engine.
//!
//! Built-in commands:  calc  houses  render  moon  crossing  eclipse
//!                     jd  sabbats  esbats  omer  calendar  phenomena
//!
//! Plugin commands: any `celestial-<n>` executable on $PATH becomes a subcommand.
//! Run `celestial --list-plugins` to see discovered plugins.

use celestial_cli::{
    cmd,
    i18n::{tr, Lang},
    plugin,
};

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name    = "celestial",
    // Placeholder — the real about text is injected at runtime by
    // `localize()` based on the user's locale (CELESTIAL_LANG / LC_ALL / LANG).
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
    /// Moon phase, illumination, and next principal phases
    Moon(cmd::moon::MoonArgs),
    /// Sefirat HaOmer — 49-day Omer count
    Omer(cmd::omer::OmerArgs),
    /// Multi-tradition religious calendars
    Calendar(cmd::calendar::CalendarArgs),
    /// Apparent planetary phenomena (magnitude, phase, illumination, elongation)
    Phenomena(cmd::phenomena::PhenomenaArgs),
    /// Render a Jinja2 template with celestial chart data (SVG, HTML, …)
    Render(Box<cmd::render::RenderArgs>),
}

/// Apply locale-appropriate help text to the clap command tree.
///
/// We keep the derive-based `Cli` and `Command` definitions for parsing,
/// but override every user-visible `about` string at runtime. The English
/// strings on the derive structs above are therefore only fallbacks for
/// the unlikely case where i18n hasn't been initialised — every release
/// build runs through `localize()` before clap gets to print anything.
fn localize(cmd: clap::Command, lang: Lang) -> clap::Command {
    cmd.about(tr("top.about", lang))
        .mut_arg("list_plugins", |a| a.help(tr("flag.list_plugins", lang)))
        .mut_subcommand("calc", |c| c.about(tr("cmd.calc.about", lang)))
        .mut_subcommand("houses", |c| c.about(tr("cmd.houses.about", lang)))
        .mut_subcommand("sabbats", |c| c.about(tr("cmd.sabbats.about", lang)))
        .mut_subcommand("esbats", |c| c.about(tr("cmd.esbats.about", lang)))
        .mut_subcommand("jd", |c| c.about(tr("cmd.jd.about", lang)))
        .mut_subcommand("crossing", |c| c.about(tr("cmd.crossing.about", lang)))
        .mut_subcommand("eclipse", |c| c.about(tr("cmd.eclipse.about", lang)))
        .mut_subcommand("moon", |c| c.about(tr("cmd.moon.about", lang)))
        .mut_subcommand("omer", |c| c.about(tr("cmd.omer.about", lang)))
        .mut_subcommand("calendar", |c| c.about(tr("cmd.calendar.about", lang)))
        .mut_subcommand("phenomena", |c| c.about(tr("cmd.phenomena.about", lang)))
        .mut_subcommand("render", |c| c.about(tr("cmd.render.about", lang)))
}

fn main() {
    let raw: Vec<String> = std::env::args().collect();
    let lang = Lang::detect();

    // --list-plugins
    if raw.len() == 2 && raw[1] == "--list-plugins" {
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
    // Update BUILTIN_COMMANDS below when adding a new subcommand.
    const BUILTIN_COMMANDS: &[&str] = &[
        "calc",
        "houses",
        "sabbats",
        "esbats",
        "jd",
        "crossing",
        "eclipse",
        "moon",
        "omer",
        "calendar",
        "phenomena",
        "render",
        "--help",
        "-h",
        "--version",
        "-V",
        "--list-plugins",
        "",
    ];
    let sub = raw.get(1).map_or("", String::as_str);
    let is_builtin = BUILTIN_COMMANDS.contains(&sub);
    if !is_builtin && !sub.starts_with('-') {
        if let Err(msg) = plugin::try_exec(sub, &raw[2..]) {
            eprintln!("error: {msg}");
            std::process::exit(1);
        }
    }

    // Normal clap dispatch — but with localised help text injected.
    let cmd = localize(Cli::command(), lang);
    let matches = cmd.get_matches_from(&raw);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            // Errors from clap (e.g. invalid value) — print and exit. clap's
            // own error format is preserved; we only translate help text.
            e.exit();
        }
    };

    let result = match cli.command {
        Command::Calc(a) => cmd::calc::run(a),
        Command::Houses(a) => cmd::houses::run(a),
        Command::Sabbats(a) => cmd::sabbats::run_sabbats(a),
        Command::Esbats(a) => cmd::sabbats::run_esbats(a),
        Command::Jd(a) => cmd::jd::run(a),
        Command::Crossing(a) => cmd::crossing::run(a),
        Command::Eclipse(a) => cmd::eclipse::run(a),
        Command::Moon(a) => cmd::moon::run(a),
        Command::Omer(a) => cmd::omer::run(a),
        Command::Calendar(a) => cmd::calendar::run(a),
        Command::Phenomena(a) => cmd::phenomena::run(a),
        Command::Render(a) => cmd::render::run(*a),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
