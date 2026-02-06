//! Chrome Extension Manifest V3 Parser
//!
//! Parses and validates Chrome Extension manifest.json files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{ExtensionError, ExtensionResult};

/// Chrome Extension Manifest (supports V3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Manifest version (must be 3)
    pub manifest_version: u32,

    /// Extension name
    pub name: String,

    /// Extension description
    #[serde(default)]
    pub description: String,

    /// Extension version
    pub version: String,

    /// Permissions required by the extension
    #[serde(default)]
    pub permissions: Vec<Permission>,

    /// Host permissions (URL patterns)
    #[serde(default)]
    pub host_permissions: Vec<String>,

    /// Optional permissions
    #[serde(default)]
    pub optional_permissions: Vec<Permission>,

    /// Side panel configuration
    #[serde(default)]
    pub side_panel: Option<SidePanelConfig>,

    /// Action (toolbar button) configuration
    #[serde(default)]
    pub action: Option<ActionConfig>,

    /// Background service worker
    #[serde(default)]
    pub background: Option<BackgroundConfig>,

    /// Content scripts
    #[serde(default)]
    pub content_scripts: Vec<ContentScriptConfig>,

    /// Web accessible resources
    #[serde(default)]
    pub web_accessible_resources: Vec<WebAccessibleResource>,

    /// Content security policy
    #[serde(default)]
    pub content_security_policy: Option<ContentSecurityPolicy>,

    /// Commands (keyboard shortcuts)
    #[serde(default)]
    pub commands: HashMap<String, CommandConfig>,

    /// Icons
    #[serde(default)]
    pub icons: HashMap<String, String>,

    /// Default locale
    #[serde(default)]
    pub default_locale: Option<String>,

    /// Options page
    #[serde(default)]
    pub options_page: Option<String>,

    /// Options UI
    #[serde(default)]
    pub options_ui: Option<OptionsUiConfig>,

    /// DevTools page
    #[serde(default)]
    pub devtools_page: Option<String>,

    /// Chrome URL overrides
    #[serde(default)]
    pub chrome_url_overrides: Option<ChromeUrlOverrides>,

    /// Minimum Chrome version
    #[serde(default)]
    pub minimum_chrome_version: Option<String>,

    /// Author
    #[serde(default)]
    pub author: Option<String>,

    /// Homepage URL
    #[serde(default)]
    pub homepage_url: Option<String>,

    /// Short name
    #[serde(default)]
    pub short_name: Option<String>,

    /// Version name (display version)
    #[serde(default)]
    pub version_name: Option<String>,
}

/// Alias for Manifest V3
pub type ManifestV3 = Manifest;

/// Permission types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Permission {
    /// Named permission
    Named(String),
}

impl Permission {
    /// Get the permission name
    pub fn name(&self) -> &str {
        match self {
            Permission::Named(name) => name,
        }
    }

    /// Check if this is the storage permission
    pub fn is_storage(&self) -> bool {
        self.name() == "storage"
    }

    /// Check if this is the tabs permission
    pub fn is_tabs(&self) -> bool {
        self.name() == "tabs"
    }

    /// Check if this is the sidePanel permission
    pub fn is_side_panel(&self) -> bool {
        self.name() == "sidePanel"
    }

    /// Check if this is the scripting permission
    pub fn is_scripting(&self) -> bool {
        self.name() == "scripting"
    }
}

impl From<&str> for Permission {
    fn from(s: &str) -> Self {
        Permission::Named(s.to_string())
    }
}

/// Side panel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SidePanelConfig {
    /// Default path to the side panel HTML
    pub default_path: Option<String>,
}

/// Action (toolbar button) configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionConfig {
    /// Default icon
    #[serde(default)]
    pub default_icon: Option<IconConfig>,

    /// Default title (tooltip)
    #[serde(default)]
    pub default_title: Option<String>,

    /// Default popup HTML
    #[serde(default)]
    pub default_popup: Option<String>,
}

