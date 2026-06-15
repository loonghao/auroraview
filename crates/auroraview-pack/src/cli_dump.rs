//! Pack-time CLI metadata collection (RFC 0018 §5.1 / §13.3).
//!
//! To make the runtime `-h`/`list` path zero-latency, the command table is
//! harvested **at pack time** and embedded into the overlay. This module runs
//! the already-bundled entry point once with `AURORAVIEW_CLI_DUMP=1` inside the
//! target Python environment, captures the `{"type":"cli_metadata", ...}` JSON
//! it prints, and converts it into [`CliCommandMeta`] for
//! [`PackConfig::cli_commands`].
//!
//! The whole thing is **best-effort**: when the dump cannot run (cross-platform
//! packing, missing runtime, a failing entry point) the caller logs a warning
//! and proceeds with an empty command table. Only the §12.4 alias conflict
//! check is fatal — a successful dump that contains conflicting aliases means
//! the developer made a mistake worth surfacing.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use tempfile::TempDir;

use crate::config::{CliCommandMeta, CliParamMeta};
use crate::overlay::OverlayData;
use crate::python_standalone::PythonTarget;
use crate::{BundleStrategy, PackMode, PythonBundleConfig};

/// Reserved CLI verbs/flags an alias must never collide with (RFC 0018 §12.4).
/// Mirrors the Python-side `_RESERVED_CLI_VERBS`.
const RESERVED_CLI_VERBS: &[&str] = &["run", "list", "help", "version"];

/// Raw JSON shape emitted by the Python `dump_cli_metadata` entry point.
#[derive(Debug, Deserialize)]
struct CliDumpPayload {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    commands: Vec<CliCommandMeta>,
}

/// Outcome of a pack-time dump attempt.
pub enum CliDumpOutcome {
    /// The dump ran and produced a (possibly empty) command table.
    Collected(Vec<CliCommandMeta>),
    /// The dump was skipped; the string explains why (for a warning log).
    Skipped(String),
}

/// Collect CLI command metadata for a built overlay (best-effort).
///
/// Returns [`CliDumpOutcome::Skipped`] (never an error) for any non-fatal
/// reason the dump can't run — not FullStack, not Standalone, cross-platform,
/// no runtime archive, subprocess failure. Returns `Err` only for a fatal
/// alias conflict (§12.4) in an otherwise successful dump.
pub fn collect_cli_metadata(
    overlay: &OverlayData,
    python: &PythonBundleConfig,
) -> Result<CliDumpOutcome, String> {
    // Only FullStack apps have Python commands to dump.
    if !matches!(overlay.config.mode, PackMode::FullStack { .. }) {
        return Ok(CliDumpOutcome::Skipped("not a FullStack app".into()));
    }

    // The dump executes the bundled interpreter, which only exists for the
    // Standalone strategy (others rely on a system Python at runtime).
    if python.strategy != BundleStrategy::Standalone {
        return Ok(CliDumpOutcome::Skipped(format!(
            "bundle strategy {:?} has no embedded interpreter to run at pack time",
            python.strategy
        )));
    }

    // Cross-platform packing cannot execute the target interpreter on the host.
    let host = match PythonTarget::current() {
        Ok(t) => t,
        Err(e) => return Ok(CliDumpOutcome::Skipped(format!("unknown host target: {e}"))),
    };
    if let Some(meta) = runtime_target(overlay) {
        if meta != host.triple() {
            return Ok(CliDumpOutcome::Skipped(format!(
                "cross-platform pack (runtime target {meta} != host {})",
                host.triple()
            )));
        }
    }

    match run_dump(overlay, python, host) {
        Ok(commands) => {
            check_alias_conflicts(&commands)?;
            Ok(CliDumpOutcome::Collected(commands))
        }
        Err(reason) => Ok(CliDumpOutcome::Skipped(reason)),
    }
}

