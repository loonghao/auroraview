//! WeChat MiniProgram Builder
//!
//! Builds WeChat Mini Programs from web assets.
//! Requires WeChat DevTools installed.

use std::path::PathBuf;

use crate::builder::common::{BuildContext, BuildOutput, BuildResult};
use crate::builder::traits::{Builder, BuilderCapability};
use crate::PackError;

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
        let cli = self.get_cli()?;

        let app_id = self
            .app_id
            .clone()
            .or_else(|| ctx.config.app.identifier.clone())
            .ok_or_else(|| PackError::Config("WeChat App ID is required".into()))?;

        let output_dir = ctx.output_dir.join("wechat-miniprogram");
        std::fs::create_dir_all(&output_dir)?;

        // 1. Generate project.config.json
        let project_config = WeChatProjectConfig {
            app_id: app_id.clone(),
            project_name: if ctx.config.app.name.is_empty() {
                "AuroraView App".into()
            } else {
                ctx.config.app.name.clone()
            },
            miniprogramroot: "./".into(),
            setting: WeChatProjectSetting {
                url_check: true,
                es6: true,
                enhance: true,
                postcss: true,
                preload_background_data: false,
                minified: true,
                auto_audit: false,
            },
        };

        let config_json = serde_json::to_string_pretty(&project_config).map_err(|e| {
            PackError::Build(format!("Failed to serialize project.config.json: {}", e))
        })?;
        std::fs::write(output_dir.join("project.config.json"), &config_json)?;

        // 2. Generate app.json with pages
        let app_json = serde_json::json!({
            "pages": ["pages/index/index"],
            "window": {
                "backgroundTextStyle": "light",
                "navigationBarBackgroundColor": "#fff",
                "navigationBarTitleText": project_config.project_name,
                "navigationBarTextStyle": "black"
            }
        });
        let app_json_str = serde_json::to_string_pretty(&app_json)
            .map_err(|e| PackError::Build(format!("Failed to serialize app.json: {}", e)))?;
        std::fs::write(output_dir.join("app.json"), &app_json_str)?;

        // 3. Generate minimal app.js / app.wxss
        std::fs::write(output_dir.join("app.js"), "App({})")?;
        std::fs::write(output_dir.join("app.wxss"), "")?;

        // 4. Create pages/index directory and copy frontend assets as web-view
        let index_dir = output_dir.join("pages").join("index");
        std::fs::create_dir_all(&index_dir)?;

        // If frontend bundle exists, embed via <web-view>
        let index_wxml = if ctx.frontend.is_some() {
            "<web-view src=\"/index.html\"></web-view>".into()
        } else {
            "<view class=\"container\"><text>AuroraView App</text></view>".to_string()
        };

        std::fs::write(index_dir.join("index.wxml"), &index_wxml)?;
        std::fs::write(index_dir.join("index.wxss"), "")?;
        std::fs::write(index_dir.join("index.js"), "Page({})")?;
        std::fs::write(index_dir.join("index.json"), "{}")?;

        // 5. Copy additional assets from build context
        for (path, content) in &ctx.assets {
            let dest = output_dir.join(path);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, content)?;
        }

        tracing::info!(
            "WeChat MiniProgram project generated at {}",
            output_dir.display()
        );
        tracing::info!("Use '{}' to preview or upload", cli.display());

        Ok(BuildOutput::new(output_dir, "wechat-miniprogram")
            .with_assets(ctx.assets.len())
            .with_duration(ctx.start_time.elapsed())
            .with_info("app_id", &app_id)
            .with_info("cli_path", &cli.to_string_lossy()))
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
