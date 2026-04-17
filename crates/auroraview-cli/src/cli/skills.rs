//! `auroraview skills` subcommand
//!
//! Ships AuroraView's first-party skills (Markdown playbooks for AI agents) as
//! part of the CLI binary itself — no separate Python package, no extra install
//! step. Skills are embedded via `include_dir!` at build time.
//!
//! ## Usage
//!
//! ```bash
//! # Show available skills
//! auroraview skills list
//!
//! # Install into a known agent tool's skills directory
//! auroraview skills install --target claude
//! auroraview skills install --target cursor
//! auroraview skills install --target all
//!
//! # Install into an arbitrary path
//! auroraview skills install --path ./my-project/.claude/skills
//!
//! # Print where the bundled skills would be extracted
//! auroraview skills path
//! ```
//!
//! The current bundle contains:
//! - `qt-to-auroraview-migration` — convert Qt/PySide/PyQt projects to AuroraView
//!
//! Additional skills (e.g. `package-auroraview-app`) are added by dropping a new
//! `<skill-name>/SKILL.md` into `crates/auroraview-cli/skills/`. No code change
//! is needed — `include_dir!` picks them up at build time.

use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::{Path, PathBuf};

/// Bundled skills, embedded into the binary at compile time.
static BUNDLED_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skills");

#[derive(Args, Debug)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub command: SkillsCommand,
}

#[derive(Subcommand, Debug)]
pub enum SkillsCommand {
    /// List skills bundled with this CLI
    List,

    /// Install bundled skills into an agent tool's skills directory
    Install(InstallArgs),

    /// Print the on-disk path that `install` would write to for a target
    Path(PathArgs),
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Agent tool to install skills into (use `all` to hit every known target)
    #[arg(long, value_enum, conflicts_with = "path")]
    pub target: Option<AgentTarget>,

    /// Explicit directory to install into (e.g. `./.cursor/skills`)
    #[arg(long, conflicts_with = "target")]
    pub path: Option<PathBuf>,

    /// Install globally (into the user's home dir) instead of the current project
    #[arg(long, conflicts_with = "path")]
    pub global: bool,

    /// Overwrite existing skill files without prompting
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct PathArgs {
    /// Agent tool to resolve the path for
    #[arg(long, value_enum, default_value_t = AgentTarget::Claude)]
    pub target: AgentTarget,

    /// Resolve the global (user-home) path instead of the project-local one
    #[arg(long)]
    pub global: bool,
}

/// Agent tools we know how to install skills into.
///
/// Each variant maps to a conventional `<dot-dir>/skills/` layout used by that
/// agent. Project-local installs drop the skills next to the working dir;
/// `--global` installs them under the user's home directory.
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum AgentTarget {
    Claude,
    Cursor,
    Codex,
    Windsurf,
    Cline,
    Roo,
    Opencode,
    Augment,
    Kiro,
    Trae,
    Codebuddy,
    /// Agent-neutral `.agents/skills/` layout
    Agents,
    /// Install into every known target
    All,
}

impl AgentTarget {
    fn dot_dir(self) -> &'static str {
        match self {
            AgentTarget::Claude => ".claude",
            AgentTarget::Cursor => ".cursor",
            AgentTarget::Codex => ".codex",
            AgentTarget::Windsurf => ".windsurf",
            AgentTarget::Cline => ".cline",
            AgentTarget::Roo => ".roo",
            AgentTarget::Opencode => ".opencode",
            AgentTarget::Augment => ".augment",
            AgentTarget::Kiro => ".kiro",
            AgentTarget::Trae => ".trae",
            AgentTarget::Codebuddy => ".codebuddy",
            AgentTarget::Agents => ".agents",
            AgentTarget::All => "", // handled separately
        }
    }

    fn expand(self) -> Vec<AgentTarget> {
        match self {
            AgentTarget::All => vec![
                AgentTarget::Claude,
                AgentTarget::Cursor,
                AgentTarget::Codex,
                AgentTarget::Windsurf,
                AgentTarget::Cline,
                AgentTarget::Roo,
                AgentTarget::Opencode,
                AgentTarget::Augment,
                AgentTarget::Kiro,
                AgentTarget::Trae,
                AgentTarget::Codebuddy,
                AgentTarget::Agents,
            ],
            other => vec![other],
        }
    }
}

fn resolve_target_path(target: AgentTarget, global: bool) -> Result<PathBuf> {
    let base = if global {
        dirs::home_dir().ok_or_else(|| anyhow!("could not determine user home directory"))?
    } else {
        std::env::current_dir().context("failed to read current working directory")?
    };
    Ok(base.join(target.dot_dir()).join("skills"))
}

pub fn run_skills(args: SkillsArgs) -> Result<()> {
    match args.command {
        SkillsCommand::List => list_skills(),
        SkillsCommand::Install(install_args) => install_skills(install_args),
        SkillsCommand::Path(path_args) => {
            let path = resolve_target_path(path_args.target, path_args.global)?;
            println!("{}", path.display());
            Ok(())
        }
    }
}

