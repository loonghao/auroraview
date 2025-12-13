//! WebView configuration structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

// ============================================================
// System Tray Configuration
// ============================================================

/// System tray menu item type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrayMenuItemType {
    /// Normal clickable menu item
    Normal,
    /// Separator line
    Separator,
    /// Checkbox item (can be checked/unchecked)
    Checkbox { checked: bool },
    /// Submenu containing child items
    Submenu { items: Vec<TrayMenuItem> },
}

/// A single menu item in the system tray context menu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayMenuItem {
    /// Unique identifier for this menu item
    pub id: String,
    /// Display text (ignored for separators)
    pub text: String,
    /// Whether the item is enabled
    pub enabled: bool,
    /// Item type (normal, separator, checkbox, submenu)
    pub item_type: TrayMenuItemType,
    /// Optional keyboard accelerator (e.g., "Ctrl+Q")
    pub accelerator: Option<String>,
}

impl TrayMenuItem {
    /// Create a normal menu item
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            enabled: true,
            item_type: TrayMenuItemType::Normal,
            accelerator: None,
        }
    }

    /// Create a separator
    pub fn separator() -> Self {
        Self {
            id: String::new(),
            text: String::new(),
            enabled: true,
            item_type: TrayMenuItemType::Separator,
            accelerator: None,
        }
    }

    /// Create a checkbox item
    pub fn checkbox(id: impl Into<String>, text: impl Into<String>, checked: bool) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            enabled: true,
            item_type: TrayMenuItemType::Checkbox { checked },
            accelerator: None,
        }
    }

    /// Create a submenu
    pub fn submenu(
        id: impl Into<String>,
        text: impl Into<String>,
        items: Vec<TrayMenuItem>,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            enabled: true,
            item_type: TrayMenuItemType::Submenu { items },
            accelerator: None,
        }
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set accelerator
    pub fn with_accelerator(mut self, accelerator: impl Into<String>) -> Self {
        self.accelerator = Some(accelerator.into());
        self
    }
}

/// System tray configuration
#[derive(Debug, Clone, Default)]
pub struct TrayConfig {
    /// Enable system tray icon
    pub enabled: bool,
    /// Tooltip text shown on hover
    pub tooltip: Option<String>,
    /// Path to tray icon (PNG recommended, 32x32 or 64x64)
    /// If None, uses the window icon or default AuroraView icon
    pub icon: Option<PathBuf>,
    /// Context menu items
    pub menu_items: Vec<TrayMenuItem>,
    /// Hide window to tray instead of closing
    pub hide_on_close: bool,
    /// Show window when tray icon is clicked
    pub show_on_click: bool,
    /// Show window when tray icon is double-clicked
    pub show_on_double_click: bool,
}

/// Protocol handler callback type
/// Takes a URI string and returns optional response (data, mime_type, status)
pub type ProtocolCallback = Arc<dyn Fn(&str) -> Option<(Vec<u8>, String, u16)> + Send + Sync>;

