//! cargo xtask — developer automation for celestial-workspace
//!
//! USAGE
//!
//!   cargo xtask parity           Check Python / JS / PHP export identical functions.
//!                                 Exits 1 if any binding is missing functions.
//!   cargo xtask codegen          Print stubs for functions missing from each binding.
//!                                 Exits 1 when gaps exist (useful as a CI gate).
//!   cargo xtask codegen --apply  Write generated stubs into the binding files.
//!                                 Review the diff before committing.
//!   cargo xtask stubs            Regenerate bindings/php/phpstan-stubs.php.
//!   cargo xtask test-stubs       Validate phpstan-stubs.php for PHP 8.0 syntax.
//!   cargo xtask pyi              Regenerate bindings/python/python/celestial_py/celestial_py.pyi.
//!   cargo xtask dts              Regenerate bindings/js/index.d.ts.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

// ─── entry point ─────────────────────────────────────────────────────────────

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("parity") => cmd_parity(),
        Some("coverage") => cmd_coverage(args.any(|a| a == "--write-allow")),
        Some("shapes") => cmd_shapes(args.any(|a| a == "--check")),
        Some("apidoc") => cmd_apidoc(args.any(|a| a == "--check")),
        Some("codegen") => cmd_codegen(args.any(|a| a == "--apply")),
        Some("stubs") => cmd_stubs(),
        Some("test-stubs") => cmd_test_stubs(),
        Some("pyi") => cmd_pyi(args.any(|a| a == "--check")),
        Some("dts") => cmd_dts(args.any(|a| a == "--check")),
        _ => {
            eprintln!(
                "USAGE\n  cargo xtask parity\n  cargo xtask coverage [--write-allow]   Core→binding coverage + arity parity\n  cargo xtask shapes [--check]           Return-shape contract snapshot\n  cargo xtask apidoc [--check]           Generate docs/generated/binding_api.md\n  cargo xtask codegen [--apply]\n  cargo xtask stubs\n  cargo xtask test-stubs\n  cargo xtask pyi        Regenerate bindings/python/python/celestial_py/celestial_py.pyi\n  cargo xtask dts        Regenerate bindings/js/index.d.ts"
            );
            std::process::exit(1);
        }
    }
}

// ─── workspace root ───────────────────────────────────────────────────────────

fn workspace_root() -> PathBuf {
    let mut dir = std::env::current_exe().unwrap();
    dir.pop();
    loop {
        let toml = dir.join("Cargo.toml");
        if toml.exists()
            && fs::read_to_string(&toml)
                .unwrap_or_default()
                .contains("[workspace]")
        {
            return dir;
        }
        if !dir.pop() {
            return std::env::current_dir().unwrap();
        }
    }
}

// ─── binding descriptor ───────────────────────────────────────────────────────

struct Binding {
    name: String,
    src_path: PathBuf,
    fns: BTreeSet<String>,
}

fn internal_fns() -> BTreeSet<String> {
    ["to_napi", "to_py", "tap"]
        .iter()
        .map(std::string::ToString::to_string)
        .collect()
}

/// Find the function name on or after line `start`, scanning at most 5 lines.
fn scan_fn_name(lines: &[&str], start: usize) -> Option<(String, usize)> {
    let n = lines.len();
    if let Some(j) = (start..n.min(start + 5)).next() {
        let l = lines[j].trim();
        let rest = l
            .strip_prefix("pub fn ")
            .or_else(|| l.strip_prefix("fn "))?;
        let name = rest.split('(').next()?.trim().to_string();
        return Some((name, j));
    }
    None
}

/// Find a `celestial::<name>(` delegate call within ~15 lines after `start`.
/// Returns the called name only if it differs from `fname` (i.e. is an alias).
fn scan_delegate(lines: &[&str], start: usize, fname: &str) -> Option<String> {
    let n = lines.len();
    for line in lines.iter().take(n.min(start + 15)).skip(start) {
        let l = line.trim();
        if let Some(after) = l.strip_prefix("celestial::") {
            let called: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !called.is_empty() && called != fname {
                return Some(called);
            }
            return None;
        }
        if l == "}" {
            return None;
        }
    }
    None
}

/// Legacy alias functions that exist for SwissEph / backward-compat naming.
/// Codegen will never flag these as missing or generate new stubs for them.
/// Maps alias_name → canonical_name.
fn legacy_aliases() -> std::collections::BTreeMap<String, String> {
    // Auto-detect by scanning for #[php_function] wrappers that immediately delegate
    // to celestial::<different_name>(...) — these are aliases by construction.
    let root = workspace_root();
    let php_src = fs::read_to_string(root.join("bindings/php/src/lib.rs")).unwrap_or_default();
    let mut map = std::collections::BTreeMap::new();

    let lines: Vec<&str> = php_src.lines().collect();
    let n = lines.len();
    let mut i = 0;
    while i < n {
        let t = lines[i].trim();
        let is_marker = t == "#[php_function]" || t.starts_with("#[allow");
        if !is_marker {
            i += 1;
            continue;
        }
        if let Some((fname, j)) = scan_fn_name(&lines, i + 1) {
            if let Some(real) = scan_delegate(&lines, j + 1, &fname) {
                map.insert(fname, real);
            }
        }
        i += 1;
    }
    map
}

// ─── parsing ─────────────────────────────────────────────────────────────────

fn fn_name_from_line(line: &str) -> Option<String> {
    let l = line.trim();
    let rest = l
        .strip_prefix("pub fn ")
        .or_else(|| l.strip_prefix("fn "))?;
    let name: String = rest.split([' ', '(', '<']).next().unwrap_or("").to_string();
    let valid = !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_');
    if valid {
        Some(name)
    } else {
        None
    }
}

fn decorated_fns(src: &str, decorator: &str) -> BTreeSet<String> {
    let prefix = format!("#[{decorator}");
    let mut out = BTreeSet::new();
    let lines: Vec<&str> = src.lines().collect();
    let n = lines.len();
    for (i, head) in lines.iter().enumerate() {
        let t = head.trim();
        if !(t.starts_with(&prefix) || t.starts_with("#[allow")) {
            continue;
        }
        for line in lines.iter().take(n.min(i + 6)).skip(i) {
            if let Some(name) = fn_name_from_line(line) {
                out.insert(name);
                break;
            }
        }
    }
    out
}

fn py_wrapped_fns(src: &str) -> BTreeSet<String> {
    src.split("wrap_pyfunction!(")
        .skip(1)
        .filter_map(|chunk| {
            let name = chunk.split(',').next()?.trim().to_string();
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                Some(name)
            } else {
                None
            }
        })
        .collect()
}

// ─── load bindings ────────────────────────────────────────────────────────────

struct BindingSpec {
    name: &'static str,
    rel_path: &'static str,
    decorator: &'static str,
}

