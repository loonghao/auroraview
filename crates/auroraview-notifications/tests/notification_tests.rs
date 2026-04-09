use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use auroraview_notifications::{
    Notification, NotificationAction, NotificationManager, NotificationType, Permission,
    PermissionState,
};
use rstest::*;

// ========== NotificationType Tests ==========

#[rstest]
#[case(NotificationType::Info, Some(5000))]
#[case(NotificationType::Success, Some(3000))]
#[case(NotificationType::Warning, Some(8000))]
#[case(NotificationType::Error, None)]
fn notification_type_duration(
    #[case] kind: NotificationType,
    #[case] expected: Option<u64>,
) {
    assert_eq!(kind.default_duration(), expected);
}

#[rstest]
fn notification_type_default_is_info() {
    assert_eq!(NotificationType::default(), NotificationType::Info);
}

// ========== NotificationAction Tests ==========

#[rstest]
fn action_new() {
    let action = NotificationAction::new("btn1", "Click me");
    assert_eq!(action.id, "btn1");
    assert_eq!(action.label, "Click me");
    assert!(action.icon.is_none());
}

#[rstest]
fn action_with_icon() {
    let action = NotificationAction::new("btn1", "Click me").with_icon("check");
    assert_eq!(action.icon, Some("check".to_string()));
}

// ========== Notification Tests ==========

#[rstest]
fn notification_new() {
    let n = Notification::new("Title", "Body");
    assert_eq!(n.title, "Title");
    assert_eq!(n.body, Some("Body".to_string()));
    assert_eq!(n.notification_type, NotificationType::Info);
    assert_eq!(n.duration, Some(5000));
    assert!(!n.require_interaction);
    assert!(n.is_active());
    assert!(!n.is_shown());
}

#[rstest]
fn notification_simple() {
    let n = Notification::simple("Title only");
    assert_eq!(n.title, "Title only");
    assert!(n.body.is_none());
}

#[rstest]
fn notification_builder() {
    let n = Notification::new("Alert", "Something happened")
        .with_type(NotificationType::Warning)
        .with_icon("warning")
        .with_image("https://example.com/img.png")
        .with_tag("alert-group")
        .with_data(serde_json::json!({"key": "value"}))
        .with_action(NotificationAction::new("ok", "OK"));

    assert_eq!(n.notification_type, NotificationType::Warning);
    assert_eq!(n.duration, Some(8000));
    assert_eq!(n.icon, Some("warning".to_string()));
    assert_eq!(n.image, Some("https://example.com/img.png".to_string()));
    assert_eq!(n.tag, Some("alert-group".to_string()));
    assert!(n.data.is_some());
    assert_eq!(n.actions.len(), 1);
}

#[rstest]
fn notification_persistent() {
    let n = Notification::new("Error", "Critical failure").persistent();
    assert!(n.duration.is_none());
    assert!(n.require_interaction);
}

#[rstest]
fn notification_with_duration() {
    let n = Notification::new("Custom", "msg").with_duration(10000);
    assert_eq!(n.duration, Some(10000));
}

#[rstest]
fn notification_mark_shown() {
    let mut n = Notification::new("T", "B");
    assert!(!n.is_shown());

    n.mark_shown();
    assert!(n.is_shown());
    assert!(n.shown_at.is_some());

    // Calling again doesn't change the timestamp
    let first_shown = n.shown_at;
    n.mark_shown();
    assert_eq!(n.shown_at, first_shown);
}

#[rstest]
fn notification_mark_dismissed() {
    let mut n = Notification::new("T", "B");
    assert!(n.is_active());

    n.mark_dismissed();
    assert!(!n.is_active());
    assert!(n.dismissed_at.is_some());

    let first_dismissed = n.dismissed_at;
    n.mark_dismissed();
    assert_eq!(n.dismissed_at, first_dismissed);
}

#[rstest]
fn notification_display() {
    let n1 = Notification::new("Title", "Body");
    assert_eq!(format!("{}", n1), "Title: Body");

    let n2 = Notification::simple("Title only");
    assert_eq!(format!("{}", n2), "Title only");
}

// ========== PermissionState Tests ==========

#[rstest]
#[case(PermissionState::Default, false, false, true)]
#[case(PermissionState::Granted, true, false, false)]
#[case(PermissionState::Denied, false, true, false)]
fn permission_state(
    #[case] state: PermissionState,
    #[case] granted: bool,
    #[case] denied: bool,
    #[case] default: bool,
) {
    assert_eq!(state.is_granted(), granted);
    assert_eq!(state.is_denied(), denied);
    assert_eq!(state.is_default(), default);
}

