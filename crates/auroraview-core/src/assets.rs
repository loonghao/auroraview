//! Static assets for AuroraView
//!
//! This module provides embedded static assets including:
//! - Loading HTML page
//! - JavaScript utilities (event bridge, context menu, etc.)

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
}
