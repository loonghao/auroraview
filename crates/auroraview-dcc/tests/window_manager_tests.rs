//! Tests for WindowManager

use auroraview_dcc::{DccConfig, WindowManager};

#[test]
fn test_window_manager_new() {
    let manager = WindowManager::new();
    assert_eq!(manager.count(), 0);
    assert!(manager.list().is_empty());
}

#[test]
fn test_window_manager_create() {
    let _manager = WindowManager::new();

    // Create without parent (will fail on Windows, but tests structure)
    let _config = DccConfig::new().title("Test Window").size(400, 300);

    // On non-Windows, this should work (stub implementation)
    #[cfg(not(target_os = "windows"))]
    {
        let result = _manager.create(_config);
        assert!(result.is_ok());

        let id = result.unwrap();
        assert_eq!(manager.count(), 1);
        assert!(manager.list().contains(&id));

        let info = manager.get(&id);
        assert!(info.is_some());
        assert_eq!(info.unwrap().title, "Test Window");
    }
}

#[test]
fn test_window_manager_close() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let config = DccConfig::new().title("To Close");
        let id = _manager.create(config).unwrap();

        assert_eq!(_manager.count(), 1);

        let result = _manager.close(&id);
        assert!(result.is_ok());
        assert_eq!(_manager.count(), 0);

        // Close non-existent
        let result = _manager.close(&id);
        assert!(result.is_err());
    }
}

#[test]
fn test_window_manager_multiple_windows() {
    let _manager = WindowManager::new();

    #[cfg(not(target_os = "windows"))]
    {
        let id1 = _manager.create(DccConfig::new().title("Window 1")).unwrap();
        let id2 = _manager.create(DccConfig::new().title("Window 2")).unwrap();
        let id3 = _manager.create(DccConfig::new().title("Window 3")).unwrap();

        assert_eq!(_manager.count(), 3);

        let all = _manager.all();
        assert_eq!(all.len(), 3);

        // Close middle window
        _manager.close(&id2).unwrap();
        assert_eq!(_manager.count(), 2);
        assert!(_manager.get(&id1).is_some());
        assert!(_manager.get(&id2).is_none());
        assert!(_manager.get(&id3).is_some());
    }
}

#[test]
fn test_window_manager_visibility() {
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
fn test_window_manager_resize() {
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
fn test_window_manager_navigate() {
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
fn test_window_manager_shared_router() {
    use auroraview_dcc::IpcRouter;
    use std::sync::Arc;

    let router = Arc::new(IpcRouter::new());
    router.register("shared.method", |_| serde_json::json!({"ok": true}));

    let manager = WindowManager::with_router(router.clone());

    // All windows share the same router
    assert!(manager.router().has_handler("shared.method"));
}
