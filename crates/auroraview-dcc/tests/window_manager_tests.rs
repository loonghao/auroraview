//! Tests for WindowManager

use auroraview_dcc::{DccConfig, WindowManager};
use rstest::rstest;

// ─── Basic construction ───────────────────────────────────────────────────────

#[test]
fn new_starts_empty() {
    let manager = WindowManager::new();
    assert_eq!(manager.count(), 0);
    assert!(manager.list().is_empty());
}

#[test]
fn default_starts_empty() {
    let manager = WindowManager::default();
    assert_eq!(manager.count(), 0);
}

// ─── Create window ────────────────────────────────────────────────────────────

#[test]
fn create_window() {
    let _manager = WindowManager::new();
    let _config = DccConfig::new().title("Test Window").size(400, 300);

    #[cfg(not(target_os = "windows"))]
    {
        let result = _manager.create(_config);
        assert!(result.is_ok());

        let id = result.unwrap();
        assert_eq!(_manager.count(), 1);
        assert!(_manager.list().contains(&id));

        let info = _manager.get(&id);
        assert!(info.is_some());
        assert_eq!(info.unwrap().title, "Test Window");
    }
}

#[test]
fn create_window_has_correct_size() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager
            .create(DccConfig::new().title("Sized").size(1920, 1080))
            .unwrap();
        let info = _manager.get(&id).unwrap();
        assert_eq!(info.width, 1920);
        assert_eq!(info.height, 1080);
    }
}

#[test]
fn create_window_starts_hidden() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager.create(DccConfig::new().title("Hidden")).unwrap();
        let info = _manager.get(&id).unwrap();
        assert!(!info.visible, "New windows should be hidden by default");
    }
}

#[test]
fn create_multiple_windows_unique_ids() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id1 = _manager.create(DccConfig::new().title("W1")).unwrap();
        let id2 = _manager.create(DccConfig::new().title("W2")).unwrap();
        let id3 = _manager.create(DccConfig::new().title("W3")).unwrap();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
        assert_eq!(_manager.count(), 3);
    }
}

// ─── Close window ─────────────────────────────────────────────────────────────

#[test]
fn close_window() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let config = DccConfig::new().title("To Close");
        let id = _manager.create(config).unwrap();

        assert_eq!(_manager.count(), 1);

        let result = _manager.close(&id);
        assert!(result.is_ok());
        assert_eq!(_manager.count(), 0);
    }
}

#[test]
fn close_nonexistent_returns_error() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let config = DccConfig::new().title("To Close");
        let id = _manager.create(config).unwrap();
        _manager.close(&id).unwrap();

        // Close again — should error
        let result = _manager.close(&id);
        assert!(result.is_err());
    }
}

#[test]
fn close_removes_from_list() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id1 = _manager.create(DccConfig::new().title("Keep")).unwrap();
        let id2 = _manager.create(DccConfig::new().title("Remove")).unwrap();

        _manager.close(&id2).unwrap();

        let list = _manager.list();
        assert!(list.contains(&id1));
        assert!(!list.contains(&id2));
    }
}

// ─── Multiple windows lifecycle ───────────────────────────────────────────────

#[test]
fn multiple_windows_close_middle() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id1 = _manager.create(DccConfig::new().title("Window 1")).unwrap();
        let id2 = _manager.create(DccConfig::new().title("Window 2")).unwrap();
        let id3 = _manager.create(DccConfig::new().title("Window 3")).unwrap();

        assert_eq!(_manager.count(), 3);

        let all = _manager.all();
        assert_eq!(all.len(), 3);

        _manager.close(&id2).unwrap();
        assert_eq!(_manager.count(), 2);
        assert!(_manager.get(&id1).is_some());
        assert!(_manager.get(&id2).is_none());
        assert!(_manager.get(&id3).is_some());
    }
}

#[test]
fn close_all_windows_empties_manager() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let ids: Vec<_> = (0..5)
            .map(|i| {
                _manager
                    .create(DccConfig::new().title(&format!("Win {}", i)))
                    .unwrap()
            })
            .collect();

        assert_eq!(_manager.count(), 5);

        for id in &ids {
            _manager.close(id).unwrap();
        }

        assert_eq!(_manager.count(), 0);
        assert!(manager_all_empty(&_manager));
    }
}

#[cfg(not(target_os = "windows"))]
fn manager_all_empty(manager: &WindowManager) -> bool {
    manager.all().is_empty()
}

// ─── Show / hide ──────────────────────────────────────────────────────────────

#[test]
fn show_and_hide_window() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager.create(DccConfig::new().title("Test")).unwrap();

        let info = _manager.get(&id).unwrap();
        assert!(!info.visible);

        _manager.show(&id).unwrap();
        let info = _manager.get(&id).unwrap();
        assert!(info.visible);

        _manager.hide(&id).unwrap();
        let info = _manager.get(&id).unwrap();
        assert!(!info.visible);
    }
}