/// Embedding mode on Windows.
///
/// # Recommended Mode
///
/// **Use `Child` mode** - This is the official recommended approach by both wry and WebView2:
/// - wry's `build_as_child()` is designed for this use case
/// - WebView2's "Windowed Hosting" is the simplest and most reliable option
/// - Automatic resize handling when parent window resizes
///
/// # Qt Integration
///
/// For Qt5/Qt6 applications (Maya, Houdini, Nuke, etc.):
/// 1. Create a `QWidget` as the container
/// 2. Get its HWND via `winId()`
/// 3. Pass the HWND as `parent_hwnd` with `EmbedMode::Child`
/// 4. The WebView will be created as a true child window (WS_CHILD)
///
/// This approach works with both Qt5 and Qt6, avoiding the complexity of
/// `createWindowContainer` which has different behavior across Qt versions.
/// Window embedding mode for DCC integration.
///
/// # Overview
///
/// Windows provides two distinct parent-child relationships for windows:
///
/// | Mode | Relationship | Use Case |
/// |------|-------------|----------|
/// | `Child` | Parent-Child (containment) | Embed WebView inside a Qt widget |
/// | `Owner` | Owner-Owned (management) | Floating tool windows, dialogs |
///
/// # Official Documentation
///
/// - [Window Features (Microsoft)](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features)
/// - [SetWindowLongPtrW (GWLP_HWNDPARENT)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw)
/// - [Window Styles (WS_CHILD, WS_POPUP)](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-styles)
/// - [Extended Window Styles (WS_EX_TOOLWINDOW)](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)
///
/// # Child vs Owner Comparison
///
/// | Feature | Child (WS_CHILD) | Owner (GWLP_HWNDPARENT) |
/// |---------|------------------|-------------------------|
/// | **Coordinate System** | Relative to parent's client area | Relative to screen |
/// | **Clipping** | Clipped to parent bounds | Independent, can extend beyond owner |
/// | **Z-Order** | Within parent's client area only | Always above owner window |
/// | **Visibility** | Hidden when parent is hidden | Hidden when owner is minimized |
/// | **Lifecycle** | Destroyed with parent | Destroyed with owner |
/// | **Movement** | Cannot be moved independently | Can be dragged freely |
/// | **Taskbar** | Never shown | Shown unless WS_EX_TOOLWINDOW |
/// | **Alt+Tab** | Never shown | Shown unless WS_EX_TOOLWINDOW |
///
/// # When to Use Each Mode
///
/// ## Use `Child` mode when:
/// - Embedding WebView inside a Qt widget (Maya, Houdini, Nuke panels)
/// - WebView should fill a specific area in the host application
/// - WebView should resize automatically with the container
/// - User should NOT be able to drag the WebView independently
///
/// ## Use `Owner` mode when:
/// - Creating floating tool windows (AI assistants, property editors)
/// - Creating modal/modeless dialogs
/// - Window should follow owner's minimize/restore state
/// - Window should stay above the owner in Z-order
/// - Window needs to be positioned freely on screen
///
/// # Example: Floating Tool Window
///
/// ```python
/// from auroraview import AuroraView
///
/// class FloatingPanel(AuroraView):
///     def __init__(self, owner_hwnd: int):
///         super().__init__(
///             html=PANEL_HTML,
///             width=350,
///             height=200,
///             frame=False,           # Frameless window
///             transparent=True,      # Transparent background
///             always_on_top=True,    # Stay on top
///             parent_hwnd=owner_hwnd,
///             embed_mode="owner",    # Owner relationship
///             tool_window=True,      # Hide from taskbar/Alt+Tab
///         )
/// ```
#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmbedMode {
    /// No parent specified (standalone top-level window).
    ///
    /// Use this for independent windows that are not associated with any host application.
    /// The window will appear in the taskbar and Alt+Tab list.
    None,

    /// Create as real child window (WS_CHILD) - **RECOMMENDED for embedding**.
    ///
    /// This is the official recommended mode for embedding WebView2 into Qt widgets:
    /// - Uses wry's `build_as_child()` which is designed for embedding
    /// - WebView becomes a true child window that cannot be moved independently
    /// - Automatic resize when parent resizes
    /// - Coordinates are relative to parent's client area
    /// - Works with Qt5/Qt6 by passing QWidget's `winId()` as `parent_hwnd`
    ///
    /// # Windows API
    /// - Window style: `WS_CHILD`
    /// - Created via: `CreateWindowEx` with parent HWND
    ///
    /// # Requirements
    /// - Must be created on the same thread as the parent window
    /// - Parent must be a valid window handle
    Child,

    /// Create as owned window (GWLP_HWNDPARENT) - **RECOMMENDED for floating windows**.
    ///
    /// This mode creates a top-level window with an owner relationship:
    /// - Window stays above the owner in Z-order
    /// - Hidden when owner is minimized
    /// - Destroyed when owner is destroyed
    /// - Can be positioned freely on screen
    /// - Coordinates are relative to screen (not owner)
    ///
    /// # Windows API
    /// - Window style: `WS_POPUP` (no `WS_CHILD`)
    /// - Owner set via: `SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, owner_hwnd)`
    /// - Optional: `WS_EX_TOOLWINDOW` to hide from taskbar/Alt+Tab
    ///
    /// # Use Cases
    /// - Floating tool windows in DCC applications
    /// - AI assistant panels (like in Photoshop)
    /// - Property editors and inspectors
    /// - Modal/modeless dialogs
    ///
    /// # Official Documentation
    /// - [Owned Windows](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows)
    /// - [SetWindowLongPtrW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw)
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

    /// Background color in hex format (e.g., "#1e1e1e", "#ffffff")
    /// Used as window background while WebView is loading
    /// Default: None (system default, usually white)
    pub background_color: Option<String>,

    /// Parent window handle (HWND on Windows) for embedding/ownership
    pub parent_hwnd: Option<u64>,

    /// Embedding mode (Windows): Child vs Owner vs None
    ///
    /// See [`EmbedMode`] for detailed documentation.
    pub embed_mode: EmbedMode,

    /// Tool window style (WS_EX_TOOLWINDOW on Windows).
    ///
    /// When enabled, the window:
    /// - Does NOT appear in the taskbar
    /// - Does NOT appear in Alt+Tab window switcher
    /// - Has a smaller title bar (if decorations are enabled)
    ///
    /// This is commonly used with `embed_mode: Owner` for floating tool windows.
    ///
    /// # Official Documentation
    /// - [WS_EX_TOOLWINDOW](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)
    #[cfg(target_os = "windows")]
    pub tool_window: bool,

    /// Show shadow for undecorated (frameless) windows (Windows only).
    ///
    /// When `decorations` is false, Windows can still show a subtle shadow
    /// around the window. Set this to `false` to disable the shadow completely,
    /// which is required for truly transparent frameless windows.
    ///
    /// Default: true (show shadow for undecorated windows)
    ///
    /// # When to disable
    /// - Transparent overlay windows (e.g., floating logo buttons)
    /// - Custom-shaped windows
    /// - Windows that should blend seamlessly with the desktop
    ///
    /// # Example
    /// ```python
    /// from auroraview import run_desktop
    ///
    /// # Transparent logo button with no shadow
    /// run_desktop(
    ///     html=LOGO_HTML,
    ///     width=64,
    ///     height=64,
    ///     frame=False,           # No decorations
    ///     transparent=True,      # Transparent background
    ///     undecorated_shadow=False,  # No shadow
    ///     tool_window=True,      # Hide from taskbar
    /// )
    /// ```
    #[cfg(target_os = "windows")]
    pub undecorated_shadow: bool,

    /// Enable IPC message batching for better performance
    pub ipc_batching: bool,

    /// Maximum number of messages per batch
    pub ipc_batch_size: usize,

    /// Maximum batch age in milliseconds (flush interval)
    pub ipc_batch_interval_ms: u64,

    /// Asset root directory for auroraview:// protocol
    pub asset_root: Option<PathBuf>,

    /// User data directory for WebView (cookies, cache, localStorage, etc.)
    /// If None, uses system default (usually %LOCALAPPDATA%\{app}\EBWebView on Windows)
    /// Set this to isolate WebView data per application or user profile
    pub data_directory: Option<PathBuf>,

    /// Custom protocol handlers (scheme -> callback)
    #[allow(clippy::type_complexity)]
    pub custom_protocols: HashMap<String, ProtocolCallback>,

    /// API methods to register (namespace -> method names)
    /// Used to dynamically inject JavaScript wrapper methods
    pub api_methods: HashMap<String, Vec<String>>,

    /// Allow opening new windows (e.g., via window.open)
    /// Default: false (blocks new windows)
    pub allow_new_window: bool,

    /// Enable file:// protocol support
    /// Default: false (blocks file:// for security)
    /// WARNING: Enabling this bypasses WebView's default security restrictions
    pub allow_file_protocol: bool,

    /// Automatically show window after creation
    /// Default: true (show window after loading screen is ready)
    /// Set to false for DCC embedding where window visibility is controlled externally
    pub auto_show: bool,

    /// Headless mode - run without visible window
    /// Default: false (show window)
    /// When true, the window is created but never shown, useful for automated testing
    /// Note: WebView2 doesn't support true headless mode, so this creates a hidden window
    pub headless: bool,

    /// Remote debugging port for CDP (Chrome DevTools Protocol) connections
    /// Default: None (disabled)
    /// When set, enables remote debugging on the specified port
    /// Playwright/Puppeteer can connect via: `browser.connect_over_cdp(f"http://localhost:{port}")`
    /// Note: This sets WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS environment variable
    pub remote_debugging_port: Option<u16>,

    // ============================================================
    // Security Configuration
    // ============================================================
    /// Content Security Policy (CSP) header
    /// Default: None (uses browser default)
    /// Example: "default-src 'self'; script-src 'self' 'unsafe-inline'"
    pub content_security_policy: Option<String>,

    /// CORS allowed origins
    /// Default: empty (no CORS restrictions within WebView)
    /// Example: `vec!["https://api.example.com", "http://localhost:3000"]`
    pub cors_allowed_origins: Vec<String>,

    /// Enable clipboard access from JavaScript
    /// Default: false (blocks navigator.clipboard API)
    pub allow_clipboard: bool,

    /// Enable geolocation access from JavaScript
    /// Default: false (blocks navigator.geolocation API)
    pub allow_geolocation: bool,

    /// Enable notification access from JavaScript
    /// Default: false (blocks Notification API)
    pub allow_notifications: bool,

    /// Enable microphone/camera access from JavaScript
    /// Default: false (blocks MediaDevices API)
    pub allow_media_devices: bool,

    /// Block external navigation (http/https URLs not in allowed list)
    /// Default: false (allow all navigation)
    pub block_external_navigation: bool,

    /// Allowed external navigation domains
    /// Only used when block_external_navigation is true
    /// Example: vec!["example.com", "api.example.com"]
    pub allowed_navigation_domains: Vec<String>,

    /// Custom window icon path (PNG format, recommended 32x32 pixels)
    /// If None, uses the default AuroraView icon
    /// Supported formats: PNG (recommended), ICO, JPEG, BMP, GIF
    /// Recommended sizes: 32x32 (taskbar), 64x64 (alt-tab), 256x256 (high-DPI)
    pub icon: Option<PathBuf>,

    /// Enable plugin JavaScript APIs (fs, dialog, clipboard, shell)
    /// Default: true (inject plugin APIs)
    /// When enabled, injects JavaScript wrappers for native plugin commands
    pub enable_plugins: bool,

    /// List of enabled plugin names
    /// Default: all plugins ["fs", "dialog", "clipboard", "shell"]
    /// Empty list means all plugins are enabled when enable_plugins is true
    pub enabled_plugin_names: Vec<String>,

    // ============================================================
    // System Tray Configuration
    // ============================================================
    /// System tray configuration
    /// Default: None (no system tray)
    pub tray: Option<TrayConfig>,
}

