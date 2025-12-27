//! AuroraView Plugin System
//!
//! This crate provides a plugin architecture for extending AuroraView with
//! native desktop capabilities. Inspired by Tauri's plugin system.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    JavaScript API                            │
//! │  window.auroraview.fs.readFile()                            │
//! │  window.auroraview.clipboard.write()                        │
//! │  window.auroraview.shell.open()                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │              Plugin Command Router                           │
//! │  invoke("plugin:fs|read_file", { path, ... })               │
//! ├────────────┬────────────┬────────────┬──────────────────────┤
//! │ fs_plugin  │ clipboard  │ shell      │ dialog               │
//! ├────────────┴────────────┴────────────┴──────────────────────┤
//! │               auroraview-plugins                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Crate Structure
//!
//! - `auroraview-plugin-core` - Core plugin framework (traits, router, scope)
//! - `auroraview-plugin-fs` - File system plugin
//! - `auroraview-plugins` - This crate, aggregates all plugins
//!
//! ## Available Plugins
//!
//! - **fs**: File system operations (read, write, list, etc.)
//! - **clipboard**: System clipboard access (read/write text, images)
//! - **shell**: Execute commands, open URLs/files
//! - **dialog**: Native file/folder dialogs
//! - **process**: Process spawning with IPC support
//! - **browser_bridge**: Browser extension WebSocket bridge
//!
//! ## Command Format
//!
//! Plugin commands use the format: `plugin:<plugin_name>|<command_name>`
//!
//! Example: `plugin:fs|read_file`

// Re-export core types
pub use auroraview_plugin_core::{
    PathScope, PluginCommand, PluginError, PluginErrorCode, PluginEventCallback, PluginHandler,
    PluginRequest, PluginResponse, PluginResult, PluginRouter, ScopeConfig, ScopeError, ShellScope,
};

// Re-export fs plugin
pub use auroraview_plugin_fs as fs;

// Built-in plugins (still in this crate for now)
pub mod browser_bridge;
pub mod clipboard;
pub mod dialog;
pub mod extensions;
pub mod process;
pub mod shell;

use std::sync::Arc;

/// Create a plugin router with all built-in plugins registered
pub fn create_router() -> PluginRouter {
    let mut router = PluginRouter::new();
    let event_callback = router.event_callback_ref();

    // Register built-in plugins
    router.register("fs", Arc::new(fs::FsPlugin::new()));
    router.register("clipboard", Arc::new(clipboard::ClipboardPlugin::new()));
    router.register("shell", Arc::new(shell::ShellPlugin::new()));
    router.register("dialog", Arc::new(dialog::DialogPlugin::new()));

    // Create process plugin with shared event callback
    let process_plugin = process::ProcessPlugin::with_event_callback(Arc::clone(&event_callback));
    router.register("process", Arc::new(process_plugin));

    // Create browser bridge plugin with shared event callback
    let browser_bridge_plugin =
        browser_bridge::BrowserBridgePlugin::with_event_callback(event_callback);
    router.register("browser_bridge", Arc::new(browser_bridge_plugin));

    // Register extensions plugin for Chrome Extension API compatibility
    router.register("extensions", Arc::new(extensions::ExtensionsPlugin::new()));

    router
}

/// Create a plugin router with custom scope configuration
pub fn create_router_with_scope(scope: ScopeConfig) -> PluginRouter {
    let mut router = create_router();
    router.set_scope(scope);
    router
}