#[test]
fn show_nonexistent_returns_error() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let result = _manager.show(&"nonexistent".to_string());
        assert!(result.is_err());
    }
}

#[test]
fn hide_nonexistent_returns_error() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let result = _manager.hide(&"nonexistent".to_string());
        assert!(result.is_err());
    }
}

#[test]
fn toggle_visibility_multiple_times() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager.create(DccConfig::new().title("Toggle")).unwrap();

        for _ in 0..3 {
            _manager.show(&id).unwrap();
            assert!(_manager.get(&id).unwrap().visible);
            _manager.hide(&id).unwrap();
            assert!(!_manager.get(&id).unwrap().visible);
        }
    }
}

// ─── Resize ───────────────────────────────────────────────────────────────────

#[test]
fn resize_window() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager
            .create(DccConfig::new().title("Test").size(400, 300))
            .unwrap();

        let info = _manager.get(&id).unwrap();
        assert_eq!(info.width, 400);
        assert_eq!(info.height, 300);

        _manager.resize(&id, 800, 600).unwrap();

        let info = _manager.get(&id).unwrap();
        assert_eq!(info.width, 800);
        assert_eq!(info.height, 600);
    }
}

#[test]
fn resize_nonexistent_returns_error() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let result = _manager.resize(&"ghost".to_string(), 100, 100);
        assert!(result.is_err());
    }
}

#[rstest]
#[case(100, 100)]
#[case(800, 600)]
#[case(1920, 1080)]
#[case(3840, 2160)]
fn resize_various_sizes(#[case] _w: u32, #[case] _h: u32) {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager.create(DccConfig::new().title("T")).unwrap();
        _manager.resize(&id, _w, _h).unwrap();
        let info = _manager.get(&id).unwrap();
        assert_eq!(info.width, _w);
        assert_eq!(info.height, _h);
    }
}

// ─── Navigate ─────────────────────────────────────────────────────────────────

#[test]
fn navigate_window() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager.create(DccConfig::new().title("Test")).unwrap();

        _manager.navigate(&id, "https://example.com").unwrap();

        let info = _manager.get(&id).unwrap();
        assert_eq!(info.url, Some("https://example.com".to_string()));
    }
}

#[test]
fn navigate_nonexistent_returns_error() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let result = _manager.navigate(&"ghost".to_string(), "https://example.com");
        assert!(result.is_err());
    }
}

#[test]
fn navigate_updates_url() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager.create(DccConfig::new().title("Nav")).unwrap();

        _manager.navigate(&id, "https://first.com").unwrap();
        assert_eq!(
            _manager.get(&id).unwrap().url,
            Some("https://first.com".to_string())
        );

        _manager.navigate(&id, "https://second.com").unwrap();
        assert_eq!(
            _manager.get(&id).unwrap().url,
            Some("https://second.com".to_string())
        );
    }
}

// ─── Eval JavaScript ─────────────────────────────────────────────────────────

#[test]
fn eval_existing_window() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager.create(DccConfig::new().title("Eval")).unwrap();
        let result = _manager.eval(&id, "console.log('hello')");
        assert!(result.is_ok());
    }
}

#[test]
fn eval_nonexistent_returns_error() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let result = _manager.eval(&"ghost".to_string(), "1+1");
        assert!(result.is_err());
    }
}

// ─── Init ─────────────────────────────────────────────────────────────────────

#[test]
fn init_existing_window() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager.create(DccConfig::new().title("Init")).unwrap();
        let result = _manager.init(&id);
        assert!(result.is_ok());
    }
}

#[test]
fn init_nonexistent_returns_error() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let result = _manager.init(&"ghost".to_string());
        assert!(result.is_err());
    }
}

// ─── has_window / get_info / window_ids ──────────────────────────────────────

#[test]
fn has_window_returns_correct() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager.create(DccConfig::new().title("Exists")).unwrap();
        assert!(_manager.has_window(&id));
        assert!(!_manager.has_window("no-such-window"));
    }

    // On Windows, has_window always returns false for any id (no windows created in test)
    #[cfg(target_os = "windows")]
    {
        assert!(!_manager.has_window("no-such-window"));
    }
}

#[test]
fn get_info_alias() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager
            .create(DccConfig::new().title("InfoAlias"))
            .unwrap();
        let info_get = _manager.get(&id);
        let info_get_info = _manager.get_info(&id);
        assert!(info_get.is_some());
        assert!(info_get_info.is_some());
        assert_eq!(info_get.unwrap().title, info_get_info.unwrap().title);
    }
}

#[test]
fn get_info_not_found() {
    let manager = WindowManager::new();
    let result = manager.get_info("no-such-id");
    assert!(result.is_none());
}

