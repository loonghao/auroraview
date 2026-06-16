//! Headless `run` execution for the packed CLI path (RFC 0018 §7 / §15.2).
//!
//! `app run <name|alias> [args...]` invokes a single registered command in a
//! one-shot Python process and exits — it does **not** reuse the persistent
//! JSON-RPC server the GUI path uses (§9.5), avoiding state pollution.
//!
//! Flow:
//! 1. read the overlay and normalize `<alias>` → canonical name from the
//!    embedded command table (§12, §15.2);
//! 2. extract the bundled Python runtime + sources (shared hash-based cache as
//!    the GUI path, so a warm cache means no re-extraction);
//! 3. launch the entry point with `AURORAVIEW_CLI_INVOKE` + `AURORAVIEW_CLI_ARGS`
//!    so Python builds `webview`/`commands` *without* `show()` and invokes the
//!    command in-process (argument parsing per §6.3 lives Python-side, where the
//!    signature is available);
//! 4. inherit stdout/stderr straight to the terminal and map the child exit
//!    code to the §4.4 convention.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use auroraview_pack::{BundleStrategy, OverlayData, PackMode, PythonBundleConfig};

use super::extract::{extract_lib_packages, extract_resources_parallel, extract_standalone_python};
use super::utils::{build_module_search_paths, get_python_extract_dir_with_hash};

/// Exit code for "command not found" / argument errors (§4.4).
const EXIT_USAGE: i32 = 2;

/// Entry point for `app run ...`. `args` is everything after the `run` verb
/// (the command name/alias followed by its arguments).
///
/// Diverging exit codes are emitted via `std::process::exit` to honor §4.4
/// precisely; the `Result` is reserved for unexpected internal failures.
pub fn run_command(args: &[String]) -> Result<()> {
    super::attach_parent_console();

    // Real exe name for hint text, so suggestions match what the user typed.
    let prog = super::cli::program_name();

    let Some(requested) = args.first() else {
        eprintln!("auroraview: 'run' requires a command name");
        eprintln!("Try '{prog} -h' to list available commands.");
        std::process::exit(EXIT_USAGE);
    };

    let exe_path = std::env::current_exe().context("locate current executable")?;
    let overlay = match auroraview_pack::OverlayReader::read(&exe_path)? {
        Some(o) => o,
        None => {
            eprintln!("auroraview: no packed overlay found");
            std::process::exit(EXIT_USAGE);
        }
    };

    // Resolve alias → canonical name against the embedded table (§15.2).
    let canonical = match resolve_command(&overlay, requested) {
        Some(name) => name,
        None => {
            eprintln!("auroraview: command not found: {requested}");
            eprintln!("Try '{prog} list' to see available commands.");
            std::process::exit(EXIT_USAGE);
        }
    };

    let python = match &overlay.config.mode {
        PackMode::FullStack { python, .. } => python.as_ref(),
        _ => {
            eprintln!("auroraview: this application has no Python backend to run commands");
            std::process::exit(EXIT_USAGE);
        }
    };

    let command_args: Vec<String> = args[1..].to_vec();
    let code = invoke(&overlay, python, &canonical, &command_args)?;
    std::process::exit(code);
}

/// Resolve a requested token to a canonical command name, accepting either the
/// canonical name itself or any alias. Returns `None` if no CLI command matches.
fn resolve_command(overlay: &OverlayData, requested: &str) -> Option<String> {
    overlay.config.cli_commands.iter().find_map(|cmd| {
        if cmd.name == requested || cmd.aliases.iter().any(|a| a == requested) {
            Some(cmd.name.clone())
        } else {
            None
        }
    })
}