// Manual Debug implementation (ProtocolCallback doesn't implement Debug)
impl std::fmt::Debug for WebViewConfig {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = fmt.debug_struct("WebViewConfig");
        f.field("title", &self.title)
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
            .field("embed_mode", &self.embed_mode);

        #[cfg(target_os = "windows")]
        f.field("tool_window", &self.tool_window);

        f.field("ipc_batching", &self.ipc_batching)
            .field("ipc_batch_size", &self.ipc_batch_size)
            .field("ipc_batch_interval_ms", &self.ipc_batch_interval_ms)
            .field("asset_root", &self.asset_root)
            .field(
                "custom_protocols",
                &format!("{} protocols", self.custom_protocols.len()),
            )
            .field("api_methods", &self.api_methods)
            .field("auto_show", &self.auto_show)
            .field("icon", &self.icon)
            .field("tray", &self.tray)
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
            background_color: None,    // System default (usually white)
            ipc_batching: true,        // Enable by default
            ipc_batch_size: 10,        // 10 messages per batch
            ipc_batch_interval_ms: 16, // ~60 FPS (16.67ms)
            parent_hwnd: None,
            #[cfg(target_os = "windows")]
            embed_mode: EmbedMode::None,
            #[cfg(not(target_os = "windows"))]
            embed_mode: EmbedMode::None,
            #[cfg(target_os = "windows")]
            tool_window: false,
            #[cfg(target_os = "windows")]
            undecorated_shadow: true,
            asset_root: None,
            data_directory: None,
            custom_protocols: HashMap::new(),
            api_methods: HashMap::new(),
            allow_new_window: false,     // Block new windows by default
            allow_file_protocol: false,  // Block file:// protocol by default for security
            auto_show: true,             // Show window after loading screen is ready
            headless: false,             // Show window by default
            remote_debugging_port: None, // CDP debugging disabled by default
            // Security defaults
            content_security_policy: None,
            cors_allowed_origins: Vec::new(),
            allow_clipboard: false,
            allow_geolocation: false,
            allow_notifications: false,
            allow_media_devices: false,
            block_external_navigation: false,
            allowed_navigation_domains: Vec::new(),
            icon: None,                       // Use default AuroraView icon
            enable_plugins: true,             // Enable plugin APIs by default
            enabled_plugin_names: Vec::new(), // Empty = all plugins enabled
            tray: None,                       // No system tray by default
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

