//! Package and Frontend configuration
//!
//! This module contains `PackageConfig` and `FrontendConfig` structs
//! that define the package metadata and frontend source for AuroraView Pack.

use serde::{Deserialize, Serialize};

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Package name (used for executable name)
    pub name: String,

    /// Package version
    #[serde(default = "default_version")]
    pub version: String,

    /// Window title
    #[serde(default)]
    pub title: Option<String>,

    /// Application identifier (e.g., "com.example.myapp")
    #[serde(default)]
    pub identifier: Option<String>,

    /// Package description
    #[serde(default)]
    pub description: Option<String>,

    /// Package authors
    #[serde(default)]
    pub authors: Vec<String>,

    /// License
    #[serde(default)]
    pub license: Option<String>,

    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,

    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,

    /// Custom user agent
    #[serde(default)]
    pub user_agent: Option<String>,

    /// Allow opening new windows
    #[serde(default)]
    pub allow_new_window: bool,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Frontend configuration
///
/// Specifies where to load frontend content from.
/// Either `path` (local) or `url` (remote) must be specified, but not both.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontendConfig {
    /// Path to local frontend assets (directory or HTML file)
    #[serde(default)]
    pub path: Option<PathBuf>,

    /// Remote URL to load (mutually exclusive with path)
    #[serde(default)]
    pub url: Option<String>,
}