/// Extract Python, launch the one-shot invoke process, and return its exit code.
fn invoke(
    overlay: &OverlayData,
    python: &PythonBundleConfig,
    command: &str,
    command_args: &[String],
) -> Result<i32> {
    let python_exe = match python.strategy {
        BundleStrategy::Standalone => {
            // First launch extracts the runtime (seconds); warm cache is instant.
            // A one-line hint to stderr avoids looking hung (RFC §9.2).
            let cache = get_python_extract_dir_with_hash(&overlay.content_hash);
            if !cache.join(".cache_valid").exists() {
                eprintln!("auroraview: preparing runtime (first run may take a few seconds)...");
            }
            extract_standalone_python(overlay).context("extract bundled Python runtime")?
        }
        _ => PathBuf::from("python"),
    };

    let temp_dir = get_python_extract_dir_with_hash(&overlay.content_hash);
    fs::create_dir_all(&temp_dir)?;
    let marker = temp_dir.join(".cache_valid");
    let cache_valid = marker.exists();
    let (resources_dir, site_packages_dir) = if cache_valid {
        // Warm cache: reuse the previously extracted dirs (matches GUI path).
        (temp_dir.join("resources"), temp_dir.join("site-packages"))
    } else {
        extract_python_sources(overlay, &temp_dir)?;
        let resources = extract_resources_parallel(overlay, &temp_dir)?;
        let site_packages = extract_lib_packages(overlay, &temp_dir)?;
        // Mark the cache valid so the next run (and the GUI path) skip extraction.
        let _ = fs::write(&marker, "1");
        (resources, site_packages)
    };

    let module_paths = build_module_search_paths(
        &python.module_search_paths,
        &temp_dir,
        &resources_dir,
        &site_packages_dir,
    );
    let pythonpath = module_paths.join(if cfg!(windows) { ";" } else { ":" });

    let code = build_invoke_code(&temp_dir, &python.entry_point);

    let args_json = serde_json::to_string(command_args).unwrap_or_else(|_| "[]".to_string());

    let mut cmd = Command::new(&python_exe);
    cmd.args(["-c", &code])
        .current_dir(&temp_dir)
        // Headless invoke streams results straight to the user's terminal.
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("AURORAVIEW_PACKED", "1")
        .env("AURORAVIEW_CLI_INVOKE", command)
        .env("AURORAVIEW_CLI_ARGS", args_json)
        .env("AURORAVIEW_RESOURCES_DIR", &resources_dir)
        .env("PYTHONPATH", &pythonpath)
        .env("PYTHONUNBUFFERED", "1")
        .env("PYTHONIOENCODING", "utf-8");

    if let Some(home) = python_exe.parent() {
        cmd.env("PYTHONHOME", home);
    }

    let status = cmd
        .status()
        .with_context(|| format!("failed to launch interpreter: {}", python_exe.display()))?;

    // §4.4: the Python invoke path already maps its own outcomes to 0/1/2 and
    // exits with them. Propagate the child's code; default to 1 if it was
    // terminated by a signal (no code available).
    Ok(status.code().unwrap_or(1))
}

/// Write every `python/<rel>` overlay asset to `root/<rel>`.
///
/// Mirrors the GUI extraction path. Files already present with identical
/// content are left untouched (warm-cache reuse); a locked file with matching
/// content is treated as success.
fn extract_python_sources(overlay: &OverlayData, root: &Path) -> Result<()> {
    let assets: Vec<_> = overlay
        .assets
        .iter()
        .filter(|(p, _)| p.starts_with("python/"))
        .collect();

    let dirs: HashSet<PathBuf> = assets
        .iter()
        .filter_map(|(p, _)| {
            let rel = p.strip_prefix("python/").unwrap_or(p);
            root.join(rel).parent().map(Path::to_path_buf)
        })
        .collect();
    for dir in &dirs {
        fs::create_dir_all(dir)?;
    }

    for (path, content) in assets {
        let rel = path.strip_prefix("python/").unwrap_or(path);
        let dest = root.join(rel);
        if let Ok(existing) = fs::read(&dest) {
            if existing == *content {
                continue;
            }
        }
        if let Err(e) = fs::write(&dest, content) {
            // A locked file whose content already matches is fine.
            if e.raw_os_error() == Some(32) {
                if let Ok(existing) = fs::read(&dest) {
                    if existing == *content {
                        continue;
                    }
                }
            }
            return Err(e).with_context(|| format!("write {}", dest.display()));
        }
    }
    Ok(())
}