fn load_bindings(root: &Path) -> Vec<Binding> {
    let skip = internal_fns();
    let legacy = legacy_aliases();
    // We skip the alias names themselves — they are intentional duplicates
    // of canonical names and should not appear in the parity union.
    let skip_aliases: BTreeSet<String> = legacy.keys().cloned().collect();
    let specs = [
        BindingSpec {
            name: "Python",
            rel_path: "bindings/python/src/lib.rs",
            decorator: "pyfunction",
        },
        BindingSpec {
            name: "JS",
            rel_path: "bindings/js/src/lib.rs",
            decorator: "napi",
        },
        BindingSpec {
            name: "PHP",
            rel_path: "bindings/php/src/lib.rs",
            decorator: "php_function",
        },
    ];

    specs
        .iter()
        .map(|spec| {
            let src_path = root.join(spec.rel_path);
            let src = fs::read_to_string(&src_path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", spec.rel_path));

            let mut fns = decorated_fns(&src, spec.decorator);
            if spec.name == "Python" {
                fns.extend(py_wrapped_fns(&src));
            }
            fns.retain(|f| !skip.contains(f) && !skip_aliases.contains(f));

            Binding {
                name: spec.name.to_string(),
                src_path,
                fns,
            }
        })
        .collect()
}

// ─── parity command ───────────────────────────────────────────────────────────

fn cmd_parity() {
    let bindings = load_bindings(&workspace_root());

    println!("celestial binding parity check");
    println!("==============================");
    for b in &bindings {
        println!("  {:<8} {} functions", b.name, b.fns.len());
    }

    let union: BTreeSet<String> = bindings
        .iter()
        .flat_map(|b| b.fns.iter().cloned())
        .collect();

    // Map: fn_name -> which bindings are missing it
    let mut gaps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for fn_name in &union {
        let missing: Vec<String> = bindings
            .iter()
            .filter(|b| !b.fns.contains(fn_name))
            .map(|b| b.name.clone())
            .collect();
        if !missing.is_empty() {
            gaps.insert(fn_name.clone(), missing);
        }
    }

    if gaps.is_empty() {
        println!(
            "\n✓ All three bindings export the same {} functions.",
            union.len()
        );
        return;
    }

    println!("\n✗ {} function(s) differ across bindings:\n", gaps.len());
    // Group by which binding(s) are missing it for compact output
    let mut by_missing: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (fn_name, missing) in &gaps {
        by_missing
            .entry(missing.join(", "))
            .or_default()
            .push(fn_name.clone());
    }
    for (missing_from, fns) in &by_missing {
        println!(
            "  Missing from [{missing_from}] ({} function{}):",
            fns.len(),
            if fns.len() == 1 { "" } else { "s" }
        );
        for fn_name in fns {
            println!("    {fn_name}");
        }
    }
    std::process::exit(1);
}

// ─── codegen command ──────────────────────────────────────────────────────────

fn make_codegen_stubs(missing: &[&String], js_src: &str, target: &str) -> Vec<String> {
    missing
        .iter()
        .map(|fn_name| match extract_fn_body(js_src, fn_name) {
            Some(body) => format!("// ── {fn_name} ──\n{}\n", convert_to(&body, target)),
            None => format!("// ── {fn_name} — no JS reference; add manually ──\n"),
        })
        .collect()
}

fn write_codegen_stubs(binding: &Binding, stubs: &[String]) {
    let original = fs::read_to_string(&binding.src_path).unwrap();
    let anchor = match binding.name.as_str() {
        "PHP" => "\n#[php_module]",
        "Python" => "\n#[pymodule]",
        _ => "\n// end",
    };
    let joined = stubs.join("\n");
    let new_src = if let Some(pos) = original.rfind(anchor) {
        format!("{}{}\n{}", &original[..pos], joined, &original[pos..])
    } else {
        format!("{original}\n{joined}")
    };
    fs::write(&binding.src_path, new_src).unwrap_or_else(|e| panic!("write failed: {e}"));
    println!("  ✓ stubs written to {}", binding.src_path.display());
}

fn cmd_codegen(apply: bool) {
    let root = workspace_root();
    let bindings = load_bindings(&root);

    let union: BTreeSet<String> = bindings
        .iter()
        .flat_map(|b| b.fns.iter().cloned())
        .collect();

    let js_src = fs::read_to_string(root.join("bindings/js/src/lib.rs")).unwrap_or_default();

    let mut any_gap = false;

    for binding in &bindings {
        let missing: Vec<&String> = union
            .iter()
            .filter(|fn_name| !binding.fns.contains(*fn_name))
            .collect();
        if missing.is_empty() {
            continue;
        }
        any_gap = true;

        println!(
            "\n── {} ({} missing) ──────────────────────────────",
            binding.name,
            missing.len()
        );

        let stubs = make_codegen_stubs(&missing, &js_src, &binding.name);
        for s in &stubs {
            print!("{s}");
        }

        if apply {
            write_codegen_stubs(binding, &stubs);
        }
    }

    if !any_gap {
        println!("✓ All bindings are already at parity — nothing to generate.");
        return;
    }
    if !apply {
        eprintln!("\nRun `cargo xtask codegen --apply` to write stubs into binding files.");
        eprintln!("Review the generated diff carefully before committing.");
        std::process::exit(1);
    }
}

// ─── JS function extractor ────────────────────────────────────────────────────

fn extract_fn_body(src: &str, name: &str) -> Option<String> {
    let needle = format!("fn {name}(");
    let fn_pos = src.find(&needle)?;
    let before = &src[..fn_pos];
    let block_start = before.rfind("\n#[").map_or(0, |p| p + 1);
    let brace_off = src[fn_pos..].find('{')?;
    let body_start = fn_pos + brace_off;

    let mut depth = 0usize;
    let mut end = body_start;
    for (i, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = body_start + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    Some(src[block_start..end].trim().to_string())
}

/// Count the number of parameters in a Rust function signature string.
fn count_params(src: &str) -> usize {
    if let (Some(open), Some(close)) = (src.find('('), src.rfind(')')) {
        let params = &src[open + 1..close];
        split_params(params)
            .iter()
            .filter(|p| {
                let p = p.trim();
                !p.is_empty() && p.contains(':')
            })
            .count()
    } else {
        0
    }
}

// ─── stub converter ───────────────────────────────────────────────────────────

fn convert_to(js_stub: &str, target: &str) -> String {
    // Strip napi-specific and clippy allow decorators
    let stripped: String = js_stub
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("#[napi") && !t.starts_with("#[allow(clippy")
        })
        .collect::<Vec<_>>()
        .join("\n");

    match target {
        "Python" => {
            let body = stripped
                // Unwrap napi struct return types → Vec<f64> (pyo3 list encoding)
                .replace("-> napi::Result<HouseResult>", "-> PyResult<Vec<f64>>")
                .replace("-> napi::Result<AzAlt>", "-> PyResult<Vec<f64>>")
                .replace("-> napi::Result<EclipseResult>", "-> PyResult<Vec<f64>>")
                .replace("-> napi::Result<EclipseHow>", "-> PyResult<Vec<f64>>")
                .replace(
                    "-> napi::Result<EclipseResultAttr>",
                    "-> PyResult<Vec<f64>>",
                )
                .replace("-> napi::Result<EclipseWhere>", "-> PyResult<Vec<f64>>")
                .replace("-> napi::Result<Stations>", "-> PyResult<Vec<f64>>")
                .replace("-> napi::Result<PlanetPos>", "-> PyResult<Vec<f64>>")
                .replace("-> napi::Result<StarPos>", "-> PyResult<Vec<f64>>")
                // napi::Either → PyObject
                .replace("napi::Either<String, i32>", "PyObject")
                // Standard napi → pyo3 mappings
                .replace("-> napi::Result<", "-> PyResult<")
                .replace("napi::Result<", "PyResult<")
                .replace("napi::Error::from_reason(", "PyRuntimeError::new_err(")
                .replace(".map_err(to_napi)", ".map_err(to_py)")
                .replace("pub fn ", "fn ");
            let allow = if count_params(&body) > 7 {
                "#[allow(clippy::too_many_arguments)]\n"
            } else {
                ""
            };
            format!("{allow}#[pyfunction]\n{body}")
        }
        "PHP" => {
            let body = stripped
                // Unwrap napi struct return types to Vec<f64> (flat array encoding)
                .replace("-> napi::Result<HouseResult>",     "-> PhpResult<Vec<f64>>")
                .replace("-> napi::Result<AzAlt>",           "-> PhpResult<Vec<f64>>")
                .replace("-> napi::Result<EclipseResult>",   "-> PhpResult<Vec<f64>>")
                .replace("-> napi::Result<EclipseHow>",      "-> PhpResult<Vec<f64>>")
                .replace("-> napi::Result<EclipseResultAttr>","-> PhpResult<Vec<f64>>")
                .replace("-> napi::Result<EclipseWhere>",    "-> PhpResult<Vec<f64>>")
                .replace("-> napi::Result<Stations>",        "-> PhpResult<Vec<f64>>")
                .replace("-> napi::Result<PlanetPos>",       "-> PhpResult<Vec<f64>>")
                .replace("-> napi::Result<StarPos>",         "-> PhpResult<Vec<f64>>")
                // napi::Either<A,B> → mixed
                .replace("napi::Either<String, i32>", "mixed")
                .replace("napi::Either<", "mixed /* was Either<")
                // Standard napi error/result wrappers
                .replace("-> napi::Result<", "-> PhpResult<")
                .replace("napi::Result<", "PhpResult<")
                .replace("napi::Error::from_reason(", "PhpException::from(")
                .replace(
                    ".map_err(to_napi)",
                    ".map_err(|e| PhpException::from(e.to_string()))",
                )
                .replace(": i32", ": i64")
                .replace(": u32", ": i64")
                .replace("pub fn ", "fn ");
            let allow = if count_params(&body) > 7 {
                "#[allow(clippy::too_many_arguments)]\n"
            } else {
                ""
            };
            format!("{allow}#[php_function]\n{body}")
        }
        _ => stripped,
    }
}

/// Parse a single `pub const NAME: TYPE = ...;` line into (name, php_type).
/// Returns None for lines that don't match the expected shape.
fn parse_php_const_decl(line: &str) -> Option<(String, &'static str)> {
    let rest = line.trim().strip_prefix("pub const ")?;
    let colon = rest.find(':')?;
    let name = rest[..colon].trim().to_string();
    let after = &rest[colon + 1..];
    let eq = after.find('=')?;
    let rust_type = after[..eq].trim();
    let php_type = if rust_type.contains("f64") || rust_type.contains("f32") {
        "float"
    } else {
        "int"
    };
    Some((name, php_type))
}

/// Parse all `#[php_const] pub const NAME: type = value;` from the PHP binding source.
/// Returns (name, php_type, php_value) tuples for use in phpstan define() stubs.
fn parse_php_consts(src: &str) -> Vec<(String, String, String)> {
    let root = workspace_root();
    let consts_src = fs::read_to_string(root.join("core/src/constants.rs")).unwrap_or_default();
    let core_vals = parse_core_constants(&consts_src);

    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    for (i, head) in lines.iter().enumerate() {
        if head.trim() != "#[php_const]" {
            continue;
        }
        let Some(decl_line) = lines.get(i + 1) else {
            continue;
        };
        let Some((name, php_type)) = parse_php_const_decl(decl_line) else {
            continue;
        };
        let php_val = core_vals
            .get(&name)
            .cloned()
            .unwrap_or_else(|| "0".to_string());
        out.push((name, php_type.to_string(), php_val));
    }
    out
}

/// Parse one `pub const NAME: type = value;` line into (name, value-string).
fn parse_core_const_line(line: &str) -> Option<(String, String)> {
    let rest = line.trim().strip_prefix("pub const ")?;
    let colon = rest.find(':')?;
    let name = rest[..colon].trim().to_string();
    let after = &rest[colon + 1..];
    let eq = after.find('=')?;
    let value_raw = after[eq + 1..].trim().trim_end_matches(';');
    let is_expr = value_raw.contains('+')
        || value_raw.contains('*')
        || value_raw.starts_with("crate::")
        || !value_raw.chars().all(|c| c.is_ascii_digit() || c == '-');
    let val = if is_expr {
        value_raw
            .parse::<i64>()
            .map_or_else(|_| "0".to_string(), |v| v.to_string())
    } else {
        value_raw.to_string()
    };
    Some((name, val))
}

/// Parse `pub const NAME: type = value;` from a Rust constants file.
/// Returns a map of name → value string.
fn parse_core_constants(src: &str) -> std::collections::HashMap<String, String> {
    src.lines().filter_map(parse_core_const_line).collect()
}

// ─── stubs command ────────────────────────────────────────────────────────────

fn emit_stub_constants(
    out: &mut String,
    seen: &mut std::collections::BTreeSet<String>,
    php_src: &str,
) {
    for (cname, ctype, cval) in parse_php_consts(php_src) {
        if seen.insert(cname.clone()) {
            out.push_str(&format!(
                "\n/** @var {ctype} */\ndefine('{cname}', {cval});\n"
            ));
        }
    }
}

fn emit_stub_function(
    out: &mut String,
    seen: &mut std::collections::BTreeSet<String>,
    entry: &PhpFnEntry,
) {
    let fn_name = &entry.name;
    let prefixed = format!("celestial_{fn_name}");

    let php_params = entry
        .params
        .iter()
        .map(|(typ, nam)| format!("{} ${}", rust_type_to_php(typ, false), sanitise_var(nam)))
        .collect::<Vec<_>>()
        .join(", ");
    let ret = rust_type_to_php(&entry.ret, true);
    let docblock = format!("/** @return {} */", php_array_doctype(&entry.ret));

    if seen.insert(fn_name.clone()) {
        out.push_str(&format!(
            "\n{docblock}\nfunction {fn_name}({php_params}): {ret} {{}}\n"
        ));
    }
    if seen.insert(prefixed.clone()) {
        out.push_str(&format!(
            "\n{docblock}\nfunction {prefixed}({php_params}): {ret} {{}}\n"
        ));
    }
}

fn emit_stub_aliases(out: &mut String, seen: &mut std::collections::BTreeSet<String>) {
    for (alias, canonical) in &legacy_aliases() {
        let prefixed_alias = format!("celestial_{alias}");
        let doc = format!("/** Legacy alias for {canonical}(). @see {canonical} */");
        if seen.insert(alias.clone()) {
            out.push_str(&format!(
                "\n{doc}\nfunction {alias}(mixed ...$args): mixed {{}}\n"
            ));
        }
        if seen.insert(prefixed_alias.clone()) {
            out.push_str(&format!(
                "\n{doc}\nfunction {prefixed_alias}(mixed ...$args): mixed {{}}\n"
            ));
        }
    }
}

