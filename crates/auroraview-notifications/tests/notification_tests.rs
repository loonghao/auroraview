use auroraview_notifications::{
    Notification, NotificationAction, NotificationManager, NotificationType, Permission,
    PermissionState,
};
use rstest::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// ========== NotificationType Tests ==========

#[rstest]
#[case(NotificationType::Info, Some(5000))]
#[case(NotificationType::Success, Some(3000))]
#[case(NotificationType::Warning, Some(8000))]
#[case(NotificationType::Error, None)]
fn test_notification_type_duration(#[case] kind: NotificationType, #[case] expected: Option<u64>) {
    assert_eq!(kind.default_duration(), expected);
}

#[test]
fn test_notification_type_default_is_info() {
    assert_eq!(NotificationType::default(), NotificationType::Info);
}

// ========== NotificationAction Tests ==========

#[test]
fn test_action_new() {
    let action = NotificationAction::new("btn1", "Click me");
    assert_eq!(action.id, "btn1");
    assert_eq!(action.label, "Click me");
    assert!(action.icon.is_none());
}

#[test]
fn test_action_with_icon() {
    let action = NotificationAction::new("btn1", "Click me").with_icon("check");
    assert_eq!(action.icon, Some("check".to_string()));
}

// ========== Notification Tests ==========

#[test]
fn test_notification_new() {
    let n = Notification::new("Title", "Body");
    assert_eq!(n.title, "Title");
    assert_eq!(n.body, Some("Body".to_string()));
    assert_eq!(n.notification_type, NotificationType::Info);
    assert_eq!(n.duration, Some(5000));
    assert!(!n.require_interaction);
    assert!(n.is_active());
    assert!(!n.is_shown());
}

#[test]
fn test_notification_simple() {
    let n = Notification::simple("Title only");
    assert_eq!(n.title, "Title only");
    assert!(n.body.is_none());
}

