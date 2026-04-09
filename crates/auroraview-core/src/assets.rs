//! Static assets for AuroraView
//!
//! This module provides embedded static assets including:
//! - Loading HTML page (Vite-built React page)
//! - Error HTML page (Next.js-style diagnostics)
//! - Browser controller HTML page
//! - JavaScript utilities (event bridge, context menu, etc.)
//! - BOM (Browser Object Model) scripts
//! - HTML error page templates (generated at runtime)

use rust_embed::RustEmbed;

use crate::utils::escape_js_string;

// Re-export HTML templates module
#[path = "assets/html/mod.rs"]
pub mod html;

pub use html::{
    connection_error_page, internal_error_page, loading_with_error, not_found_page,
    python_error_page, startup_error_page,
};

/// Embedded static assets
#[derive(RustEmbed)]
#[folder = "src/assets/"]
pub struct Assets;

// ========================================
// HTML Pages (from auroraview-assets)
// ========================================

/// Get the loading HTML page content
///
/// Returns a modern React-based loading screen with Aurora animation
/// and progress indication, built via Vite from auroraview-assets.
///
/// When built frontend assets are not available in a source checkout,
/// fall back to a lightweight embedded loading page instead of panicking.
pub fn get_loading_html() -> String {
    auroraview_assets::get_page_html(auroraview_assets::Page::Loading)
        .unwrap_or_else(|_| fallback_loading_html())
}

fn fallback_loading_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Loading... | AuroraView</title>
    <style>
        body {
            margin: 0;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
            color: #e2e8f0;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
        }
        #root {
            width: min(520px, calc(100% - 48px));
        }
        .panel {
            background: rgba(15, 23, 42, 0.88);
            border: 1px solid rgba(148, 163, 184, 0.2);
            border-radius: 20px;
            padding: 32px;
            text-align: center;
            box-shadow: 0 24px 60px rgba(0, 0, 0, 0.35);
        }
        .spinner {
            width: 44px;
            height: 44px;
            margin: 0 auto 20px;
            border-radius: 999px;
            border: 4px solid rgba(148, 163, 184, 0.25);
            border-top-color: #38bdf8;
            animation: spin 1s linear infinite;
        }
        @keyframes spin {
            to { transform: rotate(360deg); }
        }
        .status {
            margin: 0;
            color: #cbd5e1;
            line-height: 1.6;
        }
    </style>
</head>
<body>
    <div id="root">
        <main class="panel">
            <div class="spinner"></div>
            <h1>Loading AuroraView</h1>
            <p class="status">Built frontend loading assets were not found; using embedded fallback page.</p>
        </main>
    </div>
</body>
</html>"#
        .to_string()
}


/// Get the error HTML page content
///
/// Returns a Next.js-style error overlay with full diagnostics,
/// stack traces, and developer-friendly error information.
///
/// The page accepts URL parameters to customize the error display:
/// - `code`: HTTP status code (e.g., "500", "404")
/// - `title`: Error title (e.g., "Internal Server Error")
/// - `message`: User-friendly error message
/// - `details`: Technical details (shown in a code block)
/// - `url`: The URL that caused the error (for retry functionality)
///
/// When built frontend assets are not available in a source checkout,
/// fall back to an embedded shell that keeps `build_error_page()` compatible.
pub fn get_error_html() -> String {
    auroraview_assets::get_page_html(auroraview_assets::Page::Error)
        .unwrap_or_else(|_| fallback_error_html())
}

fn fallback_error_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Error | AuroraView</title>
    <style>
        body {
            margin: 0;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 24px;
            background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
            color: #e2e8f0;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
        }
        .panel {
            width: min(720px, 100%);
            background: rgba(15, 23, 42, 0.88);
            border: 1px solid rgba(148, 163, 184, 0.2);
            border-radius: 20px;
            padding: 32px;
            box-shadow: 0 24px 60px rgba(0, 0, 0, 0.35);
        }
        .badge {
            display: inline-flex;
            align-items: center;
            gap: 8px;
            padding: 6px 12px;
            border-radius: 999px;
            background: rgba(248, 113, 113, 0.12);
            color: #fca5a5;
            font-size: 12px;
            text-transform: uppercase;
            letter-spacing: 0.08em;
        }
        h1 {
            margin: 18px 0 10px;
            font-size: 32px;
            line-height: 1.2;
        }
        #error-code {
            color: #f97316;
        }
        #error-message {
            margin: 0;
            color: #cbd5e1;
            font-size: 16px;
            line-height: 1.6;
        }
        #details-wrapper {
            display: none;
            margin-top: 20px;
        }
        #error-details {
            margin: 0;
            padding: 16px;
            background: rgba(15, 23, 42, 0.8);
            border: 1px solid rgba(148, 163, 184, 0.18);
            border-radius: 12px;
            color: #e2e8f0;
            white-space: pre-wrap;
            word-break: break-word;
            font-family: 'Cascadia Code', 'SFMono-Regular', Consolas, monospace;
            font-size: 13px;
            line-height: 1.5;
        }
        .actions {
            display: flex;
            gap: 12px;
            margin-top: 24px;
            flex-wrap: wrap;
        }
        button {
            border: 0;
            border-radius: 10px;
            padding: 10px 16px;
            cursor: pointer;
            font-weight: 600;
        }
        .primary {
            background: #38bdf8;
            color: #082f49;
        }
        .secondary {
            background: rgba(148, 163, 184, 0.14);
            color: #e2e8f0;
        }
    </style>
