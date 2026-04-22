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
        Some("codegen") => cmd_codegen(args.any(|a| a == "--apply")),
        Some("stubs") => cmd_stubs(),
        _ => {
            eprintln!(
                "USAGE\n  cargo xtask parity\n  cargo xtask codegen [--apply]\n  cargo xtask stubs"
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
        .map(|s| s.to_string())
        .collect()
}

// ─── parsing ─────────────────────────────────────────────────────────────────

fn decorated_fns(src: &str, decorator: &str) -> BTreeSet<String> {
    let prefix = format!("#[{decorator}");
    let mut out = BTreeSet::new();
    let lines: Vec<&str> = src.lines().collect();
    let n = lines.len();
    let mut i = 0;
    while i < n {
        let t = lines[i].trim();
        if t.starts_with(&prefix) || t.starts_with("#[allow") {
            for j in i..n.min(i + 6) {
                let l = lines[j].trim();
                let rest = l.strip_prefix("pub fn ").or_else(|| l.strip_prefix("fn "));
                if let Some(rest) = rest {
                    let name: String = rest.split([' ', '(', '<']).next().unwrap_or("").to_string();
                    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        out.insert(name);
                        break;
                    }
                }
            }
        }
        i += 1;
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
            fns.retain(|f| !skip.contains(f));

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

        let stubs: Vec<String> = missing
            .iter()
            .map(|fn_name| match extract_fn_body(&js_src, fn_name) {
                Some(body) => format!("// ── {fn_name} ──\n{}\n", convert_to(&body, &binding.name)),
                None => format!("// ── {fn_name} — no JS reference; add manually ──\n"),
            })
            .collect();

        for s in &stubs {
            print!("{s}");
        }

        if apply {
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
    let block_start = before.rfind("\n#[").map(|p| p + 1).unwrap_or(0);
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
                .replace("-> napi::Result<", "-> PyResult<")
                .replace("napi::Result<", "PyResult<")
                .replace("napi::Error::from_reason(", "PyRuntimeError::new_err(")
                .replace(".map_err(to_napi)", ".map_err(to_py)")
                .replace("pub fn ", "fn ");
            format!("#[pyfunction]\n{body}")
        }
        "PHP" => {
            let body = stripped
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
            format!("#[php_function]\n{body}")
        }
        _ => stripped,
    }
}

// ─── stubs command ────────────────────────────────────────────────────────────

/// Regenerate `bindings/php/phpstan-stubs.php` from the PHP binding source.
///
/// Rules enforced:
///  - Every exported `pub fn` gets a `function celestial_<n>(…): <type> {}` stub.
///  - Return types: `Vec<*>` / `array` / `Result<Vec<*>>` → `array`; scalars pass through.
///  - Nullable returns (`Option<T>`) → `?<phptype>`.
///  - Typed array hints (`int[]`, `float[]`, `string[]`) are NEVER used in
///    function signatures — only in `@return` docblocks where they are valid.
///  - Parameter type `Vec<T>` / typed arrays → `array`.
///  - Numeric or invalid variable names are sanitised.
fn cmd_stubs() {
    let root = workspace_root();
    let php_src = fs::read_to_string(root.join("bindings/php/src/lib.rs"))
        .expect("cannot read bindings/php/src/lib.rs");
    let out_path = root.join("bindings/php/phpstan-stubs.php");

    let mut stubs: Vec<String> = vec!["<?php\n".to_string()];
    stubs.push("// Auto-generated by `cargo xtask stubs` — do not edit by hand.\n// Regenerate with:  cargo xtask stubs\n".to_string());

    // Parse every #[php_function] decorated function
    let fn_regex = regex_find_php_fns(&php_src);
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for entry in fn_regex {
        let fn_name = &entry.name;
        let stub_name = format!("celestial_{fn_name}");
        if seen.contains(&stub_name) {
            continue; // skip duplicates
        }
        seen.insert(stub_name.clone());

        let php_params = entry
            .params
            .iter()
            .map(|(typ, nam)| format!("{} ${}", rust_type_to_php(typ, false), sanitise_var(nam)))
            .collect::<Vec<_>>()
            .join(", ");

        let php_ret = rust_type_to_php(&entry.ret, true);
        let docblock = format!("/** @return {} */", php_array_doctype(&entry.ret));

        stubs.push(format!(
            "\n{docblock}\nfunction {stub_name}({php_params}): {php_ret} {{}}\n"
        ));
    }

    let output = stubs.join("");
    fs::write(&out_path, &output).expect("cannot write phpstan-stubs.php");
    println!(
        "✓ {} functions written to {}",
        seen.len(),
        out_path.display()
    );
}

