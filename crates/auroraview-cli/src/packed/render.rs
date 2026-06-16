//! Render the embedded CLI command table for `-h` / `list` (RFC 0018 §13.4).
//!
//! These functions consume the [`CliCommandMeta`] table read from the overlay
//! and produce the human-readable (`-h`, `list`) and machine-readable
//! (`list --json`) views. They never touch Python — the table was harvested at
//! pack time (§5) — so rendering is allocation-only and instant.

use auroraview_pack::CliCommandMeta;

/// Render the `-h` / `--help` view: usage banner plus every command with its
/// aliases, help text, and per-parameter detail.
///
/// `program` is the actual executable name (RFC 0018 §4) so the usage lines
/// match what the user typed, instead of a hardcoded `app`.
pub fn render_help(program: &str, commands: &[CliCommandMeta]) -> String {
    let mut out = String::new();
    out.push_str("USAGE:\n");
    out.push_str(&format!("    {program} run <command> [--key value ...]\n"));
    out.push_str(&format!("    {program} list [--json]\n"));
    out.push_str(&format!("    {program} -h | --help\n"));
    out.push_str(&format!("    {program} -V | --version\n"));

    if commands.is_empty() {
        out.push_str("\nNo CLI commands are available in this application.\n");
        return out;
    }

    out.push_str("\nCOMMANDS:\n");
    for cmd in commands {
        out.push_str(&format!("    {}", command_title(cmd)));
        if !cmd.help.is_empty() {
            out.push_str(&format!("   {}", cmd.help));
        }
        out.push('\n');
        for param in &cmd.params {
            out.push_str(&format!("        {}\n", param_line(param)));
        }
    }
    out
}

/// Render the `list` view: one command per line (`name (alias) — help`).
pub fn render_list(commands: &[CliCommandMeta]) -> String {
    if commands.is_empty() {
        return "No CLI commands are available in this application.\n".to_string();
    }
    let mut out = String::new();
    for cmd in commands {
        out.push_str(&command_title(cmd));
        if !cmd.help.is_empty() {
            out.push_str(&format!("   {}", cmd.help));
        }
        out.push('\n');
    }
    out
}

/// Render `list --json`: the §13.2 metadata array, verbatim and machine-readable.
pub fn render_list_json(commands: &[CliCommandMeta]) -> String {
    serde_json::to_string_pretty(commands).unwrap_or_else(|_| "[]".to_string())
}

/// `name (alias1, alias2)` or just `name` when there are no aliases.
fn command_title(cmd: &CliCommandMeta) -> String {
    if cmd.aliases.is_empty() {
        cmd.name.clone()
    } else {
        format!("{} ({})", cmd.name, cmd.aliases.join(", "))
    }
}

/// `--name <type>  [required]/[default X]  help`.
fn param_line(param: &auroraview_pack::CliParamMeta) -> String {
    let tag = if param.required {
        "[required]".to_string()
    } else if param.default.is_null() {
        "[optional]".to_string()
    } else {
        format!("[default {}]", render_default(&param.default))
    };

    let mut line = format!("--{} <{}>  {}", param.name, param.r#type, tag);
    if !param.help.is_empty() {
        line.push_str(&format!("  {}", param.help));
    }
    line
}

/// Compact one-line rendering of a JSON default value for help text.
fn render_default(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auroraview_pack::CliParamMeta;
    use serde_json::json;

    fn sample() -> Vec<CliCommandMeta> {
        vec![CliCommandMeta {
            name: "export-document-image".to_string(),
            aliases: vec!["exi".to_string()],
            help: "Export the current document as an image".to_string(),
            params: vec![
                CliParamMeta {
                    name: "path".to_string(),
                    r#type: "str".to_string(),
                    required: true,
                    default: json!(null),
                    help: "Output directory".to_string(),
                },
                CliParamMeta {
                    name: "dpi".to_string(),
                    r#type: "int".to_string(),
                    required: false,
                    default: json!(300),
                    help: "Resolution (DPI)".to_string(),
                },
            ],
        }]
    }

    #[test]
    fn help_includes_command_aliases_and_params() {
        let out = render_help("myapp", &sample());
        assert!(out.contains("USAGE:"));
        // The usage banner uses the real program name, not a hardcoded `app`.
        assert!(out.contains("myapp run <command>"));
        assert!(out.contains("export-document-image (exi)"));
        assert!(out.contains("--path <str>  [required]"));
        assert!(out.contains("--dpi <int>  [default 300]"));
        assert!(out.contains("Output directory"));
    }

    #[test]
    fn help_handles_empty_table() {
        let out = render_help("myapp", &[]);
        assert!(out.contains("No CLI commands"));
    }

    #[test]
    fn list_one_line_per_command() {
        let out = render_list(&sample());
        assert_eq!(out.lines().count(), 1);
        assert!(out.contains("export-document-image (exi)"));
        assert!(out.contains("Export the current document"));
    }

    #[test]
    fn list_json_round_trips() {
        let out = render_list_json(&sample());
        let parsed: Vec<CliCommandMeta> = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed, sample());
    }

    #[test]
    fn list_json_empty_is_array() {
        assert_eq!(render_list_json(&[]), "[]");
    }

    #[test]
    fn title_without_aliases_is_bare_name() {
        let cmd = CliCommandMeta {
            name: "sync".to_string(),
            aliases: vec![],
            help: String::new(),
            params: vec![],
        };
        assert_eq!(command_title(&cmd), "sync");
    }
}