/// Icon configuration (can be a string or size map)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IconConfig {
    /// Single icon path
    Single(String),
    /// Size-specific icons
    Sized(HashMap<String, String>),
}

/// Background service worker configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackgroundConfig {
    /// Service worker script path
    pub service_worker: Option<String>,

    /// Module type (for ES modules)
    #[serde(default)]
    pub r#type: Option<String>,

    /// Scripts (for MV2 compatibility, deprecated in MV3)
    #[serde(default)]
    pub scripts: Vec<String>,

    /// Persistent (MV2 only, ignored in MV3)
    #[serde(default)]
    pub persistent: Option<bool>,
}

/// Content script configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScriptConfig {
    /// URL patterns to match
    pub matches: Vec<String>,

    /// JavaScript files to inject
    #[serde(default)]
    pub js: Vec<String>,

    /// CSS files to inject
    #[serde(default)]
    pub css: Vec<String>,

    /// When to run the scripts
    #[serde(default)]
    pub run_at: Option<RunAt>,

    /// Whether to run in all frames
    #[serde(default)]
    pub all_frames: bool,

    /// URL patterns to exclude
    #[serde(default)]
    pub exclude_matches: Vec<String>,

    /// Glob patterns to include
    #[serde(default)]
    pub include_globs: Vec<String>,

    /// Glob patterns to exclude
    #[serde(default)]
    pub exclude_globs: Vec<String>,

    /// Match about:blank
    #[serde(default)]
    pub match_about_blank: bool,

    /// Match origin as fallback
    #[serde(default)]
    pub match_origin_as_fallback: bool,

    /// World to run in
    #[serde(default)]
    pub world: Option<ScriptWorld>,
}

/// When to run content scripts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunAt {
    /// Run at document start (before DOM is built)
    DocumentStart,
    /// Run at document end (after DOM is built, before subresources)
    #[default]
    DocumentEnd,
    /// Run at document idle (after page load)
    DocumentIdle,
}

/// Script execution world
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScriptWorld {
    /// Isolated world (default, separate from page scripts)
    #[default]
    Isolated,
    /// Main world (same as page scripts)
    Main,
}

/// Web accessible resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAccessibleResource {
    /// Resource paths
    pub resources: Vec<String>,

    /// URL patterns that can access these resources
    #[serde(default)]
    pub matches: Vec<String>,

    /// Extension IDs that can access these resources
    #[serde(default)]
    pub extension_ids: Vec<String>,

    /// Whether to use dynamic URL
    #[serde(default)]
    pub use_dynamic_url: bool,
}

/// Content security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSecurityPolicy {
    /// CSP for extension pages
    #[serde(default)]
    pub extension_pages: Option<String>,

    /// CSP for sandboxed pages
    #[serde(default)]
    pub sandbox: Option<String>,
}

/// Command (keyboard shortcut) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    /// Command description
    #[serde(default)]
    pub description: Option<String>,

    /// Suggested key combinations
    #[serde(default)]
    pub suggested_key: Option<SuggestedKey>,
}

/// Suggested keyboard shortcut
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedKey {
    /// Default key combination
    #[serde(default)]
    pub default: Option<String>,

    /// Windows key combination
    #[serde(default)]
    pub windows: Option<String>,

    /// macOS key combination
    #[serde(default)]
    pub mac: Option<String>,

    /// Linux key combination
    #[serde(default)]
    pub linux: Option<String>,

    /// ChromeOS key combination
    #[serde(default)]
    pub chromeos: Option<String>,
}

/// Options UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsUiConfig {
    /// Options page path
    pub page: String,

    /// Open in tab (vs embedded)
    #[serde(default)]
    pub open_in_tab: bool,
}

/// Chrome URL overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChromeUrlOverrides {
    /// New tab page override
    #[serde(default)]
    pub newtab: Option<String>,

    /// History page override
    #[serde(default)]
    pub history: Option<String>,

    /// Bookmarks page override
    #[serde(default)]
    pub bookmarks: Option<String>,
}

