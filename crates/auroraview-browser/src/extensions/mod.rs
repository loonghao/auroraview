//! Extension system for browser plugins
//!
//! This module provides two extension systems:
//!
//! 1. **Native Extensions** (`Extension` trait) - Rust-based browser plugins
//!    that can respond to browser events, add toolbar buttons, etc.
//!
//! 2. **Chrome Extensions** (`ChromeExtensionBridge`) - Chrome-style extensions
//!    with manifest.json support, requires the `plugins` feature.

mod extension;
mod plugins_bridge;
mod registry;

pub use extension::{Extension, ExtensionManifest};
pub use plugins_bridge::{ChromeExtensionBridge, ChromeExtensionInfo};
pub use registry::ExtensionRegistry;