/// Read the bundled runtime's target triple from `python_runtime.json`.
fn runtime_target(overlay: &OverlayData) -> Option<String> {
    let raw = overlay
        .assets
        .iter()
        .find(|(p, _)| p == "python_runtime.json")
        .map(|(_, c)| c.clone())?;
    let meta: serde_json::Value = serde_json::from_slice(&raw).ok()?;
    meta.get("target")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// Extract the bundled runtime + sources to a temp dir and run the entry point
/// with `AURORAVIEW_CLI_DUMP=1`, returning the parsed command table.
fn run_dump(
    overlay: &OverlayData,
    python: &PythonBundleConfig,
    host: PythonTarget,
) -> Result<Vec<CliCommandMeta>, String> {
    let temp = TempDir::new().map_err(|e| format!("temp dir: {e}"))?;
    let root = temp.path();

    extract_runtime(overlay, root)?;
    extract_python_sources(overlay, root)?;

    let python_exe = root.join(host.python_path());
    if !python_exe.exists() {
        return Err(format!(
            "bundled interpreter missing at {}",
            python_exe.display()
        ));
    }

    let code = build_dump_code(root, &python.entry_point);

    let output = Command::new(&python_exe)
        .args(["-c", &code])
        .current_dir(root)
        .env("AURORAVIEW_CLI_DUMP", "1")
        .env("PYTHONUNBUFFERED", "1")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .map_err(|e| format!("failed to launch interpreter: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "dump process exited with {}: {}",
            output.status,
            stderr.trim()
        ));
    }

    parse_dump_stdout(&output.stdout)
}

/// Parse the `{"type":"cli_metadata", "commands":[...]}` line from stdout.
///
/// Tolerates leading log noise by scanning for the last non-empty line that
/// parses as the expected payload.
fn parse_dump_stdout(stdout: &[u8]) -> Result<Vec<CliCommandMeta>, String> {
    let text = String::from_utf8_lossy(stdout);
    for line in text.lines().rev() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            continue;
        }
        if let Ok(payload) = serde_json::from_str::<CliDumpPayload>(line) {
            if payload.kind == "cli_metadata" {
                return Ok(normalize(payload.commands));
            }
        }
    }
    Err("no cli_metadata JSON found in dump output".into())
}

/// Backfill defaults the Python side may omit (e.g. an absent param `type`).
fn normalize(commands: Vec<CliCommandMeta>) -> Vec<CliCommandMeta> {
    commands
        .into_iter()
        .map(|mut c| {
            c.params = c
                .params
                .into_iter()
                .map(|mut p| {
                    if p.r#type.is_empty() {
                        p.r#type = "any".to_string();
                    }
                    p
                })
                .collect::<Vec<CliParamMeta>>();
            c
        })
        .collect()
}

/// Build the `python -c` snippet that imports/runs the entry point. Mirrors the
/// runtime launcher in `auroraview-cli` so dump and run see the same module.
fn build_dump_code(root: &Path, entry_point: &str) -> String {
    let root_str = root.display();
    if let Some((module, function)) = entry_point.split_once(':') {
        let module = module.replace(['/', '\\'], ".");
        let module = module.trim_end_matches(".py");
        let function = if function.is_empty() { "main" } else { function };
        format!(
            "import sys; sys.path.insert(0, r'{root_str}'); from {module} import {function}; {function}()"
        )
    } else {
        let script = root.join(entry_point);
        format!(
            "import sys; sys.path.insert(0, r'{root_str}'); import runpy; runpy.run_path(r'{}', run_name='__main__')",
            script.display()
        )
    }
}

/// Unpack `python_runtime.tar.gz` into `root`.
fn extract_runtime(overlay: &OverlayData, root: &Path) -> Result<(), String> {
    let archive = overlay
        .assets
        .iter()
        .find(|(p, _)| p == "python_runtime.tar.gz")
        .map(|(_, c)| c.clone())
        .ok_or_else(|| "no python_runtime.tar.gz in overlay".to_string())?;

    let decoder = flate2::read::GzDecoder::new(&archive[..]);
    let mut tar = tar::Archive::new(decoder);
    tar.unpack(root)
        .map_err(|e| format!("failed to unpack runtime: {e}"))
}

