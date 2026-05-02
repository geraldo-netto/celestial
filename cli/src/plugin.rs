//! Plugin discovery and dispatch.
//!
//! Any executable named `celestial-<name>` on $PATH becomes a subcommand.
//! `celestial <name> [args…]` execs it with the remaining arguments verbatim.

use std::path::PathBuf;

const PREFIX: &str = "celestial-";

#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub path: PathBuf,
}

/// Every `celestial-*` plugin visible on $PATH, sorted by name.
pub fn discover() -> Vec<Plugin> {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    let mut plugins = Vec::new();
    for dir in std::env::split_paths(&path_var) {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let s = fname.to_string_lossy();
            if !s.starts_with(PREFIX) {
                continue;
            }
            let name = s[PREFIX.len()..].to_string();
            if name.is_empty() || !seen.insert(name.clone()) {
                continue;
            }
            let path = entry.path();
            if is_exec(&path) {
                plugins.push(Plugin { name, path });
            }
        }
    }
    plugins.sort_by(|a, b| a.name.cmp(&b.name));
    plugins
}

/// Try to exec `celestial-<subcommand>` with `args`.  
/// Only returns on error or when no matching plugin is found.
pub fn try_exec(subcommand: &str, args: &[String]) -> Result<(), String> {
    let target = format!("{PREFIX}{subcommand}");
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let candidate = dir.join(&target);
        if is_exec(&candidate) {
            let err = do_exec(&candidate, args);
            return Err(format!("failed to exec `{target}`: {err}"));
        }
    }
    Err(format!(
        "unknown command `{subcommand}` — no built-in and no `{target}` on PATH\n\
         Run `celestial --help` for available commands."
    ))
}

fn is_exec(p: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        p.metadata()
            .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        p.is_file()
    }
}

#[cfg(unix)]
fn do_exec(path: &std::path::Path, args: &[String]) -> std::io::Error {
    use std::os::unix::process::CommandExt;
    let mut cmd = std::process::Command::new(path);
    cmd.args(args);
    cmd.exec()
}

