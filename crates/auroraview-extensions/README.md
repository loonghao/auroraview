# AuroraView Extensions

[![Crates.io](https://img.shields.io/crates/v/auroraview-extensions.svg)](https://crates.io/crates/auroraview-extensions)
[![Documentation](https://docs.rs/auroraview-extensions/badge.svg)](https://docs.rs/auroraview-extensions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Browser extension compatibility layer providing 1:1 Chrome Extension API support for WebView applications.

## Features

- **Chrome Extension API Compatibility** - Run browser extensions with minimal or no modifications
- **Manifest V3 Support** - Full support for the latest Chrome extension manifest format
- **WXT Framework Compatibility** - Extensions built with WXT work out of the box
- **Storage Backend** - Local, sync, session, and managed storage APIs
- **Service Worker Support** - Background service worker lifecycle management
- **Content Script Injection** - Automatic script injection based on URL patterns

## Supported APIs

| API | Status | Description |
|-----|--------|-------------|
| `chrome.runtime` | ✅ Full | Extension lifecycle and messaging |
| `chrome.storage` | ✅ Full | Local, sync, session, and managed storage |
| `chrome.tabs` | ✅ Full | Tab management (single-tab mode) |
| `chrome.sidePanel` | ✅ Full | Side panel API |
| `chrome.action` | ✅ Full | Extension action (toolbar button) |
| `chrome.scripting` | ✅ Full | Script injection |
| `chrome.contextMenus` | ✅ Full | Context menu API |
| `chrome.notifications` | ✅ Full | System notifications |
| `chrome.alarms` | ✅ Full | Scheduled alarms |
| `chrome.webRequest` | ⚡ Basic | Request interception |
| `chrome.windows` | ⚡ Basic | Window management |
| `chrome.commands` | ✅ Full | Keyboard shortcuts |
| `chrome.permissions` | ✅ Full | Permission management |
| `chrome.identity` | ⚡ Basic | OAuth authentication |
| `chrome.i18n` | ✅ Full | Internationalization |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
auroraview-extensions = "0.4"
```

## Usage

```rust
use auroraview_extensions::{ExtensionHost, ExtensionConfig};

// Create extension host
let host = ExtensionHost::new(ExtensionConfig {
    extensions_dir: "/path/to/extensions".into(),
    storage_dir: "/path/to/storage".into(),
});

// Load extensions
host.load_extensions().await?;

// Get side panel HTML for an extension
if let Some(panel) = host.get_side_panel("extension-id") {
    // Create WebView for side panel
}
```

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    WebView Application                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Main View  │  │ Side Panel  │  │  Extension Popup    │  │
│  │  (WebView)  │  │  (WebView)  │  │     (WebView)       │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                     │             │
│         └────────────────┼─────────────────────┘             │
│                          │                                   │
│                ┌─────────▼─────────┐                         │
│                │  Extension Host   │                         │
│                │  (This Crate)     │                         │
│                ├───────────────────┤                         │
│                │ • Manifest Parser │                         │
│                │ • API Polyfills   │                         │
│                │ • Message Router  │                         │
│                │ • Storage Backend │                         │
│                │ • Script Injector │                         │
│                │ • Service Worker  │                         │
│                └───────────────────┘                         │
└─────────────────────────────────────────────────────────────┘
```

## License

MIT License - see [LICENSE](LICENSE) for details.
