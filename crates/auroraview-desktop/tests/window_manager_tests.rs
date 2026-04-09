//! Tests for WindowManager

use std::sync::Arc;
use std::thread;

use auroraview_desktop::config::DesktopConfig;
use auroraview_desktop::ipc::IpcRouter;
use auroraview_desktop::window_manager::{WindowId, WindowManager};
use rstest::rstest;

// ============================================================================
// WindowManager::new / default
// ============================================================================

#[rstest]
fn new_manager_is_empty() {
    let mgr = WindowManager::new();
    assert_eq!(mgr.count(), 0);
    assert!(mgr.list().is_empty());
}

#[rstest]
fn default_same_as_new() {
    let mgr = WindowManager::default();
    assert_eq!(mgr.count(), 0);
}

#[rstest]
fn with_router_uses_provided_router() {
    let router = Arc::new(IpcRouter::new());
    let mgr = WindowManager::with_router(router.clone());
    // Should share the same router (pointer identity check not possible here,
    // but we can verify the manager's router works)
    let r = mgr.router();
    assert!(r.methods().is_empty());
}

// ============================================================================
// WindowManager::create
// ============================================================================

#[rstest]
fn create_returns_id() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new().title("Test")).unwrap();
    assert!(!id.is_empty());
    assert_eq!(mgr.count(), 1);
}

#[rstest]
fn create_multiple_windows_unique_ids() {
    let mgr = WindowManager::new();
    let id1 = mgr.create(DesktopConfig::new().title("W1")).unwrap();
    let id2 = mgr.create(DesktopConfig::new().title("W2")).unwrap();
    assert_ne!(id1, id2);
    assert_eq!(mgr.count(), 2);
}

#[rstest]
fn created_window_not_visible() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new().title("Test")).unwrap();
    let info = mgr.get(&id).unwrap();
    assert!(!info.visible);
}

#[rstest]
fn created_window_has_correct_title() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new().title("My Title")).unwrap();
    let info = mgr.get(&id).unwrap();
    assert_eq!(info.title, "My Title");
}

#[rstest]
fn created_window_has_correct_size() {
    let mgr = WindowManager::new();
    let config = DesktopConfig {
        width: 1280,
        height: 720,
        ..DesktopConfig::new()
    };
    let id = mgr.create(config).unwrap();
    let info = mgr.get(&id).unwrap();
    assert_eq!(info.width, 1280);
    assert_eq!(info.height, 720);
}

#[rstest]
fn created_window_with_url() {
    let mgr = WindowManager::new();
    let id = mgr
        .create(DesktopConfig::new().url("https://example.com"))
        .unwrap();
    let info = mgr.get(&id).unwrap();
    assert_eq!(info.url, Some("https://example.com".to_string()));
}

// ============================================================================
// WindowManager::show / hide
// ============================================================================

#[rstest]
fn show_makes_visible() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new()).unwrap();
    mgr.show(&id).unwrap();
    assert!(mgr.get(&id).unwrap().visible);
}

#[rstest]
fn hide_makes_invisible() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new()).unwrap();
    mgr.show(&id).unwrap();
    mgr.hide(&id).unwrap();
    assert!(!mgr.get(&id).unwrap().visible);
}

#[rstest]
fn show_nonexistent_window_errors() {
    let mgr = WindowManager::new();
    let fake_id: WindowId = "window_999".to_string();
    let result = mgr.show(&fake_id);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("window_999"));
}

#[rstest]
fn hide_nonexistent_window_errors() {
    let mgr = WindowManager::new();
    let fake_id: WindowId = "no_such_window".to_string();
    let result = mgr.hide(&fake_id);
    assert!(result.is_err());
}

// ============================================================================
// WindowManager::close
// ============================================================================

#[rstest]
fn close_removes_window() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new()).unwrap();
    assert_eq!(mgr.count(), 1);
    mgr.close(&id).unwrap();
    assert_eq!(mgr.count(), 0);
    assert!(mgr.get(&id).is_none());
}

#[rstest]
fn close_nonexistent_errors() {
    let mgr = WindowManager::new();
    let fake_id: WindowId = "missing".to_string();
    assert!(mgr.close(&fake_id).is_err());
}

