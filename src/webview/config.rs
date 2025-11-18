//! WebView configuration structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Protocol handler callback type
/// Takes a URI string and returns optional response (data, mime_type, status)
pub type ProtocolCallback = Arc<dyn Fn(&str) -> Option<(Vec<u8>, String, u16)> + Send + Sync>;

/// Embedding mode on Windows.
#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmbedMode {
    /// No parent/owner specified (standalone top-level window)
    None,
    /// Create as real child window (WS_CHILD). Requires same-thread parenting; can freeze GUIs if cross-thread.
    Child,
    /// Create as owned top-level window (GWLP_HWNDPARENT). Safe across threads; follows minimize/activate of owner.
    Owner,
}

/// Dummy enum for non-Windows (compile-time placeholder)
#[cfg(not(target_os = "windows"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmbedMode {
    None,
}

/// WebView configuration
#[derive(Clone)]
pub struct WebViewConfig {
    /// Window title
    pub title: String,

    /// Window width in pixels
    pub width: u32,

    /// Window height in pixels
    pub height: u32,

    /// URL to load (optional)
    pub url: Option<String>,

    /// HTML content to load (optional)
    pub html: Option<String>,

    /// Enable developer tools
    pub dev_tools: bool,

    /// Enable context menu
    pub context_menu: bool,

    /// Window resizable
    pub resizable: bool,

    /// Window decorations (title bar, borders)
    pub decorations: bool,

    /// Always on top
    pub always_on_top: bool,

    /// Transparent window
    pub transparent: bool,

    /// Parent window handle (HWND on Windows) for embedding/ownership
    pub parent_hwnd: Option<u64>,

    /// Embedding mode (Windows): Child vs Owner vs None
    pub embed_mode: EmbedMode,

    /// Enable IPC message batching for better performance
    pub ipc_batching: bool,

    /// Maximum number of messages per batch
    pub ipc_batch_size: usize,

    /// Maximum batch age in milliseconds (flush interval)
    pub ipc_batch_interval_ms: u64,

    /// Asset root directory for auroraview:// protocol
    pub asset_root: Option<PathBuf>,

    /// Custom protocol handlers (scheme -> callback)
    #[allow(clippy::type_complexity)]
    pub custom_protocols: HashMap<String, ProtocolCallback>,

    /// API methods to register (namespace -> method names)
    /// Used to dynamically inject JavaScript wrapper methods
    pub api_methods: HashMap<String, Vec<String>>,
}

// Manual Debug implementation (ProtocolCallback doesn't implement Debug)
impl std::fmt::Debug for WebViewConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebViewConfig")
            .field("title", &self.title)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("url", &self.url)
            .field(
                "html",
                &self
                    .html
                    .as_ref()
                    .map(|h| format!("{}...", &h.chars().take(50).collect::<String>())),
            )
            .field("dev_tools", &self.dev_tools)
            .field("context_menu", &self.context_menu)
            .field("resizable", &self.resizable)
            .field("decorations", &self.decorations)
            .field("always_on_top", &self.always_on_top)
            .field("transparent", &self.transparent)
            .field("parent_hwnd", &self.parent_hwnd)
            .field("embed_mode", &self.embed_mode)
            .field("ipc_batching", &self.ipc_batching)
            .field("ipc_batch_size", &self.ipc_batch_size)
            .field("ipc_batch_interval_ms", &self.ipc_batch_interval_ms)
            .field("asset_root", &self.asset_root)
            .field(
                "custom_protocols",
                &format!("{} protocols", self.custom_protocols.len()),
            )
            .field("api_methods", &self.api_methods)
            .finish()
    }
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            title: "AuroraView".to_string(),
            width: 800,
            height: 600,
            url: None,
            html: None,
            dev_tools: true,
            context_menu: true,
            resizable: true,
            decorations: true,
            always_on_top: false,
            transparent: false,
            ipc_batching: true,        // Enable by default
            ipc_batch_size: 10,        // 10 messages per batch
            ipc_batch_interval_ms: 16, // ~60 FPS (16.67ms)
            parent_hwnd: None,
            #[cfg(target_os = "windows")]
            embed_mode: EmbedMode::None,
            #[cfg(not(target_os = "windows"))]
            embed_mode: EmbedMode::None,
            asset_root: None,
            custom_protocols: HashMap::new(),
            api_methods: HashMap::new(),
        }
    }
}