fn cmd_stubs() {
    let root = workspace_root();
    let php_src = fs::read_to_string(root.join("bindings/php/src/lib.rs"))
        .expect("cannot read bindings/php/src/lib.rs");
    let out_path = root.join("bindings/php/phpstan-stubs.php");

    let mut out = String::from("<?php\n");
    out.push_str("// Auto-generated by `cargo xtask stubs` — do not edit.\n// Regenerate: cargo xtask stubs\n");

    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    emit_stub_constants(&mut out, &mut seen, &php_src);
    for entry in regex_find_php_fns(&php_src) {
        emit_stub_function(&mut out, &mut seen, &entry);
    }
    emit_stub_aliases(&mut out, &mut seen);

    fs::write(&out_path, &out).expect("cannot write phpstan-stubs.php");
    println!("✓ {} symbols written to {}", seen.len(), out_path.display());
}

struct PhpFnEntry {
    name: String,
    params: Vec<(String, String)>,
    ret: String,
}

fn regex_find_php_fns(src: &str) -> Vec<PhpFnEntry> {
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let n = lines.len();
    let mut i = 0;

    while i < n {
        if lines[i].trim() == "#[php_function]" || lines[i].trim().starts_with("#[php_function(") {
            // Collect the function signature (may span multiple lines)
            let mut sig = String::new();
            let mut j = i + 1;
            let mut depth = 0usize;
            while j < n {
                let l = lines[j];
                sig.push_str(l);
                sig.push('\n');
                depth += l.chars().filter(|&c| c == '(').count();
                depth = depth.saturating_sub(l.chars().filter(|&c| c == ')').count());
                if depth == 0 && sig.contains('(') {
                    break;
                }
                j += 1;
            }
            if let Some(entry) = parse_fn_sig(&sig) {
                out.push(entry);
            }
        }
        i += 1;
    }
    out
}

/// Find the index of the `)` matching the `(` at `p_open` in `s`.
fn matching_paren(s: &str, p_open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, ch) in s[p_open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(p_open + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Try to parse a single trimmed parameter chunk into (ptype, pname).
/// Returns `None` for self/py/empty/invalid identifiers.
fn parse_param_chunk(p: &str) -> Option<(String, String)> {
    let p = p.trim().trim_start_matches("mut").trim();
    if p.is_empty() {
        return None;
    }
    let colon = p.find(':')?;
    let pname = p[..colon].trim().to_string();
    let ptype = p[colon + 1..].trim().to_string();
    if pname == "py" || pname == "self" {
        return None;
    }
    let starts_with_digit = pname.chars().next().is_none_or(|c| c.is_ascii_digit());
    let valid_chars = pname.chars().all(|c| c.is_alphanumeric() || c == '_');
    if pname.is_empty() || starts_with_digit || !valid_chars {
        return None;
    }
    Some((ptype, pname))
}

fn parse_fn_sig(sig: &str) -> Option<PhpFnEntry> {
    let sig_clean: String = sig
        .lines()
        .map(|l| l.find("//").map_or(l, |c| &l[..c]))
        .collect::<Vec<_>>()
        .join(" ");
    let sig_clean = sig_clean.trim();

    let fn_kw = sig_clean.find("fn ")?;
    let after = &sig_clean[fn_kw + 3..];
    let n_end = after.find('(')?;
    let name = after[..n_end].trim().to_string();
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }

    let p_open = sig_clean.find('(')?;
    let p_close = matching_paren(sig_clean, p_open)?;
    let params_str = sig_clean[p_open + 1..p_close].trim();

    let params: Vec<(String, String)> = split_params(params_str)
        .iter()
        .filter_map(|p| parse_param_chunk(p))
        .collect();

    let ret = if let Some(arrow) = sig_clean[p_close..].find("->") {
        let start = p_close + arrow + 2;
        let chunk = &sig_clean[start..];
        let end = chunk.find('{').unwrap_or(chunk.len());
        chunk[..end].trim().to_string()
    } else {
        "()".to_string()
    };

    Some(PhpFnEntry { name, params, ret })
}

/// Split a parameter string on commas, respecting `<>`, `()`, `[]` nesting.
fn split_params(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut depth = 0usize;
    for ch in s.chars() {
        match ch {
            '<' | '(' | '[' => {
                depth += 1;
                cur.push(ch);
            }
            '>' | ')' | ']' => {
                depth = depth.saturating_sub(1);
                cur.push(ch);
            }
            ',' if depth == 0 => {
                parts.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur.trim().to_string());
    }
    parts
}

/// Convert a Rust type string to a PHP type hint (for use in function signatures).
/// RULE: typed arrays (`int[]`, `float[]`, `string[]`) are NEVER emitted here.
/// Those appear only in @return docblocks (`php_array_doctype`).
fn rust_type_to_php(rust: &str, is_return: bool) -> &'static str {
    let t = rust
        .trim()
        .trim_start_matches("crate::")
        .trim_start_matches("celestial_core::");

    // Unwrap PhpResult<T> / Result<T> → look at T
    let inner = if let Some(s) = t
        .strip_prefix("PhpResult<")
        .and_then(|s| s.strip_suffix('>'))
    {
        s.trim()
    } else if let Some(s) = t.strip_prefix("Result<").and_then(|s| s.strip_suffix('>')) {
        s.trim()
    } else {
        t
    };

    // Option<T> → nullable variant
    if let Some(inner_opt) = inner
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        // Return the nullable version
        return match rust_type_to_php_str(inner_opt.trim()) {
            "array" => "?array",
            "string" => "?string",
            "float" => "?float",
            "int" => "?int",
            "bool" => "?bool",
            _ => "mixed",
        };
    }

    // () / void
    if inner == "()" || inner == "void" || (!is_return && inner.is_empty()) {
        return "void";
    }

    rust_type_to_php_str(inner)
}

fn rust_type_to_php_str(t: &str) -> &'static str {
    match t.trim() {
        "f64" | "f32" => "float",
        "i32" | "i64" | "u32" | "u64" | "u8" | "i8" | "usize" | "isize" => "int",
        "bool" => "bool",
        "String" | "&str" | "&'static str" => "string",
        "()" => "void",
        t if t.starts_with("Vec<") => "array",
        t if t.starts_with("HashMap") => "array",
        t if t.starts_with('[') => "array", // fixed arrays [f64;N]
        t if t.ends_with("Result") => "array",
        _ => "array", // all structs → array
    }
}

/// Strip a single-level `Wrapper<...>` if present.
fn strip_wrapper<'a>(t: &'a str, wrapper: &str) -> Option<&'a str> {
    t.strip_prefix(wrapper)
        .and_then(|s| s.strip_suffix('>'))
        .map(str::trim)
}

/// Map a non-nullable Rust core type to the PHPDoc base type.
fn php_doc_base_type(core: &str) -> &'static str {
    match core {
        "f64" | "f32" => "float",
        "i32" | "i64" | "u32" | "u64" | "u8" | "i8" | "usize" | "isize" => "int",
        "bool" => "bool",
        "String" | "&str" | "&'static str" => "string",
        "()" => "void",
        t if t.starts_with("Vec<f") => "float[]",
        t if t.starts_with("Vec<i") || t.starts_with("Vec<u") => "int[]",
        t if t.starts_with("Vec<String") || t.starts_with("Vec<&str") => "string[]",
        t if t.starts_with("HashMap<") => "array<string,mixed>",
        _ => "array",
    }
}

/// Return the PHPDoc @return type string (can use int[], float[], etc.).
fn php_array_doctype(rust: &str) -> &'static str {
    let t = rust.trim();
    let inner = strip_wrapper(t, "PhpResult<")
        .or_else(|| strip_wrapper(t, "Result<"))
        .unwrap_or(t);
    let (nullable, core) = match strip_wrapper(inner, "Option<") {
        Some(s) => (true, s),
        None => (false, inner),
    };
    let base = php_doc_base_type(core);
    if !nullable || base == "void" {
        return base;
    }
    match base {
        "float" => "float|null",
        "int" => "int|null",
        "bool" => "bool|null",
        "string" => "string|null",
        "float[]" => "float[]|null",
        "int[]" => "int[]|null",
        "string[]" => "string[]|null",
        "array<string,mixed>" => "array<string,mixed>|null",
        _ => "array|null",
    }
}

/// Sanitise a Rust parameter name to a valid PHP variable name.
fn sanitise_var(name: &str) -> std::borrow::Cow<'_, str> {
    let s = name.trim().trim_start_matches('_');
    if s.is_empty() || s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        // Rare: needs a "p" prefix — allocate only then
        std::borrow::Cow::Owned(format!("p{s}"))
    } else {
        // Common: valid identifier — borrow directly, no allocation
        std::borrow::Cow::Borrowed(s)
    }
}

// ─── test-stubs command ───────────────────────────────────────────────────────

/// Validate `phpstan-stubs.php` for PHP 8.0 syntax correctness without
/// requiring a PHP binary. Checks every rule that has caused CI failures.
/// Exits 1 on any error.
fn cmd_test_stubs() {
    let root = workspace_root();
    let path = root.join("bindings/php/phpstan-stubs.php");
    let src = fs::read_to_string(&path).expect("cannot read phpstan-stubs.php");

    let mut errors: Vec<String> = Vec::new();

    check_php_opening(&src, &mut errors);
    check_typed_array_hints(&src, &mut errors);
    check_dollar_digit(&src, &mut errors);
    check_rust_type_remnants(&src, &mut errors);
    let seen_fns = check_duplicate_functions(&src, &mut errors);
    check_double_prefix(&src, &mut errors);
    check_function_bodies(&src, &mut errors);
    check_suspicious_signatures(&src, &mut errors);

    report_stubs_result(&errors, seen_fns.len());
}

fn check_php_opening(src: &str, errors: &mut Vec<String>) {
    if !src.starts_with("<?php") {
        errors.push("missing <?php opening tag".into());
    }
}

fn check_typed_array_hints(src: &str, errors: &mut Vec<String>) {
    for (i, line) in src.lines().enumerate() {
        let t = line.trim();
        if !t.starts_with("function ") {
            continue;
        }
        let Some(start) = t.find('(') else { continue };
        if t.rfind(')').is_none() {
            continue;
        }
        let params_and_ret = &t[start..];
        if params_and_ret.contains("int[]")
            || params_and_ret.contains("float[]")
            || params_and_ret.contains("string[]")
        {
            errors.push(format!(
                "line {}: typed array hint in function signature: {}",
                i + 1,
                &t[..t.len().min(80)]
            ));
        }
    }
}

fn check_dollar_digit(src: &str, errors: &mut Vec<String>) {
    for (line_no, snippet) in regex_lite_find_dollar_digit(src) {
        errors.push(format!(
            "line {line_no}: invalid variable name starting with digit: {snippet}"
        ));
    }
}

fn check_rust_type_remnants(src: &str, errors: &mut Vec<String>) {
    const RUST_TYPES: &[&str] = &[
        "i32",
        "u32",
        "i64",
        "u64",
        "usize",
        "Vec<",
        "Option<",
        "PhpResult",
    ];
    for rt in RUST_TYPES {
        for (i, line) in src.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("function ") && t.contains(rt) {
                errors.push(format!(
                    "line {}: Rust type {:?} in PHP stub: {}",
                    i + 1,
                    rt,
                    &t[..t.len().min(80)]
                ));
            }
        }
    }
}