impl Manifest {
    /// Parse manifest from JSON string
    pub fn from_json(json: &str) -> ExtensionResult<Self> {
        serde_json::from_str(json).map_err(|e| ExtensionError::ManifestParse(e.to_string()))
    }

    /// Parse manifest from file
    pub fn from_file(path: &Path) -> ExtensionResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Validate the manifest
    pub fn validate(&self) -> ExtensionResult<()> {
        // Check manifest version
        if self.manifest_version != 3 {
            return Err(ExtensionError::ManifestInvalid(format!(
                "Only Manifest V3 is supported, got V{}",
                self.manifest_version
            )));
        }

        // Check required fields
        if self.name.is_empty() {
            return Err(ExtensionError::ManifestInvalid(
                "Extension name is required".to_string(),
            ));
        }

        if self.version.is_empty() {
            return Err(ExtensionError::ManifestInvalid(
                "Extension version is required".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if extension has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p.name() == permission)
    }

    /// Check if extension has side panel
    pub fn has_side_panel(&self) -> bool {
        self.side_panel.is_some() && self.has_permission("sidePanel")
    }

    /// Check if extension has action (toolbar button)
    pub fn has_action(&self) -> bool {
        self.action.is_some()
    }

    /// Check if extension has background service worker
    pub fn has_background(&self) -> bool {
        self.background
            .as_ref()
            .map(|b| b.service_worker.is_some())
            .unwrap_or(false)
    }

    /// Get the side panel path
    pub fn get_side_panel_path(&self) -> Option<&str> {
        self.side_panel
            .as_ref()
            .and_then(|sp| sp.default_path.as_deref())
    }

    /// Get the action popup path
    pub fn get_popup_path(&self) -> Option<&str> {
        self.action
            .as_ref()
            .and_then(|a| a.default_popup.as_deref())
    }

    /// Get the background service worker path
    pub fn get_service_worker_path(&self) -> Option<&str> {
        self.background
            .as_ref()
            .and_then(|b| b.service_worker.as_deref())
    }

    /// Check if a URL matches any host permission
    pub fn matches_host_permission(&self, url: &str) -> bool {
        for pattern in &self.host_permissions {
            if url_matches_pattern(url, pattern) {
                return true;
            }
        }
        false
    }
}

/// Check if a URL matches a Chrome extension URL pattern
fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    // Handle special patterns
    if pattern == "<all_urls>" {
        return true;
    }

    // Parse pattern: scheme://host/path
    // Examples:
    // - *://*.example.com/*
    // - http://localhost:*/*
    // - https://specific.site.com/page/*

    let pattern = pattern.trim();

    // Simple wildcard matching (basic implementation)
    let regex_pattern = pattern
        .replace(".", r"\.")
        .replace("*://", r"https?://")
        .replace("*", r".*");

    if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
        re.is_match(url)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let json = r#"{
            "manifest_version": 3,
            "name": "Test Extension",
            "version": "1.0.0",
            "permissions": ["storage", "tabs"],
            "side_panel": {
                "default_path": "sidepanel.html"
            }
        }"#;

        let manifest = Manifest::from_json(json).unwrap();
        assert_eq!(manifest.name, "Test Extension");
        assert_eq!(manifest.version, "1.0.0");
        assert!(manifest.has_permission("storage"));
        assert!(manifest.has_permission("tabs"));
    }

    #[test]
    fn test_url_pattern_matching() {
        // *.example.com matches subdomains like sub.example.com, not example.com itself
        assert!(url_matches_pattern(
            "https://sub.example.com/page",
            "*://*.example.com/*"
        ));
        assert!(url_matches_pattern(
            "http://localhost:3000/api",
            "http://localhost:*/*"
        ));
        assert!(url_matches_pattern("https://anything.com", "<all_urls>"));
        // Test exact domain match
        assert!(url_matches_pattern(
            "https://example.com/page",
            "*://example.com/*"
        ));
    }
}