</head>
<body>
    <main class="panel">
        <div class="badge">AuroraView fallback</div>
        <h1><span id="error-code">500</span> <span id="error-title">Internal Error</span></h1>
        <p id="error-message">AuroraView could not load the built error page and is using an embedded fallback.</p>
        <section id="details-wrapper">
            <pre id="error-details"></pre>
        </section>
        <div class="actions">
            <button class="primary" onclick="location.reload()">Retry</button>
            <button class="secondary" onclick="window.history.back()">Go Back</button>
        </div>
    </main>
</body>
</html>"#
        .to_string()
}


/// Get the browser controller HTML page content
///
/// Returns an Edge-style browser controller UI with modern React components.
///
/// Features:
/// - Tab bar with rounded tabs, favicons, and close buttons
/// - Navigation toolbar (back, forward, reload, home)
/// - URL/search bar with modern styling
/// - Theme support (light/dark)
pub fn get_browser_controller_html() -> String {
    auroraview_assets::get_page_html(auroraview_assets::Page::BrowserController)
        .expect("Browser controller page should be available from auroraview-assets")
}

/// Build an error page HTML with specific error information
///
/// This function generates a complete error page by injecting error details
/// into the base error template via URL parameters.
///
/// # Arguments
/// * `code` - HTTP status code (e.g., 500, 404)
/// * `title` - Error title
/// * `message` - User-friendly error message
/// * `details` - Optional technical details
/// * `url` - Optional URL that caused the error
pub fn build_error_page(
    code: u16,
    title: &str,
    message: &str,
    details: Option<&str>,
    url: Option<&str>,
) -> String {
    let base_html = get_error_html();

    // Build JavaScript to update the error display
    let js_update = format!(
        r#"<script>
        (function() {{
            // Store error info globally for copy function
            window._errorInfo = {{
                code: '{}',
                title: '{}',
                message: '{}',
                details: '{}',
                url: '{}'
            }};

            document.addEventListener('DOMContentLoaded', function() {{
                var info = window._errorInfo;
                document.getElementById('error-code').textContent = info.code;
                document.getElementById('error-title').textContent = info.title;
                document.getElementById('error-message').textContent = info.message;

                var detailsWrapper = document.getElementById('details-wrapper');
                var detailsEl = document.getElementById('error-details');

                if (info.details || info.url) {{
                    var text = '';
                    if (info.url) text += 'URL: ' + info.url + '\\n';
                    if (info.details) text += info.details;
                    detailsEl.textContent = text.trim();
                    detailsWrapper.style.display = 'block';
                }} else {{
                    detailsWrapper.style.display = 'none';
                }}
            }});

            // Override getErrorInfo to use injected data
            window.getErrorInfo = function() {{
                return window._errorInfo;
            }};
        }})();
        </script>"#,
        code,
        escape_js_string(title),
        escape_js_string(message),
        escape_js_string(details.unwrap_or("")),
        escape_js_string(url.unwrap_or(""))
    );

    // Insert the JavaScript before </body>
    base_html.replace("</body>", &format!("{}</body>", js_update))
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

/// Get the file drop handler JavaScript code
///
/// This script provides file drag and drop handling capabilities.
/// It intercepts drag/drop events and sends file information to Python.
/// Events emitted:
/// - file_drop_hover: When files are dragged over the window
/// - file_drop: When files are dropped
/// - file_drop_cancelled: When drag operation is cancelled
/// - file_paste: When files are pasted from clipboard
pub fn get_file_drop_js() -> String {
    Assets::get("js/bom/file_drop.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

// ========================================
// Testing Scripts
// ========================================

/// Get the Midscene AI testing bridge JavaScript code
///
/// This script provides browser-side utilities for AI-powered UI testing:
/// - DOM analysis and element location
/// - Screenshot capture
/// - Element interaction helpers
/// - Page state inspection
///
/// The bridge is accessible via `window.__midscene_bridge__` or
/// `window.auroraview.midscene` when the event bridge is loaded.
pub fn get_midscene_bridge_js() -> String {
    Assets::get("js/features/midscene_bridge.js")
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

/// Get the event utilities JavaScript code
///
/// This script provides utility functions for event handling:
/// - debounce: Delays function execution until after wait milliseconds
/// - throttle: Limits function execution to at most once per wait milliseconds
/// - once: Restricts function to single invocation
/// - onDebounced/onThrottled: Convenience wrappers for event handlers
pub fn get_event_utils_js() -> String {
    Assets::get("js/core/event_utils.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

// ========================================
// Feature Scripts
// ========================================

/// Get the screenshot JavaScript code
///
/// This script provides screenshot capture functionality using html2canvas.
/// It dynamically loads html2canvas from CDN and provides methods for:
/// - Full page screenshots
/// - Element screenshots
/// - Viewport screenshots
pub fn get_screenshot_js() -> String {
    Assets::get("js/features/screenshot.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the network interception JavaScript code
///
/// This script provides network request interception and mocking capabilities.
/// It intercepts fetch() and XMLHttpRequest to enable:
/// - Request interception with pattern matching
/// - Response mocking
/// - Network monitoring and logging
pub fn get_network_intercept_js() -> String {
    Assets::get("js/features/network_intercept.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the test callback JavaScript code
///
/// This script provides callback mechanism for AuroraTest framework
/// to receive JavaScript evaluation results asynchronously.
pub fn get_test_callback_js() -> String {
    Assets::get("js/features/test_callback.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the splash overlay JavaScript code
///
/// This script injects a loading overlay that displays while the page loads.
/// The overlay automatically fades out when the page is fully loaded.
/// Useful for showing a branded loading experience during slow network loads.
///
/// Features:
/// - Aurora-themed animated loading screen
/// - Automatically removes itself on page load
/// - Exposes `window.__auroraview_splash.show()` and `.hide()` for manual control
pub fn get_splash_overlay_js() -> String {
    Assets::get("js/features/splash_overlay.js")
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

/// Get the dialog plugin JavaScript code
pub fn get_dialog_plugin_js() -> String {
    Assets::get("js/plugins/dialog.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the clipboard plugin JavaScript code
pub fn get_clipboard_plugin_js() -> String {
    Assets::get("js/plugins/clipboard.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get the shell plugin JavaScript code
pub fn get_shell_plugin_js() -> String {
    Assets::get("js/plugins/shell.js")
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default()
}

/// Get plugin JavaScript by name
pub fn get_plugin_js(name: &str) -> Option<String> {
    match name {
        "fs" => Some(get_fs_plugin_js()),
        "dialog" => Some(get_dialog_plugin_js()),
        "clipboard" => Some(get_clipboard_plugin_js()),
        "shell" => Some(get_shell_plugin_js()),
        _ => None,
    }
}

/// List available plugin names
pub fn plugin_names() -> &'static [&'static str] {
    &["fs", "dialog", "clipboard", "shell"]
}

/// Get all plugin JavaScript code concatenated
pub fn get_all_plugins_js() -> String {
    let mut scripts = Vec::new();

    // Add all plugins
    for name in plugin_names() {
        if let Some(js) = get_plugin_js(name) {
            if !js.is_empty() {
                scripts.push(js);
            }
        }
    }

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

// ========================================
// Packed Mode Initialization
// ========================================

/// Build initialization script for packed mode
///
/// This function creates a JavaScript initialization script that includes
/// the event bridge (core IPC functionality).
///
/// Note: API methods are registered dynamically by the Python backend
/// when it receives the `__auroraview_ready` event, not via static configuration.
///
/// # Example
/// ```rust,ignore
/// use auroraview_core::assets::build_packed_init_script;
///
/// let script = build_packed_init_script();
/// ```
pub fn build_packed_init_script() -> String {
    // Just return the event bridge - API methods are registered dynamically
    get_event_bridge_js()
}

/// Build the JavaScript snippet that injects a CSP `<meta>` tag into the page.
///
/// The snippet runs synchronously before any page script, ensuring the policy
/// is active from the very start of page evaluation.
///
/// # Arguments
/// * `policy` - A valid CSP directive string, e.g.
///   `"default-src 'self'; script-src 'self' 'unsafe-inline'"`
///
/// # Safety
/// The caller is responsible for ensuring `policy` does not contain
/// unescaped single-quote characters that could break out of the JS string literal.
/// Use [`escape_csp_policy`] if the value originates from untrusted input.
pub fn build_csp_injection_script(policy: &str) -> String {
    let escaped = policy.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(){{
    var m = document.createElement('meta');
    m.httpEquiv = 'Content-Security-Policy';
    m.content = '{}';
    var head = document.head || document.documentElement;
    if (head) {{ head.insertBefore(m, head.firstChild); }}
}})();"#,
        escaped
    )
}

/// Build initialization script for packed mode with an optional CSP policy.
///
/// If `csp` is `Some(policy)`, a CSP `<meta>` injection is prepended to the
/// event-bridge script so it runs before any page content is evaluated.
pub fn build_packed_init_script_with_csp(csp: Option<&str>) -> String {
    let bridge = get_event_bridge_js();
    match csp {
        Some(policy) => {
            let csp_script = build_csp_injection_script(policy);
            format!("{}\n{}", csp_script, bridge)
        }
        None => bridge,
    }
}