fn check_duplicate_functions(
    src: &str,
    errors: &mut Vec<String>,
) -> std::collections::BTreeMap<String, usize> {
    let mut seen_fns: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for (i, line) in src.lines().enumerate() {
        let t = line.trim();
        if !t.starts_with("function ") {
            continue;
        }
        let Some(paren) = t.find('(') else { continue };
        let name = t["function ".len()..paren].trim().to_string();
        if let Some(prev) = seen_fns.insert(name.clone(), i + 1) {
            errors.push(format!(
                "line {}: duplicate function {name} (first at line {prev})",
                i + 1
            ));
        }
    }
    seen_fns
}

fn check_double_prefix(src: &str, errors: &mut Vec<String>) {
    for (i, line) in src.lines().enumerate() {
        if line.contains("celestial_celestial_") {
            errors.push(format!(
                "line {}: double-prefix celestial_celestial_: {}",
                i + 1,
                line.trim().chars().take(80).collect::<String>()
            ));
        }
    }
}

fn check_function_bodies(src: &str, errors: &mut Vec<String>) {
    let fn_count = src
        .lines()
        .filter(|l| l.trim().starts_with("function "))
        .count();
    let body_count = src.matches("{}").count();
    if body_count < fn_count {
        errors.push(format!(
            "{fn_count} function declarations but only {body_count} empty bodies ({{}})"
        ));
    }
}

fn is_suspicious_signature(t: &str) -> bool {
    t.starts_with("function ")
        && (t.contains("flat:")
            || t.contains("...")
            || t.contains("$//")
            || t.contains(", $,")
            || t.contains("($,"))
}

fn check_suspicious_signatures(src: &str, errors: &mut Vec<String>) {
    for (i, line) in src.lines().enumerate() {
        if is_suspicious_signature(line.trim()) {
            errors.push(format!(
                "line {}: suspicious/malformed signature: {}",
                i + 1,
                line.trim().chars().take(80).collect::<String>()
            ));
        }
    }
}

fn report_stubs_result(errors: &[String], fn_total: usize) {
    if errors.is_empty() {
        println!("✓ phpstan-stubs.php: {fn_total} functions — all PHP 8.0 syntax checks passed");
        return;
    }
    eprintln!("✗ phpstan-stubs.php: {} error(s):\n", errors.len());
    for e in errors {
        eprintln!("  {e}");
    }
    std::process::exit(1);
}

/// Find every byte offset in `line` where a literal `$` is immediately followed
/// by an ASCII digit.
fn dollar_digit_columns(line: &str) -> Vec<usize> {
    let mut hits = Vec::new();
    let mut chars = line.chars().peekable();
    let mut col = 0usize;
    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek().is_some_and(char::is_ascii_digit) {
            hits.push(col);
        }
        col += ch.len_utf8();
    }
    hits
}

/// Find all `$<digit>` occurrences (invalid PHP variable names) and return
/// (line_number, snippet) pairs.
fn regex_lite_find_dollar_digit(src: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let t = line.trim();
        if t.starts_with("/**") || t.starts_with('*') || t.starts_with("//") {
            continue;
        }
        for col in dollar_digit_columns(line) {
            out.push((i + 1, line[col..].chars().take(20).collect::<String>()));
        }
    }
    out
}

// ─── pyi / dts generators ────────────────────────────────────────────────────

/// Decorated function metadata returned by [`scan_decorated_fns_full`].
struct DecoratedFn {
    /// The attribute line as it appears in source (e.g. `#[napi(js_name = "X")]`).
    attr_line: String,
    /// Parsed signature (name + params + return type).
    entry: PhpFnEntry,
}

/// Scan a Rust source file for every function decorated with a given attribute.
///
/// The attribute tag is matched flexibly: `#[attr]`, `#[attr(...)]`, `#[attr ...]`.
/// Returns the attribute line (trimmed) alongside the parsed signature so callers
/// can extract attribute options like `js_name = "X"`.
/// Has `line` started with `#[<prefix>]`, `#[<prefix>(`, or `#[<prefix> `?
fn starts_with_attr(line: &str, prefix: &str) -> bool {
    line.starts_with(&format!("#[{prefix}]"))
        || line.starts_with(&format!("#[{prefix}("))
        || line.starts_with(&format!("#[{prefix} "))
}

/// Skip further attribute lines, returning the index of the first `fn`-like line.
fn find_fn_line(lines: &[&str], from: usize) -> Option<usize> {
    let mut j = from;
    while j < lines.len() {
        let l = lines[j].trim_start();
        // Skip the rest of the attribute stack + doc/blank lines so we
        // land on the *item the attribute decorates*.
        if l.starts_with("#[") || l.starts_with("//") || l.is_empty() {
            j += 1;
            continue;
        }
        // The decorated item must itself be the fn. If it's a struct /
        // impl / enum / const / type / mod (a `#[napi(object)]` struct,
        // `impl From … { fn from }`, etc.), this attr is NOT a function
        // export — bail so the caller skips it. Without this guard the
        // scanner walked into a following `impl` block and mis-emitted
        // its `fn from` as a binding export.
        if l.contains("fn ") {
            return Some(j);
        }
        return None;
    }
    None
}

/// Collect the signature text (joined lines) starting at `from`, up through
/// the line that contains the opening `{`. Returns (signature, last_index).
fn collect_signature(lines: &[&str], from: usize) -> (String, usize) {
    let mut sig = String::new();
    let mut k = from;
    while k < lines.len() {
        sig.push_str(lines[k]);
        sig.push('\n');
        if lines[k].contains('{') {
            break;
        }
        k += 1;
    }
    (sig, k)
}

fn scan_decorated_fns_full(src: &str, attr_prefix: &str) -> Vec<DecoratedFn> {
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let n = lines.len();
    let mut i = 0;
    while i < n {
        let line = lines[i].trim_start();
        if !starts_with_attr(line, attr_prefix) {
            i += 1;
            continue;
        }
        let attr_line = line.to_string();
        let Some(j) = find_fn_line(&lines, i + 1) else {
            i += 1;
            continue;
        };
        let (sig, k) = collect_signature(&lines, j);
        if let Some(entry) = parse_fn_sig(&sig) {
            out.push(DecoratedFn { attr_line, entry });
        }
        i = k + 1;
    }
    out
}

/// Thin wrapper for callers that only need the parsed signatures.
fn scan_decorated_fns(src: &str, attr_prefix: &str) -> Vec<PhpFnEntry> {
    scan_decorated_fns_full(src, attr_prefix)
        .into_iter()
        .map(|d| d.entry)
        .collect()
}

/// Extract `js_name = "value"` from an attribute line, if present.
fn extract_js_name(attr_line: &str) -> Option<String> {
    let after = attr_line.split_once("js_name")?.1;
    let first_quote = after.find('"')?;
    let rest = &after[first_quote + 1..];
    let end_quote = rest.find('"')?;
    Some(rest[..end_quote].to_string())
}

/// Write `contents` to `path`, or verify it matches an existing file in `--check` mode.
///
/// In check mode, exits with code 1 and prints a message if the file is stale.
fn write_or_check(path: &Path, contents: &str, check: bool, kind: &str, cmd: &str, count: usize) {
    if check {
        let existing = fs::read_to_string(path).unwrap_or_default();
        if existing.trim() == contents.trim() {
            println!("✓ {count} {kind} in sync ({})", path.display());
        } else {
            eprintln!(
                "✗ {} is out of sync — run `cargo xtask {cmd}` to regenerate",
                path.display()
            );
            std::process::exit(1);
        }
    } else {
        fs::write(path, contents).expect("cannot write generated file");
        println!("✓ {count} {kind} written to {}", path.display());
    }
}

/// Language-specific type names & constructors for [`rust_type_to_lang`].
struct LangMap {
    /// `Result<T>` wrapper prefixes to unwrap (e.g. `"PyResult<"`, `"napi::Result<"`).
    result_prefixes: &'static [&'static str],
    float: &'static str,
    int: &'static str,
    bool: &'static str,
    string: &'static str,
    unit: &'static str, // `()` return
    fallback: &'static str,
    map: &'static str,
    /// `fn(inner_py) -> "list[{inner_py}]"` etc.
    vec_fmt: fn(&str) -> String,
    /// `fn(parts) -> "tuple[A, B, C]"` etc.
    tuple_fmt: fn(&[String]) -> String,
    /// `fn(inner) -> "{inner} | None"` etc.
    optional_fmt: fn(&str) -> String,
}

const PYI_LANG: LangMap = LangMap {
    result_prefixes: &["PyResult<", "Result<"],
    float: "float",
    int: "int",
    bool: "bool",
    string: "str",
    unit: "None",
    fallback: "object",
    map: "dict[str, object]",
    vec_fmt: |inner| format!("list[{inner}]"),
    tuple_fmt: |parts| format!("tuple[{}]", parts.join(", ")),
    optional_fmt: |inner| format!("{inner} | None"),
};

const DTS_LANG: LangMap = LangMap {
    result_prefixes: &["napi::Result<", "Result<"],
    float: "number",
    int: "number",
    bool: "boolean",
    string: "string",
    unit: "void",
    fallback: "unknown",
    map: "Record<string, number>",
    vec_fmt: |inner| format!("Array<{inner}>"),
    tuple_fmt: |parts| format!("[{}]", parts.join(", ")),
    optional_fmt: |inner| format!("{inner} | null"),
};

/// Convert a Rust type string to a target-language type according to [`LangMap`].
///
/// Handles `Result<T>`/`PyResult<T>`/`napi::Result<T>` unwrapping, `Option<T>`
/// nullability, `Vec<T>`, `HashMap`/`BTreeMap`, tuples `(A, B, ...)`, and
/// primitives. Falls back to [`LangMap::fallback`] for unknown types.
fn rust_type_to_lang(rust: &str, lang: &LangMap) -> String {
    let t = rust
        .trim()
        .trim_start_matches("crate::")
        .trim_start_matches("celestial_core::");

    // Unwrap Result<T> / PyResult<T> / napi::Result<T>
    let inner = lang
        .result_prefixes
        .iter()
        .find_map(|p| t.strip_prefix(p).and_then(|s| s.strip_suffix('>')))
        .map_or(t, str::trim);

    // Option<T> → nullable
    if let Some(inner_opt) = inner
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        return (lang.optional_fmt)(&rust_type_to_lang(inner_opt, lang));
    }

    match inner {
        "f64" | "f32" => lang.float.to_string(),
        "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
            lang.int.to_string()
        }
        "bool" => lang.bool.to_string(),
        "String" | "&str" | "&'static str" | "str" => lang.string.to_string(),
        "()" | "void" => lang.unit.to_string(),
        t if t.starts_with("Vec<") => (lang.vec_fmt)(&rust_type_to_lang(&t[4..t.len() - 1], lang)),
        t if t.starts_with("HashMap<") || t.starts_with("BTreeMap<") => lang.map.to_string(),
        t if t.starts_with('(') && t.ends_with(')') => {
            let parts: Vec<String> = split_params(&t[1..t.len() - 1])
                .iter()
                .map(|p| rust_type_to_lang(p, lang))
                .collect();
            (lang.tuple_fmt)(&parts)
        }
        _ => lang.fallback.to_string(),
    }
}

