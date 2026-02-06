//! iOS Builder

use super::common::{BuildContext, BuildOutput, BuildResult};
use super::traits::{Builder, BuilderCapability};
use crate::PackError;

pub struct IOSBuilder {
    pub team_id: Option<String>,
}

impl IOSBuilder {
    pub fn new() -> Self {
        Self { team_id: None }
    }
    pub fn team_id(mut self, id: &str) -> Self {
        self.team_id = Some(id.into());
        self
    }
}

impl Default for IOSBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder for IOSBuilder {
    fn id(&self) -> &'static str {
        "ios"
    }
    fn name(&self) -> &'static str {
        "iOS"
    }
    fn targets(&self) -> &'static [&'static str] {
        &["ios", "ipa", "iphone", "ipad"]
    }

    fn capabilities(&self) -> Vec<BuilderCapability> {
        vec![
            BuilderCapability::Standalone,
            BuilderCapability::CodeSign,
            BuilderCapability::AppStore,
        ]
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "macos")
    }
    fn required_tools(&self) -> Vec<&'static str> {
        vec!["xcodebuild", "xcrun"]
    }

    fn check_tools(&self) -> BuildResult<()> {
        if !self.is_available() {
            return Err(PackError::Build("iOS builds require macOS".into()));
        }
        Ok(())
    }

    fn validate(&self, _ctx: &BuildContext) -> BuildResult<()> {
        Ok(())
    }

    fn build(&self, _ctx: &mut BuildContext) -> BuildResult<BuildOutput> {
        self.check_tools()?;
        Err(PackError::Build("iOS builder not yet implemented".into()))
    }
}
