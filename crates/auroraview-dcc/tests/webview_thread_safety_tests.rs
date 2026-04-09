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
    use std::sync::Arc;
    use std::thread;

    use auroraview_dcc::config::{DccConfig, DccType};
    use rstest::rstest;

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

    // -------------------------------------------------------------------------
    // Basic state tests
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // DccMessage variant coverage
    // -------------------------------------------------------------------------

    #[rstest]
    fn navigate_empty_url_enqueues() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        webview.navigate("").unwrap();
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn navigate_long_url_enqueues() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        let long_url = format!("https://example.com/{}", "a".repeat(2048));
        webview.navigate(&long_url).unwrap();
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn load_html_empty_enqueues() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        webview.load_html("").unwrap();
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn load_html_large_content_enqueues() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        let large_html = format!("<html><body>{}</body></html>", "x".repeat(64 * 1024));
        webview.load_html(&large_html).unwrap();
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn eval_js_empty_enqueues() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        webview.eval("").unwrap();
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn eval_js_unicode_enqueues() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        webview.eval("console.log('你好世界 🦀')").unwrap();
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn show_hide_toggle_enqueues() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        for _ in 0..10 {
            webview.show().unwrap();
            webview.hide().unwrap();
        }
        // Drain all 20 messages
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn close_enqueues_and_drains() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        webview.close().unwrap();
        webview.process_events();
        assert!(!webview.process_events());
    }

    // -------------------------------------------------------------------------
    // Concurrent mixed operations
    // -------------------------------------------------------------------------

    #[rstest]
    fn concurrent_mixed_operations() {
        let webview = Arc::new(auroraview_dcc::DccWebView::new(test_config()).unwrap());
        let mut handles = Vec::new();

        // Thread 1: navigate
        {
            let wv = webview.clone();
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    wv.navigate(&format!("https://site.com/{}", i)).unwrap();
                }
            }));
        }

        // Thread 2: load HTML
        {
            let wv = webview.clone();
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    wv.load_html(&format!("<p>page {}</p>", i)).unwrap();
                }
            }));
        }

        // Thread 3: eval JS
        {
            let wv = webview.clone();
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    wv.eval(&format!("console.log({})", i)).unwrap();
                }
            }));
        }

        // Thread 4: resize
        {
            let wv = webview.clone();
            handles.push(thread::spawn(move || {
                for i in 0..50u32 {
                    wv.resize(800 + i, 600 + i).unwrap();
                }
            }));
        }

        // Thread 5: show/hide
        {
            let wv = webview.clone();
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    if i % 2 == 0 {
                        wv.show().unwrap();
                    } else {
                        wv.hide().unwrap();
                    }
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Drain all queued messages
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn concurrent_high_load_navigations() {
        let webview = Arc::new(auroraview_dcc::DccWebView::new(test_config()).unwrap());
        let mut handles = Vec::new();

        for thread_id in 0..20 {
            let wv = webview.clone();
            handles.push(thread::spawn(move || {
                for msg_id in 0..200 {
                    wv.navigate(&format!("https://t{}.com/{}", thread_id, msg_id))
                        .unwrap();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // 20 threads * 200 messages = 4000 messages total, drain all
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn concurrent_eval_and_resize() {
        let webview = Arc::new(auroraview_dcc::DccWebView::new(test_config()).unwrap());
        let mut handles = Vec::new();

        for i in 0..5u32 {
            let wv = webview.clone();
            handles.push(thread::spawn(move || {
                for j in 0..100u32 {
                    wv.eval(&format!("window.x = {}", i * 100 + j)).unwrap();
                    wv.resize(400 + j, 300 + j).unwrap();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        webview.process_events();
        assert!(!webview.process_events());
    }

    // -------------------------------------------------------------------------
    // Multi-instance tests
    // -------------------------------------------------------------------------

    #[rstest]
    fn multiple_instances_independent() {
        let wv1 = Arc::new(auroraview_dcc::DccWebView::new(test_config()).unwrap());
        let wv2 = Arc::new(auroraview_dcc::DccWebView::new(test_config()).unwrap());
        let wv3 = Arc::new(auroraview_dcc::DccWebView::new(test_config()).unwrap());

        wv1.navigate("https://a.com").unwrap();
        wv2.navigate("https://b.com").unwrap();
        wv3.navigate("https://c.com").unwrap();

        // Each instance drains its own channel independently
        wv1.process_events();
        wv2.process_events();
        wv3.process_events();

        assert!(!wv1.process_events());
        assert!(!wv2.process_events());
        assert!(!wv3.process_events());
    }

    #[rstest]
    fn multiple_instances_concurrent_send() {
        let instances: Vec<Arc<auroraview_dcc::DccWebView>> = (0..5)
            .map(|_| Arc::new(auroraview_dcc::DccWebView::new(test_config()).unwrap()))
            .collect();

        let mut handles = Vec::new();

        for (idx, wv) in instances.iter().enumerate() {
            let wv = wv.clone();
            let id = idx;
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    wv.navigate(&format!("https://inst{}.com/{}", id, i))
                        .unwrap();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        for wv in &instances {
            wv.process_events();
            assert!(!wv.process_events());
        }
    }

    // -------------------------------------------------------------------------
    // Process events behavior
    // -------------------------------------------------------------------------

    #[rstest]
    fn process_events_empty_returns_false() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        // No messages queued
        assert!(!webview.process_events());
    }

    #[rstest]
    fn process_events_drains_single_message() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        webview.navigate("https://example.com").unwrap();
        // One message — process_events will drain it and return false (empty after drain)
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn process_events_drains_all_messages() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        for i in 0..100 {
            webview.navigate(&format!("https://x.com/{}", i)).unwrap();
        }
        // process_events drains all available messages in one call
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn process_events_idempotent_on_empty_channel() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        // Calling process_events many times on an empty channel is safe
        for _ in 0..100 {
            assert!(!webview.process_events());
        }
    }

    // -------------------------------------------------------------------------
    // Atomic state ordering
    // -------------------------------------------------------------------------

    #[rstest]
    fn is_initialized_false_before_init() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        // Spin from multiple threads and assert always false before init()
        let webview = Arc::new(webview);
        let mut handles = Vec::new();
        for _ in 0..8 {
            let wv = webview.clone();
            handles.push(thread::spawn(move || {
                assert!(!wv.is_initialized());
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }

    // -------------------------------------------------------------------------
    // Config builder integration
    // -------------------------------------------------------------------------

    #[rstest]
    #[case(DccType::Maya)]
    #[case(DccType::Houdini)]
    #[case(DccType::Nuke)]
    #[case(DccType::Blender)]
    #[case(DccType::Max3ds)]
    #[case(DccType::Unreal)]
    #[case(DccType::Unknown)]
    fn webview_new_with_all_dcc_types(#[case] dcc_type: DccType) {
        let config = DccConfig {
            dcc_type,
            parent_hwnd: Some(0xABCD),
            ..DccConfig::default()
        };
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert!(!webview.is_initialized());
    }

    #[rstest]
    fn config_with_url_stored_correctly() {
        let config = DccConfig {
            url: Some("https://auroraview.dev".to_string()),
            parent_hwnd: Some(0x1000),
            ..DccConfig::default()
        };
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert_eq!(
            webview.config().url.as_deref(),
            Some("https://auroraview.dev")
        );
    }

    #[rstest]
    fn config_with_html_stored_correctly() {
        let config = DccConfig {
            html: Some("<h1>Hi</h1>".to_string()),
            parent_hwnd: Some(0x2000),
            ..DccConfig::default()
        };
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert_eq!(webview.config().html.as_deref(), Some("<h1>Hi</h1>"));
    }

    #[rstest]
    #[case(800, 600)]
    #[case(1920, 1080)]
    #[case(1, 1)]
    #[case(4096, 2160)]
    fn size_returns_initial_config_size(#[case] w: u32, #[case] h: u32) {
        let config = DccConfig {
            width: w,
            height: h,
            parent_hwnd: Some(0x5678),
            ..DccConfig::default()
        };
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert_eq!(webview.size(), (w, h));
    }

    #[rstest]
    fn devtools_flag_preserved_in_config() {
        let config = DccConfig {
            devtools: true,
            parent_hwnd: Some(0x9ABC),
            ..DccConfig::default()
        };
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert!(webview.config().devtools);
    }

    #[rstest]
    fn debug_port_preserved_in_config() {
        let config = DccConfig {
            debug_port: 9222,
            parent_hwnd: Some(0xDEF0),
            ..DccConfig::default()
        };
        let webview = auroraview_dcc::DccWebView::new(config).unwrap();
        assert_eq!(webview.config().debug_port, 9222);
    }

    // -------------------------------------------------------------------------
    // Stress tests
    // -------------------------------------------------------------------------

    #[rstest]
    fn stress_sequential_all_message_types() {
        let webview = auroraview_dcc::DccWebView::new(test_config()).unwrap();
        for i in 0..500 {
            webview.navigate(&format!("https://x.com/{}", i)).unwrap();
            webview.load_html(&format!("<p>{}</p>", i)).unwrap();
            webview.eval(&format!("x={}", i)).unwrap();
            webview
                .resize(100 + (i as u32 % 1000), 100 + (i as u32 % 800))
                .unwrap();
            if i % 2 == 0 {
                webview.show().unwrap();
            } else {
                webview.hide().unwrap();
            }
        }
        // Drain 500*5 = 2500 messages
        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn stress_concurrent_close_signals() {
        let webview = Arc::new(auroraview_dcc::DccWebView::new(test_config()).unwrap());
        let mut handles = Vec::new();

        for _ in 0..10 {
            let wv = webview.clone();
            handles.push(thread::spawn(move || {
                // Close can be called from any thread safely
                wv.close().unwrap();
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        webview.process_events();
        assert!(!webview.process_events());
    }

    #[rstest]
    fn multiple_drops_concurrent() {
        // Create multiple webviews, drop them concurrently — no UB
        let webviews: Vec<Arc<auroraview_dcc::DccWebView>> = (0..8)
            .map(|_| Arc::new(auroraview_dcc::DccWebView::new(test_config()).unwrap()))
            .collect();

        let mut handles = Vec::new();
        for wv in webviews {
            handles.push(thread::spawn(move || {
                wv.navigate("https://drop.test").unwrap();
                drop(wv);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }
}
