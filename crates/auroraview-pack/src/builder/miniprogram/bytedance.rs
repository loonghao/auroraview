//! ByteDance (Douyin/TikTok) MiniProgram Builder

use crate::builder::common::{BuildContext, BuildOutput, BuildResult};
use crate::builder::traits::{Builder, BuilderCapability};
use crate::PackError;
use std::path::PathBuf;

/// ByteDance MiniProgram builder (Douyin, TikTok, Toutiao, etc.)
pub struct ByteDanceBuilder {
    pub app_id: Option<String>,
    pub cli_path: Option<PathBuf>,
}

impl ByteDanceBuilder {
    pub fn new() -> Self {
        Self {
            app_id: None,
            cli_path: None,
        }
    }

    pub fn app_id(mut self, id: &str) -> Self {
        self.app_id = Some(id.into());
        self
    }

    pub fn find_cli() -> Option<PathBuf> {
        // ByteDance DevTools paths
        #[cfg(target_os = "windows")]
        {
            // Try common installation paths
            let paths = [
                r"C:\Program Files\抖音开发者工具\cli.bat",
                r"C:\Program Files\ByteDanceMicroApp\cli.bat",
            ];
            for p in &paths {
                let path = PathBuf::from(p);
                if path.exists() {
                    return Some(path);
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            let paths = [
                "/Applications/抖音开发者工具.app/Contents/MacOS/cli",
                "/Applications/ByteDanceMicroApp.app/Contents/MacOS/cli",
            ];
            for p in &paths {
                let path = PathBuf::from(p);
                if path.exists() {
                    return Some(path);
                }
            }
        }
        None
    }
}

impl Default for ByteDanceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder for ByteDanceBuilder {
    fn id(&self) -> &'static str {
        "bytedance"
    }
    fn name(&self) -> &'static str {
        "ByteDance MiniProgram"
    }
    fn targets(&self) -> &'static [&'static str] {
        &["bytedance", "douyin", "tiktok", "toutiao"]
    }
    fn capabilities(&self) -> Vec<BuilderCapability> {
        vec![BuilderCapability::Standalone]
    }
    fn is_available(&self) -> bool {
        self.cli_path.is_some() || Self::find_cli().is_some()
    }
    fn required_tools(&self) -> Vec<&'static str> {
        vec!["bytedance-devtools"]
    }

    fn validate(&self, _ctx: &BuildContext) -> BuildResult<()> {
        Ok(())
    }

    fn build(&self, _ctx: &mut BuildContext) -> BuildResult<BuildOutput> {
        Err(PackError::Build(
            "ByteDance MiniProgram builder not yet implemented".into(),
        ))
    }
}