/// Build the `python -c` snippet that imports/runs the entry point. Mirrors the
/// GUI launcher (`backend.rs`) and the pack-time dump so all three agree on how
/// the user module is loaded.
fn build_invoke_code(temp_dir: &Path, entry_point: &str) -> String {
    let dir = temp_dir.display();
    let has_bootstrap = temp_dir.join("__aurora_bootstrap__.py").exists();
    let bootstrap = if has_bootstrap {
        "import __aurora_bootstrap__; "
    } else {
        ""
    };

    if let Some((module, function)) = entry_point.split_once(':') {
        let module = module.replace(['/', '\\'], ".");
        let module = module.trim_end_matches(".py");
        let function = if function.is_empty() { "main" } else { function };
        format!(
            "import sys; sys.path.insert(0, r'{dir}'); {bootstrap}from {module} import {function}; {function}()"
        )
    } else {
        let script = temp_dir.join(entry_point);
        format!(
            "import sys; sys.path.insert(0, r'{dir}'); {bootstrap}import runpy; runpy.run_path(r'{}', run_name='__main__')",
            script.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auroraview_pack::{CliCommandMeta, PackConfig};

    fn overlay_with(commands: Vec<CliCommandMeta>) -> OverlayData {
        let mut config = PackConfig::url("about:blank");
        config.cli_commands = commands;
        OverlayData::new(config)
    }

    fn cmd(name: &str, aliases: &[&str]) -> CliCommandMeta {
        CliCommandMeta {
            name: name.to_string(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            help: String::new(),
            params: vec![],
        }
    }

    #[test]
    fn resolve_matches_canonical_name() {
        let o = overlay_with(vec![cmd("export", &["exp"])]);
        assert_eq!(resolve_command(&o, "export").as_deref(), Some("export"));
    }

    #[test]
    fn resolve_matches_alias() {
        let o = overlay_with(vec![cmd("export", &["exp", "e"])]);
        assert_eq!(resolve_command(&o, "e").as_deref(), Some("export"));
    }

    #[test]
    fn resolve_unknown_is_none() {
        let o = overlay_with(vec![cmd("export", &["exp"])]);
        assert!(resolve_command(&o, "missing").is_none());
    }

    #[test]
    fn invoke_code_module_function_form() {
        let code = build_invoke_code(Path::new("/tmp/app"), "pkg.main:run");
        assert!(code.contains("from pkg.main import run"));
        assert!(code.contains("run()"));
        assert!(!code.contains("__aurora_bootstrap__"));
    }

    #[test]
    fn invoke_code_script_form() {
        let code = build_invoke_code(Path::new("/tmp/app"), "main.py");
        assert!(code.contains("runpy.run_path"));
    }

    #[test]
    fn invoke_code_includes_bootstrap_when_present() {
        // A `__aurora_bootstrap__.py` beside the sources is imported first so
        // the packed runtime path matches the GUI launcher.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("__aurora_bootstrap__.py"), b"# boot").unwrap();
        let code = build_invoke_code(dir.path(), "pkg.main:run");
        assert!(code.contains("import __aurora_bootstrap__;"));
        assert!(code.contains("from pkg.main import run"));
    }

    #[test]
    fn invoke_code_empty_function_defaults_to_main() {
        let code = build_invoke_code(Path::new("/tmp/app"), "pkg.entry:");
        assert!(code.contains("from pkg.entry import main"));
        assert!(code.contains("main()"));
    }

    #[test]
    fn extract_python_sources_writes_python_prefixed_only() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = PackConfig::url("about:blank");
        config.cli_commands = vec![];
        let mut overlay = OverlayData::new(config);
        overlay.add_asset("python/pkg/mod.py", b"x = 1\n".to_vec());
        overlay.add_asset("python/main.py", b"print('hi')\n".to_vec());
        overlay.add_asset("web/index.html", b"<html></html>".to_vec());

        extract_python_sources(&overlay, dir.path()).expect("extract");

        assert!(dir.path().join("pkg/mod.py").exists());
        assert!(dir.path().join("main.py").exists());
        assert!(!dir.path().join("index.html").exists());
    }

    #[test]
    fn extract_python_sources_skips_identical_existing_content() {
        // A warm cache leaves byte-identical files untouched (no rewrite).
        let dir = tempfile::tempdir().unwrap();
        let mut overlay = OverlayData::new(PackConfig::url("about:blank"));
        overlay.add_asset("python/main.py", b"same\n".to_vec());

        // Pre-create the destination with identical content.
        std::fs::write(dir.path().join("main.py"), b"same\n").unwrap();
        extract_python_sources(&overlay, dir.path()).expect("extract");

        assert_eq!(
            std::fs::read(dir.path().join("main.py")).unwrap(),
            b"same\n"
        );
    }
}