/// Builder pattern for WebView configuration
pub struct WebViewBuilder {
    config: WebViewConfig,
}

impl WebViewBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: WebViewConfig::default(),
        }
    }

    /// Set window title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    /// Set window size
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    /// Set URL to load
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.config.url = Some(url.into());
        self
    }

    /// Set HTML content
    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.config.html = Some(html.into());
        self
    }

    /// Enable/disable developer tools
    pub fn dev_tools(mut self, enabled: bool) -> Self {
        self.config.dev_tools = enabled;
        self
    }

    /// Enable/disable context menu
    pub fn context_menu(mut self, enabled: bool) -> Self {
        self.config.context_menu = enabled;
        self
    }

    /// Set window resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.config.resizable = resizable;
        self
    }

    /// Set window decorations
    pub fn decorations(mut self, decorations: bool) -> Self {
        self.config.decorations = decorations;
        self
    }

    /// Set always on top
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.config.always_on_top = always_on_top;
        self
    }

    /// Set transparent window
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.config.transparent = transparent;
        self
    }

    /// Set asset root directory for auroraview:// protocol
    pub fn asset_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.asset_root = Some(path.into());
        self
    }

    /// Register a custom protocol handler
    ///
    /// # Arguments
    /// * `scheme` - Protocol scheme (e.g., "maya", "fbx")
    /// * `handler` - Callback function that takes URI and returns (data, mime_type, status)
    ///
    /// # Example
    /// ```ignore
    /// use std::sync::Arc;
    ///
    /// let config = WebViewBuilder::new()
    ///     .register_protocol("maya", Arc::new(|uri: &str| {
    ///         // Handle maya:// protocol
    ///         Some((b"data".to_vec(), "text/plain".to_string(), 200))
    ///     }))
    ///     .build();
    /// ```
    pub fn register_protocol(
        mut self,
        scheme: impl Into<String>,
        handler: ProtocolCallback,
    ) -> Self {
        self.config.custom_protocols.insert(scheme.into(), handler);
        self
    }

    /// Build the configuration
    pub fn build(self) -> WebViewConfig {
        self.config
    }
}

impl Default for WebViewBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let cfg = WebViewConfig::default();
        assert_eq!(cfg.title, "AuroraView");
        assert_eq!(cfg.width, 800);
        assert_eq!(cfg.height, 600);
        assert!(cfg.url.is_none());
        assert!(cfg.html.is_none());
        assert!(cfg.dev_tools);
        assert!(cfg.context_menu);
        assert!(cfg.resizable);
        assert!(cfg.decorations);
        assert!(!cfg.always_on_top);
        assert!(!cfg.transparent);
        assert!(cfg.ipc_batching);
        assert_eq!(cfg.ipc_batch_size, 10);
        assert_eq!(cfg.ipc_batch_interval_ms, 16);
        assert!(cfg.asset_root.is_none());
        assert_eq!(cfg.custom_protocols.len(), 0);
    }

    #[test]
    fn test_builder_overrides() {
        let cfg = WebViewBuilder::new()
            .title("Hello")
            .size(1024, 768)
            .url("https://example.com")
            .html("<h1>ignored when url set</h1>")
            .dev_tools(false)
            .context_menu(false)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .transparent(true)
            .build();

        assert_eq!(cfg.title, "Hello");
        assert_eq!(cfg.width, 1024);
        assert_eq!(cfg.height, 768);
        assert_eq!(cfg.url.as_deref(), Some("https://example.com"));
        assert!(cfg.html.is_some());
        assert!(!cfg.dev_tools);
        assert!(!cfg.context_menu);
        assert!(!cfg.resizable);
        assert!(!cfg.decorations);
        assert!(cfg.always_on_top);
        assert!(cfg.transparent);
    }
}
