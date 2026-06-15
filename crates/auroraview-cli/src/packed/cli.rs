//! Packed headless CLI mode entry and argv classification (RFC 0018 §4).
//!
//! A packed executable normally opens its GUI window on launch and ignores
//! argv. RFC 0018 adds a *headless CLI* path so the same `app.exe` can also
//! run registered commands from a terminal. To avoid hijacking ordinary
//! launches (file associations, drag-and-drop paths, protocol activation), the
//! CLI path is triggered **only** by an explicit reserved verb or flag as the
//! first argument (§4.3):
//!
//! | First argument                 | Result                |
//! |--------------------------------|-----------------------|
//! | (none)                         | GUI                   |
//! | `some/file.proj`               | GUI (open path later) |
//! | `run <cmd> [--k v ...]`        | CLI: invoke a command |
//! | `list [--json]`                | CLI: list commands    |
//! | `-h` / `--help`                | CLI: print help       |
//! | `-V` / `--version`             | CLI: print version    |
//!
//! Everything else falls through to GUI, so a bare path or unknown flag never
//! gets misread as a command.

use anyhow::Result;

/// How a packed executable was invoked, after classifying argv (§4.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackedInvocation {
    /// Open the GUI window (default / file-association / drag-drop launch).
    Gui,
    /// Run the headless CLI path with the given arguments (excluding argv[0]).
    Cli(Vec<String>),
}

/// Reserved subcommand verbs that trigger the CLI path (§4.3, decision #1).
const RESERVED_VERBS: &[&str] = &["run", "list"];

/// Classify a packed invocation from the full process argument list.
///
/// `args` is expected to include argv[0] (the executable path), matching
/// `std::env::args()`. Only the first *real* argument decides the mode.
pub fn classify_packed_invocation<I, S>(args: I) -> PackedInvocation
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    // Drop argv[0]; the first remaining token is the discriminator.
    let mut rest = args.into_iter().map(Into::into);
    let _exe = rest.next();

    let first = match rest.next() {
        Some(tok) => tok,
        None => return PackedInvocation::Gui, // bare launch → GUI
    };

    let is_cli_trigger = RESERVED_VERBS.contains(&first.as_str())
        || matches!(first.as_str(), "-h" | "--help" | "-V" | "--version");

    if !is_cli_trigger {
        // Bare path / unknown flag / anything else → GUI (§4.3).
        return PackedInvocation::Gui;
    }

    // Re-assemble the CLI argument vector (the trigger token + remainder).
    let mut cli_args = Vec::new();
    cli_args.push(first);
    cli_args.extend(rest);
    PackedInvocation::Cli(cli_args)
}

/// Run the packed headless CLI path (§7).
///
/// `cli_args` is the classified argument vector from
/// [`classify_packed_invocation`] (the reserved verb/flag followed by its
/// arguments, without argv[0]).
///
/// This currently implements the step-2 skeleton: console attach, `-V`/
/// `--version`, and a placeholder `-h`. `run` / `list` rendering and command
/// invocation land in later RFC steps.
pub fn run_packed_cli(cli_args: Vec<String>) -> Result<()> {
    // Reconnect stdio to the launching terminal before printing anything
    // (§3.2). Harmless if there is no parent console.
    super::attach_parent_console();

    let first = cli_args.first().map(String::as_str).unwrap_or("");

    match first {
        "-V" | "--version" => {
            println!("auroraview {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "-h" | "--help" => {
            // Placeholder until §13.4 overlay-driven rendering lands (step 6).
            print_help_placeholder();
            Ok(())
        }
        "run" | "list" => {
            // Wired in later RFC steps (7 and 6 respectively).
            eprintln!("auroraview: '{first}' is not implemented yet");
            std::process::exit(2);
        }
        _ => {
            eprintln!("auroraview: unknown CLI invocation: {first}");
            std::process::exit(2);
        }
    }
}

/// Minimal help text shown until the overlay-driven command list is wired up.
fn print_help_placeholder() {
    println!("AuroraView packed application");
    println!();
    println!("USAGE:");
    println!("    app.exe                       Launch the GUI window");
    println!("    app.exe run <command> [args]  Run a registered command");
    println!("    app.exe list [--json]         List available commands");
    println!("    app.exe -h, --help            Show this help");
    println!("    app.exe -V, --version         Show version");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(args: &[&str]) -> PackedInvocation {
        classify_packed_invocation(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn bare_launch_is_gui() {
        assert_eq!(classify(&["app.exe"]), PackedInvocation::Gui);
    }

    #[test]
    fn no_args_at_all_is_gui() {
        let empty: Vec<String> = Vec::new();
        assert_eq!(classify_packed_invocation(empty), PackedInvocation::Gui);
    }

    #[test]
    fn bare_file_path_is_gui() {
        // File association / drag-drop must never be read as a command.
        assert_eq!(
            classify(&["app.exe", "some/file.proj"]),
            PackedInvocation::Gui
        );
        assert_eq!(
            classify(&["app.exe", "C:\\docs\\thing.proj"]),
            PackedInvocation::Gui
        );
    }

    #[test]
    fn unknown_leading_flag_is_gui() {
        // Only the reserved flags trigger CLI; anything else stays GUI.
        assert_eq!(classify(&["app.exe", "--open"]), PackedInvocation::Gui);
        assert_eq!(classify(&["app.exe", "-x"]), PackedInvocation::Gui);
    }

    #[test]
    fn run_verb_triggers_cli_with_args() {
        assert_eq!(
            classify(&["app.exe", "run", "export", "--path", "./out"]),
            PackedInvocation::Cli(vec![
                "run".into(),
                "export".into(),
                "--path".into(),
                "./out".into(),
            ])
        );
    }

    #[test]
    fn list_verb_triggers_cli() {
        assert_eq!(
            classify(&["app.exe", "list", "--json"]),
            PackedInvocation::Cli(vec!["list".into(), "--json".into()])
        );
    }

    #[test]
    fn help_and_version_flags_trigger_cli() {
        for flag in ["-h", "--help", "-V", "--version"] {
            assert_eq!(
                classify(&["app.exe", flag]),
                PackedInvocation::Cli(vec![flag.into()]),
                "flag {flag} should trigger CLI",
            );
        }
    }

    #[test]
    fn reserved_verb_only_matters_as_first_token() {
        // A path that merely contains 'run' later is still GUI.
        assert_eq!(
            classify(&["app.exe", "myfile.proj", "run"]),
            PackedInvocation::Gui
        );
    }
}
