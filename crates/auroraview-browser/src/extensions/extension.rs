//! Extension trait definition

use crate::browser::BrowserEvent;

/// Extension trait for browser plugins
///
/// This trait defines the interface for browser extensions.
/// Extensions can:
/// - Add toolbar buttons
/// - React to browser events
/// - Show popups
/// - Modify page content (via JavaScript injection)
///
/// # Example
///
/// ```rust,ignore
/// use auroraview_browser::{Extension, BrowserEvent, Browser};
///
/// struct AdBlocker {
///     enabled: bool,
/// }
///
/// impl Extension for AdBlocker {
///     fn id(&self) -> &str { "adblocker" }
///     fn name(&self) -> &str { "Ad Blocker" }
///     
///     fn on_event(&mut self, event: &BrowserEvent) {
///         if let BrowserEvent::NavigationCompleted { tab_id, url } = event {
///             if self.enabled {
///                 // Inject ad-blocking script
///             }
///         }
///     }
/// }
/// ```
pub trait Extension: Send + Sync {
    /// Extension unique identifier
    fn id(&self) -> &str;

    /// Display name
    fn name(&self) -> &str;

    /// Extension version
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Extension description
    fn description(&self) -> &str {
        ""
    }

    /// Extension icon (SVG string or data URL)
    fn icon(&self) -> Option<&str> {
        None
    }

    /// Whether extension is currently enabled
    fn enabled(&self) -> bool {
        true
    }

    /// Enable/disable the extension
    fn set_enabled(&mut self, _enabled: bool) {}

    /// Called when extension is loaded
    fn on_load(&mut self) {}

    /// Called when extension is unloaded
    fn on_unload(&mut self) {}

    /// Handle browser events
    fn on_event(&mut self, _event: &BrowserEvent) {}

    /// Toolbar button click handler
    fn on_toolbar_click(&mut self) {}

    /// Get popup HTML (if any)
    /// Returns HTML content to show in a popup when toolbar button is clicked
    fn popup_html(&self) -> Option<&str> {
        None
    }

    /// Get content script to inject into pages
    /// Returns JavaScript that will be injected into every page
    fn content_script(&self) -> Option<&str> {
        None
    }

    /// Get background script
    /// Returns JavaScript that runs in the extension context
    fn background_script(&self) -> Option<&str> {
        None
    }

    /// URL patterns where content script should be injected
    /// Returns a list of URL patterns (supports * wildcard)
    fn content_script_matches(&self) -> Vec<&str> {
        vec!["*://*/*"]
    }

    /// Required permissions
    fn permissions(&self) -> Vec<&str> {
        vec![]
    }
}

/// Extension manifest (for serialization)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtensionManifest {
    /// Extension ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Version
    pub version: String,
    /// Description
    pub description: String,
    /// Icon path or data URL
    pub icon: Option<String>,
    /// Whether extension is enabled
    pub enabled: bool,
    /// Content script URL patterns
    pub content_script_matches: Vec<String>,
    /// Required permissions
    pub permissions: Vec<String>,
}

impl<T: Extension> From<&T> for ExtensionManifest {
    fn from(ext: &T) -> Self {
        Self {
            id: ext.id().to_string(),
            name: ext.name().to_string(),
            version: ext.version().to_string(),
            description: ext.description().to_string(),
            icon: ext.icon().map(String::from),
            enabled: ext.enabled(),
            content_script_matches: ext.content_script_matches().iter().map(|s| s.to_string()).collect(),
            permissions: ext.permissions().iter().map(|s| s.to_string()).collect(),
        }
    }
}