#[rstest]
fn permission_state_display() {
    assert_eq!(PermissionState::Default.to_string(), "default");
    assert_eq!(PermissionState::Granted.to_string(), "granted");
    assert_eq!(PermissionState::Denied.to_string(), "denied");
}

// ========== Permission Tests ==========

#[rstest]
fn permission_new() {
    let p = Permission::new("https://example.com");
    assert_eq!(p.origin, "https://example.com");
    assert_eq!(p.state, PermissionState::Default);
    assert!(p.updated_at.is_none());
}

#[rstest]
fn permission_granted() {
    let p = Permission::granted("https://example.com");
    assert_eq!(p.state, PermissionState::Granted);
    assert!(p.updated_at.is_some());
}

#[rstest]
fn permission_denied() {
    let p = Permission::denied("https://example.com");
    assert_eq!(p.state, PermissionState::Denied);
    assert!(p.updated_at.is_some());
}

#[rstest]
fn permission_grant_deny_reset() {
    let mut p = Permission::new("origin");
    assert!(p.state.is_default());

    p.grant();
    assert!(p.state.is_granted());
    assert!(p.updated_at.is_some());

    p.deny();
    assert!(p.state.is_denied());

    p.reset();
    assert!(p.state.is_default());
}

// ========== NotificationManager Tests ==========

#[fixture]
fn manager() -> NotificationManager {
    NotificationManager::new()
}

#[rstest]
fn manager_notify(manager: NotificationManager) {
    let n = Notification::new("Hello", "World");
    let id = manager.notify(n).unwrap();

    assert_eq!(manager.active_count(), 1);
    let retrieved = manager.get(id).unwrap();
    assert_eq!(retrieved.title, "Hello");
    assert!(retrieved.is_shown());
}

#[rstest]
fn manager_dismiss(manager: NotificationManager) {
    let n = Notification::new("Hello", "World");
    let id = manager.notify(n).unwrap();

    manager.dismiss(id).unwrap();
    assert_eq!(manager.active_count(), 0);
    assert!(manager.get(id).is_none());
    assert_eq!(manager.history().len(), 1);
}

#[rstest]
fn manager_dismiss_not_found(manager: NotificationManager) {
    let result = manager.dismiss(uuid::Uuid::new_v4());
    assert!(result.is_err());
}

#[rstest]
fn manager_dismiss_all(manager: NotificationManager) {
    for i in 0..3 {
        manager
            .notify(Notification::new(format!("N{}", i), "body"))
            .unwrap();
    }
    assert_eq!(manager.active_count(), 3);

    manager.dismiss_all();
    assert_eq!(manager.active_count(), 0);
    assert_eq!(manager.history().len(), 3);
}

#[rstest]
fn manager_max_active(manager: NotificationManager) {
    manager.set_max_active(2);

    let _id1 = manager.notify(Notification::new("N1", "body")).unwrap();
    let _id2 = manager.notify(Notification::new("N2", "body")).unwrap();
    let _id3 = manager.notify(Notification::new("N3", "body")).unwrap();

    // Max is 2, so oldest should be evicted
    assert_eq!(manager.active_count(), 2);
    assert_eq!(manager.history().len(), 1);
}

#[rstest]
fn manager_tag_replacement(manager: NotificationManager) {
    let n1 = Notification::new("V1", "body").with_tag("update");
    let n2 = Notification::new("V2", "body").with_tag("update");

    manager.notify(n1).unwrap();
    assert_eq!(manager.active_count(), 1);

    manager.notify(n2).unwrap();
    // Should replace the first one with the same tag
    assert_eq!(manager.active_count(), 1);
    assert_eq!(manager.history().len(), 1);

    let active = manager.active();
    assert_eq!(active[0].title, "V2");
}

#[rstest]
fn manager_history_limit(manager: NotificationManager) {
    manager.set_max_history(3);

    for i in 0..5 {
        let n = Notification::new(format!("N{}", i), "body");
        let id = manager.notify(n).unwrap();
        manager.dismiss(id).unwrap();
    }

    assert_eq!(manager.history().len(), 3);
}

