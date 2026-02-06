//! Android Builder

use super::common::{BuildContext, BuildOutput, BuildResult};
use super::traits::{Builder, BuilderCapability};
use crate::PackError;

pub struct AndroidBuilder {
    pub package_name: Option<String>,
}

impl AndroidBuilder {
    pub fn new() -> Self {
        Self { package_name: None }
    }
    pub fn package_name(mut self, name: &str) -> Self {
        self.package_name = Some(name.into());
        self
    }
}

impl Default for AndroidBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder for AndroidBuilder {
    fn id(&self) -> &'static str {
        "android"
    }
    fn name(&self) -> &'static str {
        "Android"
    }
    fn targets(&self) -> &'static [&'static str] {
        &["android", "apk", "aab"]
    }

    fn capabilities(&self) -> Vec<BuilderCapability> {
        vec![
            BuilderCapability::Standalone,
            BuilderCapability::CodeSign,
            BuilderCapability::AppStore,
        ]
    }

    fn is_available(&self) -> bool {
        std::env::var("ANDROID_HOME").is_ok() || std::env::var("ANDROID_SDK_ROOT").is_ok()
    }
    fn required_tools(&self) -> Vec<&'static str> {
        vec!["gradle"]
    }

    fn validate(&self, _ctx: &BuildContext) -> BuildResult<()> {
        Ok(())
    }

    fn build(&self, _ctx: &mut BuildContext) -> BuildResult<BuildOutput> {
        Err(PackError::Build(
            "Android builder not yet implemented".into(),
        ))
    }
}