fn list_skills() -> Result<()> {
    let skills = BUNDLED_SKILLS
        .dirs()
        .filter(|d| d.get_file(format!("{}/SKILL.md", d.path().display())).is_some())
        .collect::<Vec<_>>();

    if skills.is_empty() {
        println!("No skills bundled with this build.");
        return Ok(());
    }

    println!("Bundled skills ({}):", skills.len());
    for skill in skills {
        let name = skill
            .path()
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?");
        let description = skill
            .get_file(format!("{}/SKILL.md", skill.path().display()))
            .and_then(|f| f.contents_utf8())
            .map(extract_description)
            .unwrap_or_else(|| "(no description)".to_string());
        println!("  - {name}\n      {description}");
    }
    println!("\nInstall with: auroraview-cli skills install --target <agent>");
    Ok(())
}

fn extract_description(skill_md: &str) -> String {
    let mut in_frontmatter = false;
    for line in skill_md.lines() {
        if line.trim() == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter {
            if let Some(rest) = line.strip_prefix("description:") {
                return rest.trim().trim_matches('"').to_string();
            }
        }
    }
    "(no description)".to_string()
}

fn install_skills(args: InstallArgs) -> Result<()> {
    let targets: Vec<PathBuf> = match (args.path, args.target) {
        (Some(p), _) => vec![p],
        (None, Some(target)) => target
            .expand()
            .into_iter()
            .map(|t| resolve_target_path(t, args.global))
            .collect::<Result<Vec<_>>>()?,
        (None, None) => {
            return Err(anyhow!(
                "must pass either --target <agent> or --path <dir>; use --target all to hit every known agent"
            ));
        }
    };

    let mut total_written = 0usize;
    for dest in &targets {
        let written = copy_skills_into(dest, args.force)?;
        total_written += written;
        println!("✓ {} ({} files)", dest.display(), written);
    }

    println!(
        "\nInstalled {} file(s) across {} target(s).",
        total_written,
        targets.len()
    );
    Ok(())
}

fn copy_skills_into(dest: &Path, force: bool) -> Result<usize> {
    fs::create_dir_all(dest)
        .with_context(|| format!("failed to create destination {}", dest.display()))?;

    let mut written = 0;
    for entry in BUNDLED_SKILLS.entries() {
        written += extract_entry(entry, dest, force)?;
    }
    Ok(written)
}

fn extract_entry(entry: &include_dir::DirEntry<'_>, dest: &Path, force: bool) -> Result<usize> {
    match entry {
        include_dir::DirEntry::Dir(d) => {
            let target_dir = dest.join(
                d.path()
                    .file_name()
                    .ok_or_else(|| anyhow!("skill directory has no file name"))?,
            );
            fs::create_dir_all(&target_dir)?;
            let mut count = 0;
            for child in d.entries() {
                count += extract_entry_recursive(child, &target_dir, force)?;
            }
            Ok(count)
        }
        include_dir::DirEntry::File(f) => {
            // Top-level files (unusual) go straight into dest
            let target_file = dest.join(
                f.path()
                    .file_name()
                    .ok_or_else(|| anyhow!("skill file has no file name"))?,
            );
            write_if_needed(&target_file, f.contents(), force)
        }
    }
}

fn extract_entry_recursive(
    entry: &include_dir::DirEntry<'_>,
    dest: &Path,
    force: bool,
) -> Result<usize> {
    match entry {
        include_dir::DirEntry::Dir(d) => {
            let sub = dest.join(
                d.path()
                    .file_name()
                    .ok_or_else(|| anyhow!("nested skill directory has no file name"))?,
            );
            fs::create_dir_all(&sub)?;
            let mut count = 0;
            for child in d.entries() {
                count += extract_entry_recursive(child, &sub, force)?;
            }
            Ok(count)
        }
        include_dir::DirEntry::File(f) => {
            let target_file = dest.join(
                f.path()
                    .file_name()
                    .ok_or_else(|| anyhow!("nested skill file has no file name"))?,
            );
            write_if_needed(&target_file, f.contents(), force)
        }
    }
}

fn write_if_needed(target: &Path, contents: &[u8], force: bool) -> Result<usize> {
    if target.exists() && !force {
        let existing = fs::read(target).unwrap_or_default();
        if existing == contents {
            return Ok(0);
        }
        return Err(anyhow!(
            "{} already exists and differs; pass --force to overwrite",
            target.display()
        ));
    }
    fs::write(target, contents)
        .with_context(|| format!("failed to write {}", target.display()))?;
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn bundled_skills_are_non_empty() {
        assert!(BUNDLED_SKILLS.dirs().next().is_some(), "expected at least one skill dir");
    }

    #[test]
    fn install_writes_skill_md() {
        let tmp = TempDir::new().unwrap();
        let written = copy_skills_into(tmp.path(), false).unwrap();
        assert!(written >= 1);
        let skill_file = tmp
            .path()
            .join("qt-to-auroraview-migration")
            .join("SKILL.md");
        assert!(skill_file.exists());
    }

    #[test]
    fn install_twice_is_noop_without_force() {
        let tmp = TempDir::new().unwrap();
        copy_skills_into(tmp.path(), false).unwrap();
        let second = copy_skills_into(tmp.path(), false).unwrap();
        assert_eq!(second, 0, "re-running install should write zero files when contents match");
    }

    #[test]
    fn extract_description_reads_frontmatter() {
        let md = "---\nname: foo\ndescription: hello world\n---\n\n# body\n";
        assert_eq!(extract_description(md), "hello world");
    }
}
