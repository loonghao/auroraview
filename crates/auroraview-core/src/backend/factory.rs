//! Backend factory for creating WebView instances
//!
//! Implements the factory pattern for backend instantiation,
//! inspired by Qt WebView's `QWebViewFactory`.

use super::error::{WebViewError, WebViewResult};
use super::settings::WebViewSettingsImpl;
use super::traits::WebViewBackend;
use super::wry_impl::WryBackend;
use std::path::PathBuf;
use std::str::FromStr;

/// Available backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendType {
    /// Wry backend (cross-platform, current default)
    #[default]
    Wry,
    /// WebView2 backend (Windows native)
    #[cfg(target_os = "windows")]
    WebView2,
    /// WKWebView backend (macOS/iOS native)
    #[cfg(target_os = "macos")]
    WKWebView,
    /// WebKitGTK backend (Linux native)
    #[cfg(target_os = "linux")]
    WebKitGTK,
}

impl FromStr for BackendType {
    type Err = WebViewError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "wry" => Ok(Self::Wry),
            #[cfg(target_os = "windows")]
            "webview2" | "webview_2" | "wv2" => Ok(Self::WebView2),
            #[cfg(target_os = "macos")]
            "wkwebview" | "wk" | "webkit" => Ok(Self::WKWebView),
            #[cfg(target_os = "linux")]
            "webkitgtk" | "gtk" => Ok(Self::WebKitGTK),
            other => Err(WebViewError::UnsupportedBackend(other.to_string())),
        }
    }
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wry => write!(f, "wry"),
            #[cfg(target_os = "windows")]
            Self::WebView2 => write!(f, "webview2"),
            #[cfg(target_os = "macos")]
            Self::WKWebView => write!(f, "wkwebview"),
            #[cfg(target_os = "linux")]
            Self::WebKitGTK => write!(f, "webkitgtk"),
        }
    }
}

/// Backend configuration
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Backend type to use
    pub backend_type: BackendType,
    /// Window title
    pub title: String,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Initial URL
    pub url: Option<String>,
    /// Initial HTML content
    pub html: Option<String>,
    /// Parent window handle (platform-specific)
    pub parent_handle: Option<u64>,
    /// Asset root directory
    pub asset_root: Option<PathBuf>,
    /// WebView settings
    pub settings: WebViewSettingsImpl,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            backend_type: BackendType::default(),
            title: "AuroraView".to_string(),
            width: 800,
            height: 600,
            url: None,
            html: None,
            parent_handle: None,
            asset_root: None,
            settings: WebViewSettingsImpl::default(),
        }
    }
}

/// Backend factory
///
/// Creates WebView backend instances based on configuration.
/// Supports environment variable override via `AURORAVIEW_BACKEND`.
pub struct BackendFactory;

impl BackendFactory {
    /// Environment variable for backend selection
    pub const ENV_BACKEND: &'static str = "AURORAVIEW_BACKEND";

    /// Create a WebView backend based on configuration
    ///
    /// The backend type can be overridden via the `AURORAVIEW_BACKEND` environment variable.
    pub fn create(config: &BackendConfig) -> WebViewResult<Box<dyn WebViewBackend>> {
        let backend_type = Self::resolve_backend_type(config)?;

        match backend_type {
            BackendType::Wry => {
                // Create WryBackend with settings from config
                let mut backend = WryBackend::new();
                backend.apply_settings(config.settings.clone());
                Ok(Box::new(backend))
            }
            #[cfg(target_os = "windows")]
            BackendType::WebView2 => {
                // WebView2 is also handled via Wry on Windows (wry uses WebView2)
                let mut backend = WryBackend::new();
                backend.apply_settings(config.settings.clone());
                Ok(Box::new(backend))
            }
            #[cfg(target_os = "macos")]
            BackendType::WKWebView => Err(WebViewError::Internal(
                "WKWebView backend not yet implemented".into(),
            )),
            #[cfg(target_os = "linux")]
            BackendType::WebKitGTK => Err(WebViewError::Internal(
                "WebKitGTK backend not yet implemented".into(),
            )),
        }
    }

    /// Resolve backend type from config and environment
    fn resolve_backend_type(config: &BackendConfig) -> WebViewResult<BackendType> {
        // Check environment variable first
        if let Ok(env_backend) = std::env::var(Self::ENV_BACKEND) {
            return env_backend.parse();
        }

        // Use config value
        Ok(config.backend_type)
    }

    /// Get the default backend type for the current platform
    pub fn default_backend() -> BackendType {
        BackendType::default()
    }

    /// List available backends for the current platform
    pub fn available_backends() -> Vec<BackendType> {
        let mut backends = vec![BackendType::Wry];

        #[cfg(target_os = "windows")]
        backends.push(BackendType::WebView2);

        #[cfg(target_os = "macos")]
        backends.push(BackendType::WKWebView);

        #[cfg(target_os = "linux")]
        backends.push(BackendType::WebKitGTK);

        backends
    }
}