#[inline]
fn rust_type_to_pyi(rust: &str) -> String {
    rust_type_to_lang(rust, &PYI_LANG)
}

#[allow(dead_code)]
#[inline]
fn rust_type_to_ts(rust: &str) -> String {
    rust_type_to_lang(rust, &DTS_LANG)
}

/// Convert snake_case to camelCase (for napi js_name default).
fn to_camel(s: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in s.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Sanitise a Python parameter name (keyword collisions, digit prefixes).
fn sanitise_pyi_var(name: &str) -> String {
    // Python keywords that would collide
    const KW: &[&str] = &[
        "from", "import", "class", "def", "return", "pass", "lambda", "global", "nonlocal", "None",
        "True", "False", "and", "or", "not", "if", "else", "elif", "while", "for", "in", "is",
        "as", "try", "except", "finally", "with", "yield", "async", "await",
    ];
    let s = name.trim().trim_start_matches('_');
    if KW.contains(&s) {
        format!("{s}_")
    } else if s.is_empty() || s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("p{s}")
    } else {
        s.to_string()
    }
}

fn cmd_pyi(check: bool) {
    let root = workspace_root();
    let py_src = fs::read_to_string(root.join("bindings/python/src/lib.rs"))
        .expect("cannot read bindings/python/src/lib.rs");
    let out_path = root.join("bindings/python/python/celestial_py/celestial_py.pyi");

    let mut out = String::from(
        "# Auto-generated by `cargo xtask pyi` — do not edit.\n\
         # Regenerate: cargo xtask pyi\n\n",
    );

    let mut seen: BTreeSet<String> = BTreeSet::new();
    for entry in scan_decorated_fns(&py_src, "pyfunction") {
        if !seen.insert(entry.name.clone()) {
            continue;
        }
        let params: Vec<String> = entry
            .params
            .iter()
            .map(|(typ, nam)| format!("{}: {}", sanitise_pyi_var(nam), rust_type_to_pyi(typ)))
            .collect();
        let ret = rust_type_to_pyi(&entry.ret);
        out.push_str(&format!(
            "def {}({}) -> {}: ...\n",
            entry.name,
            params.join(", "),
            ret
        ));
    }

    write_or_check(&out_path, &out, check, "stubs", "pyi", seen.len());
}

// ─── helpers for napi const/struct scanning ──────────────────────────────────

/// A `#[napi] pub const NAME: TYPE = ...;` declaration.
struct NapiConst {
    name: String,
    ty: String,
}

/// Scan a Rust source file for all `#[napi] pub const X: T = ...;` items.
/// Parse a single `pub const NAME: TYPE = ...;` declaration line.
fn parse_napi_const_decl(decl: &str) -> Option<NapiConst> {
    let rest = decl.trim().strip_prefix("pub const ")?;
    let colon = rest.find(':')?;
    let name = rest[..colon].trim().to_string();
    let eq = rest[colon..].find('=')?;
    let ty = rest[colon + 1..colon + eq].trim().to_string();
    Some(NapiConst { name, ty })
}

fn scan_napi_consts(src: &str) -> Vec<NapiConst> {
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let n = lines.len();
    let mut i = 0;
    while i < n {
        let line = lines[i].trim_start();
        if !(line.starts_with("#[napi]") || line.starts_with("#[napi(")) {
            i += 1;
            continue;
        }
        let j = skip_attrs(&lines, i + 1);
        if j < n {
            if let Some(c) = parse_napi_const_decl(lines[j]) {
                out.push(c);
            }
        }
        i = j + 1;
    }
    out
}

/// A single field of a `#[napi(object)]` struct.
struct NapiStructField {
    name: String,
    ty: String,
}

/// A `#[napi(object)] pub struct X { ... }` declaration.
struct NapiStruct {
    name: String,
    fields: Vec<NapiStructField>,
}

/// Skip past `#[..]` attribute lines starting at `j`. Returns first non-attribute line index, or `n`.
fn skip_attrs(lines: &[&str], mut j: usize) -> usize {
    let n = lines.len();
    while j < n && lines[j].trim_start().starts_with("#[") {
        j += 1;
    }
    j
}

fn parse_struct_name(decl: &str) -> String {
    decl.strip_prefix("pub struct ")
        .and_then(|s| s.split([' ', '{']).next())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// Find the line index immediately after the line containing the opening brace.
fn find_field_start(lines: &[&str], decl: &str, j: usize) -> usize {
    if decl.contains('{') {
        return j + 1;
    }
    let n = lines.len();
    let mut k = j + 1;
    while k < n && !lines[k].contains('{') {
        k += 1;
    }
    k + 1
}

fn parse_struct_field(line: &str) -> Option<NapiStructField> {
    let rest = line.strip_prefix("pub ")?.trim_end_matches(',').trim();
    let colon = rest.find(':')?;
    let name = rest[..colon].trim().to_string();
    let ty = rest[colon + 1..].trim().to_string();
    if name.is_empty() || ty.is_empty() {
        return None;
    }
    Some(NapiStructField { name, ty })
}

/// Parse field list until the closing `}`. Returns (fields, line index of `}`).
fn parse_struct_fields(lines: &[&str], start: usize) -> (Vec<NapiStructField>, usize) {
    let n = lines.len();
    let mut fields = Vec::new();
    let mut k = start;
    while k < n {
        let l = lines[k].trim();
        if l.starts_with('}') {
            break;
        }
        if let Some(field) = parse_struct_field(l) {
            fields.push(field);
        }
        k += 1;
    }
    (fields, k)
}

/// Scan a Rust source file for all `#[napi(object)] pub struct X { ... }` items.
fn scan_napi_structs(src: &str) -> Vec<NapiStruct> {
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let n = lines.len();
    let mut i = 0;
    while i < n {
        if !lines[i].trim_start().starts_with("#[napi(object)]") {
            i += 1;
            continue;
        }
        let j = skip_attrs(&lines, i + 1);
        if j >= n {
            i += 1;
            continue;
        }
        let decl = lines[j].trim();
        let name = parse_struct_name(decl);
        if name.is_empty() {
            i = j + 1;
            continue;
        }
        let field_start = find_field_start(&lines, decl, j);
        let (fields, end) = parse_struct_fields(&lines, field_start);
        out.push(NapiStruct { name, fields });
        i = end + 1;
    }
    out
}

/// Convert a Rust type to TypeScript, using `known_structs` as first-class type names.
fn rust_type_to_ts_known(rust: &str, known_structs: &BTreeSet<String>) -> String {
    let t = rust
        .trim()
        .trim_start_matches("crate::")
        .trim_start_matches("celestial_core::");

    // Unwrap Result<T> / napi::Result<T>
    let inner = DTS_LANG
        .result_prefixes
        .iter()
        .find_map(|p| t.strip_prefix(p).and_then(|s| s.strip_suffix('>')))
        .map_or(t, str::trim);

    // Option<T> → T | null (in return position — parameters are handled specially)
    if let Some(inner_opt) = inner
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        return format!("{} | null", rust_type_to_ts_known(inner_opt, known_structs));
    }

    // Known struct → emit raw type name
    if known_structs.contains(inner) {
        return inner.to_string();
    }

    match inner {
        "f64" | "f32" | "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64"
        | "usize" => "number".to_string(),
        "bool" => "boolean".to_string(),
        "String" | "&str" | "&'static str" | "str" => "string".to_string(),
        "()" | "void" => "void".to_string(),
        t if t.starts_with("Vec<") => {
            format!(
                "Array<{}>",
                rust_type_to_ts_known(&t[4..t.len() - 1], known_structs)
            )
        }
        t if t.starts_with("HashMap<") || t.starts_with("BTreeMap<") => {
            "Record<string, number>".to_string()
        }
        t if t.starts_with('(') && t.ends_with(')') => {
            let parts: Vec<String> = split_params(&t[1..t.len() - 1])
                .iter()
                .map(|p| rust_type_to_ts_known(p, known_structs))
                .collect();
            format!("[{}]", parts.join(", "))
        }
        _ => "unknown".to_string(),
    }
}

fn emit_dts_struct_field(out: &mut String, f: &NapiStructField, known: &BTreeSet<String>) {
    let (ts_ty, optional) = if let Some(inner) =
        f.ty.strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
    {
        (rust_type_to_ts_known(inner, known), true)
    } else {
        (rust_type_to_ts_known(&f.ty, known), false)
    };
    let cam = to_camel(&f.name);
    if optional {
        out.push_str(&format!("  {cam}?: {ts_ty} | null;\n"));
    } else {
        out.push_str(&format!("  {cam}: {ts_ty};\n"));
    }
}

fn emit_dts_structs(out: &mut String, structs: &[NapiStruct], known: &BTreeSet<String>) {
    for st in structs {
        out.push_str(&format!("export interface {} {{\n", st.name));
        for f in &st.fields {
            emit_dts_struct_field(out, f, known);
        }
        out.push_str("}\n\n");
    }
}

fn dts_param_string(typ: &str, nam: &str, known: &BTreeSet<String>) -> String {
    let cam = to_camel(nam);
    if let Some(inner) = typ
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        let inner_ts = rust_type_to_ts_known(inner, known);
        format!("{cam}?: {inner_ts} | null")
    } else {
        format!("{cam}: {}", rust_type_to_ts_known(typ, known))
    }
}

fn emit_dts_functions(out: &mut String, src: &str, known: &BTreeSet<String>) -> usize {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for d in scan_decorated_fns_full(src, "napi") {
        let js_name = extract_js_name(&d.attr_line).unwrap_or_else(|| to_camel(&d.entry.name));
        if !seen.insert(js_name.clone()) {
            continue;
        }
        let params: Vec<String> = d
            .entry
            .params
            .iter()
            .map(|(typ, nam)| dts_param_string(typ, nam, known))
            .collect();
        let ret = rust_type_to_ts_known(&d.entry.ret, known);
        out.push_str(&format!(
            "export declare function {}({}): {};\n",
            js_name,
            params.join(", "),
            ret
        ));
    }
    seen.len()
}

