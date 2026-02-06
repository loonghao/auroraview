//! Linux Builder

use super::common::{BuildContext, BuildOutput, BuildResult};
use super::traits::{Builder, BuilderCapability};
use crate::PackError;

pub struct LinuxBuilder;

impl LinuxBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder for LinuxBuilder {
    fn id(&self) -> &'static str {
        "linux"
    }
    fn name(&self) -> &'static str {
        "Linux"
    }
    fn targets(&self) -> &'static [&'static str] {
        &["linux", "appimage", "deb", "rpm"]
    }

    fn capabilities(&self) -> Vec<BuilderCapability> {
        vec![
            BuilderCapability::Standalone,
            BuilderCapability::Installer,
            BuilderCapability::Portable,
            BuilderCapability::PythonEmbed,
            BuilderCapability::Extensions,
        ]
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "linux")
    }

    fn validate(&self, _ctx: &BuildContext) -> BuildResult<()> {
        Ok(())
    }

    fn build(&self, _ctx: &mut BuildContext) -> BuildResult<BuildOutput> {
        Err(PackError::Build("Linux builder not yet implemented".into()))
    }
}