/// Write every `python/<rel>` overlay asset to `root/<rel>`.
fn extract_python_sources(overlay: &OverlayData, root: &Path) -> Result<(), String> {
    for (path, content) in &overlay.assets {
        let Some(rel) = path.strip_prefix("python/") else {
            continue;
        };
        let dest = root.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }
        std::fs::write(&dest, content).map_err(|e| format!("write {}: {e}", dest.display()))?;
    }
    Ok(())
}

/// Fail-fast alias conflict detection over a collected command table (§12.4).
///
/// An alias must not collide with a reserved verb, any canonical command name,
/// or any other command's alias. The Python registration path already guards
/// per-registry; this catches conflicts that only surface once both registries
/// are merged at pack time.
fn check_alias_conflicts(commands: &[CliCommandMeta]) -> Result<(), String> {
    let names: HashSet<&str> = commands.iter().map(|c| c.name.as_str()).collect();
    let mut claimed: HashSet<String> = HashSet::new();

    for cmd in commands {
        for alias in &cmd.aliases {
            if RESERVED_CLI_VERBS.contains(&alias.as_str()) {
                return Err(format!(
                    "alias '{alias}' for command '{}' collides with a reserved CLI verb",
                    cmd.name
                ));
            }
            if names.contains(alias.as_str()) && alias != &cmd.name {
                return Err(format!(
                    "alias '{alias}' for command '{}' collides with command name '{alias}'",
                    cmd.name
                ));
            }
            if !claimed.insert(alias.clone()) {
                return Err(format!(
                    "alias '{alias}' for command '{}' is already used by another command",
                    cmd.name
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(name: &str, aliases: &[&str]) -> CliCommandMeta {
        CliCommandMeta {
            name: name.to_string(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            help: String::new(),
            params: Vec::new(),
        }
    }

    #[test]
    fn parse_extracts_commands_ignoring_log_noise() {
        let stdout = b"[info] starting\n{\"type\":\"cli_metadata\",\"commands\":[{\"name\":\"export\",\"aliases\":[\"exp\"],\"help\":\"\",\"params\":[]}]}\n";
        let commands = parse_dump_stdout(stdout).expect("parse");
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "export");
        assert_eq!(commands[0].aliases, vec!["exp"]);
    }

    #[test]
    fn parse_errors_without_payload() {
        assert!(parse_dump_stdout(b"just logs\n").is_err());
    }

    #[test]
    fn conflict_detects_reserved_verb() {
        let err = check_alias_conflicts(&[cmd("export", &["list"])]).unwrap_err();
        assert!(err.contains("reserved"));
    }

    #[test]
    fn conflict_detects_alias_vs_name() {
        let err =
            check_alias_conflicts(&[cmd("export", &["sync"]), cmd("sync", &[])]).unwrap_err();
        assert!(err.contains("command name"));
    }

    #[test]
    fn conflict_detects_duplicate_alias() {
        let err =
            check_alias_conflicts(&[cmd("export", &["e"]), cmd("edit", &["e"])]).unwrap_err();
        assert!(err.contains("already used"));
    }

    #[test]
    fn conflict_allows_clean_table() {
        assert!(check_alias_conflicts(&[cmd("export", &["exp"]), cmd("sync", &["sy"])]).is_ok());
    }

    #[test]
    fn build_code_module_function() {
        let code = build_dump_code(Path::new("/tmp/x"), "app.main:run");
        assert!(code.contains("from app.main import run"));
        assert!(code.contains("run()"));
    }

    #[test]
    fn build_code_script_path() {
        let code = build_dump_code(Path::new("/tmp/x"), "main.py");
        assert!(code.contains("runpy.run_path"));
    }
}
