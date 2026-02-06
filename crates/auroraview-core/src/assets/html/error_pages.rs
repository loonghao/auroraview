//! Error page HTML templates
//!
//! This module provides user-friendly error pages for various error conditions.
//! All pages follow the AuroraView design language and include helpful debugging information.

/// Generate a 404 Not Found error page
pub fn not_found_page(requested_path: &str, available_assets: Option<Vec<&str>>) -> String {
    let asset_list = if let Some(assets) = available_assets {
        let items: Vec<String> = assets
            .iter()
            .take(20)
            .map(|a| format!("<li><code>{}</code></li>", html_escape(a)))
            .collect();
        let more = if assets.len() > 20 {
            format!("<li>... and {} more files</li>", assets.len() - 20)
        } else {
            String::new()
        };
        format!(
            r#"
            <details class="asset-list">
                <summary>Available assets ({} files)</summary>
                <ul>{}{}</ul>
            </details>
            "#,
            assets.len(),
            items.join("\n"),
            more
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>404 - Not Found | AuroraView</title>
    <style>
        {base_styles}
        .error-code {{ color: #ef4444; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error-box">
            <div class="icon">📄</div>
            <h1><span class="error-code">404</span> Not Found</h1>
            <p class="message">The requested resource could not be found.</p>
            <div class="details">
                <p><strong>Requested path:</strong> <code>{requested_path}</code></p>
            </div>
            {asset_list}
            <div class="actions">
                <button onclick="location.reload()">Reload Page</button>
                <button onclick="window.history.back()">Go Back</button>
            </div>
        </div>
        <div class="debug-info">
            <p>AuroraView • <span id="timestamp"></span></p>
        </div>
    </div>
    <script>
        document.getElementById('timestamp').textContent = new Date().toLocaleString();
    </script>
</body>
</html>"#,
        base_styles = BASE_ERROR_STYLES,
        requested_path = html_escape(requested_path),
        asset_list = asset_list
    )
}

/// Generate a 500 Internal Error page
pub fn internal_error_page(error_message: &str, details: Option<&str>) -> String {
    let details_section = details
        .map(|d| format!(r#"<pre class="error-details">{}</pre>"#, html_escape(d)))
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>500 - Internal Error | AuroraView</title>
    <style>
        {base_styles}
        .error-code {{ color: #f97316; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error-box">
            <div class="icon">⚠️</div>
            <h1><span class="error-code">500</span> Internal Error</h1>
            <p class="message">{error_message}</p>
            {details_section}
            <div class="actions">
                <button onclick="location.reload()">Retry</button>
                <button onclick="window.history.back()">Go Back</button>
            </div>
        </div>
        <div class="debug-info">
            <p>AuroraView • <span id="timestamp"></span></p>
        </div>
    </div>
    <script>
        document.getElementById('timestamp').textContent = new Date().toLocaleString();
    </script>
</body>
</html>"#,
        base_styles = BASE_ERROR_STYLES,
        error_message = html_escape(error_message),
        details_section = details_section
    )
}

/// Generate a Python backend error page
pub fn python_error_page(error_type: &str, error_message: &str, traceback: Option<&str>) -> String {
    let traceback_section = traceback
        .map(|tb| {
            format!(
                r#"
                <details class="traceback" open>
                    <summary>Python Traceback</summary>
                    <pre>{}</pre>
                </details>
                "#,
                html_escape(tb)
            )
        })
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Python Error | AuroraView</title>
    <style>
        {base_styles}
        .error-code {{ color: #8b5cf6; }}
        .traceback pre {{
            background: #1e1e2e;
            color: #cdd6f4;
            padding: 16px;
            border-radius: 8px;
            overflow-x: auto;
            font-size: 12px;
            line-height: 1.5;
            text-align: left;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error-box wide">
            <div class="icon">🐍</div>
            <h1><span class="error-code">Python Error</span></h1>
            <p class="error-type"><strong>{error_type}</strong></p>
            <p class="message">{error_message}</p>
            {traceback_section}
            <div class="actions">
                <button onclick="location.reload()">Retry</button>
                <button onclick="navigator.clipboard.writeText(document.querySelector('.traceback pre')?.textContent || '')">Copy Traceback</button>
            </div>
        </div>
        <div class="debug-info">
            <p>AuroraView • <span id="timestamp"></span></p>
        </div>
    </div>
    <script>
        document.getElementById('timestamp').textContent = new Date().toLocaleString();
    </script>
</body>
</html>"#,
        base_styles = BASE_ERROR_STYLES,
        error_type = html_escape(error_type),
        error_message = html_escape(error_message),
        traceback_section = traceback_section
    )
}

/// Generate a connection error page (WebView cannot connect to backend)
pub fn connection_error_page(target: &str, error: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Connection Error | AuroraView</title>
    <style>
        {base_styles}
        .error-code {{ color: #ec4899; }}
        .retry-counter {{ font-size: 14px; color: #6b7280; margin-top: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error-box">
            <div class="icon">🔌</div>
            <h1><span class="error-code">Connection Error</span></h1>
            <p class="message">Failed to connect to the backend service.</p>
            <div class="details">
                <p><strong>Target:</strong> <code>{target}</code></p>
                <p><strong>Error:</strong> {error}</p>
            </div>
            <div class="actions">
                <button onclick="location.reload()" id="retry-btn">Retry Connection</button>
            </div>
            <p class="retry-counter">Auto-retry in <span id="countdown">5</span> seconds...</p>
        </div>
        <div class="debug-info">
            <p>AuroraView • <span id="timestamp"></span></p>
        </div>
    </div>
    <script>
        document.getElementById('timestamp').textContent = new Date().toLocaleString();
        let countdown = 5;
        const countdownEl = document.getElementById('countdown');
        const timer = setInterval(() => {{
            countdown--;
            countdownEl.textContent = countdown;
            if (countdown <= 0) {{
                clearInterval(timer);
                location.reload();
            }}
        }}, 1000);
    </script>
</body>
</html>"#,
        base_styles = BASE_ERROR_STYLES,
        target = html_escape(target),
        error = html_escape(error)
    )
}

/// Generate a startup error page (Python backend failed to start)
pub fn startup_error_page(
    error_message: &str,
    python_output: Option<&str>,
    entry_point: Option<&str>,
) -> String {
    let output_section = python_output
        .map(|out| {
            format!(
                r#"
                <details class="python-output" open>
                    <summary>Python Output</summary>
                    <pre>{}</pre>
                </details>
                "#,
                html_escape(out)
            )
        })
        .unwrap_or_default();

    let entry_info = entry_point
        .map(|ep| {
            format!(
                "<p><strong>Entry point:</strong> <code>{}</code></p>",
                html_escape(ep)
            )
        })
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Startup Error | AuroraView</title>
    <style>
        {base_styles}
        .error-code {{ color: #dc2626; }}
        .python-output pre {{
            background: #1e1e2e;
            color: #cdd6f4;
            padding: 16px;
            border-radius: 8px;
            overflow-x: auto;
            font-size: 12px;
            line-height: 1.5;
            text-align: left;
            max-height: 400px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error-box wide">
            <div class="icon">🚫</div>
            <h1><span class="error-code">Startup Failed</span></h1>
            <p class="message">The Python backend failed to start.</p>
            <div class="details">
                <p><strong>Error:</strong> {error_message}</p>
                {entry_info}
            </div>
            {output_section}
            <div class="tips">
                <h3>Troubleshooting Tips:</h3>
                <ul>
                    <li>Check if Python dependencies are installed</li>
                    <li>Verify the entry point function exists</li>
                    <li>Look for syntax errors in the Python code</li>
                    <li>Check if required environment variables are set</li>
                </ul>
            </div>
            <div class="actions">
                <button onclick="location.reload()">Retry</button>
                <button onclick="navigator.clipboard.writeText(document.querySelector('.python-output pre')?.textContent || '')">Copy Output</button>
            </div>
        </div>
        <div class="debug-info">
            <p>AuroraView • <span id="timestamp"></span></p>
        </div>
    </div>
    <script>
        document.getElementById('timestamp').textContent = new Date().toLocaleString();
    </script>
</body>
</html>"#,
        base_styles = BASE_ERROR_STYLES,
        error_message = html_escape(error_message),
        entry_info = entry_info,
        output_section = output_section
    )
}

/// Generate a generic loading page with error state support
pub fn loading_with_error(status: &str, error: Option<&str>) -> String {
    let error_section = error
        .map(|e| {
            format!(
                r#"<div class="error-message"><span class="error-icon">⚠️</span> {}</div>"#,
                html_escape(e)
            )
        })
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Loading... | AuroraView</title>
    <style>
        {base_styles}
        .spinner {{
            width: 48px;
            height: 48px;
            border: 4px solid #e5e7eb;
            border-top-color: #3b82f6;
            border-radius: 50%;
            animation: spin 1s linear infinite;
            margin: 0 auto 24px;
        }}
        @keyframes spin {{
            to {{ transform: rotate(360deg); }}
        }}
        .status {{ color: #6b7280; font-size: 14px; }}
        .error-message {{
            background: #fef2f2;
            border: 1px solid #fecaca;
            color: #dc2626;
            padding: 12px 16px;
            border-radius: 8px;
            margin-top: 16px;
            display: flex;
            align-items: center;
            gap: 8px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error-box">
            <div class="spinner"></div>
            <h1>Loading</h1>
            <p class="status">{status}</p>
            {error_section}
        </div>
    </div>
</body>
</html>"#,
        base_styles = BASE_ERROR_STYLES,
        status = html_escape(status),
        error_section = error_section
    )
}

/// Base CSS styles shared by all error pages
const BASE_ERROR_STYLES: &str = r#"
    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 20px;
    }
    .container {
        width: 100%;
        max-width: 600px;
    }
    .error-box {
        background: white;
        border-radius: 16px;
        padding: 40px;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
        text-align: center;
    }
    .error-box.wide {
        max-width: 800px;
    }
    .icon {
        font-size: 64px;
        margin-bottom: 20px;
    }
    h1 {
        font-size: 28px;
        color: #1f2937;
        margin-bottom: 12px;
    }
    .message {
        color: #6b7280;
        font-size: 16px;
        margin-bottom: 24px;
    }
    .error-type {
        color: #374151;
        font-size: 18px;
        margin-bottom: 8px;
    }
    .details {
        background: #f9fafb;
        border-radius: 8px;
        padding: 16px;
        margin-bottom: 20px;
        text-align: left;
    }
    .details p {
        margin-bottom: 8px;
        color: #4b5563;
    }
    .details p:last-child {
        margin-bottom: 0;
    }
    code {
        background: #e5e7eb;
        padding: 2px 6px;
        border-radius: 4px;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', Consolas, monospace;
        font-size: 14px;
        color: #1f2937;
    }
    .actions {
        display: flex;
        gap: 12px;
        justify-content: center;
        margin-top: 24px;
    }
    button {
        background: #3b82f6;
        color: white;
        border: none;
        padding: 12px 24px;
        border-radius: 8px;
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
        transition: background 0.2s;
    }
    button:hover {
        background: #2563eb;
    }
    button:nth-child(2) {
        background: #6b7280;
    }
    button:nth-child(2):hover {
        background: #4b5563;
    }
    .debug-info {
        text-align: center;
        margin-top: 20px;
        color: rgba(255, 255, 255, 0.7);
        font-size: 12px;
    }
    details {
        text-align: left;
        margin-top: 16px;
    }
    details summary {
        cursor: pointer;
        color: #3b82f6;
        font-weight: 500;
        padding: 8px 0;
    }
    details ul {
        list-style: none;
        padding-left: 16px;
    }
    details li {
        padding: 4px 0;
        color: #4b5563;
        font-size: 13px;
    }
    .error-details {
        background: #fef2f2;
        border: 1px solid #fecaca;
        color: #dc2626;
        padding: 12px 16px;
        border-radius: 8px;
        overflow-x: auto;
        font-size: 13px;
        text-align: left;
        margin-top: 16px;
    }
    .tips {
        background: #eff6ff;
        border: 1px solid #bfdbfe;
        border-radius: 8px;
        padding: 16px;
        margin-top: 20px;
        text-align: left;
    }
    .tips h3 {
        color: #1e40af;
        font-size: 14px;
        margin-bottom: 12px;
    }
    .tips ul {
        color: #3b82f6;
        padding-left: 20px;
    }
    .tips li {
        margin-bottom: 6px;
        font-size: 13px;
    }
"#;

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_page() {
        let page = not_found_page("/missing.js", None);
        assert!(page.contains("404"));
        assert!(page.contains("Not Found"));
        assert!(page.contains("/missing.js"));
    }

    #[test]
    fn test_not_found_page_with_assets() {
        let assets = vec!["index.html", "app.js", "style.css"];
        let page = not_found_page("/missing.js", Some(assets));
        assert!(page.contains("index.html"));
        assert!(page.contains("3 files"));
    }

    #[test]
    fn test_python_error_page() {
        let page = python_error_page(
            "ImportError",
            "No module named 'xyz'",
            Some("Traceback (most recent call last):\n  File \"main.py\", line 1\nImportError"),
        );
        assert!(page.contains("Python Error"));
        assert!(page.contains("ImportError"));
        assert!(page.contains("No module named"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }
}
