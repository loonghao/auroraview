//! Manifest module for AuroraView Pack
//!
//! This module provides support for `auroraview.pack.toml` manifest files,
//! enabling declarative configuration of packaging options.
//!
//! # Module Structure
//!
//! - `package`: Package metadata and frontend configuration
//! - `backend`: Backend configuration (Python/Go/Rust/Node)
//! - `window`: Window configuration
//! - `bundle`: Bundle configuration (icons, platform configs)
//! - `build`: Build hooks and runtime configuration
//! - `security`: Security configuration (CSP, etc.)

#[cfg(test)]
mod tests;

pub mod backend;
pub mod bundle;
pub mod package;
pub mod window;

// TODO: Extract these modules from manifest.rs
// mod build;
// mod security;

// Re-exports
pub use backend::{BackendConfig, BackendGoConfig, BackendNodeConfig, BackendPythonConfig, BackendRustConfig, BackendType};
pub use bundle::{BundleConfig, LinuxBundleConfig, MacOSBundleConfig, PlatformConfig, WindowsBundleConfig};
pub use package::{FrontendConfig, PackageConfig};
pub use window::WindowConfig;