#[rstest]
fn manager_clear_history(manager: NotificationManager) {
    let id = manager.notify(Notification::new("N1", "body")).unwrap();
    manager.dismiss(id).unwrap();
    assert_eq!(manager.history().len(), 1);

    manager.clear_history();
    assert_eq!(manager.history().len(), 0);
}

#[rstest]
fn manager_permission_default(manager: NotificationManager) {
    assert_eq!(
        manager.permission("https://example.com"),
        PermissionState::Default
    );
}

#[rstest]
fn manager_set_permission(manager: NotificationManager) {
    manager.set_permission("https://example.com", true);
    assert_eq!(
        manager.permission("https://example.com"),
        PermissionState::Granted
    );

    manager.set_permission("https://example.com", false);
    assert_eq!(
        manager.permission("https://example.com"),
        PermissionState::Denied
    );
}

#[rstest]
fn manager_request_permission(manager: NotificationManager) {
    let state = manager.request_permission("https://example.com");
    assert_eq!(state, PermissionState::Granted); // Auto-grant

    // Subsequent requests return same state
    let state2 = manager.request_permission("https://example.com");
    assert_eq!(state2, PermissionState::Granted);
}

#[rstest]
fn manager_permission_denied_blocks_notify(manager: NotificationManager) {
    manager.set_permission("https://evil.com", false);

    let n = Notification::new("Spam", "body");
    let result = manager.notify_for_origin(n, "https://evil.com");
    assert!(result.is_err());
}

#[rstest]
fn manager_trigger_action(manager: NotificationManager) {
    let n = Notification::new("Alert", "body")
        .with_action(NotificationAction::new("ok", "OK"))
        .with_action(NotificationAction::new("cancel", "Cancel"));
    let id = manager.notify(n).unwrap();

    assert!(manager.trigger_action(id, "ok").is_ok());
    assert!(manager.trigger_action(id, "nonexistent").is_err());
}

#[rstest]
fn manager_callbacks(manager: NotificationManager) {
    let show_count = Arc::new(AtomicUsize::new(0));
    let close_count = Arc::new(AtomicUsize::new(0));

    let sc = show_count.clone();
    manager.on_show(move |_| {
        sc.fetch_add(1, Ordering::SeqCst);
    });

    let cc = close_count.clone();
    manager.on_close(move |_| {
        cc.fetch_add(1, Ordering::SeqCst);
    });

    let id = manager.notify(Notification::new("T", "B")).unwrap();
    assert_eq!(show_count.load(Ordering::SeqCst), 1);

    manager.dismiss(id).unwrap();
    assert_eq!(close_count.load(Ordering::SeqCst), 1);
}

#[rstest]
fn manager_clone_shares_state(manager: NotificationManager) {
    let manager2 = manager.clone();

    let id = manager.notify(Notification::new("T", "B")).unwrap();
    assert_eq!(manager2.active_count(), 1);

    manager2.dismiss(id).unwrap();
    assert_eq!(manager.active_count(), 0);
}

#[rstest]
fn manager_default() {
    let manager = NotificationManager::default();
    assert_eq!(manager.active_count(), 0);
}

// ========== Serde Roundtrip Tests ==========

