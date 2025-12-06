//! AuroraView Pack - Standalone Executable Packaging
//!
//! This crate provides functionality to package AuroraView-based applications
//! into standalone executables, similar to Pake/Tauri.
//!
//! # Features
//!
//! - **URL Mode**: Pack a URL into a standalone desktop app
//! - **Frontend Mode**: Pack local HTML/CSS/JS into a standalone app
//! - **Full Stack Mode**: Pack frontend + Python backend (requires PyOxidizer)
//!
//! # Quick Start
//!
//! ## URL Mode - Wrap a website
//!
//! ```rust,ignore
//! use auroraview_pack::{PackConfig, PackGenerator};
//!
//! let config = PackConfig::url("https://example.com")
//!     .with_output("my-app")
//!     .with_title("My App")
//!     .with_size(1280, 720);
//!
//! let generator = PackGenerator::new(config);
//! let project_dir = generator.generate()?;
//! println!("Project generated at: {}", project_dir.display());
//! ```
//!
//! ## Frontend Mode - Bundle local assets
//!
//! ```rust,ignore
//! use auroraview_pack::{PackConfig, PackGenerator};
//!
//! let config = PackConfig::frontend("./dist")
//!     .with_output("my-app")
//!     .with_title("My Frontend App");
//!
//! let generator = PackGenerator::new(config);
//! generator.generate()?;
//! ```
//!
//! ## Full Stack Mode - Frontend + Python backend
//!
//! ```rust,ignore
//! use auroraview_pack::{PackConfig, PackGenerator};
//!
//! let config = PackConfig::fullstack("./dist", "myapp.main:run")
//!     .with_output("my-fullstack-app")
//!     .with_title("Full Stack App");
//!
//! let generator = PackGenerator::new(config);
//! generator.generate()?;
//! // Then run: pyoxidizer build --release
//! ```

mod config;
mod error;
mod generator;
mod pyembed_integration;
mod templates;

// Public API exports
pub use config::{PackConfig, PackMode, TargetPlatform, WindowStartPosition};
pub use error::PackError;
pub use generator::PackGenerator;
pub use pyembed_integration::{
    check_pyembed_availability, generate_pyoxidizer_config, PyEmbedConfig,
};

/// Result type for pack operations
pub type PackResult<T> = Result<T, PackError>;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if all required dependencies are available for packing
///
/// Returns a list of missing dependencies, if any.
pub fn check_dependencies() -> Vec<&'static str> {
    let mut missing = Vec::new();

    // Check for cargo
    if std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .is_err()
    {
        missing.push("cargo (Rust toolchain)");
    }

    missing
}

/// Check if PyOxidizer is available for fullstack mode
pub fn is_pyoxidizer_available() -> bool {
    std::process::Command::new("pyoxidizer")
        .arg("--version")
        .output()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_pack_mode_variants() {
        use std::path::PathBuf;

        let url_mode = PackMode::Url {
            url: "https://example.com".to_string(),
        };
        assert_eq!(url_mode.name(), "url");
        assert!(!url_mode.embeds_assets());

        let frontend_mode = PackMode::Frontend {
            path: PathBuf::from("/path/to/dist"),
        };
        assert_eq!(frontend_mode.name(), "frontend");
        assert!(frontend_mode.embeds_assets());

        let fullstack_mode = PackMode::FullStack {
            frontend_path: PathBuf::from("/path/to/dist"),
            backend_entry: "myapp:main".to_string(),
        };
        assert_eq!(fullstack_mode.name(), "fullstack");
        assert!(fullstack_mode.requires_pyoxidizer());
    }

    #[test]
    fn test_check_dependencies() {
        // cargo should be available in the test environment
        let missing = check_dependencies();
        assert!(
            !missing.contains(&"cargo (Rust toolchain)"),
            "cargo should be available"
        );
    }
}
