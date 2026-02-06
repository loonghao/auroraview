//! Alipay MiniProgram Builder

use crate::builder::common::{BuildContext, BuildOutput, BuildResult};
use crate::builder::traits::{Builder, BuilderCapability};
use crate::PackError;
use std::path::PathBuf;

/// Alipay MiniProgram builder
pub struct AlipayBuilder {
    pub app_id: Option<String>,
    pub cli_path: Option<PathBuf>,
}

impl AlipayBuilder {
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
        // Alipay DevTools paths
        #[cfg(target_os = "windows")]
        {
            let path = PathBuf::from(r"C:\Program Files\小程序开发者工具\cli.bat");
            if path.exists() {
                return Some(path);
            }
        }
        #[cfg(target_os = "macos")]
        {
            let path = PathBuf::from("/Applications/小程序开发者工具.app/Contents/MacOS/cli");
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
}

impl Default for AlipayBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder for AlipayBuilder {
    fn id(&self) -> &'static str {
        "alipay"
    }
    fn name(&self) -> &'static str {
        "Alipay MiniProgram"
    }
    fn targets(&self) -> &'static [&'static str] {
        &["alipay", "ali", "zhifubao"]
    }
    fn capabilities(&self) -> Vec<BuilderCapability> {
        vec![BuilderCapability::Standalone]
    }
    fn is_available(&self) -> bool {
        self.cli_path.is_some() || Self::find_cli().is_some()
    }
    fn required_tools(&self) -> Vec<&'static str> {
        vec!["alipay-devtools"]
    }

    fn validate(&self, _ctx: &BuildContext) -> BuildResult<()> {
        Ok(())
    }

    fn build(&self, _ctx: &mut BuildContext) -> BuildResult<BuildOutput> {
        Err(PackError::Build(
            "Alipay MiniProgram builder not yet implemented".into(),
        ))
    }
}