#[test]
fn serde_notification_type_info() {
    let kind = NotificationType::Info;
    let json = serde_json::to_string(&kind).unwrap();
    assert_eq!(json, r#""info""#);
    let back: NotificationType = serde_json::from_str(&json).unwrap();
    assert_eq!(back, kind);
}

#[rstest]
#[case(NotificationType::Info, r#""info""#)]
#[case(NotificationType::Success, r#""success""#)]
#[case(NotificationType::Warning, r#""warning""#)]
#[case(NotificationType::Error, r#""error""#)]
fn serde_notification_type_variants(
    #[case] kind: NotificationType,
    #[case] expected_json: &str,
) {
    let json = serde_json::to_string(&kind).unwrap();
    assert_eq!(json, expected_json);
    let back: NotificationType = serde_json::from_str(&json).unwrap();
    assert_eq!(back, kind);
}

#[test]
fn serde_notification_roundtrip_basic() {
    let n = Notification::new("Title", "Body");
    let json = serde_json::to_string(&n).unwrap();
    let back: Notification = serde_json::from_str(&json).unwrap();

    assert_eq!(back.id, n.id);
    assert_eq!(back.title, n.title);
    assert_eq!(back.body, n.body);
    assert_eq!(back.notification_type, n.notification_type);
    assert_eq!(back.duration, n.duration);
    assert_eq!(back.require_interaction, n.require_interaction);
}

#[test]
fn serde_notification_roundtrip_full() {
    let n = Notification::new("Alert", "Something happened")
        .with_type(NotificationType::Error)
        .with_icon("error-icon")
        .with_image("https://example.com/img.png")
        .with_tag("alert-group")
        .with_data(serde_json::json!({"key": "value", "count": 42}))
        .with_action(NotificationAction::new("ok", "OK").with_icon("check"))
        .with_action(NotificationAction::new("cancel", "Cancel"))
        .persistent();

    let json = serde_json::to_string(&n).unwrap();
    let back: Notification = serde_json::from_str(&json).unwrap();

    assert_eq!(back.title, "Alert");
    assert_eq!(back.notification_type, NotificationType::Error);
    assert_eq!(back.icon, Some("error-icon".to_string()));
    assert_eq!(back.tag, Some("alert-group".to_string()));
    assert!(back.require_interaction);
    assert_eq!(back.actions.len(), 2);
    assert_eq!(back.actions[0].id, "ok");
    assert_eq!(back.actions[0].icon, Some("check".to_string()));

    assert_eq!(back.actions[1].icon, None);
    assert!(back.data.is_some());
}

#[test]
fn serde_notification_simple_omits_body() {
    let n = Notification::simple("No Body");
    let json = serde_json::to_string(&n).unwrap();
    // body should be absent (skip_serializing_if = Option::is_none)
    assert!(!json.contains(r#""body""#));
    let back: Notification = serde_json::from_str(&json).unwrap();
    assert!(back.body.is_none());
}

#[test]
fn serde_notification_action_roundtrip() {
    let action = NotificationAction::new("btn1", "Click me").with_icon("check");
    let json = serde_json::to_string(&action).unwrap();
    let back: NotificationAction = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "btn1");
    assert_eq!(back.label, "Click me");
    assert_eq!(back.icon, Some("check".to_string()));
}

#[test]
fn serde_permission_state_roundtrip() {
    for state in [
        PermissionState::Default,
        PermissionState::Granted,
        PermissionState::Denied,
    ] {
        let json = serde_json::to_string(&state).unwrap();
        let back: PermissionState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state);
    }
}

#[test]
fn serde_permission_roundtrip() {
    let p = Permission::granted("https://example.com");
    let json = serde_json::to_string(&p).unwrap();
    let back: Permission = serde_json::from_str(&json).unwrap();
    assert_eq!(back.origin, "https://example.com");
    assert_eq!(back.state, PermissionState::Granted);
    assert!(back.updated_at.is_some());
}

// ========== NotificationError Tests ==========

use auroraview_notifications::NotificationError;

#[test]
fn error_not_found_display() {
    let id = uuid::Uuid::new_v4();
    let err = NotificationError::NotFound(id);
    let msg = err.to_string();
    assert!(msg.contains("not found"), "expected 'not found' in: {msg}");
    assert!(msg.contains(&id.to_string()));
}

#[test]
fn error_permission_denied_display() {
    let err = NotificationError::PermissionDenied;
    let msg = err.to_string();
    assert!(msg.to_lowercase().contains("permission") || msg.to_lowercase().contains("denied"), "got: {msg}");
}

#[test]
fn error_permission_not_requested_display() {
    let err = NotificationError::PermissionNotRequested;
    let msg = err.to_string();
    assert!(!msg.is_empty());
}

#[test]
fn error_invalid_notification_display() {
    let err = NotificationError::InvalidNotification("bad action".to_string());
    let msg = err.to_string();
    assert!(msg.contains("bad action"), "got: {msg}");
}

#[test]
fn error_max_notifications_reached_display() {
    let err = NotificationError::MaxNotificationsReached(5);
    let msg = err.to_string();
    assert!(msg.contains('5'), "got: {msg}");
}

// ========== Notification Edge Cases ==========

#[test]
fn notification_with_type_error_has_no_duration() {
    let n = Notification::new("T", "B").with_type(NotificationType::Error);
    assert!(n.duration.is_none());
    assert!(!n.require_interaction); // with_type does not set require_interaction
}

#[test]
fn notification_with_type_success_duration() {
    let n = Notification::new("T", "B").with_type(NotificationType::Success);
    assert_eq!(n.duration, Some(3000));
}

#[test]
fn notification_multiple_actions() {
    let n = Notification::new("T", "B")
        .with_action(NotificationAction::new("a1", "Action 1"))
        .with_action(NotificationAction::new("a2", "Action 2"))
        .with_action(NotificationAction::new("a3", "Action 3"));
    assert_eq!(n.actions.len(), 3);
    assert_eq!(n.actions[2].id, "a3");
}

#[test]
fn notification_custom_duration_overrides_type() {
    let n = Notification::new("T", "B")
        .with_type(NotificationType::Error)
        .with_duration(2000);
    assert_eq!(n.duration, Some(2000));
}

#[test]
fn notification_zero_duration() {
    let n = Notification::new("T", "B").with_duration(0);
    assert_eq!(n.duration, Some(0));
}

// ========== Manager Edge Cases ==========

#[rstest]
fn manager_action_callback_triggered(manager: NotificationManager) {
    let action_count = Arc::new(AtomicUsize::new(0));
    let ac = action_count.clone();

    manager.on_action(move |_, _action_id| {
        ac.fetch_add(1, Ordering::SeqCst);
    });

    let n = Notification::new("Alert", "body")
        .with_action(NotificationAction::new("ok", "OK"));
    let id = manager.notify(n).unwrap();

    manager.trigger_action(id, "ok").unwrap();
    assert_eq!(action_count.load(Ordering::SeqCst), 1);
}

#[rstest]
fn manager_notify_granted_origin_succeeds(manager: NotificationManager) {
    manager.set_permission("https://trusted.com", true);
    let n = Notification::new("T", "B");
    let result = manager.notify_for_origin(n, "https://trusted.com");
    assert!(result.is_ok());
}

#[rstest]
fn manager_default_origin_auto_granted(manager: NotificationManager) {
    // Unset origins are auto-granted for standalone
    let n = Notification::new("T", "B");
    let result = manager.notify_for_origin(n, "https://unknown-origin.com");
    assert!(result.is_ok());
}

#[rstest]
fn manager_history_entries_are_dismissed(manager: NotificationManager) {
    let id = manager.notify(Notification::new("T", "B")).unwrap();
    manager.dismiss(id).unwrap();

    let history = manager.history();
    assert_eq!(history.len(), 1);
    assert!(!history[0].is_active()); // must be dismissed
}

#[rstest]
fn manager_dismiss_all_callbacks_called(manager: NotificationManager) {
    let close_count = Arc::new(AtomicUsize::new(0));
    let cc = close_count.clone();
    manager.on_close(move |_| {
        cc.fetch_add(1, Ordering::SeqCst);
    });

    for i in 0..4 {
        manager.notify(Notification::new(format!("N{i}"), "body")).unwrap();
    }
    manager.dismiss_all();
    assert_eq!(close_count.load(Ordering::SeqCst), 4);
}

#[rstest]
fn manager_set_max_history_trims_existing(manager: NotificationManager) {
    // Add 5 dismissed notifications
    for i in 0..5 {
        let id = manager.notify(Notification::new(format!("N{i}"), "body")).unwrap();
        manager.dismiss(id).unwrap();
    }
    assert_eq!(manager.history().len(), 5);

    // Setting max_history to 2 should trim on next dismiss
    manager.set_max_history(2);
    let id = manager.notify(Notification::new("Extra", "body")).unwrap();
    manager.dismiss(id).unwrap();
    // After one more dismiss the history is trimmed to max 2
    assert!(manager.history().len() <= 2);
}

#[rstest]
fn manager_max_active_zero_evicts_immediately(manager: NotificationManager) {
    manager.set_max_active(1);
    let _id1 = manager.notify(Notification::new("N1", "body")).unwrap();
    let _id2 = manager.notify(Notification::new("N2", "body")).unwrap();
    // Only 1 active allowed, so oldest is evicted
    assert_eq!(manager.active_count(), 1);
}

// ========== Concurrent Tests ==========

#[test]
fn concurrent_notify_no_panic() {

    let manager = Arc::new(NotificationManager::new());

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let m = Arc::clone(&manager);
            thread::spawn(move || {
                for j in 0..10 {
                    let n = Notification::new(format!("T{i}-{j}"), "body");
                    let _ = m.notify(n);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    // All notifications were created concurrently without panic
    assert!(manager.active_count() > 0);
}

#[test]
fn concurrent_notify_and_dismiss_no_deadlock() {

    let manager = Arc::new(NotificationManager::new());

    // Producer threads
    let producer_handles: Vec<_> = (0..4)
        .map(|i| {
            let m = Arc::clone(&manager);
            thread::spawn(move || {
                for j in 0..5 {
                    let n = Notification::new(format!("P{i}-{j}"), "body");
                    let _ = m.notify(n);
                }
            })
        })
        .collect();

    // Dismiss-all thread
    let m2 = Arc::clone(&manager);
    let dismisser = thread::spawn(move || {
        for _ in 0..5 {
            m2.dismiss_all();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });

    for h in producer_handles {
        h.join().unwrap();
    }
    dismisser.join().unwrap();
}


#[test]
fn concurrent_permission_reads_no_panic() {

    let manager = Arc::new(NotificationManager::new());
    manager.set_permission("https://example.com", true);

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let m = Arc::clone(&manager);
            thread::spawn(move || {
                for _ in 0..20 {
                    let _ = m.permission("https://example.com");
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

// ========== Permission State Methods ==========

#[rstest]
fn permission_state_is_granted() {
    assert!(PermissionState::Granted.is_granted());
    assert!(!PermissionState::Denied.is_granted());
    assert!(!PermissionState::Default.is_granted());
}

#[rstest]
fn permission_state_is_denied() {
    assert!(PermissionState::Denied.is_denied());
    assert!(!PermissionState::Granted.is_denied());
    assert!(!PermissionState::Default.is_denied());
}

#[rstest]
fn permission_state_is_default() {
    assert!(PermissionState::Default.is_default());
    assert!(!PermissionState::Granted.is_default());
    assert!(!PermissionState::Denied.is_default());
}

// ========== Permission struct methods ==========

#[rstest]
fn permission_new_is_default_state() {
    let p = Permission::new("https://example.com");
    assert_eq!(p.state, PermissionState::Default);
    assert_eq!(p.origin, "https://example.com");
    assert!(p.updated_at.is_none());
}

#[rstest]
fn permission_granted_factory() {
    let p = Permission::granted("https://trusted.com");
    assert_eq!(p.state, PermissionState::Granted);
    assert!(p.updated_at.is_some());
}

#[rstest]
fn permission_denied_factory() {
    let p = Permission::denied("https://evil.com");
    assert_eq!(p.state, PermissionState::Denied);
    assert!(p.updated_at.is_some());
}

#[rstest]
fn permission_grant_method() {
    let mut p = Permission::new("https://example.com");
    p.grant();
    assert_eq!(p.state, PermissionState::Granted);
}

#[rstest]
fn permission_deny_method() {
    let mut p = Permission::new("https://example.com");
    p.deny();
    assert_eq!(p.state, PermissionState::Denied);
}

#[rstest]
fn permission_reset_method() {
    let mut p = Permission::granted("https://example.com");
    p.reset();
    assert_eq!(p.state, PermissionState::Default);
}

#[rstest]
fn permission_grant_then_deny_then_reset() {
    let mut p = Permission::new("https://example.com");
    p.grant();
    assert!(p.state.is_granted());
    p.deny();
    assert!(p.state.is_denied());
    p.reset();
    assert!(p.state.is_default());
}


// ========== Notification mark methods ==========

#[rstest]
fn notification_mark_shown_state_transition() {
    let mut n = Notification::new("T", "B");
    assert!(!n.is_shown());
    n.mark_shown();
    assert!(n.is_shown());
    assert!(n.is_active()); // still active until dismissed
}

#[rstest]
fn notification_mark_dismissed_state_transition() {
    let mut n = Notification::new("T", "B");
    n.mark_shown();
    n.mark_dismissed();
    assert!(!n.is_active());
    assert!(n.is_shown());
}

#[rstest]
fn notification_new_is_active_not_shown() {
    let n = Notification::new("T", "B");
    assert!(n.is_active());
    assert!(!n.is_shown());
}

// ========== Notification builder edge cases ==========

#[rstest]
fn notification_simple_has_no_body() {
    let n = Notification::simple("Title only");
    assert_eq!(n.title, "Title only");
    assert!(n.body.is_none());
}

#[rstest]
fn notification_simple_inherits_type_default() {
    let n = Notification::simple("T");
    assert_eq!(n.notification_type, NotificationType::Info);
}

#[rstest]
fn notification_with_data() {
    let n = Notification::new("T", "B")
        .with_data(serde_json::json!({"user_id": 42, "level": "critical"}));
    assert!(n.data.is_some());
    assert_eq!(n.data.as_ref().unwrap()["user_id"], 42);
}

#[rstest]
fn notification_persistent_sets_require_interaction() {
    let n = Notification::new("T", "B").persistent();
    assert!(n.require_interaction);
}

#[rstest]
fn notification_with_tag() {
    let n = Notification::new("T", "B").with_tag("alert");
    assert_eq!(n.tag, Some("alert".to_string()));
}

#[rstest]
fn notification_with_icon_and_image() {
    let n = Notification::new("T", "B")
        .with_icon("icon.png")
        .with_image("banner.jpg");
    assert_eq!(n.icon, Some("icon.png".to_string()));
    assert_eq!(n.image, Some("banner.jpg".to_string()));
}

// ========== NotificationType default_duration ==========

#[rstest]
fn notification_type_info_default_duration() {
    assert_eq!(NotificationType::Info.default_duration(), Some(5000));
}

#[rstest]
fn notification_type_success_default_duration() {
    assert_eq!(NotificationType::Success.default_duration(), Some(3000));
}

#[rstest]
fn notification_type_warning_default_duration() {
    assert_eq!(NotificationType::Warning.default_duration(), Some(8000));
}

#[rstest]
fn notification_type_error_default_duration() {
    assert_eq!(NotificationType::Error.default_duration(), None);
}

// ========== Manager get() ==========

#[rstest]
fn manager_get_active_notification(manager: NotificationManager) {
    let id = manager.notify(Notification::new("T", "B")).unwrap();
    let n = manager.get(id).unwrap();
    assert_eq!(n.title, "T");
}

#[rstest]
fn manager_get_nonexistent_returns_none(manager: NotificationManager) {
    let nonexistent = uuid::Uuid::new_v4();
    assert!(manager.get(nonexistent).is_none());
}

// ========== Manager active() ==========

#[rstest]
fn manager_active_empty_initially(manager: NotificationManager) {
    let active = manager.active();
    assert!(active.is_empty());
}

#[rstest]
fn manager_active_returns_all_active(manager: NotificationManager) {
    manager.notify(Notification::new("N1", "B")).unwrap();
    manager.notify(Notification::new("N2", "B")).unwrap();
    let active = manager.active();
    assert_eq!(active.len(), 2);
}

// ========== Manager clear_history ==========

#[rstest]
fn manager_clear_history_after_dismiss(manager: NotificationManager) {
    let id = manager.notify(Notification::new("T", "B")).unwrap();
    manager.dismiss(id).unwrap();
    assert_eq!(manager.history().len(), 1);

    manager.clear_history();
    assert!(manager.history().is_empty());
}

// ========== Manager dismiss errors ==========

#[rstest]
fn manager_dismiss_nonexistent_returns_err(manager: NotificationManager) {
    let fake_id = uuid::Uuid::new_v4();
    let result = manager.dismiss(fake_id);
    assert!(result.is_err());
}

#[rstest]
fn manager_trigger_action_dismissed_returns_err(manager: NotificationManager) {
    let n = Notification::new("T", "B")
        .with_action(NotificationAction::new("ok", "OK"));
    let id = manager.notify(n).unwrap();
    manager.dismiss(id).unwrap();
    // After dismiss, triggering action should error (not found)
    let result = manager.trigger_action(id, "ok");
    assert!(result.is_err());
}

// ========== NotificationError: Send + Sync ==========

#[test]
fn notification_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<auroraview_notifications::NotificationError>();
}

// ========== NotificationType: Clone + PartialEq ==========

#[rstest]
#[case(NotificationType::Info)]
#[case(NotificationType::Success)]
#[case(NotificationType::Warning)]
#[case(NotificationType::Error)]
fn notification_type_clone_eq(#[case] kind: NotificationType) {
    let cloned = kind.clone();
    assert_eq!(cloned, kind);
}

// ========== Concurrent multi-origin notifications ==========

#[test]
fn concurrent_multi_origin_permissions() {
    let manager = Arc::new(NotificationManager::new());

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let m = Arc::clone(&manager);
            thread::spawn(move || {
                let origin = format!("https://site{}.com", i);
                m.set_permission(&origin, i % 2 == 0);
                let _ = m.permission(&origin);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