fn cmd_dts(check: bool) {
    let root = workspace_root();
    let js_src = fs::read_to_string(root.join("bindings/js/src/lib.rs"))
        .expect("cannot read bindings/js/src/lib.rs");
    let out_path = root.join("bindings/js/index.d.ts");

    let mut out = String::from(
        "// Auto-generated by `cargo xtask dts` — do not edit.\n\
         // Regenerate: cargo xtask dts\n\n",
    );

    let structs = scan_napi_structs(&js_src);
    let known_structs: BTreeSet<String> = structs.iter().map(|s| s.name.clone()).collect();

    emit_dts_structs(&mut out, &structs, &known_structs);

    let consts = scan_napi_consts(&js_src);
    for c in &consts {
        let ts_ty = rust_type_to_ts_known(&c.ty, &known_structs);
        out.push_str(&format!("export declare const {}: {};\n", c.name, ts_ty));
    }
    if !consts.is_empty() {
        out.push('\n');
    }

    let fn_count = emit_dts_functions(&mut out, &js_src, &known_structs);

    let total = structs.len() + consts.len() + fn_count;
    write_or_check(&out_path, &out, check, "declarations", "dts", total);
}

// ─── coverage command (A: core→binding coverage · B: arity parity) ────────────

/// Path (relative to workspace root) of the checked-in "core fns intentionally
/// not bound" allow-list. Each future core public fn must be either bound in all
/// three languages or listed here with a reason — otherwise `coverage` fails.
const CORE_UNBOUND_FILE: &str = "xtask/core_unbound_allow.txt";

/// Add `tok` to `set` if it looks like a free function name (snake_case,
/// lowercase first char — excludes CamelCase types and UPPER_CASE consts).
fn push_core_fn(set: &mut BTreeSet<String>, tok: &str) {
    // `name as alias` re-exports export the alias; take the right-hand side.
    let t = tok
        .trim()
        .trim_end_matches('}')
        .rsplit(" as ")
        .next()
        .unwrap_or("")
        .trim();
    if t.is_empty() || !t.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return;
    }
    if t.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        set.insert(t.to_string());
    }
}

/// Parse the flat public function surface of core from its `pub use` re-exports
/// in `core/src/lib.rs`. This is the source of truth for what bindings *could*
/// expose; anything here that no binding wraps is a coverage gap.
fn parse_core_flat_fns(lib_src: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut rest = lib_src;
    while let Some(pos) = rest.find("pub use ") {
        rest = &rest[pos + "pub use ".len()..];
        let Some(semi) = rest.find(';') else { break };
        let decl = &rest[..semi];
        rest = &rest[semi + 1..];
        if let Some(brace) = decl.find('{') {
            let close = decl.rfind('}').unwrap_or(decl.len());
            for tok in decl[brace + 1..close].split(',') {
                push_core_fn(&mut names, tok);
            }
        } else if let Some(cc) = decl.rfind("::") {
            push_core_fn(&mut names, &decl[cc + 2..]);
        }
    }
    names
}

/// The set of core fn names reachable from *any* binding: direct wrappers plus
/// the canonical targets that legacy aliases delegate to.
fn bound_core_names(bindings: &[Binding]) -> BTreeSet<String> {
    let mut s: BTreeSet<String> = bindings.iter().flat_map(|b| b.fns.iter().cloned()).collect();
    for (_alias, canonical) in legacy_aliases() {
        s.insert(canonical);
    }
    s
}

fn read_allow_list(root: &Path) -> BTreeSet<String> {
    fs::read_to_string(root.join(CORE_UNBOUND_FILE))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

/// Checked-in list of fns whose cross-binding arity difference is known and
/// accepted (a language idiom or a tracked gap). First whitespace token per
/// non-comment line is the fn name; the rest is a free-text reason.
const ARITY_ALLOW_FILE: &str = "xtask/arity_allow.txt";

fn read_arity_allow(root: &Path) -> BTreeSet<String> {
    fs::read_to_string(root.join(ARITY_ALLOW_FILE))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_whitespace().next())
        .map(String::from)
        .collect()
}

/// Checked-in list of constants whose cross-binding absence is intentional.
const CONST_ALLOW_FILE: &str = "xtask/const_allow.txt";

fn read_const_allow(root: &Path) -> BTreeSet<String> {
    fs::read_to_string(root.join(CONST_ALLOW_FILE))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_whitespace().next())
        .map(String::from)
        .collect()
}

/// Names of the constants Python registers via `m.add("NAME", …)`.
fn python_const_names(src: &str) -> BTreeSet<String> {
    src.split("m.add(\"")
        .skip(1)
        .filter_map(|c| {
            let name: String = c
                .chars()
                .take_while(|ch| ch.is_alphanumeric() || *ch == '_')
                .collect();
            (name.chars().next().is_some_and(|c| c.is_ascii_uppercase())).then_some(name)
        })
        .collect()
}

/// Exported constant name set for each binding (JS `#[napi]`, PHP `#[php_const]`,
/// Python `m.add`). Source of truth for GATE-1 constant parity.
fn binding_const_names(root: &Path) -> Vec<(&'static str, BTreeSet<String>)> {
    let read = |rel: &str| fs::read_to_string(root.join(rel)).unwrap_or_default();
    let js = scan_napi_consts(&read("bindings/js/src/lib.rs"))
        .into_iter()
        .map(|c| c.name)
        .collect();
    let php = parse_php_consts(&read("bindings/php/src/lib.rs"))
        .into_iter()
        .map(|(n, _, _)| n)
        .collect();
    let py = python_const_names(&read("bindings/python/src/lib.rs"));
    vec![("JS", js), ("PHP", php), ("Python", py)]
}

/// Constants exported by some but not all bindings (and not allow-listed).
fn const_parity_gaps(
    sets: &[(&str, BTreeSet<String>)],
    allow: &BTreeSet<String>,
) -> Vec<(String, Vec<String>)> {
    let union: BTreeSet<&String> = sets.iter().flat_map(|(_, s)| s.iter()).collect();
    let mut out = Vec::new();
    for name in union {
        if allow.contains(name) {
            continue;
        }
        let missing: Vec<String> = sets
            .iter()
            .filter(|(_, s)| !s.contains(name))
            .map(|(b, _)| (*b).to_string())
            .collect();
        if !missing.is_empty() {
            out.push((name.clone(), missing));
        }
    }
    out
}

/// A param's shape category with the napi-optional idiom removed: JS wraps
/// trailing args in `Option<>` where Python/PHP take them required, which is a
/// binding convention, not a semantic difference — so `opt<T>` compares as `T`.
fn normalized_param_cat(ty: &str) -> String {
    let c = shape_category(ty);
    c.strip_prefix("opt<")
        .and_then(|s| s.strip_suffix('>'))
        .map_or(c.clone(), str::to_string)
}

/// Per-binding *ordered* parameter signature for every decorated fn: each param
/// reduced to a cross-language shape category so `i32`↔`i64` don't count as
/// drift but an argument swap or added/dropped param does (GATE-2). self/py are
/// excluded by the signature parser. Value is a "float,int,array"-style key.
fn binding_signatures(root: &Path) -> Vec<(String, BTreeMap<String, String>)> {
    let specs = [
        ("Python", "bindings/python/src/lib.rs", "pyfunction"),
        ("JS", "bindings/js/src/lib.rs", "napi"),
        ("PHP", "bindings/php/src/lib.rs", "php_function"),
    ];
    specs
        .iter()
        .map(|(name, rel, deco)| {
            let src = fs::read_to_string(root.join(rel)).unwrap_or_default();
            let map = scan_decorated_fns(&src, deco)
                .into_iter()
                .map(|e| {
                    let sig = e
                        .params
                        .iter()
                        .map(|(ty, _)| normalized_param_cat(ty))
                        .collect::<Vec<_>>()
                        .join(",");
                    (e.name, sig)
                })
                .collect();
            ((*name).to_string(), map)
        })
        .collect()
}

/// Find fns whose ordered parameter signature disagrees between two or more
/// bindings. Returns (fn_name, ["Python: float,int", "JS: int,float", ...]).
fn arity_mismatches(root: &Path) -> Vec<(String, Vec<String>)> {
    let sigs = binding_signatures(root);
    let all_fns: BTreeSet<&String> = sigs.iter().flat_map(|(_, m)| m.keys()).collect();
    let mut out = Vec::new();
    for fname in all_fns {
        let present: Vec<(&str, &String)> = sigs
            .iter()
            .filter_map(|(lang, m)| m.get(fname).map(|s| (lang.as_str(), s)))
            .collect();
        let distinct: BTreeSet<&String> = present.iter().map(|(_, s)| *s).collect();
        if present.len() >= 2 && distinct.len() > 1 {
            out.push((
                fname.clone(),
                present
                    .iter()
                    .map(|(l, s)| format!("{l}: ({s})"))
                    .collect(),
            ));
        }
    }
    out
}

fn regenerate_allow_list(root: &Path, unbound: &BTreeSet<String>) {
    let mut body = String::from(
        "# Core public fns intentionally NOT exposed in the language bindings.\n\
         # Source of truth for `cargo xtask coverage`. Regenerate the baseline with\n\
         # `cargo xtask coverage --write-allow`, but prefer BINDING a new core fn over\n\
         # adding it here. Every entry is a deliberate \"Rust-only\" decision (low-level\n\
         # helpers, path/config setters, unwrapped library-only APIs — see TODO WIRE-1).\n\
         # One fn name per line.\n\n",
    );
    for f in unbound {
        body.push_str(f);
        body.push('\n');
    }
    fs::write(root.join(CORE_UNBOUND_FILE), body).expect("cannot write allow-list");
    println!(
        "✓ wrote {} intentionally-unbound core fns to {CORE_UNBOUND_FILE}",
        unbound.len()
    );
}

fn cmd_coverage(write_allow: bool) {
    let root = workspace_root();
    let lib = fs::read_to_string(root.join("core/src/lib.rs")).expect("cannot read core/src/lib.rs");
    let core = parse_core_flat_fns(&lib);
    let bindings = load_bindings(&root);
    let bound = bound_core_names(&bindings);

    let unbound: BTreeSet<String> = core.iter().filter(|f| !bound.contains(*f)).cloned().collect();

    if write_allow {
        regenerate_allow_list(&root, &unbound);
        return;
    }

    let allow = read_allow_list(&root);
    let gap: Vec<&String> = unbound.iter().filter(|f| !allow.contains(*f)).collect();
    let stale: Vec<&String> = allow
        .iter()
        .filter(|a| bound.contains(*a) || !core.contains(*a))
        .collect();
    let arity_allow = read_arity_allow(&root);
    let arity: Vec<(String, Vec<String>)> = arity_mismatches(&root)
        .into_iter()
        .filter(|(name, _)| !arity_allow.contains(name))
        .collect();
    let const_sets = binding_const_names(&root);
    let const_gap = const_parity_gaps(&const_sets, &read_const_allow(&root));

    println!("celestial binding coverage check");
    println!("================================");
    println!("  core flat public fns : {}", core.len());
    println!("  bound (any binding)  : {}", bound.intersection(&core).count());
    println!("  intentionally unbound: {}", allow.len());
    for (b, s) in &const_sets {
        println!("  constants ({b}) : {}", s.len());
    }

    report_coverage(&gap, &stale, &arity, &const_gap);
}

