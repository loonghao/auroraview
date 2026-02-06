//! Hook system for pack lifecycle customization
//!
//! Hooks allow running custom commands or scripts at various points
//! during the pack process.
//!
//! Supports vx integration: when `use_vx` is enabled, commands are automatically
//! prefixed with `vx` to leverage the unified tool management system.

#![allow(dead_code)]

use crate::{vx_tool::VxTool, PackError, PackResult};
use std::collections::HashMap;
use std::process::{Command, Stdio};

/// Hook stage in pack lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookStage {
    /// Before any pack operation starts
    BeforePack,
    /// Before assets are collected
    BeforeCollect,
    /// After assets are collected
    AfterCollect,
    /// Before overlay is written
    BeforeOverlay,
    /// After overlay is written
    AfterOverlay,
    /// After pack completes
    AfterPack,
    /// On error (for cleanup)
    OnError,
}

impl HookStage {
    /// Get stage name for config lookup
    pub fn config_name(&self) -> &'static str {
        match self {
            Self::BeforePack => "before_pack",
            Self::BeforeCollect => "before_collect",
            Self::AfterCollect => "after_collect",
            Self::BeforeOverlay => "before_overlay",
            Self::AfterOverlay => "after_overlay",
            Self::AfterPack => "after_pack",
            Self::OnError => "on_error",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "before_pack" => Some(Self::BeforePack),
            "before_collect" => Some(Self::BeforeCollect),
            "after_collect" => Some(Self::AfterCollect),
            "before_overlay" => Some(Self::BeforeOverlay),
            "after_overlay" => Some(Self::AfterOverlay),
            "after_pack" => Some(Self::AfterPack),
            "on_error" => Some(Self::OnError),
            _ => None,
        }
    }
}

impl std::fmt::Display for HookStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.config_name())
    }
}

/// Hook configuration
#[derive(Debug, Clone)]
pub struct HookConfig {
    /// Commands to run at each stage
    pub commands: HashMap<HookStage, Vec<String>>,
    /// Working directory for commands
    pub working_dir: Option<std::path::PathBuf>,
    /// Environment variables to set
    pub env: HashMap<String, String>,
    /// Whether to fail on hook errors
    pub fail_on_error: bool,
    /// Timeout for each command (seconds)
    pub timeout_secs: Option<u64>,
    /// Whether to run commands via vx automatically
    pub use_vx: bool,
    /// Vx-specific hook commands (run directly with vx)
    pub vx_commands: HashMap<HookStage, Vec<String>>,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            commands: HashMap::new(),
            working_dir: None,
            env: HashMap::new(),
            fail_on_error: true,
            timeout_secs: Some(300), // 5 minutes default
            use_vx: false,
            vx_commands: HashMap::new(),
        }
    }
}

impl HookConfig {
    /// Create a new hook config
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a command for a stage
    pub fn add_command(&mut self, stage: HookStage, command: &str) {
        self.commands
            .entry(stage)
            .or_default()
            .push(command.to_string());
    }

    /// Add a vx-specific command for a stage
    pub fn add_vx_command(&mut self, stage: HookStage, command: &str) {
        self.vx_commands
            .entry(stage)
            .or_default()
            .push(command.to_string());
    }

    /// Set working directory
    pub fn working_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Set environment variable
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// Set fail on error behavior
    pub fn fail_on_error(mut self, fail: bool) -> Self {
        self.fail_on_error = fail;
        self
    }

    /// Set whether to use vx for running commands
    pub fn use_vx(mut self, use_vx: bool) -> Self {
        self.use_vx = use_vx;
        self
    }

    /// Check if stage has any commands
    pub fn has_commands(&self, stage: HookStage) -> bool {
        let has_regular = self
            .commands
            .get(&stage)
            .map(|cmds| !cmds.is_empty())
            .unwrap_or(false);
        let has_vx = self
            .vx_commands
            .get(&stage)
            .map(|cmds| !cmds.is_empty())
            .unwrap_or(false);
        has_regular || has_vx
    }
}

/// Hook runner for executing lifecycle hooks
pub struct HookRunner {
    config: HookConfig,
    vx_tool: Option<VxTool>,
}

