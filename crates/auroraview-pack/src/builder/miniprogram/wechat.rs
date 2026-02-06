//! WeChat MiniProgram Builder
//!
//! Builds WeChat Mini Programs from web assets.
//! Requires WeChat DevTools installed.

use crate::builder::common::{BuildContext, BuildOutput, BuildResult};
use crate::builder::traits::{Builder, BuilderCapability};
use crate::PackError;
use std::path::PathBuf;

/// WeChat MiniProgram builder
pub struct WeChatBuilder {
    /// WeChat App ID
    pub app_id: Option<String>,
    /// DevTools CLI path
    pub cli_path: Option<PathBuf>,
    /// Private key for CI upload
    pub private_key: Option<PathBuf>,
}

impl WeChatBuilder {
    pub fn new() -> Self {
        Self {
            app_id: None,
            cli_path: None,
            private_key: None,
        }
    }

    pub fn app_id(mut self, id: &str) -> Self {
        self.app_id = Some(id.into());
        self
    }

    pub fn cli_path(mut self, path: PathBuf) -> Self {
        self.cli_path = Some(path);
        self
    }

    pub fn private_key(mut self, path: PathBuf) -> Self {
        self.private_key = Some(path);
        self
    }

    /// Find WeChat DevTools CLI
    pub fn find_cli() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let paths = [
                r"C:\Program Files (x86)\Tencent\微信web开发者工具\cli.bat",
                r"C:\Program Files\Tencent\微信web开发者工具\cli.bat",
            ];
            for p in paths {
                let path = PathBuf::from(p);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let path = PathBuf::from("/Applications/wechatwebdevtools.app/Contents/MacOS/cli");
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    fn get_cli(&self) -> BuildResult<PathBuf> {
        self.cli_path
            .clone()
            .or_else(Self::find_cli)
            .ok_or_else(|| {
                PackError::Build(
                    "WeChat DevTools CLI not found. Please install WeChat DevTools.".into(),
                )
            })
    }
}

impl Default for WeChatBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder for WeChatBuilder {
    fn id(&self) -> &'static str {
        "wechat"
    }

    fn name(&self) -> &'static str {
        "WeChat MiniProgram"
    }

    fn targets(&self) -> &'static [&'static str] {
        &["wechat", "weixin", "wx", "miniprogram"]
    }

    fn capabilities(&self) -> Vec<BuilderCapability> {
        vec![BuilderCapability::Standalone, BuilderCapability::HotReload]
    }

    fn is_available(&self) -> bool {
        self.cli_path.is_some() || Self::find_cli().is_some()
    }

    fn required_tools(&self) -> Vec<&'static str> {
        vec!["wechat-devtools"]
    }

    fn check_tools(&self) -> BuildResult<()> {
        self.get_cli()?;
        Ok(())
    }

    fn validate(&self, ctx: &BuildContext) -> BuildResult<()> {
        // Check app_id
        if self.app_id.is_none() && ctx.config.app.identifier.is_none() {
            return Err(PackError::Config(
                "WeChat App ID is required. Set it in config or builder.".into(),
            ));
        }
        Ok(())
    }

    fn build(&self, ctx: &mut BuildContext) -> BuildResult<BuildOutput> {
        self.check_tools()?;
        let _cli = self.get_cli()?;

        let output_dir = ctx.output_dir.join("wechat-miniprogram");
        std::fs::create_dir_all(&output_dir)?;

        // TODO: Implement WeChat MiniProgram build
        // 1. Generate project.config.json
        // 2. Generate app.json with pages
        // 3. Transform web components to WXML/WXSS
        // 4. Copy assets
        // 5. Run CLI build/upload

        tracing::warn!("WeChat MiniProgram builder is not yet fully implemented");

        Err(PackError::Build(
            "WeChat MiniProgram builder not yet implemented".into(),
        ))
    }
}

/// WeChat project.config.json structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeChatProjectConfig {
    pub app_id: String,
    pub project_name: String,
    pub miniprogramroot: String,
    pub setting: WeChatProjectSetting,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeChatProjectSetting {
    pub url_check: bool,
    pub es6: bool,
    pub enhance: bool,
    pub postcss: bool,
    pub preload_background_data: bool,
    pub minified: bool,
    pub auto_audit: bool,
}