// ============================================================================
// WindowManager::get / get_info / has_window
// ============================================================================

#[rstest]
fn get_returns_none_for_missing() {
    let mgr = WindowManager::new();
    assert!(mgr.get(&"nope".to_string()).is_none());
}

#[rstest]
fn get_info_returns_info() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new().title("A")).unwrap();
    let info = mgr.get_info(&id).unwrap();
    assert_eq!(info.title, "A");
}

#[rstest]
fn has_window_true_after_create() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new()).unwrap();
    assert!(mgr.has_window(&id));
}

#[rstest]
fn has_window_false_after_close() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new()).unwrap();
    mgr.close(&id).unwrap();
    assert!(!mgr.has_window(&id));
}

// ============================================================================
// WindowManager::list / window_ids / all
// ============================================================================

#[rstest]
fn list_includes_all_windows() {
    let mgr = WindowManager::new();
    let id1 = mgr.create(DesktopConfig::new().title("W1")).unwrap();
    let id2 = mgr.create(DesktopConfig::new().title("W2")).unwrap();
    let ids = mgr.list();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[rstest]
fn window_ids_same_as_list() {
    let mgr = WindowManager::new();
    mgr.create(DesktopConfig::new()).unwrap();
    let a = mgr.list();
    let b = mgr.window_ids();
    assert_eq!(a.len(), b.len());
}

#[rstest]
fn all_returns_window_infos() {
    let mgr = WindowManager::new();
    mgr.create(DesktopConfig::new().title("X")).unwrap();
    let infos = mgr.all();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].title, "X");
}

// ============================================================================
// WindowManager::navigate
// ============================================================================

#[rstest]
fn navigate_updates_url() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new()).unwrap();
    mgr.navigate(&id, "https://new.example.com").unwrap();
    let info = mgr.get(&id).unwrap();
    assert_eq!(info.url, Some("https://new.example.com".to_string()));
}

#[rstest]
fn navigate_nonexistent_errors() {
    let mgr = WindowManager::new();
    let fake: WindowId = "ghost".to_string();
    assert!(mgr.navigate(&fake, "https://x.com").is_err());
}

// ============================================================================
// WindowManager::resize
// ============================================================================

#[rstest]
fn resize_updates_dimensions() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new()).unwrap();
    mgr.resize(&id, 1920, 1080).unwrap();
    let info = mgr.get(&id).unwrap();
    assert_eq!(info.width, 1920);
    assert_eq!(info.height, 1080);
}

#[rstest]
fn resize_nonexistent_errors() {
    let mgr = WindowManager::new();
    let fake: WindowId = "ghost".to_string();
    assert!(mgr.resize(&fake, 800, 600).is_err());
}

// ============================================================================
// WindowManager::set_title
// ============================================================================

#[rstest]
fn set_title_updates() {
    let mgr = WindowManager::new();
    let id = mgr.create(DesktopConfig::new().title("Old")).unwrap();
    mgr.set_title(&id, "New Title").unwrap();
    let info = mgr.get(&id).unwrap();
    assert_eq!(info.title, "New Title");
}

#[rstest]
fn set_title_nonexistent_errors() {
    let mgr = WindowManager::new();
    let fake: WindowId = "nope".to_string();
    assert!(mgr.set_title(&fake, "x").is_err());
}

// ============================================================================
// WindowManager::router
// ============================================================================

#[rstest]
fn router_shared_across_windows() {
    let router = Arc::new(IpcRouter::new());
    router.register("ping", |_| serde_json::json!({"pong": true}));
    let mgr = WindowManager::with_router(router);
    let r = mgr.router();
    assert!(r.has_handler("ping"));
}

// ============================================================================
// Concurrent access
// ============================================================================

#[rstest]
fn concurrent_create_is_safe() {
    let mgr = Arc::new(WindowManager::new());
    let mut handles = vec![];

    for i in 0..8 {
        let m = mgr.clone();
        handles.push(thread::spawn(move || {
            m.create(DesktopConfig::new().title(format!("W{}", i)))
                .unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(mgr.count(), 8);
}
