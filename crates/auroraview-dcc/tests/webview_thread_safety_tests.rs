//! Thread safety tests for DccWebView
//!
//! Validates that the lock-free message channel and atomic initialization
//! state work correctly under concurrent access.
//!
//! Note: Tests that require real WebView2 initialization (e.g., init() with
//! a valid parent HWND) are only run in integration environments with a
//! real desktop. Tests here focus on the message channel, atomic state,
//! and configuration aspects that do not require a live WebView2 runtime.

#[cfg(target_os = "windows")]
mod tests {
    use auroraview_dcc::config::{DccConfig, DccType};
    use rstest::rstest;
    use std::sync::Arc;
    use std::thread;

    /// Helper to create a DccConfig with a fake parent HWND for testing
    fn test_config() -> DccConfig {
        DccConfig {
            title: "Test".to_string(),
            width: 800,
            height: 600,
            url: None,
            html: None,
            parent_hwnd: Some(0x1234),
            dcc_type: DccType::Unknown,
            dcc_version: None,
            panel_name: None,
            devtools: false,
            data_dir: None,
            debug_port: 0,
            background_color: None,
        }
    }

    #[rstest]
    fn initialized_defaults_to_false() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert!(!webview.is_initialized());
    }

    #[rstest]
    fn size_returns_config_defaults_before_init() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert_eq!(webview.size(), (800, 600));
    }

    #[rstest]
    fn parent_hwnd_returns_config_value() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert_eq!(webview.parent_hwnd(), Some(0x1234));
    }

    #[rstest]
    fn config_is_accessible() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert_eq!(webview.config().title, "Test");
        assert_eq!(webview.config().width, 800);
        assert_eq!(webview.config().height, 600);
    }

    #[rstest]
    fn message_channel_enqueue_without_init() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();

        // Can enqueue messages even before init (they queue up)
        webview.navigate("https://example.com").unwrap();
        webview.load_html("<h1>Hello</h1>").unwrap();
        webview.eval("console.log('test')").unwrap();
        webview.resize(1024, 768).unwrap();
        webview.show().unwrap();
        webview.hide().unwrap();
        webview.close().unwrap();
    }

    #[rstest]
    fn resize_queues_message_before_init() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        webview.resize(1920, 1080).unwrap();
        // Before init(), inner state is None so cached size falls back to config defaults.
        // The resize message is queued and will take effect when process_events() runs after init().
        assert_eq!(webview.size(), (800, 600));
    }

    #[rstest]
    fn concurrent_message_sends() {
        let config = test_config();
        let webview = Arc::new(auroraview_dcc::DccWebView::new(config).unwrap());

        let mut handles = Vec::new();
        for i in 0..10 {
            let wv = webview.clone();
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    let url = format!("https://example.com/{}/{}", i, j);
                    wv.navigate(&url).unwrap();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Process all 1000 messages — without init, Navigate calls are
        // no-ops (inner is None so the webview ref is None), but the
        // channel drains correctly.
        webview.process_events();
        // All drained
        assert!(!webview.process_events());
    }

    #[rstest]
    fn drop_without_init_is_safe() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        // Just drop — should not panic
        drop(webview);
    }

    #[rstest]
    fn new_requires_parent_hwnd() {
        let mut config = test_config();
        config.parent_hwnd = None;
        let result = auroraview_dcc::DccWebView::new(config);
        assert!(result.is_err());
    }
}
