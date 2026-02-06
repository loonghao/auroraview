//! Builder module - Platform-specific build implementations
//!
//! This module contains the core build logic for each platform:
//! - `win` - Windows (WebView2, MSIX, portable)
//! - `mac` - macOS (app bundle, DMG, pkg)
//! - `linux` - Linux (AppImage, deb, rpm)
//! - `ios` - iOS (Xcode project, IPA)
//! - `android` - Android (Gradle project, APK/AAB)
//! - `web` - Web (PWA, static)
//! - `miniprogram` - Mini programs (WeChat, Alipay, ByteDance)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                         Packer                              │
//! │  (Orchestration, config migration, plugin hooks)            │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      BuildContext                           │
//! │  (Shared state, assets, overlay, temp files)                │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!           ┌──────────────────┼──────────────────┐
//!           ▼                  ▼                  ▼
//!    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//!    │  WinBuilder │    │  MacBuilder │    │  IOSBuilder │
//!    └─────────────┘    └─────────────┘    └─────────────┘
//! ```

pub mod android;
pub mod common;
pub mod ios;
pub mod linux;
pub mod mac;
pub mod miniprogram;
pub mod traits;
pub mod web;
pub mod win;

pub use common::{BuildContext, BuildOutput, BuildResult};
pub use traits::{Builder, BuilderCapability};

// Re-export platform builders
pub use android::AndroidBuilder;
pub use ios::IOSBuilder;
pub use linux::LinuxBuilder;
pub use mac::MacBuilder;
pub use miniprogram::{AlipayBuilder, ByteDanceBuilder, WeChatBuilder};
pub use web::WebBuilder;
pub use win::WinBuilder;

use std::collections::HashMap;
use std::sync::Arc;

/// Builder registry for managing platform builders
pub struct BuilderRegistry {
    builders: HashMap<String, Arc<dyn Builder>>,
}

impl BuilderRegistry {
    /// Create empty registry
    pub fn new() -> Self {
        Self {
            builders: HashMap::new(),
        }
    }

    /// Create registry with all default builders
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Desktop builders
        registry.register(Arc::new(WinBuilder::new()));
        registry.register(Arc::new(MacBuilder::new()));
        registry.register(Arc::new(LinuxBuilder::new()));

        // Mobile builders
        registry.register(Arc::new(IOSBuilder::new()));
        registry.register(Arc::new(AndroidBuilder::new()));

        // Web builder
        registry.register(Arc::new(WebBuilder::new()));

        // MiniProgram builders
        registry.register(Arc::new(WeChatBuilder::new()));
        registry.register(Arc::new(AlipayBuilder::new()));
        registry.register(Arc::new(ByteDanceBuilder::new()));

        registry
    }

    /// Register a builder
    pub fn register(&mut self, builder: Arc<dyn Builder>) {
        self.builders.insert(builder.id().to_string(), builder);
    }

    /// Get builder by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn Builder>> {
        self.builders.get(id).cloned()
    }

    /// Get all builders
    pub fn all(&self) -> impl Iterator<Item = &Arc<dyn Builder>> {
        self.builders.values()
    }

    /// Get available builders (those that can build on current system)
    pub fn available(&self) -> Vec<Arc<dyn Builder>> {
        self.builders
            .values()
            .filter(|b| b.is_available())
            .cloned()
            .collect()
    }

    /// Find best builder for target
    pub fn find_for_target(&self, target: &str) -> Option<Arc<dyn Builder>> {
        self.builders
            .values()
            .find(|b| b.targets().contains(&target))
            .cloned()
    }
}

impl Default for BuilderRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