/// Print constant-parity gaps (GATE-1). Returns true if any exist.
fn report_const_gap(const_gap: &[(String, Vec<String>)]) -> bool {
    if const_gap.is_empty() {
        return false;
    }
    println!(
        "\n✗ {} constant(s) not exported by every binding:\n",
        const_gap.len()
    );
    for (name, missing) in const_gap {
        println!("    {name}: missing from {}", missing.join(", "));
    }
    println!("\n  → add the missing `#[napi]`/`#[php_const]`/`m.add(...)` const, OR");
    println!("    list it in {CONST_ALLOW_FILE} if the omission is intentional.");
    true
}

fn report_coverage(
    gap: &[&String],
    stale: &[&String],
    arity: &[(String, Vec<String>)],
    const_gap: &[(String, Vec<String>)],
) {
    let mut failed = report_const_gap(const_gap);
    if !gap.is_empty() {
        failed = true;
        println!(
            "\n✗ {} core fn(s) exposed by no binding and not in the allow-list:\n",
            gap.len()
        );
        for f in gap {
            println!("    {f}");
        }
        println!("\n  → bind it in Python + JS + PHP (see `cargo xtask codegen`), OR");
        println!("    add it to {CORE_UNBOUND_FILE} with a reason if it is Rust-only.");
    }
    if !stale.is_empty() {
        failed = true;
        println!(
            "\n✗ {} allow-list entr(y/ies) are now bound or gone from core — remove them:\n",
            stale.len()
        );
        for f in stale {
            println!("    {f}");
        }
    }
    if !arity.is_empty() {
        failed = true;
        println!(
            "\n✗ {} fn(s) differ in parameter signature (count/order/type) across bindings:\n",
            arity.len()
        );
        for (name, per) in arity {
            println!("    {name}: {}", per.join("  |  "));
        }
    }
    if failed {
        std::process::exit(1);
    }
    println!("\n✓ every core public fn is bound or explicitly allow-listed; arities agree.");
}

// ─── shapes command (C: return-shape contract snapshot) ───────────────────────

/// Collapse a Rust return type into a coarse cross-language shape category.
/// Coarse on purpose: it must be stable when only names change, but flip when
/// the *kind* of value changes (scalar↔tuple↔array↔object) — the DOC-10 class.
fn shape_category(ret: &str) -> String {
    let t = ret
        .trim()
        .trim_start_matches("crate::")
        .trim_start_matches("celestial_core::");
    let inner = ["PyResult<", "napi::Result<", "PhpResult<", "Result<"]
        .iter()
        .find_map(|p| t.strip_prefix(p).and_then(|s| s.strip_suffix('>')))
        .map_or(t, str::trim);
    if let Some(o) = inner.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
        return format!("opt<{}>", shape_category(o));
    }
    match inner {
        "" | "()" | "void" => "void".to_string(),
        "f64" | "f32" => "float".to_string(),
        "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
            "int".to_string()
        }
        "bool" => "bool".to_string(),
        "String" | "&str" | "&'static str" | "str" => "string".to_string(),
        s if s.starts_with("Vec<") => "array".to_string(),
        s if s.starts_with("HashMap") || s.starts_with("BTreeMap") => "map".to_string(),
        s if s.starts_with('[') => "array".to_string(),
        s if s.starts_with('(') && s.ends_with(')') => {
            format!("tuple{}", split_params(&s[1..s.len() - 1]).len())
        }
        _ => "object".to_string(),
    }
}

/// Build `fn_name -> (py_cat, js_cat, php_cat)` for every decorated fn.
fn collect_shapes(root: &Path) -> BTreeMap<String, [String; 3]> {
    let specs = [
        (0usize, "bindings/python/src/lib.rs", "pyfunction"),
        (1, "bindings/js/src/lib.rs", "napi"),
        (2, "bindings/php/src/lib.rs", "php_function"),
    ];
    let mut map: BTreeMap<String, [String; 3]> = BTreeMap::new();
    for (idx, rel, deco) in specs {
        let src = fs::read_to_string(root.join(rel)).unwrap_or_default();
        for e in scan_decorated_fns(&src, deco) {
            map.entry(e.name)
                .or_insert_with(|| ["-".into(), "-".into(), "-".into()])[idx] =
                shape_category(&e.ret);
        }
    }
    map
}

fn render_shapes(map: &BTreeMap<String, [String; 3]>) -> String {
    let mut out = String::from(
        "# Auto-generated by `cargo xtask shapes` — do not edit.\n\
         # Return-shape contract per binding (py | js | php). A diff here means a\n\
         # binding's return KIND changed (scalar/tuple/array/object) — update the\n\
         # matching docs/generated file and language guide in the same commit.\n\n",
    );
    let width = map.keys().map(String::len).max().unwrap_or(0);
    for (name, [py, js, php]) in map {
        out.push_str(&format!(
            "{name:<width$}  py={py:<10} js={js:<10} php={php}\n"
        ));
    }
    out
}

fn cmd_shapes(check: bool) {
    let root = workspace_root();
    let map = collect_shapes(&root);
    let out = render_shapes(&map);
    let path = root.join("xtask/binding_shapes.txt");
    write_or_check(&path, &out, check, "shape entries", "shapes", map.len());
}

// ─── apidoc command (D: generated binding API reference) ──────────────────────

/// (section title, binding source path, decorator, Rust→lang type mapper).
type ApidocSection = (&'static str, &'static str, &'static str, fn(&str) -> String);

/// `rust_type_to_php` as an owned-String fn pointer (matches the pyi/ts mappers).
fn php_ret_type(t: &str) -> String {
    rust_type_to_php(t, true).to_string()
}

/// Render one language's fn signature line using a target-language type mapper.
fn apidoc_sig(entry: &PhpFnEntry, ty: impl Fn(&str) -> String) -> String {
    let params: Vec<String> = entry
        .params
        .iter()
        .map(|(t, n)| format!("{n}: {}", ty(t)))
        .collect();
    format!("{}({}) -> {}", entry.name, params.join(", "), ty(&entry.ret))
}

fn apidoc_constants_table(root: &Path) -> String {
    let consts_src = fs::read_to_string(root.join("core/src/constants.rs")).unwrap_or_default();
    let vals = parse_core_constants(&consts_src);
    let js = fs::read_to_string(root.join("bindings/js/src/lib.rs")).unwrap_or_default();
    let mut out = String::from("## Constants (value from `core/src/constants.rs`)\n\n| Constant | Value |\n|---|---|\n");
    for c in scan_napi_consts(&js) {
        let v = vals.get(&c.name).cloned().unwrap_or_else(|| "?".into());
        out.push_str(&format!("| `{}` | {} |\n", c.name, v));
    }
    out
}

/// Render `## <title>` section listing every decorated fn's signature.
/// Returns (rendered_markdown, fn_count).
fn apidoc_fn_section(
    root: &Path,
    title: &str,
    rel: &str,
    deco: &str,
    ty: fn(&str) -> String,
) -> (String, usize) {
    let src = fs::read_to_string(root.join(rel)).unwrap_or_default();
    let mut fns = scan_decorated_fns(&src, deco);
    fns.sort_by(|a, b| a.name.cmp(&b.name));
    let lines: String = fns
        .iter()
        .map(|e| format!("{}\n", apidoc_sig(e, ty)))
        .collect();
    let n = fns.len();
    (format!("\n## {title} — {n} functions\n\n```\n{lines}```\n"), n)
}

fn cmd_apidoc(check: bool) {
    let root = workspace_root();
    let mut out = String::from(
        "<!-- Auto-generated by `cargo xtask apidoc` — do not edit. -->\n\
         <!-- Regenerate: cargo xtask apidoc. This file is the source of truth for\n\
         binding constant values and signatures; the hand-written language guides\n\
         link here rather than restating them (prevents DOC-9/DOC-10 drift). -->\n\n\
         # Binding API — generated reference\n\n",
    );
    out.push_str(&apidoc_constants_table(&root));

    let sections: [ApidocSection; 3] = [
        ("Python", "bindings/python/src/lib.rs", "pyfunction", rust_type_to_pyi),
        ("JavaScript / TypeScript", "bindings/js/src/lib.rs", "napi", rust_type_to_ts),
        ("PHP", "bindings/php/src/lib.rs", "php_function", php_ret_type),
    ];
    let mut total = 0;
    for (title, rel, deco, ty) in sections {
        let (section, n) = apidoc_fn_section(&root, title, rel, deco, ty);
        out.push_str(&section);
        total += n;
    }

    let path = root.join("docs/generated/binding_api.md");
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).ok();
    }
    write_or_check(&path, &out, check, "signatures", "apidoc", total);
}

