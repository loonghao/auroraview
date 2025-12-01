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
//! # Example
//!
//! ```rust,ignore
//! use auroraview_pack::{PackConfig, PackMode, PackGenerator};
//!
//! let config = PackConfig {
//!     mode: PackMode::Url("https://example.com".to_string()),
//!     output_name: "my-app".to_string(),
//!     title: Some("My App".to_string()),
//!     ..Default::default()
//! };
//!
//! let generator = PackGenerator::new(config);
//! generator.generate()?;
//! ```

mod config;
mod error;
mod generator;
mod pyembed_integration;
mod templates;

pub use config::{PackConfig, PackMode};
pub use error::PackError;
pub use generator::PackGenerator;
pub use pyembed_integration::{
    check_pyembed_availability, generate_pyoxidizer_config, PyEmbedConfig,
};

/// Result type for pack operations
pub type PackResult<T> = Result<T, PackError>;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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

        let _url = PackMode::Url {
            url: "https://example.com".to_string(),
        };
        let _frontend = PackMode::Frontend {
            path: PathBuf::from("/path/to/dist"),
        };
        let _fullstack = PackMode::FullStack {
            frontend_path: PathBuf::from("/path/to/dist"),
            backend_entry: "myapp:main".to_string(),
        };
    }
}