struct PhpFnEntry {
    name: String,
    params: Vec<(String, String)>, // (rust_type, param_name)
    ret: String,
}

/// Walk through the PHP binding source and extract all #[php_function] fn signatures.
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

fn parse_fn_sig(sig: &str) -> Option<PhpFnEntry> {
    // Match: `pub fn name(args) -> ret`  or `fn name(args) -> ret`
    let sig_clean = sig.replace('\n', " ");
    let re_name = sig_clean.find("fn ")?;
    let after_fn = &sig_clean[re_name + 3..];
    let name_end = after_fn.find('(')?;
    let name = after_fn[..name_end].trim().to_string();

    // Extract parameter list
    let paren_start = sig_clean.find('(')?;
    let paren_end = sig_clean.rfind(')')?;
    let params_str = sig_clean[paren_start + 1..paren_end].trim();

    let params: Vec<(String, String)> = if params_str.is_empty() {
        vec![]
    } else {
        params_str
            .split(',')
            .filter_map(|p| {
                let p = p.trim();
                if p.is_empty() {
                    return None;
                }
                // Rust param: `name: Type` or `mut name: Type`
                let p = p.trim_start_matches("mut").trim();
                let colon = p.find(':')?;
                let pname = p[..colon].trim().to_string();
                let ptype = p[colon + 1..].trim().to_string();
                // Skip the PyO3/pyo3 Python<'_> and lifetime params
                if pname == "py" || ptype.starts_with("Python") {
                    return None;
                }
                Some((ptype, pname))
            })
            .collect()
    };

    // Extract return type
    let ret = if let Some(arrow) = sig_clean.rfind("->") {
        let after = &sig_clean[arrow + 2..];
        // Strip up to the opening { or end of line
        let end = after.find('{').unwrap_or(after.len());
        after[..end].trim().to_string()
    } else {
        "()".to_string()
    };

    Some(PhpFnEntry { name, params, ret })
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

/// Return the PHPDoc @return type string (can use int[], float[], etc.).
fn php_array_doctype(rust: &str) -> &'static str {
    let t = rust.trim();
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

    let (nullable, core) = if let Some(s) = inner
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        (true, s.trim())
    } else {
        (false, inner)
    };

    let doc = match core {
        "f64" | "f32" => {
            if nullable {
                "float|null"
            } else {
                "float"
            }
        }
        "i32" | "i64" | "u32" | "u64" | "u8" | "i8" | "usize" | "isize" => {
            if nullable {
                "int|null"
            } else {
                "int"
            }
        }
        "bool" => {
            if nullable {
                "bool|null"
            } else {
                "bool"
            }
        }
        "String" | "&str" | "&'static str" => {
            if nullable {
                "string|null"
            } else {
                "string"
            }
        }
        "()" => "void",
        t if t.starts_with("Vec<f") => {
            if nullable {
                "float[]|null"
            } else {
                "float[]"
            }
        }
        t if t.starts_with("Vec<i") || t.starts_with("Vec<u") => {
            if nullable {
                "int[]|null"
            } else {
                "int[]"
            }
        }
        t if t.starts_with("Vec<String") || t.starts_with("Vec<&str") => {
            if nullable {
                "string[]|null"
            } else {
                "string[]"
            }
        }
        t if t.starts_with("Vec<") => {
            if nullable {
                "array|null"
            } else {
                "array"
            }
        }
        _ => {
            if nullable {
                "array|null"
            } else {
                "array"
            }
        }
    };
    doc
}

/// Sanitise a Rust parameter name to a valid PHP variable name.
fn sanitise_var(name: &str) -> &'static str {
    // Leak is fine for a short-lived CLI tool.
    let s = name.trim().trim_start_matches('_');
    // If it starts with a digit or is empty, prefix with 'p'
    let fixed = if s.is_empty()
        || s.chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        format!("p{s}")
    } else {
        s.to_string()
    };
    Box::leak(fixed.into_boxed_str())
}
