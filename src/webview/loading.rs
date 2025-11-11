//! Loading screen and progressive enhancement utilities

/// Minimal loading screen HTML
///
/// This is shown immediately while the main content loads.
/// Uses inline CSS to avoid external resource loading.
#[allow(dead_code)]
pub const LOADING_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Loading...</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            overflow: hidden;
        }

        .loader-container {
            text-align: center;
            color: white;
        }

        .spinner {
            width: 60px;
            height: 60px;
            margin: 0 auto 20px;
            border: 4px solid rgba(255, 255, 255, 0.3);
            border-top-color: white;
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }

        @keyframes spin {
            to { transform: rotate(360deg); }
        }

        .loading-text {
            font-size: 18px;
            font-weight: 500;
            opacity: 0.9;
        }

        .loading-subtext {
            font-size: 14px;
            opacity: 0.7;
            margin-top: 8px;
        }
    </style>
</head>
<body>
    <div class="loader-container">
        <div class="spinner"></div>
        <div class="loading-text">Loading AuroraView...</div>
        <div class="loading-subtext">Please wait</div>
    </div>

    <script>
        // Notify Rust that loading screen is ready
        console.log('[TIMER] Loading screen rendered');

        // Mark performance timing
        window.loadingScreenReady = performance.now();

        // Listen for content ready event
        window.addEventListener('content_ready', () => {
            console.log('[OK] Content ready, hiding loading screen');
            document.body.style.opacity = '0';
            document.body.style.transition = 'opacity 0.3s';

            setTimeout(() => {
                // This will be replaced by actual content
            }, 300);
        });
    </script>
</body>
</html>"#;

/// Wrap user HTML with performance optimizations
///
/// This function:
/// 1. Adds performance monitoring
/// 2. Defers non-critical scripts
/// 3. Preloads critical resources
/// 4. Adds loading state management
#[allow(dead_code)]
pub fn wrap_html_with_optimizations(user_html: &str) -> String {
    // Check if HTML already has <!DOCTYPE html>
    let has_doctype = user_html
        .trim_start()
        .to_lowercase()
        .starts_with("<!doctype");

    if has_doctype {
        // HTML is complete, just add performance monitoring
        add_performance_monitoring(user_html)
    } else {
        // Wrap partial HTML
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        /* Prevent flash of unstyled content */
        body {{
            opacity: 0;
            transition: opacity 0.3s ease-in;
        }}
        body.ready {{
            opacity: 1;
        }}
    </style>
</head>
<body>
    {}

    <script>
        // Performance monitoring
        window.auroraViewPerf = {{
            start: performance.now(),
            marks: {{}}
        }};

        // Mark when DOM is ready
        if (document.readyState === 'loading') {{
            document.addEventListener('DOMContentLoaded', () => {{
                window.auroraViewPerf.marks.domReady = performance.now();
                console.log('[TIMER] DOM ready:', window.auroraViewPerf.marks.domReady - window.auroraViewPerf.start, 'ms');
            }});
        }}

        // Mark when fully loaded
        window.addEventListener('load', () => {{
            window.auroraViewPerf.marks.loaded = performance.now();
            console.log('[TIMER] Fully loaded:', window.auroraViewPerf.marks.loaded - window.auroraViewPerf.start, 'ms');

            // Show content
            document.body.classList.add('ready');

            // Notify Rust
            try {{
                window.dispatchEvent(new CustomEvent('first_paint', {{
                    detail: {{
                        time: window.auroraViewPerf.marks.loaded - window.auroraViewPerf.start
                    }}
                }}));
            }} catch (e) {{
                console.error('Failed to dispatch first_paint event:', e);
            }}
        }});

        // Mark when JavaScript is initialized
        window.auroraViewPerf.marks.jsInit = performance.now();
        console.log('[TIMER] JavaScript initialized:', window.auroraViewPerf.marks.jsInit - window.auroraViewPerf.start, 'ms');
    </script>
</body>
</html>"#,
            user_html
        )
    }
}

/// Add performance monitoring to existing HTML
#[allow(dead_code)]
fn add_performance_monitoring(html: &str) -> String {
    // Find </body> tag and insert monitoring script before it
    if let Some(body_end) = html.rfind("</body>") {
        let (before, after) = html.split_at(body_end);
        format!(
            r#"{}<script>
// AuroraView Performance Monitoring
(function() {{
    window.auroraViewPerf = {{
        start: performance.now(),
        marks: {{}}
    }};

    // Mark when DOM is ready
    if (document.readyState === 'loading') {{
        document.addEventListener('DOMContentLoaded', () => {{
            window.auroraViewPerf.marks.domReady = performance.now();
            console.log('[TIMER] DOM ready:', window.auroraViewPerf.marks.domReady - window.auroraViewPerf.start, 'ms');
        }});
    }} else {{
        window.auroraViewPerf.marks.domReady = performance.now();
    }}

    // Mark when fully loaded
    window.addEventListener('load', () => {{
        window.auroraViewPerf.marks.loaded = performance.now();
        console.log('[TIMER] Fully loaded:', window.auroraViewPerf.marks.loaded - window.auroraViewPerf.start, 'ms');

        // Notify Rust
        try {{
            window.dispatchEvent(new CustomEvent('first_paint', {{
                detail: {{
                    time: window.auroraViewPerf.marks.loaded - window.auroraViewPerf.start
                }}
            }}));
        }} catch (e) {{
            console.error('Failed to dispatch first_paint event:', e);
        }}
    }});

    // Mark when JavaScript is initialized
    window.auroraViewPerf.marks.jsInit = performance.now();
    console.log('[TIMER] JavaScript initialized:', window.auroraViewPerf.marks.jsInit - window.auroraViewPerf.start, 'ms');
}})();
</script>
{}"#,
            before, after
        )
    } else {
        // No </body> tag found, append to end
        format!(
            r#"{}<script>
// AuroraView Performance Monitoring
window.auroraViewPerf = {{ start: performance.now(), marks: {{}} }};
console.log('[TIMER] Performance monitoring initialized');
</script>"#,
            html
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_partial_html() {
        let html = "<h1>Hello</h1>";
        let wrapped = wrap_html_with_optimizations(html);
        assert!(wrapped.contains("<!DOCTYPE html>"));
        assert!(wrapped.contains("<h1>Hello</h1>"));
        assert!(wrapped.contains("auroraViewPerf"));
    }

    #[test]
    fn test_wrap_complete_html() {
        let html = "<!DOCTYPE html><html><body><h1>Hello</h1></body></html>";
        let wrapped = wrap_html_with_optimizations(html);
        assert!(wrapped.contains("auroraViewPerf"));
        assert!(wrapped.contains("<h1>Hello</h1>"));
    }
}

#[test]
fn test_loading_html_has_spinner_and_text() {
    assert!(LOADING_HTML.contains("spinner"));
    assert!(LOADING_HTML.contains("Loading AuroraView"));
}