#[test]
fn window_ids_alias() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id1 = _manager.create(DccConfig::new().title("A")).unwrap();
        let id2 = _manager.create(DccConfig::new().title("B")).unwrap();

        let list = _manager.list();
        let ids = _manager.window_ids();

        let mut list_sorted = list.clone();
        let mut ids_sorted = ids.clone();
        list_sorted.sort();
        ids_sorted.sort();

        assert_eq!(list_sorted, ids_sorted);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }
}

// ─── process_events ───────────────────────────────────────────────────────────

#[test]
fn process_events_does_not_panic() {
    let manager = WindowManager::new();
    // Should not panic on empty manager
    manager.process_events();
}

#[test]
fn process_events_with_windows_does_not_panic() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        _manager.create(DccConfig::new().title("W1")).unwrap();
        _manager.create(DccConfig::new().title("W2")).unwrap();
        _manager.process_events(); // non-Windows stub; must not panic
    }
}

// ─── Shared IPC router ────────────────────────────────────────────────────────

#[test]
fn shared_router_has_registered_handler() {
    use auroraview_dcc::IpcRouter;
    use std::sync::Arc;

    let router = Arc::new(IpcRouter::new());
    router.register("shared.method", |_| serde_json::json!({"ok": true}));

    let manager = WindowManager::with_router(router.clone());

    assert!(manager.router().has_handler("shared.method"));
}

#[test]
fn router_shared_across_manager() {
    use auroraview_dcc::IpcRouter;
    use std::sync::Arc;

    let router = Arc::new(IpcRouter::new());
    router.register("method.a", |_| serde_json::json!(null));
    router.register("method.b", |_| serde_json::json!(null));

    let manager = WindowManager::with_router(Arc::clone(&router));

    assert!(manager.router().has_handler("method.a"));
    assert!(manager.router().has_handler("method.b"));
    assert!(!manager.router().has_handler("method.c"));
}

// ─── Concurrent operations ────────────────────────────────────────────────────

#[test]
fn concurrent_create_and_close() {
    use std::sync::Arc;

    let _manager = Arc::new(WindowManager::new());

    #[cfg(not(target_os = "windows"))]
    {
        use std::thread;
        let mut handles = Vec::new();

        for i in 0..8 {
            let m = Arc::clone(&_manager);
            let handle = thread::spawn(move || {
                let id = m
                    .create(DccConfig::new().title(&format!("Thread-{}", i)))
                    .unwrap();
                // verify it was created
                assert!(m.has_window(&id));
                id
            });
            handles.push(handle);
        }

        let ids: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(_manager.count(), 8);

        // Concurrent close
        let mut close_handles = Vec::new();
        for id in ids {
            let m = Arc::clone(&_manager);
            let handle = thread::spawn(move || {
                m.close(&id).unwrap();
            });
            close_handles.push(handle);
        }
        for h in close_handles {
            h.join().unwrap();
        }

        assert_eq!(_manager.count(), 0);
    }
}

#[test]
fn concurrent_navigate() {
    use std::sync::Arc;

    let _manager = Arc::new(WindowManager::new());

    #[cfg(not(target_os = "windows"))]
    {
        use std::thread;
        let id = _manager
            .create(DccConfig::new().title("Concurrent Nav"))
            .unwrap();
        let mut handles = Vec::new();

        for i in 0..10 {
            let m = Arc::clone(&_manager);
            let id_clone = id.clone();
            let handle = thread::spawn(move || {
                m.navigate(&id_clone, &format!("https://example.com/{}", i))
                    .unwrap();
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        // After all threads, URL should be set to something
        let info = _manager.get(&id).unwrap();
        assert!(info.url.is_some());
        assert!(info.url.unwrap().starts_with("https://example.com/"));
    }
}

#[test]
fn concurrent_show_hide() {
    use std::sync::Arc;

    let _manager = Arc::new(WindowManager::new());

    #[cfg(not(target_os = "windows"))]
    {
        use std::thread;
        let id = _manager
            .create(DccConfig::new().title("Show/Hide"))
            .unwrap();
        let mut handles = Vec::new();

        for i in 0..10 {
            let m = Arc::clone(&_manager);
            let id_clone = id.clone();
            let handle = thread::spawn(move || {
                if i % 2 == 0 {
                    m.show(&id_clone).unwrap();
                } else {
                    m.hide(&id_clone).unwrap();
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        // Just verify no panic and window still exists
        assert!(_manager.has_window(&id));
    }
}

// ─── rstest: window creation with various configs ─────────────────────────────

#[rstest]
#[case("Maya Tool", 400, 300)]
#[case("Houdini Panel", 800, 600)]
#[case("Blender Widget", 1280, 720)]
#[case("Unreal Tool", 1920, 1080)]
fn create_window_various_configs(#[case] _title: &str, #[case] _w: u32, #[case] _h: u32) {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id = _manager
            .create(DccConfig::new().title(_title).size(_w, _h))
            .unwrap();
        let info = _manager.get(&id).unwrap();
        assert_eq!(info.title, _title);
        assert_eq!(info.width, _w);
        assert_eq!(info.height, _h);
    }
}
