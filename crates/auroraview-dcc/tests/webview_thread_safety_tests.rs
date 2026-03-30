//! Thread safety tests for DccWebView
//!
//! Validates that the lock-free message channel and atomic initialization
//! state work correctly under concurrent access.

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
    fn init_sets_initialized_flag() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        webview.init().unwrap();
        assert!(webview.is_initialized());
    }

    #[rstest]
    fn message_channel_send_receive() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        webview.init().unwrap();

        // Send messages
        webview.navigate("https://example.com").unwrap();
        webview.eval("console.log('test')").unwrap();
        webview.resize(1024, 768).unwrap();

        // Process should handle them without error
        let has_more = webview.process_events();
        // After draining all, no more messages
        assert!(!has_more);
    }

    #[rstest]
    fn concurrent_message_sends() {
        let config = test_config();
        let webview = Arc::new(auroraview_dcc::DccWebView::new(config).unwrap());
        webview.init().unwrap();

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

        // Process all 1000 messages
        webview.process_events();
        // All drained
        assert!(!webview.process_events());
    }

    #[rstest]
    fn size_returns_config_defaults_before_init() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert_eq!(webview.size(), (800, 600));
    }

    #[rstest]
    fn resize_updates_cached_size() {
        let config = test_config();
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        webview.init().unwrap();
        webview.resize(1920, 1080).unwrap();
        assert_eq!(webview.size(), (1920, 1080));
    }
}
