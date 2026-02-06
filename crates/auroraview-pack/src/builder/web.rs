//! Web Builder (PWA/Static)

use super::common::{BuildContext, BuildOutput, BuildResult};
use super::traits::{Builder, BuilderCapability};
use crate::PackError;

pub struct WebBuilder;

impl WebBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WebBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder for WebBuilder {
    fn id(&self) -> &'static str {
        "web"
    }
    fn name(&self) -> &'static str {
        "Web"
    }
    fn targets(&self) -> &'static [&'static str] {
        &["web", "pwa", "static"]
    }

    fn capabilities(&self) -> Vec<BuilderCapability> {
        vec![BuilderCapability::Standalone, BuilderCapability::HotReload]
    }

    fn is_available(&self) -> bool {
        true
    }

    fn validate(&self, _ctx: &BuildContext) -> BuildResult<()> {
        Ok(())
    }

    fn build(&self, _ctx: &mut BuildContext) -> BuildResult<BuildOutput> {
        Err(PackError::Build("Web builder not yet implemented".into()))
    }
}
