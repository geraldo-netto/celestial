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
        _ => {
            eprintln!("USAGE\n  cargo xtask parity\n  cargo xtask codegen [--apply]");
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
