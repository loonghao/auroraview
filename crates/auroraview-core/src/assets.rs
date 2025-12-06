//! Static assets for AuroraView
//!
//! This module provides embedded static assets including:
//! - Loading HTML page
//! - JavaScript utilities (event bridge, context menu, etc.)
//! - BOM (Browser Object Model) scripts

use rust_embed::RustEmbed;

/// Embedded static assets
#[derive(RustEmbed)]
#[folder = "src/assets/"]
pub struct Assets;

/// Get the loading HTML page content
pub fn get_loading_html() -> String {
    Assets::get("html/loading.html")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_else(|| {
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: white;
            font-family: system-ui, -apple-system, sans-serif;
        }
        .spinner {
            width: 50px;
            height: 50px;
            border: 3px solid rgba(255,255,255,0.3);
            border-radius: 50%;
            border-top-color: #fff;
            animation: spin 1s linear infinite;
        }
        @keyframes spin {
            to { transform: rotate(360deg); }
        }
        .container { text-align: center; }
        h1 { font-size: 1.5rem; margin-top: 1rem; }
    </style>
</head>
<body>
    <div class="container">
        <div class="spinner"></div>
        <h1>Loading...</h1>
    </div>
</body>
</html>"#
                .to_string()
        })
}

/// Get the event bridge JavaScript code
pub fn get_event_bridge_js() -> String {
    Assets::get("js/core/event_bridge.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the context menu JavaScript code
pub fn get_context_menu_js() -> String {
    Assets::get("js/features/context_menu.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the emit event JavaScript code
pub fn get_emit_event_js() -> String {
    Assets::get("js/runtime/emit_event.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the load URL JavaScript code
pub fn get_load_url_js() -> String {
    Assets::get("js/runtime/load_url.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

// ========================================
// BOM (Browser Object Model) Scripts
// ========================================

/// Get the navigation tracker JavaScript code
pub fn get_navigation_tracker_js() -> String {
    Assets::get("js/bom/navigation_tracker.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the DOM events JavaScript code
pub fn get_dom_events_js() -> String {
    Assets::get("js/bom/dom_events.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the browsing data JavaScript code
pub fn get_browsing_data_js() -> String {
    Assets::get("js/bom/browsing_data.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the navigation API JavaScript code
pub fn get_navigation_api_js() -> String {
    Assets::get("js/bom/navigation_api.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the zoom API JavaScript code
pub fn get_zoom_api_js() -> String {
    Assets::get("js/bom/zoom_api.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

// ========================================
// Core Bridge Scripts
// ========================================

/// Get the bridge stub JavaScript code
///
/// This stub creates a minimal window.auroraview namespace before the full
/// event bridge is loaded. Use this in DCC environments where timing may vary.
///
/// The stub:
/// - Creates a placeholder `window.auroraview` with queuing support
/// - Queues any `call()`, `send_event()`, `on()` calls made before bridge init
/// - Provides `whenReady()` Promise API for safe async initialization
/// - Automatically replays queued calls when real bridge initializes
///
/// # Example
///
/// ```javascript
/// // In DCC frontend code (before bridge is ready)
/// window.auroraview.whenReady().then(function(av) {
///     av.call('api.myMethod', { param: 'value' });
/// });
/// ```
pub fn get_bridge_stub_js() -> String {
    Assets::get("js/core/bridge_stub.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the state bridge JavaScript code
pub fn get_state_bridge_js() -> String {
    Assets::get("js/core/state_bridge.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the command bridge JavaScript code
pub fn get_command_bridge_js() -> String {
    Assets::get("js/core/command_bridge.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the channel bridge JavaScript code
pub fn get_channel_bridge_js() -> String {
    Assets::get("js/core/channel_bridge.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

// ========================================
// Plugin Scripts
// ========================================

/// Get the file system plugin JavaScript code
pub fn get_fs_plugin_js() -> String {
    Assets::get("js/plugins/fs.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get all plugin JavaScript code concatenated
pub fn get_all_plugins_js() -> String {
    let mut scripts = Vec::new();

    // Add file system plugin
    let fs_js = get_fs_plugin_js();
    if !fs_js.is_empty() {
        scripts.push(fs_js);
    }

    // Future plugins will be added here

    scripts.join("\n\n")
}

// ========================================
// Generic Asset Access
// ========================================

/// Get any JavaScript asset by path
///
/// # Arguments
/// * `path` - Path relative to js/ directory (e.g., "core/event_bridge.js")
pub fn get_js_asset(path: &str) -> Option<String> {
    let full_path = format!("js/{}", path);
    Assets::get(&full_path).map(|f| String::from_utf8_lossy(&f.data).to_string())
}

/// Get TypeScript definition file
pub fn get_typescript_definitions() -> String {
    Assets::get("types/auroraview.d.ts")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Build JavaScript to load a URL
pub fn build_load_url_script(url: &str) -> String {
    format!(
        r#"window.location.href = "{}";"#,
        url.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_html_not_empty() {
        let html = get_loading_html();
        assert!(!html.is_empty());
        assert!(html.contains("Loading") || html.contains("loading"));
    }

    #[test]
    fn test_build_load_url_script() {
        let script = build_load_url_script("https://example.com");
        assert!(script.contains("https://example.com"));
        assert!(script.contains("window.location.href"));
    }

    #[test]
    fn test_bom_scripts_available() {
        // These may be empty if assets aren't embedded, but shouldn't panic
        let _ = get_navigation_tracker_js();
        let _ = get_dom_events_js();
        let _ = get_browsing_data_js();
        let _ = get_navigation_api_js();
        let _ = get_zoom_api_js();
    }
}
