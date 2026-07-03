//! TOML `--config` loading and argument-override precedence:
//! `ConfigFile`/`RenderSection`, `load_config`, `--var` overrides and
//! date+time merging.
//!
//! Split out of the former 3.5k-line `mod.rs` god file — pure code
//! movement, no behaviour change (re-exported by the facade).

use super::*;
use std::path::{Component, Path, PathBuf};

// ─── Config file ──────────────────────────────────────────────────────────────

#[derive(Debug, Default, serde::Deserialize)]
struct ConfigFile {
    render: Option<RenderSection>,
    vars: Option<toml::value::Table>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct RenderSection {
    date: Option<String>,
    timezone: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    out: Option<PathBuf>,
    template: Option<PathBuf>,
    hsys: Option<char>,
}

// ─── Entry point ─────────────────────────────────────────────────────────────

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 2 — additional chart-type context builders
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 2 — SVG renderers for new chart types
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert any `toml::Value` leaf into its owned `String` representation
/// (used when merging `[vars]` from a config file).
fn toml_value_to_string(v: &toml::Value) -> String {
    match v {
        toml::Value::String(x) => x.clone(),
        toml::Value::Integer(x) => x.to_string(),
        toml::Value::Float(x) => x.to_string(),
        toml::Value::Boolean(x) => x.to_string(),
        other => other.to_string(),
    }
}

/// Apply `Some(value)` to `*field` only if `should_override` is `true`.
fn override_if<T>(field: &mut T, candidate: Option<T>, should_override: bool) {
    if should_override {
        if let Some(v) = candidate {
            *field = v;
        }
    }
}

fn reject_unsafe_config_path(field: &str, path: &Path) -> Result<(), CliError> {
    if path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(CliError::Config(format!(
            "config `{field}` must be a relative path without `..` (got `{}`); \
             pass an unrestricted path via the `--{field}` flag instead",
            path.display()
        )));
    }
    Ok(())
}

/// Read a TOML config file and merge its defaults into `args` (only fields
/// still at their sentinel defaults are overridden), returning its `[vars]`
/// map. Empty map if no config file is specified.
pub(crate) fn load_config(args: &mut RenderArgs) -> Result<BTreeMap<String, String>, CliError> {
    let mut file_vars = BTreeMap::new();
    let Some(cfg_path) = args.config.clone() else {
        return Ok(file_vars);
    };
    let text = std::fs::read_to_string(&cfg_path)
        .map_err(|e| format!("cannot read config `{}`: {e}", cfg_path.display()))?;
    let cfg: ConfigFile = toml::from_str(&text).map_err(|e| format!("invalid config TOML: {e}"))?;

    if let Some(r) = cfg.render {
        // Snapshot field values before taking &mut borrows so the predicate
        // doesn't conflict with the mutable borrow.
        let date_at_default = args.date == "now";
        let lat_at_default = args.lat == 0.0;
        let lon_at_default = args.lon == 0.0;
        let hsys_at_default = args.hsys == 'P';

        // Each line: "use the config value for this field if the user didn't
        // already specify one on the command line".
        override_if(&mut args.date, r.date, date_at_default);
        if args.timezone.is_none() {
            args.timezone = r.timezone;
        }
        override_if(&mut args.lat, r.lat, lat_at_default);
        override_if(&mut args.lon, r.lon, lon_at_default);
        override_if(&mut args.hsys, r.hsys, hsys_at_default);
        if args.template.is_none() {
            if let Some(template) = &r.template {
                reject_unsafe_config_path("template", template)?;
            }
            args.template = r.template;
        }
        if args.out.is_none() {
            // SEC-5: a config file may be untrusted. An `out` taken
            // from it must stay a relative path inside the working
            // dir — reject absolute paths and any `..` component so a
            // crafted `[render] out = "/etc/…"` / "../../…" can't make
            // the tool write (and `create_dir_all`) outside cwd. An
            // explicit CLI `--out` is the user's own intent and is
            // left unrestricted (this branch only runs when it's None).
            if let Some(o) = &r.out {
                reject_unsafe_config_path("out", o)?;
            }
            args.out = r.out;
        }
    }
    if let Some(t) = cfg.vars {
        for (k, v) in t {
            file_vars.insert(k, toml_value_to_string(&v));
        }
    }
    Ok(file_vars)
}

/// Apply `--var KEY=VALUE` CLI overrides on top of config-file vars.
pub(crate) fn apply_var_overrides(
    vars: &[String],
    mut base: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, CliError> {
    for kv in vars {
        let (k, v) = kv
            .split_once('=')
            .ok_or_else(|| format!("`--var` must be KEY=VALUE, got `{kv}`"))?;
        base.insert(k.to_string(), v.to_string());
    }
    Ok(base)
}

/// Merge a separate `--time HH:MM` CLI argument into `--date` when the
/// date is a simple bare calendar date. Time is ignored when `date` is
/// "now", a raw JD, or already contains a time component.
pub(crate) fn merge_date_and_time(date: &str, time: Option<&str>) -> String {
    let Some(t) = time else {
        return date.to_string();
    };
    let base = date.trim();
    if base == "now" || base.parse::<f64>().is_ok() || base.contains(' ') {
        date.to_string()
    } else {
        format!("{base} {t}")
    }
}