// ─── unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── rust_type_to_pyi ──────────────────────────────────────────────────────

    #[test]
    fn pyi_primitives() {
        assert_eq!(rust_type_to_pyi("f64"), "float");
        assert_eq!(rust_type_to_pyi("i32"), "int");
        assert_eq!(rust_type_to_pyi("u64"), "int");
        assert_eq!(rust_type_to_pyi("bool"), "bool");
        assert_eq!(rust_type_to_pyi("String"), "str");
        assert_eq!(rust_type_to_pyi("&str"), "str");
        assert_eq!(rust_type_to_pyi("&'static str"), "str");
        assert_eq!(rust_type_to_pyi("()"), "None");
    }

    #[test]
    fn pyi_option() {
        assert_eq!(rust_type_to_pyi("Option<f64>"), "float | None");
        assert_eq!(rust_type_to_pyi("Option<String>"), "str | None");
    }

    #[test]
    fn pyi_vec() {
        assert_eq!(rust_type_to_pyi("Vec<f64>"), "list[float]");
        assert_eq!(rust_type_to_pyi("Vec<String>"), "list[str]");
        assert_eq!(rust_type_to_pyi("Vec<Vec<i32>>"), "list[list[int]]");
    }

    #[test]
    fn pyi_tuple() {
        assert_eq!(rust_type_to_pyi("(i32, String)"), "tuple[int, str]");
        assert_eq!(
            rust_type_to_pyi("(u32, u32, u32, u32, u32)"),
            "tuple[int, int, int, int, int]"
        );
    }

    #[test]
    fn pyi_pyresult() {
        assert_eq!(rust_type_to_pyi("PyResult<f64>"), "float");
        assert_eq!(rust_type_to_pyi("Result<String>"), "str");
    }

    #[test]
    fn pyi_map() {
        assert_eq!(
            rust_type_to_pyi("HashMap<String, f64>"),
            "dict[str, object]"
        );
    }

    #[test]
    fn pyi_unknown_falls_back() {
        assert_eq!(rust_type_to_pyi("PlanetPos"), "object");
    }

    // ── rust_type_to_ts ───────────────────────────────────────────────────────

    #[test]
    fn ts_primitives() {
        assert_eq!(rust_type_to_ts("f64"), "number");
        assert_eq!(rust_type_to_ts("i32"), "number");
        assert_eq!(rust_type_to_ts("bool"), "boolean");
        assert_eq!(rust_type_to_ts("String"), "string");
        assert_eq!(rust_type_to_ts("()"), "void");
    }

    #[test]
    fn ts_option_is_null() {
        assert_eq!(rust_type_to_ts("Option<f64>"), "number | null");
        assert_eq!(rust_type_to_ts("Option<Vec<i32>>"), "Array<number> | null");
    }

    #[test]
    fn ts_vec() {
        assert_eq!(rust_type_to_ts("Vec<f64>"), "Array<number>");
    }

    #[test]
    fn ts_tuple() {
        assert_eq!(
            rust_type_to_ts("(i32, String, bool)"),
            "[number, string, boolean]"
        );
    }

    #[test]
    fn ts_napi_result() {
        assert_eq!(rust_type_to_ts("napi::Result<f64>"), "number");
    }

    #[test]
    fn ts_unknown_falls_back() {
        assert_eq!(rust_type_to_ts("SomeUnknownStruct"), "unknown");
    }

    // ── to_camel ──────────────────────────────────────────────────────────────

    #[test]
    fn camel_case_conversion() {
        assert_eq!(to_camel("house_name"), "houseName");
        assert_eq!(to_camel("jd_to_coptic"), "jdToCoptic");
        assert_eq!(to_camel("x"), "x");
        assert_eq!(to_camel(""), "");
    }

    // ── extract_js_name ───────────────────────────────────────────────────────

    #[test]
    fn js_name_override_parsed() {
        assert_eq!(
            extract_js_name(r#"#[napi(js_name = "houseName")]"#),
            Some("houseName".to_string())
        );
        assert_eq!(extract_js_name("#[napi]"), None);
        assert_eq!(extract_js_name("#[napi(other = \"x\")]"), None);
    }

    // ── sanitise_pyi_var ──────────────────────────────────────────────────────

    #[test]
    fn pyi_var_keyword_collision() {
        assert_eq!(sanitise_pyi_var("from"), "from_");
        assert_eq!(sanitise_pyi_var("class"), "class_");
        assert_eq!(sanitise_pyi_var("year"), "year");
    }

    #[test]
    fn pyi_var_digit_prefix() {
        assert_eq!(sanitise_pyi_var("2nd"), "p2nd");
    }

    // ── parse_fn_sig smoke test ───────────────────────────────────────────────

    #[test]
    fn parse_simple_fn_sig() {
        let sig = "fn example(x: f64, y: i32) -> bool {";
        let entry = parse_fn_sig(sig).expect("should parse");
        assert_eq!(entry.name, "example");
        assert_eq!(entry.params.len(), 2);
        assert_eq!(entry.params[0].0, "f64");
        assert_eq!(entry.params[0].1, "x");
        assert_eq!(entry.ret, "bool");
    }

    #[test]
    fn parse_fn_with_py_context_skipped() {
        let sig = "fn example(py: Python<'_>, x: f64) -> PyObject {";
        let entry = parse_fn_sig(sig).expect("should parse");
        assert_eq!(entry.params.len(), 1, "py param should be skipped");
        assert_eq!(entry.params[0].1, "x");
    }

    // ── split_params ──────────────────────────────────────────────────────────

    #[test]
    fn split_params_respects_nesting() {
        let parts = split_params("a: i32, b: Vec<(f64, i32)>, c: HashMap<String, Vec<i32>>");
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[1], "b: Vec<(f64, i32)>");
        assert_eq!(parts[2], "c: HashMap<String, Vec<i32>>");
    }

    // ── napi const scanning ───────────────────────────────────────────────────

    #[test]
    fn scan_napi_consts_basic() {
        let src = r#"
#[napi]
pub const SUN: i32 = 0;

#[napi]
pub const GREG_CAL: i32 = celestial::GREG_CAL;

#[napi]
pub fn some_fn() -> i32 { 0 }
"#;
        let consts = scan_napi_consts(src);
        assert_eq!(consts.len(), 2);
        assert_eq!(consts[0].name, "SUN");
        assert_eq!(consts[0].ty, "i32");
        assert_eq!(consts[1].name, "GREG_CAL");
    }

    // ── napi struct scanning ──────────────────────────────────────────────────

    #[test]
    fn scan_napi_structs_basic() {
        let src = r#"
#[napi(object)]
pub struct PlanetPos {
    pub lon: f64,
    pub lat: f64,
    pub dist: f64,
}

#[napi(object)]
pub struct RiseTrans {
    pub rise: Option<f64>,
    pub set: Option<f64>,
}
"#;
        let structs = scan_napi_structs(src);
        assert_eq!(structs.len(), 2);
        assert_eq!(structs[0].name, "PlanetPos");
        assert_eq!(structs[0].fields.len(), 3);
        assert_eq!(structs[0].fields[0].name, "lon");
        assert_eq!(structs[0].fields[0].ty, "f64");
        assert_eq!(structs[1].name, "RiseTrans");
        assert_eq!(structs[1].fields[0].ty, "Option<f64>");
    }

    // ── rust_type_to_ts_known recognizes struct names ────────────────────────

    #[test]
    fn ts_known_struct_preserved() {
        let mut structs = BTreeSet::new();
        structs.insert("PlanetPos".to_string());
        assert_eq!(rust_type_to_ts_known("PlanetPos", &structs), "PlanetPos");
        assert_eq!(
            rust_type_to_ts_known("Vec<PlanetPos>", &structs),
            "Array<PlanetPos>"
        );
        assert_eq!(
            rust_type_to_ts_known("Option<PlanetPos>", &structs),
            "PlanetPos | null"
        );
    }

    #[test]
    fn ts_known_unknown_struct_falls_back() {
        let structs = BTreeSet::new();
        assert_eq!(rust_type_to_ts_known("SomeStruct", &structs), "unknown");
    }

    /// Regression: a `#[napi(object)]` struct followed by an
    /// `impl From<…> for It { fn from(..) }` must NOT make the fn
    /// scanner emit a bogus `from` export (the scanner used to walk
    /// past the struct into the impl). A genuine `#[napi] pub fn`
    /// nearby must still be captured.
    // ── coverage: core flat-fn parsing (A) ───────────────────────────────────

    #[test]
    fn core_flat_fns_keeps_fns_drops_types_and_consts() {
        let src = r#"
pub use position::{calc_ut, calc, PlanetPos};
pub use constants::{SUN, MOON, FLG_SPEED};
pub use time::julday;
pub use geo::{tz_abbr_find, TzAbbr, TZ_TABLE};
"#;
        let fns = parse_core_flat_fns(src);
        assert!(fns.contains("calc_ut"));
        assert!(fns.contains("calc"));
        assert!(fns.contains("julday"));
        assert!(fns.contains("tz_abbr_find"));
        // Types (CamelCase) and consts (UPPER) excluded.
        assert!(!fns.contains("PlanetPos"));
        assert!(!fns.contains("SUN"));
        assert!(!fns.contains("FLG_SPEED"));
        assert!(!fns.contains("TzAbbr"));
        assert!(!fns.contains("TZ_TABLE"));
    }

    #[test]
    fn core_fn_alias_reexport_takes_alias_name() {
        let mut set = BTreeSet::new();
        push_core_fn(&mut set, "old_name as new_name");
        assert!(set.contains("new_name"));
        assert!(!set.contains("old_name"));
    }

    // ── ordered signature parity (GATE-2) ─────────────────────────────────────

    #[test]
    fn normalized_param_strips_napi_optional() {
        // JS trailing Option<> is a binding idiom, not a semantic difference.
        assert_eq!(normalized_param_cat("Option<i32>"), normalized_param_cat("i32"));
        assert_eq!(normalized_param_cat("Option<f64>"), "float");
        assert_eq!(normalized_param_cat("i64"), "int");
        // An argument swap still shows as a different sequence.
        assert_ne!(
            ["float", "int"].join(","),
            ["int", "float"].join(",")
        );
    }

    // ── constant parity (GATE-1) ──────────────────────────────────────────────

    #[test]
    fn python_const_names_extracted() {
        let src = "m.add(\"SUN\", 0)?;\n    m.add(\"lower\", 1)?;\n    m.add(\"FLG_XYZ\", 4096)?;";
        let c = python_const_names(src);
        assert!(c.contains("SUN"));
        assert!(c.contains("FLG_XYZ"));
        assert!(!c.contains("lower"));
    }

    #[test]
    fn const_gap_flags_missing_and_respects_allow() {
        let sets = vec![
            ("JS", ["A", "B"].iter().map(|s| s.to_string()).collect()),
            ("PHP", ["A"].iter().map(|s| s.to_string()).collect()),
        ];
        let gaps = const_parity_gaps(&sets, &BTreeSet::new());
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].0, "B");
        let allow: BTreeSet<String> = ["B"].iter().map(|s| s.to_string()).collect();
        assert!(const_parity_gaps(&sets, &allow).is_empty());
    }

    // ── shapes: return-shape category (C) ─────────────────────────────────────

    #[test]
    fn shape_category_distinguishes_kinds() {
        assert_eq!(shape_category("PyResult<f64>"), "float");
        assert_eq!(shape_category("napi::Result<Vec<f64>>"), "array");
        assert_eq!(shape_category("(f64, f64)"), "tuple2");
        assert_eq!(shape_category("(u8, usize, String, String)"), "tuple4");
        assert_eq!(shape_category("PhpResult<Vec<f64>>"), "array");
        assert_eq!(shape_category("bool"), "bool");
        assert_eq!(shape_category("Option<f64>"), "opt<float>");
        assert_eq!(shape_category("PlanetPos"), "object");
        assert_eq!(shape_category("()"), "void");
    }

    /// The exact drift that DOC-10 documented wrong: a 2-tuple return must NOT
    /// classify the same as an object — so a Vec→struct change flips the snapshot.
    #[test]
    fn shape_category_tuple_is_not_object() {
        assert_ne!(shape_category("(f64, f64)"), shape_category("PlanetPos"));
    }

    #[test]
    fn scan_skips_impl_from_after_napi_struct() {
        let src = r#"
#[napi(object)]
pub struct PlanetPos {
    pub lon: f64,
}

impl From<celestial::PlanetPos> for PlanetPos {
    fn from(p: celestial::PlanetPos) -> Self {
        PlanetPos { lon: p.lon }
    }
}

/// real export
#[napi]
pub fn calc_ut(tjd: f64) -> f64 {
    tjd
}
"#;
        let fns = scan_decorated_fns(src, "napi");
        let names: Vec<&str> = fns.iter().map(|e| e.name.as_str()).collect();
        assert!(
            !names.contains(&"from"),
            "scanner must not emit impl-From `from` as an export, got {names:?}"
        );
        assert!(
            names.contains(&"calc_ut"),
            "a genuine #[napi] pub fn must still be captured, got {names:?}"
        );
    }
}
