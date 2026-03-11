//! AuroraView Extensions - Browser Extension Compatibility Layer
//!
//! This crate provides 1:1 compatibility with Chrome Extension APIs (Manifest V3),
//! allowing browser extensions to run in AuroraView with minimal or no modifications.
//!
//! ## Supported APIs
//!
//! | API | Status | Description |
//! |-----|--------|-------------|
//! | `chrome.runtime` | ✅ Full | Extension lifecycle and messaging |
//! | `chrome.storage` | ✅ Full | Local, sync, session, and managed storage |
//! | `chrome.tabs` | ✅ Full | Tab management (single-tab mode) |
//! | `chrome.sidePanel` | ✅ Full | Side panel API |
//! | `chrome.action` | ✅ Full | Extension action (toolbar button) |
//! | `chrome.scripting` | ✅ Full | Script injection |
//! | `chrome.contextMenus` | ✅ Full | Context menu API |
//! | `chrome.notifications` | ✅ Full | System notifications |
//! | `chrome.alarms` | ✅ Full | Scheduled alarms |
//! | `chrome.webRequest` | ⚡ Basic | Request interception |
//! | `chrome.windows` | ⚡ Basic | Window management |
//! | `chrome.commands` | ✅ Full | Keyboard shortcuts |
//! | `chrome.permissions` | ✅ Full | Permission management |
//! | `chrome.identity` | ⚡ Basic | OAuth authentication |
//! | `chrome.declarativeNetRequest` | ⚡ Basic | Declarative request blocking |
//! | `chrome.offscreen` | ⚡ Basic | Offscreen documents |
//! | `chrome.i18n` | ✅ Full | Internationalization |
//!
//! ## WXT Framework Compatibility
//!
//! This crate includes a compatibility layer for the WXT framework, allowing
//! extensions built with WXT to run without modification:
//!
//! - `wxt/storage` - Storage API wrapper
//! - `wxt/browser` - Browser API alias
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    AuroraView Application                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
//! │  │  Main View  │  │ Side Panel  │  │  Extension Popup    │  │
//! │  │  (WebView)  │  │  (WebView)  │  │     (WebView)       │  │
//! │  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
//! │         │                │                     │             │
//! │         └────────────────┼─────────────────────┘             │
//! │                          │                                   │
//! │                ┌─────────▼─────────┐                         │
//! │                │  Extension Host   │                         │
//! │                │  (This Crate)     │                         │
//! │                ├───────────────────┤                         │
//! │                │ • Manifest Parser │                         │
//! │                │ • API Polyfills   │                         │
//! │                │ • Message Router  │                         │
//! │                │ • Storage Backend │                         │
//! │                │ • Script Injector │                         │
//! │                │ • Service Worker  │                         │
//! │                └───────────────────┘                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use auroraview_extensions::{ExtensionHost, ExtensionConfig};
//!
//! // Create extension host
//! let host = ExtensionHost::new(ExtensionConfig {
//!     extensions_dir: "/path/to/extensions".into(),
//!     storage_dir: "/path/to/storage".into(),
//! });
//!
//! // Load extensions
//! host.load_extensions().await?;
//!
//! // Get side panel HTML for an extension
//! if let Some(panel) = host.get_side_panel("extension-id") {
//!     // Create WebView for side panel
//! }
//! ```

pub mod apis;
pub mod error;
pub mod host;
pub mod injection;
pub mod installer;
pub mod js_assets;
pub mod manifest;
pub mod polyfill;

pub mod runtime;
pub mod service_worker;
pub mod storage;
pub mod view_manager;

pub use error::{ExtensionError, ExtensionResult};
pub use host::{ExtensionConfig, ExtensionHost, LoadedExtension};
pub use injection::ScriptInjector;
pub use installer::{
    archive_suffix_from_url, extract_crx_archive, extract_store_extension_id, find_extension_root,
    resolve_extension_source_url, validate_extension_dir, ExtensionSourceKind,
    ResolvedExtensionSource,
};

pub use manifest::{Manifest, ManifestV3, Permission};

pub use polyfill::{
    generate_content_script_polyfill, generate_polyfill_from_sdk, generate_wxt_shim,
};
pub use runtime::ExtensionRuntime;
pub use service_worker::{
    create_service_worker_manager, MessageSender, ServiceWorkerManager, ServiceWorkerMessage,
    ServiceWorkerMessageType, ServiceWorkerRegistration, ServiceWorkerState,
    SharedServiceWorkerManager,
};
pub use storage::StorageBackend;
pub use view_manager::{
    CdpConnectionInfo, CreateWebViewCallback, ExtensionViewConfig, ExtensionViewInfo,
    ExtensionViewManager, ExtensionViewState, ExtensionViewType, OpenDevToolsCallback,
};

/// Extension ID type (typically a 32-character string)
pub type ExtensionId = String;

/// Re-export for convenience
pub mod prelude {
    pub use crate::{
        generate_polyfill_from_sdk, generate_wxt_shim, ExtensionConfig, ExtensionError,
        ExtensionHost, ExtensionId, ExtensionResult, LoadedExtension, Manifest, ManifestV3,
        Permission, ServiceWorkerManager, ServiceWorkerState,
    };
}