#[cfg(not(unix))]
fn do_exec(path: &std::path::Path, args: &[String]) -> std::io::Error {
    match std::process::Command::new(path)
        .args(args)
        .spawn()
        .and_then(|mut c| c.wait())
    {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => e,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Mutex, OnceLock};

    /// Serializes tests that mutate the process-global PATH env var.
    /// Without this, parallel tests leak each other's PATH settings and fail
    /// intermittently (`discover_finds_*` seeing plugins from `discover_sorted_*`, etc.).
    fn path_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    /// Run a closure while holding `path_lock()`. Poison-safe: if a prior test
    /// panicked while holding the lock, we still recover and proceed.
    fn with_path_lock<F: FnOnce()>(f: F) {
        let _guard = path_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        f();
    }

    /// Create a temporary executable file, run the test, then clean up.
    fn with_fake_plugin<F: FnOnce(&std::path::Path)>(name: &str, f: F) {
        let dir =
            std::env::temp_dir().join(format!("celestial_plugin_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(name);
        fs::write(&path, b"#!/bin/sh\necho hello\n").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        f(&dir);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_empty_path() {
        with_path_lock(|| {
            // Temporarily clear PATH — discover() must return empty vec without panicking.
            let saved = std::env::var_os("PATH").unwrap_or_default();
            std::env::set_var("PATH", "");
            let plugins = discover();
            std::env::set_var("PATH", &saved);
            assert!(plugins.is_empty(), "empty PATH should yield no plugins");
        });
    }

    #[test]
    fn discover_finds_celestial_plugin() {
        with_path_lock(|| {
            with_fake_plugin("celestial-testplugin", |dir| {
                let saved = std::env::var_os("PATH").unwrap_or_default();
                let new_path = format!("{}:{}", dir.display(), saved.to_string_lossy());
                std::env::set_var("PATH", &new_path);
                let plugins = discover();
                std::env::set_var("PATH", &saved);

                let found = plugins.iter().any(|p| p.name == "testplugin");
                assert!(
                    found,
                    "should discover celestial-testplugin, got: {plugins:?}"
                );
            });
        });
    }

    #[test]
    fn discover_ignores_non_celestial_prefix() {
        with_path_lock(|| {
            with_fake_plugin("other-tool", |dir| {
                let saved = std::env::var_os("PATH").unwrap_or_default();
                let new_path = format!("{}:{}", dir.display(), saved.to_string_lossy());
                std::env::set_var("PATH", &new_path);
                let plugins = discover();
                std::env::set_var("PATH", &saved);

                let found = plugins.iter().any(|p| p.name == "tool");
                assert!(!found, "should not discover non-celestial- prefixed file");
            });
        });
    }

    #[test]
    fn discover_ignores_non_executable() {
        with_path_lock(|| {
            let dir = std::env::temp_dir().join(format!("celestial_noexec_{}", std::process::id()));
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join("celestial-notexec");
            std::fs::write(&path, b"not executable").unwrap();
            // mode 0o644 — readable but not executable
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o644);
            std::fs::set_permissions(&path, perms).unwrap();

            let saved = std::env::var_os("PATH").unwrap_or_default();
            let new_path = format!("{}:{}", dir.display(), saved.to_string_lossy());
            std::env::set_var("PATH", &new_path);
            let plugins = discover();
            std::env::set_var("PATH", &saved);
            let _ = std::fs::remove_dir_all(&dir);

            assert!(
                !plugins.iter().any(|p| p.name == "notexec"),
                "non-executable file must not appear as plugin"
            );
        });
    }

    #[test]
    fn discover_deduplicates_by_name() {
        with_path_lock(|| {
            // Two dirs on PATH, both have celestial-dup — only first wins.
            let dir1 = std::env::temp_dir().join(format!("cel_dup1_{}", std::process::id()));
            let dir2 = std::env::temp_dir().join(format!("cel_dup2_{}", std::process::id()));
            for d in [&dir1, &dir2] {
                let _ = fs::create_dir_all(d);
                let p = d.join("celestial-dup");
                fs::write(&p, b"#!/bin/sh\n").unwrap();
                let mut perms = fs::metadata(&p).unwrap().permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&p, perms).unwrap();
            }

            let saved = std::env::var_os("PATH").unwrap_or_default();
            let new_path = format!(
                "{}:{}:{}",
                dir1.display(),
                dir2.display(),
                saved.to_string_lossy()
            );
            std::env::set_var("PATH", &new_path);
            let plugins = discover();
            std::env::set_var("PATH", &saved);
            let _ = fs::remove_dir_all(&dir1);
            let _ = fs::remove_dir_all(&dir2);

            let dup_count = plugins.iter().filter(|p| p.name == "dup").count();
            assert_eq!(dup_count, 1, "duplicate plugin name must appear only once");
            // First directory wins
            let p = plugins.iter().find(|p| p.name == "dup").unwrap();
            assert!(p.path.starts_with(&dir1), "first PATH dir must win");
        });
    }

    #[test]
    fn discover_sorted_by_name() {
        with_path_lock(|| {
            let dir = std::env::temp_dir().join(format!("cel_sort_{}", std::process::id()));
            let _ = fs::create_dir_all(&dir);
            for name in ["celestial-zzz", "celestial-aaa", "celestial-mmm"] {
                let p = dir.join(name);
                fs::write(&p, b"#!/bin/sh\n").unwrap();
                let mut perms = fs::metadata(&p).unwrap().permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&p, perms).unwrap();
            }

            let saved = std::env::var_os("PATH").unwrap_or_default();
            std::env::set_var("PATH", dir.display().to_string());
            let plugins = discover();
            std::env::set_var("PATH", &saved);
            let _ = fs::remove_dir_all(&dir);

            let names: Vec<&str> = plugins.iter().map(|p| p.name.as_str()).collect();
            assert!(
                names.windows(2).all(|w| w[0] <= w[1]),
                "plugins must be sorted by name, got: {names:?}"
            );
        });
    }

    #[test]
    fn try_exec_returns_err_for_unknown() {
        // A clearly non-existent subcommand must return Err (not panic).
        let saved = std::env::var_os("PATH").unwrap_or_default();
        std::env::set_var("PATH", ""); // ensure nothing on PATH
        let result = try_exec("definitely_not_a_real_plugin_xyz", &[]);
        std::env::set_var("PATH", &saved);
        assert!(result.is_err(), "unknown plugin must return Err");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("definitely_not_a_real_plugin_xyz"),
            "error message must name the subcommand"
        );
    }
}
