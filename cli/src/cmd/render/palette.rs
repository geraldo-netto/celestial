//! Wheel layout geometry constants + palette (colour-var) merge helpers.
//!
//! Pure SVG layout coordinates and string/JSON plumbing — no ephemeris
//! or precision compute. Extracted from the render god-file (ARCH-8);
//! re-exported by `mod.rs` so `super::CX` / `super::palette_vars` call
//! sites are unchanged.

use serde_json::{json, Value};

pub(crate) const CX: f64 = 450.0;
pub(crate) const CY: f64 = 424.0;
pub(crate) const RO: f64 = 320.0; // outer ring
pub(crate) const RM: f64 = 290.0; // sign band outer
pub(crate) const RI: f64 = 262.0; // sign band inner
pub(crate) const RH: f64 = 238.0; // house cusp inner
pub(crate) const RP: f64 = 212.0; // planet ring
pub(crate) const RC: f64 = 88.0; // inner circle

/// Build a palette `BTreeMap<String, Value>` from a slice of default
/// `(key, color)` pairs, merging user-provided `vars` over the top.
/// User vars take precedence so callers can override any color via `--var key=value`.
pub(crate) fn palette_with_defaults(
    defaults: &[(&str, &str)],
    vars: &std::collections::BTreeMap<String, String>,
) -> std::collections::BTreeMap<String, Value> {
    let mut palette = std::collections::BTreeMap::new();
    for (k, v) in defaults {
        palette.insert((*k).to_string(), json!(v));
    }
    for (k, v) in vars {
        palette.insert(k.clone(), json!(v));
    }
    palette
}

/// DUP-7: the title-default + palette-merge + JSON-object boilerplate
/// every tradition builder repeated. Inserts `title` only when the
/// user didn't supply one, merges `defaults` under the user vars, and
/// returns the ready `vars` JSON object — byte-identical to the
/// former hand-written trio.
pub(crate) fn palette_vars(
    mut user_vars: std::collections::BTreeMap<String, String>,
    title_default: &str,
    defaults: &[(&str, &str)],
) -> Value {
    user_vars
        .entry("title".to_string())
        .or_insert_with(|| title_default.to_string());
    Value::Object(palette_with_defaults(defaults, &user_vars).into_iter().collect())
}

/// DUP-7: `palette_with_defaults` + `Value::Object(… .collect())` in
/// one call, for builders whose `title` is injected upstream (so they
/// don't self-insert one like [`palette_vars`]). Byte-identical to the
/// former two-step.
pub(crate) fn palette_obj(
    vars: &std::collections::BTreeMap<String, String>,
    defaults: &[(&str, &str)],
) -> Value {
    Value::Object(palette_with_defaults(defaults, vars).into_iter().collect())
}