impl HookRunner {
    /// Create a new hook runner
    pub fn new(config: HookConfig) -> Self {
        // Initialize vx tool if needed
        let vx_tool = if config.use_vx || !config.vx_commands.is_empty() {
            match VxTool::new() {
                Ok(vx) => {
                    tracing::info!("Hook runner initialized with vx {}", vx.version());
                    Some(vx)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize vx tool for hooks: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self { config, vx_tool }
    }

    /// Create a new hook runner without vx support
    pub fn new_without_vx(config: HookConfig) -> Self {
        Self {
            config,
            vx_tool: None,
        }
    }

    /// Run hooks for a stage
    pub fn run(&self, stage: HookStage) -> PackResult<()> {
        // Collect commands from both sources
        let regular_commands: Vec<&String> = self
            .config
            .commands
            .get(&stage)
            .map(|cmds| cmds.iter().collect())
            .unwrap_or_default();

        let vx_specific_commands: Vec<&String> = self
            .config
            .vx_commands
            .get(&stage)
            .map(|cmds| cmds.iter().collect())
            .unwrap_or_default();

        let total_commands = regular_commands.len() + vx_specific_commands.len();
        if total_commands == 0 {
            return Ok(());
        }

        tracing::info!("Running {} hooks ({} commands)", stage, total_commands);

        // Run regular commands (via vx if enabled)
        for cmd in regular_commands {
            let result = if self.config.use_vx {
                self.run_vx_command(cmd)
            } else {
                self.run_command(cmd)
            };

            match result {
                Ok(_) => {
                    tracing::debug!("Hook command succeeded: {}", cmd);
                }
                Err(e) => {
                    if self.config.fail_on_error {
                        return Err(e);
                    }
                    tracing::warn!("Hook command failed (continuing): {}: {}", cmd, e);
                }
            }
        }

        // Run vx-specific commands
        for cmd in vx_specific_commands {
            match self.run_vx_command(cmd) {
                Ok(_) => {
                    tracing::debug!("Hook vx command succeeded: {}", cmd);
                }
                Err(e) => {
                    if self.config.fail_on_error {
                        return Err(e);
                    }
                    tracing::warn!("Hook vx command failed (continuing): {}: {}", cmd, e);
                }
            }
        }

        Ok(())
    }

    /// Run a single command
    fn run_command(&self, cmd: &str) -> PackResult<()> {
        tracing::debug!("Running hook command: {}", cmd);

        let status = if cfg!(target_os = "windows") {
            let mut command = Command::new("cmd");
            command.args(["/C", cmd]);

            if let Some(ref dir) = self.config.working_dir {
                command.current_dir(dir);
            }

            for (key, value) in &self.config.env {
                command.env(key, value);
            }

            command.stdout(Stdio::inherit());
            command.stderr(Stdio::inherit());
            command.status()
        } else {
            let mut command = Command::new("sh");
            command.args(["-c", cmd]);

            if let Some(ref dir) = self.config.working_dir {
                command.current_dir(dir);
            }

            for (key, value) in &self.config.env {
                command.env(key, value);
            }

            command.stdout(Stdio::inherit());
            command.stderr(Stdio::inherit());
            command.status()
        }
        .map_err(|e| PackError::Config(format!("Failed to run hook command '{}': {}", cmd, e)))?;

        if !status.success() {
            return Err(PackError::Config(format!(
                "Hook command failed (exit code {:?}): {}",
                status.code(),
                cmd
            )));
        }

        Ok(())
    }

    /// Run a command through vx
    fn run_vx_command(&self, cmd: &str) -> PackResult<()> {
        let vx = match &self.vx_tool {
            Some(vx) => vx,
            None => {
                return Err(PackError::Config(
                    "Vx tool not available. Cannot run vx hook commands.".to_string(),
                ));
            }
        };

        tracing::debug!("Running hook vx command: {}", cmd);

        // Parse the command to extract the tool and arguments
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Err(PackError::Config("Empty vx command".to_string()));
        }

        let tool = parts[0];
        let args = &parts[1..];

        let status = Command::new(vx.path())
            .arg(tool)
            .args(args)
            .current_dir(
                self.config
                    .working_dir
                    .as_deref()
                    .unwrap_or_else(|| std::path::Path::new(".")),
            )
            .envs(&self.config.env)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| {
                PackError::Config(format!("Failed to run vx hook command '{}': {}", cmd, e))
            })?;

        if !status.success() {
            return Err(PackError::Config(format!(
                "Vx hook command failed (exit code {:?}): {}",
                status.code(),
                cmd
            )));
        }

        Ok(())
    }

    /// Run hooks with context variables
    pub fn run_with_context(
        &self,
        stage: HookStage,
        context: &HashMap<String, String>,
    ) -> PackResult<()> {
        // Collect commands from both sources
        let regular_commands: Vec<&String> = self
            .config
            .commands
            .get(&stage)
            .map(|cmds| cmds.iter().collect())
            .unwrap_or_default();

        let vx_specific_commands: Vec<&String> = self
            .config
            .vx_commands
            .get(&stage)
            .map(|cmds| cmds.iter().collect())
            .unwrap_or_default();

        let total_commands = regular_commands.len() + vx_specific_commands.len();
        if total_commands == 0 {
            return Ok(());
        }

        tracing::info!("Running {} hooks ({} commands)", stage, total_commands);

        // Run regular commands (via vx if enabled)
        for cmd in regular_commands {
            let expanded_cmd = self.expand_variables(cmd, context);
            let result = if self.config.use_vx {
                self.run_vx_command(&expanded_cmd)
            } else {
                self.run_command(&expanded_cmd)
            };

            match result {
                Ok(_) => {
                    tracing::debug!("Hook command succeeded: {}", expanded_cmd);
                }
                Err(e) => {
                    if self.config.fail_on_error {
                        return Err(e);
                    }
                    tracing::warn!("Hook command failed (continuing): {}: {}", expanded_cmd, e);
                }
            }
        }

        // Run vx-specific commands
        for cmd in vx_specific_commands {
            let expanded_cmd = self.expand_variables(cmd, context);
            match self.run_vx_command(&expanded_cmd) {
                Ok(_) => {
                    tracing::debug!("Hook vx command succeeded: {}", expanded_cmd);
                }
                Err(e) => {
                    if self.config.fail_on_error {
                        return Err(e);
                    }
                    tracing::warn!(
                        "Hook vx command failed (continuing): {}: {}",
                        expanded_cmd,
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// Expand variables in a command string
    fn expand_variables(&self, cmd: &str, context: &HashMap<String, String>) -> String {
        let mut result = cmd.to_string();

        for (key, value) in context {
            let pattern = format!("${{{}}}", key);
            result = result.replace(&pattern, value);

            let pattern_simple = format!("${}", key);
            result = result.replace(&pattern_simple, value);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_stage_names() {
        assert_eq!(HookStage::BeforePack.config_name(), "before_pack");
        assert_eq!(HookStage::AfterPack.config_name(), "after_pack");
    }

    #[test]
    fn test_hook_stage_parse() {
        assert_eq!(
            HookStage::from_str("before_pack"),
            Some(HookStage::BeforePack)
        );
        assert_eq!(HookStage::from_str("invalid"), None);
    }

    #[test]
    fn test_hook_config_builder() {
        let mut config = HookConfig::new()
            .working_dir(std::path::PathBuf::from("/tmp"))
            .env("MY_VAR", "my_value")
            .fail_on_error(false)
            .use_vx(true);

        config.add_command(HookStage::BeforePack, "echo hello");
        config.add_vx_command(HookStage::BeforeCollect, "uv pip install requests");

        assert!(config.has_commands(HookStage::BeforePack));
        assert!(config.has_commands(HookStage::BeforeCollect));
        assert!(!config.has_commands(HookStage::AfterPack));
        assert!(!config.fail_on_error);
        assert!(config.use_vx);
    }

    #[test]
    fn test_variable_expansion() {
        let runner = HookRunner::new(HookConfig::default());
        let mut context = HashMap::new();
        context.insert("NAME".to_string(), "test".to_string());
        context.insert("VERSION".to_string(), "1.0.0".to_string());

        let cmd = "echo ${NAME} version ${VERSION}";
        let expanded = runner.expand_variables(cmd, &context);
        assert_eq!(expanded, "echo test version 1.0.0");
    }
}