#[test]
fn test_notification_builder() {
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

#[test]
fn test_notification_persistent() {
    let n = Notification::new("Error", "Critical failure").persistent();
    assert!(n.duration.is_none());
    assert!(n.require_interaction);
}

#[test]
fn test_notification_with_duration() {
    let n = Notification::new("Custom", "msg").with_duration(10000);
    assert_eq!(n.duration, Some(10000));
}

#[test]
fn test_notification_mark_shown() {
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

#[test]
fn test_notification_mark_dismissed() {
    let mut n = Notification::new("T", "B");
    assert!(n.is_active());

    n.mark_dismissed();
    assert!(!n.is_active());
    assert!(n.dismissed_at.is_some());

    let first_dismissed = n.dismissed_at;
    n.mark_dismissed();
    assert_eq!(n.dismissed_at, first_dismissed);
}

#[test]
fn test_notification_display() {
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
fn test_permission_state(
    #[case] state: PermissionState,
    #[case] granted: bool,
    #[case] denied: bool,
    #[case] default: bool,
) {
    assert_eq!(state.is_granted(), granted);
    assert_eq!(state.is_denied(), denied);
    assert_eq!(state.is_default(), default);
}

#[test]
fn test_permission_state_display() {
    assert_eq!(PermissionState::Default.to_string(), "default");
    assert_eq!(PermissionState::Granted.to_string(), "granted");
    assert_eq!(PermissionState::Denied.to_string(), "denied");
}

// ========== Permission Tests ==========

#[test]
fn test_permission_new() {
    let p = Permission::new("https://example.com");
    assert_eq!(p.origin, "https://example.com");
    assert_eq!(p.state, PermissionState::Default);
    assert!(p.updated_at.is_none());
}

#[test]
fn test_permission_granted() {
    let p = Permission::granted("https://example.com");
    assert_eq!(p.state, PermissionState::Granted);
    assert!(p.updated_at.is_some());
}

#[test]
fn test_permission_denied() {
    let p = Permission::denied("https://example.com");
    assert_eq!(p.state, PermissionState::Denied);
    assert!(p.updated_at.is_some());
}

#[test]
fn test_permission_grant_deny_reset() {
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
fn test_manager_notify(manager: NotificationManager) {
    let n = Notification::new("Hello", "World");
    let id = manager.notify(n).unwrap();

    assert_eq!(manager.active_count(), 1);
    let retrieved = manager.get(id).unwrap();
    assert_eq!(retrieved.title, "Hello");
    assert!(retrieved.is_shown());
}

#[rstest]
fn test_manager_dismiss(manager: NotificationManager) {
    let n = Notification::new("Hello", "World");
    let id = manager.notify(n).unwrap();

    manager.dismiss(id).unwrap();
    assert_eq!(manager.active_count(), 0);
    assert!(manager.get(id).is_none());
    assert_eq!(manager.history().len(), 1);
}

#[rstest]
fn test_manager_dismiss_not_found(manager: NotificationManager) {
    let result = manager.dismiss(uuid::Uuid::new_v4());
    assert!(result.is_err());
}

#[rstest]
fn test_manager_dismiss_all(manager: NotificationManager) {
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
fn test_manager_max_active(manager: NotificationManager) {
    manager.set_max_active(2);

    let _id1 = manager.notify(Notification::new("N1", "body")).unwrap();
    let _id2 = manager.notify(Notification::new("N2", "body")).unwrap();
    let _id3 = manager.notify(Notification::new("N3", "body")).unwrap();

    // Max is 2, so oldest should be evicted
    assert_eq!(manager.active_count(), 2);
    assert_eq!(manager.history().len(), 1);
}

#[rstest]
fn test_manager_tag_replacement(manager: NotificationManager) {
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
fn test_manager_history_limit(manager: NotificationManager) {
    manager.set_max_history(3);

    for i in 0..5 {
        let n = Notification::new(format!("N{}", i), "body");
        let id = manager.notify(n).unwrap();
        manager.dismiss(id).unwrap();
    }

    assert_eq!(manager.history().len(), 3);
}

#[rstest]
fn test_manager_clear_history(manager: NotificationManager) {
    let id = manager.notify(Notification::new("N1", "body")).unwrap();
    manager.dismiss(id).unwrap();
    assert_eq!(manager.history().len(), 1);

    manager.clear_history();
    assert_eq!(manager.history().len(), 0);
}

#[rstest]
fn test_manager_permission_default(manager: NotificationManager) {
    assert_eq!(
        manager.permission("https://example.com"),
        PermissionState::Default
    );
}

#[rstest]
fn test_manager_set_permission(manager: NotificationManager) {
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
fn test_manager_request_permission(manager: NotificationManager) {
    let state = manager.request_permission("https://example.com");
    assert_eq!(state, PermissionState::Granted); // Auto-grant

    // Subsequent requests return same state
    let state2 = manager.request_permission("https://example.com");
    assert_eq!(state2, PermissionState::Granted);
}

#[rstest]
fn test_manager_permission_denied_blocks_notify(manager: NotificationManager) {
    manager.set_permission("https://evil.com", false);

    let n = Notification::new("Spam", "body");
    let result = manager.notify_for_origin(n, "https://evil.com");
    assert!(result.is_err());
}

#[rstest]
fn test_manager_trigger_action(manager: NotificationManager) {
    let n = Notification::new("Alert", "body")
        .with_action(NotificationAction::new("ok", "OK"))
        .with_action(NotificationAction::new("cancel", "Cancel"));
    let id = manager.notify(n).unwrap();

    assert!(manager.trigger_action(id, "ok").is_ok());
    assert!(manager.trigger_action(id, "nonexistent").is_err());
}

#[rstest]
fn test_manager_callbacks(manager: NotificationManager) {
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
fn test_manager_clone_shares_state(manager: NotificationManager) {
    let manager2 = manager.clone();

    let id = manager.notify(Notification::new("T", "B")).unwrap();
    assert_eq!(manager2.active_count(), 1);

    manager2.dismiss(id).unwrap();
    assert_eq!(manager.active_count(), 0);
}

#[test]
fn test_manager_default() {
    let manager = NotificationManager::default();
    assert_eq!(manager.active_count(), 0);
}
