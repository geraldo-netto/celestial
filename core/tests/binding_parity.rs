//! Compile-time (well, test-time) check that all three language bindings
//! export the same set of functions.
//!
//! This test is intentionally simple: it reads the three binding source files,
//! extracts exported function names, and asserts the sets are equal.
//! It is not a substitute for `cargo xtask parity` but catches regressions
//! during a normal `cargo test` run without requiring the xtask binary.

use std::{collections::BTreeSet, path::PathBuf};

fn workspace_root() -> PathBuf {
    // core/tests/ -> core/ -> workspace root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn decorated_fns(src: &str, decorator: &str) -> BTreeSet<String> {
    let prefix = format!("#[{decorator}");
    let mut out = BTreeSet::new();
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim();
        if t.starts_with(&prefix) || t.starts_with("#[allow") {
            let n = lines.len();
            for line in lines.iter().take(n.min(i + 6)).skip(i) {
                let l = line.trim();
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

fn internal_fns() -> BTreeSet<String> {
    ["to_napi", "to_py", "tap"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn load_fns(root: &std::path::Path, rel: &str, decorator: &str, include_wrap: bool) -> BTreeSet<String> {
    let src =
        std::fs::read_to_string(root.join(rel)).unwrap_or_else(|_| panic!("cannot read {rel}"));
    let skip = internal_fns();
    let mut fns = decorated_fns(&src, decorator);
    if include_wrap {
        fns.extend(py_wrapped_fns(&src));
    }
    fns.retain(|f| !skip.contains(f));
    fns
}

#[test]
fn all_bindings_export_same_functions() {
    let root = workspace_root();

    let py = load_fns(&root, "bindings/python/src/lib.rs", "pyfunction", true);
    let js = load_fns(&root, "bindings/js/src/lib.rs", "napi", false);
    let php = load_fns(&root, "bindings/php/src/lib.rs", "php_function", false);

    // Build human-readable diff messages before asserting
    let mut errors: Vec<String> = Vec::new();

    let union: BTreeSet<_> = py
        .iter()
        .chain(js.iter())
        .chain(php.iter())
        .cloned()
        .collect();
    for fn_name in &union {
        let in_py = py.contains(fn_name);
        let in_js = js.contains(fn_name);
        let in_php = php.contains(fn_name);
        if !(in_py && in_js && in_php) {
            let mut missing = Vec::new();
            if !in_py {
                missing.push("Python");
            }
            if !in_js {
                missing.push("JS");
            }
            if !in_php {
                missing.push("PHP");
            }
            errors.push(format!(
                "  {fn_name:<40} missing from: {}",
                missing.join(", ")
            ));
        }
    }

    if !errors.is_empty() {
        let total = union.len();
        panic!(
            "\nBinding parity failure — {}/{total} functions differ:\n\n{}\n\n\
             Fix with:  cargo xtask codegen --apply\n",
            errors.len(),
            errors.join("\n"),
        );
    }

    // Also assert counts match as a sanity check
    assert_eq!(
        py.len(),
        js.len(),
        "Python ({}) and JS ({}) have different function counts",
        py.len(),
        js.len()
    );
    assert_eq!(
        js.len(),
        php.len(),
        "JS ({}) and PHP ({}) have different function counts",
        js.len(),
        php.len()
    );
}
