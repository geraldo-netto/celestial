//! Plugin discovery and dispatch.
//!
//! Any executable named `celestial-<name>` on $PATH becomes a subcommand.
//! `celestial <name> [args…]` execs it with the remaining arguments verbatim.

use crate::error::CliError;
use std::path::{Path, PathBuf};

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
            let Some(name) = plugin_name_from_file_name(&s) else {
                continue;
            };
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
pub fn try_exec(subcommand: &str, args: &[String]) -> Result<(), CliError> {
    let target = format!("{PREFIX}{subcommand}");
    let candidate_names = executable_candidate_names(&target);
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        for candidate_name in &candidate_names {
            let candidate = dir.join(candidate_name);
            if is_exec(&candidate) {
                let err = do_exec(&candidate, args);
                return Err(CliError::Msg(format!("failed to exec `{target}`: {err}")));
            }
        }
    }
    Err(CliError::Msg(format!(
        "unknown command `{subcommand}` — no built-in and no `{target}` on PATH\n\
         Run `celestial --help` for available commands."
    )))
}

fn plugin_name_from_file_name(file_name: &str) -> Option<String> {
    plugin_name_from_file_name_with_extensions(file_name, &executable_extensions())
}

fn plugin_name_from_file_name_with_extensions(
    file_name: &str,
    extensions: &[String],
) -> Option<String> {
    let raw = file_name.strip_prefix(PREFIX)?;
    let name = strip_known_extension(raw, extensions).unwrap_or(raw);
    (!name.is_empty()).then(|| name.to_string())
}

fn executable_candidate_names(target: &str) -> Vec<String> {
    executable_candidate_names_with_extensions(target, &executable_extensions())
}

fn executable_candidate_names_with_extensions(target: &str, extensions: &[String]) -> Vec<String> {
    let mut names = vec![target.to_string()];
    for ext in extensions {
        if !ends_with_ignore_ascii_case(target, ext) {
            names.push(format!("{target}{ext}"));
        }
    }
    names
}

fn executable_extensions() -> Vec<String> {
    #[cfg(unix)]
    {
        Vec::new()
    }
    #[cfg(not(unix))]
    {
        std::env::var_os("PATHEXT")
            .map(|v| {
                v.to_string_lossy()
                    .split(';')
                    .filter(|ext| ext.starts_with('.') && ext.len() > 1)
                    .map(str::to_string)
                    .collect()
            })
            .filter(|exts: &Vec<String>| !exts.is_empty())
            .unwrap_or_else(|| {
                vec![
                    ".COM".to_string(),
                    ".EXE".to_string(),
                    ".BAT".to_string(),
                    ".CMD".to_string(),
                ]
            })
    }
}

fn strip_known_extension<'a>(name: &'a str, extensions: &[String]) -> Option<&'a str> {
    extensions
        .iter()
        .find(|ext| ends_with_ignore_ascii_case(name, ext))
        .map(|ext| &name[..name.len() - ext.len()])
}

fn ends_with_ignore_ascii_case(value: &str, suffix: &str) -> bool {
    value
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
}

#[cfg(any(not(unix), test))]
fn has_known_extension(path: &Path, extensions: &[String]) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| strip_known_extension(name, extensions).is_some())
}

fn is_exec(p: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        p.metadata()
            .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        p.is_file() && has_known_extension(p, &executable_extensions())
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
        let _guard = path_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f();
    }

    fn plugin_file_name(name: &str) -> String {
        #[cfg(unix)]
        {
            name.to_string()
        }
        #[cfg(not(unix))]
        {
            if has_known_extension(std::path::Path::new(name), &executable_extensions()) {
                name.to_string()
            } else {
                format!("{name}.exe")
            }
        }
    }

    fn make_executable(path: &std::path::Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }
        #[cfg(not(unix))]
        {
            let _ = path;
        }
    }

    fn make_non_executable(path: &std::path::Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o644);
            fs::set_permissions(path, perms).unwrap();
        }
        #[cfg(not(unix))]
        {
            let _ = path;
        }
    }

    /// Create a temporary executable file, run the test, then clean up.
    fn with_fake_plugin<F: FnOnce(&std::path::Path)>(name: &str, f: F) {
        let dir =
            std::env::temp_dir().join(format!("celestial_plugin_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(plugin_file_name(name));
        fs::write(&path, b"#!/bin/sh\necho hello\n").unwrap();
        make_executable(&path);
        f(&dir);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn plugin_name_strips_pathext_extension() {
        let exts = vec![".EXE".to_string(), ".CMD".to_string()];
        let name = plugin_name_from_file_name_with_extensions("celestial-synastry.exe", &exts);
        assert_eq!(name.as_deref(), Some("synastry"));
    }

    #[test]
    fn executable_candidates_probe_pathext_suffixes() {
        let exts = vec![".EXE".to_string(), ".CMD".to_string()];
        let names = executable_candidate_names_with_extensions("celestial-synastry", &exts);
        assert_eq!(
            names,
            vec![
                "celestial-synastry".to_string(),
                "celestial-synastry.EXE".to_string(),
                "celestial-synastry.CMD".to_string(),
            ]
        );
    }

    #[test]
    fn known_extension_rejects_non_pathext_suffix() {
        let exts = vec![".EXE".to_string()];
        assert!(has_known_extension(
            Path::new("celestial-synastry.exe"),
            &exts
        ));
        assert!(!has_known_extension(
            Path::new("celestial-synastry.txt"),
            &exts
        ));
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
            make_non_executable(&path);

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
                let p = d.join(plugin_file_name("celestial-dup"));
                fs::write(&p, b"#!/bin/sh\n").unwrap();
                make_executable(&p);
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
                let p = dir.join(plugin_file_name(name));
                fs::write(&p, b"#!/bin/sh\n").unwrap();
                make_executable(&p);
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
        // TEST-1: this test also mutates the process-global PATH, so it
        // must take the same lock — without it, it raced the
        // `discover_*` tests (clearing/restoring PATH mid-`discover`)
        // and they failed intermittently under the parallel runner.
        with_path_lock(|| {
            // A clearly non-existent subcommand must return Err (not panic).
            let saved = std::env::var_os("PATH").unwrap_or_default();
            std::env::set_var("PATH", ""); // ensure nothing on PATH
            let result = try_exec("definitely_not_a_real_plugin_xyz", &[]);
            std::env::set_var("PATH", &saved);
            assert!(result.is_err(), "unknown plugin must return Err");
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains("definitely_not_a_real_plugin_xyz"),
                "error message must name the subcommand"
            );
        });
    }

    /// Direct call to `do_exec` with a non-existent path → `ENOENT`.
    /// `cmd.exec()` returns the `io::Error` instead of replacing the test
    /// runner, which is the only return path we can exercise in-process.
    #[test]
    fn do_exec_returns_error_for_missing_path() {
        let p = std::path::Path::new("/nonexistent/celestial-doesnotexist-xyz");
        let err = do_exec(p, &[]);
        assert!(
            err.kind() == std::io::ErrorKind::NotFound
                || err.kind() == std::io::ErrorKind::PermissionDenied,
            "expected NotFound/PermissionDenied, got: {err:?}"
        );
    }
}
