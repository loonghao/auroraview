//! macOS Builder

use super::common::{BuildContext, BuildOutput, BuildResult};
use super::traits::{Builder, BuilderCapability};
use crate::PackError;

pub struct MacBuilder;

impl MacBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder for MacBuilder {
    fn id(&self) -> &'static str {
        "mac"
    }
    fn name(&self) -> &'static str {
        "macOS"
    }
    fn targets(&self) -> &'static [&'static str] {
        &["mac", "macos", "darwin", "osx"]
    }

    fn capabilities(&self) -> Vec<BuilderCapability> {
        vec![
            BuilderCapability::Standalone,
            BuilderCapability::Installer,
            BuilderCapability::CodeSign,
            BuilderCapability::Notarize,
            BuilderCapability::PythonEmbed,
            BuilderCapability::Extensions,
            BuilderCapability::AppStore,
        ]
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "macos")
    }
    fn required_tools(&self) -> Vec<&'static str> {
        vec!["codesign", "hdiutil"]
    }

    fn validate(&self, _ctx: &BuildContext) -> BuildResult<()> {
        Ok(())
    }

    fn build(&self, _ctx: &mut BuildContext) -> BuildResult<BuildOutput> {
        Err(PackError::Build("macOS builder not yet implemented".into()))
    }
}