    /// Set user data directory for WebView (cookies, cache, localStorage, etc.)
    /// Use this to isolate WebView data per application or user profile
    pub fn data_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.data_directory = Some(path.into());
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

    /// Allow or block new windows (e.g., window.open)
    pub fn allow_new_window(mut self, allow: bool) -> Self {
        self.config.allow_new_window = allow;
        self
    }

    /// Enable or disable file:// protocol support
    /// WARNING: Enabling this bypasses WebView's default security restrictions
    pub fn allow_file_protocol(mut self, allow: bool) -> Self {
        self.config.allow_file_protocol = allow;
        self
    }

    // ============================================================
    // Security Configuration
    // ============================================================

    /// Set Content Security Policy (CSP) header
    /// Example: "default-src 'self'; script-src 'self' 'unsafe-inline'"
    pub fn content_security_policy(mut self, csp: impl Into<String>) -> Self {
        self.config.content_security_policy = Some(csp.into());
        self
    }

    /// Add CORS allowed origin
    pub fn cors_allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.config.cors_allowed_origins.push(origin.into());
        self
    }

    /// Set multiple CORS allowed origins
    pub fn cors_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.config.cors_allowed_origins = origins;
        self
    }

    /// Enable or disable clipboard access from JavaScript
    pub fn allow_clipboard(mut self, allow: bool) -> Self {
        self.config.allow_clipboard = allow;
        self
    }

    /// Enable or disable geolocation access from JavaScript
    pub fn allow_geolocation(mut self, allow: bool) -> Self {
        self.config.allow_geolocation = allow;
        self
    }

    /// Enable or disable notification access from JavaScript
    pub fn allow_notifications(mut self, allow: bool) -> Self {
        self.config.allow_notifications = allow;
        self
    }

    /// Enable or disable media device (camera/microphone) access
    pub fn allow_media_devices(mut self, allow: bool) -> Self {
        self.config.allow_media_devices = allow;
        self
    }

    /// Block external navigation (http/https URLs not in allowed list)
    pub fn block_external_navigation(mut self, block: bool) -> Self {
        self.config.block_external_navigation = block;
        self
    }

    /// Add allowed navigation domain (only used when block_external_navigation is true)
    pub fn allow_navigation_domain(mut self, domain: impl Into<String>) -> Self {
        self.config.allowed_navigation_domains.push(domain.into());
        self
    }

    /// Set multiple allowed navigation domains
    pub fn allowed_navigation_domains(mut self, domains: Vec<String>) -> Self {
        self.config.allowed_navigation_domains = domains;
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
    use rstest::*;

    #[fixture]
    fn default_config() -> WebViewConfig {
        WebViewConfig::default()
    }

    #[fixture]
    fn builder() -> WebViewBuilder {
        WebViewBuilder::new()
    }

    #[rstest]
    fn test_default_config_values(default_config: WebViewConfig) {
        assert_eq!(default_config.title, "AuroraView");
        assert_eq!(default_config.width, 800);
        assert_eq!(default_config.height, 600);
        assert!(default_config.url.is_none());
        assert!(default_config.html.is_none());
        assert!(default_config.dev_tools);
        assert!(default_config.context_menu);
        assert!(default_config.resizable);
        assert!(default_config.decorations);
        assert!(!default_config.always_on_top);
        assert!(!default_config.transparent);
        assert!(default_config.ipc_batching);
        assert_eq!(default_config.ipc_batch_size, 10);
        assert_eq!(default_config.ipc_batch_interval_ms, 16);
        assert!(default_config.asset_root.is_none());
        assert_eq!(default_config.custom_protocols.len(), 0);
        // Test new fields default values
        assert!(!default_config.allow_new_window);
        assert!(!default_config.allow_file_protocol);
    }

    #[rstest]
    fn test_builder_overrides(builder: WebViewBuilder) {
        let cfg = builder
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

    #[rstest]
    #[case("Test Title")]
    #[case("Another Window")]
    #[case("")]
    fn test_builder_title_variations(builder: WebViewBuilder, #[case] title: &str) {
        let cfg = builder.title(title).build();
        assert_eq!(cfg.title, title);
    }

    #[rstest]
    #[case(1920, 1080)]
    #[case(1280, 720)]
    #[case(640, 480)]
    fn test_builder_size_variations(
        builder: WebViewBuilder,
        #[case] width: u32,
        #[case] height: u32,
    ) {
        let cfg = builder.size(width, height).build();
        assert_eq!(cfg.width, width);
        assert_eq!(cfg.height, height);
    }

    #[rstest]
    fn test_builder_asset_root(builder: WebViewBuilder) {
        let path = PathBuf::from("/tmp/assets");
        let cfg = builder.asset_root(path.clone()).build();
        assert_eq!(cfg.asset_root, Some(path));
    }

    #[rstest]
    fn test_builder_register_protocol(builder: WebViewBuilder) {
        let handler = Arc::new(|uri: &str| {
            if uri.starts_with("custom://") {
                Some((b"data".to_vec(), "text/plain".to_string(), 200))
            } else {
                None
            }
        });

        let cfg = builder.register_protocol("custom", handler).build();
        assert_eq!(cfg.custom_protocols.len(), 1);
        assert!(cfg.custom_protocols.contains_key("custom"));
    }

    #[rstest]
    #[cfg(target_os = "windows")]
    fn test_embed_mode_default() {
        let cfg = WebViewConfig::default();
        assert_eq!(cfg.embed_mode, EmbedMode::None);
    }

    #[rstest]
    fn test_allow_new_window_builder(builder: WebViewBuilder) {
        // Test enabling new window
        let cfg = builder.allow_new_window(true).build();
        assert!(cfg.allow_new_window);

        // Test disabling new window (default)
        let cfg2 = WebViewBuilder::new().build();
        assert!(!cfg2.allow_new_window);
    }

    #[rstest]
    fn test_allow_file_protocol_builder(builder: WebViewBuilder) {
        // Test enabling file protocol
        let cfg = builder.allow_file_protocol(true).build();
        assert!(cfg.allow_file_protocol);

        // Test disabling file protocol (default)
        let cfg2 = WebViewBuilder::new().build();
        assert!(!cfg2.allow_file_protocol);
    }

    #[rstest]
    fn test_new_features_combined(builder: WebViewBuilder) {
        // Test all new features together
        let cfg = builder
            .allow_new_window(true)
            .allow_file_protocol(true)
            .always_on_top(true)
            .build();

        assert!(cfg.allow_new_window);
        assert!(cfg.allow_file_protocol);
        assert!(cfg.always_on_top);
    }

    #[rstest]
    #[case(true, true)]
    #[case(true, false)]
    #[case(false, true)]
    #[case(false, false)]
    fn test_window_control_combinations(
        builder: WebViewBuilder,
        #[case] allow_new_window: bool,
        #[case] allow_file_protocol: bool,
    ) {
        let cfg = builder
            .allow_new_window(allow_new_window)
            .allow_file_protocol(allow_file_protocol)
            .build();

        assert_eq!(cfg.allow_new_window, allow_new_window);
        assert_eq!(cfg.allow_file_protocol, allow_file_protocol);
    }

    #[rstest]
    #[cfg(target_os = "windows")]
    fn test_undecorated_shadow_default(default_config: WebViewConfig) {
        assert!(
            default_config.undecorated_shadow,
            "undecorated_shadow should default to true"
        );
    }

    #[rstest]
    #[cfg(target_os = "windows")]
    fn test_undecorated_shadow_disabled() {
        let cfg = WebViewConfig {
            undecorated_shadow: false,
            ..Default::default()
        };
        assert!(!cfg.undecorated_shadow);
    }

    #[rstest]
    #[cfg(target_os = "windows")]
    fn test_transparent_frameless_window() {
        // Test configuration for transparent frameless window (like logo button)
        let cfg = WebViewConfig {
            transparent: true,
            decorations: false,
            undecorated_shadow: false,
            tool_window: true,
            ..Default::default()
        };
        assert!(cfg.transparent);
        assert!(!cfg.decorations);
        assert!(!cfg.undecorated_shadow);
        assert!(cfg.tool_window);
    }
}
