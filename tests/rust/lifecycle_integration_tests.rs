//! Integration tests for lifecycle management
//!
//! These tests verify the LifecycleManager functionality across different scenarios.

use _core::webview::lifecycle::{CloseReason, LifecycleManager, LifecycleState};
use rstest::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

/// Fixture: Create a new lifecycle manager
#[fixture]
fn manager() -> LifecycleManager {
    LifecycleManager::new()
}

#[rstest]
fn test_initial_state(manager: LifecycleManager) {
    assert_eq!(manager.state(), LifecycleState::Creating);
}

#[rstest]
#[case(LifecycleState::Active)]
#[case(LifecycleState::Destroying)]
#[case(LifecycleState::Destroyed)]
fn test_set_state(manager: LifecycleManager, #[case] new_state: LifecycleState) {
    manager.set_state(new_state);
    assert_eq!(manager.state(), new_state);
}

#[rstest]
fn test_state_transition_sequence(manager: LifecycleManager) {
    assert_eq!(manager.state(), LifecycleState::Creating);

    manager.set_state(LifecycleState::Active);
    assert_eq!(manager.state(), LifecycleState::Active);

    manager.set_state(LifecycleState::Destroying);
    assert_eq!(manager.state(), LifecycleState::Destroying);

    manager.set_state(LifecycleState::Destroyed);
    assert_eq!(manager.state(), LifecycleState::Destroyed);
}

#[rstest]
fn test_request_close_from_active(manager: LifecycleManager) {
    manager.set_state(LifecycleState::Active);
    assert!(manager.request_close(CloseReason::UserRequest).is_ok());
    assert_eq!(manager.state(), LifecycleState::CloseRequested);
}

#[rstest]
fn test_request_close_on_destroyed_fails(manager: LifecycleManager) {
    manager.set_state(LifecycleState::Destroyed);
    let result = manager.request_close(CloseReason::UserRequest);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already destroyed"));
}

#[rstest]
fn test_request_close_when_destroying_is_noop(manager: LifecycleManager) {
    manager.set_state(LifecycleState::Destroying);
    assert!(manager.request_close(CloseReason::SystemShutdown).is_ok());
    assert_eq!(manager.check_close_requested(), None);
    assert_eq!(manager.state(), LifecycleState::Destroying);
}

#[rstest]
#[case(CloseReason::UserRequest)]
#[case(CloseReason::AppRequest)]
#[case(CloseReason::ParentClosed)]
#[case(CloseReason::SystemShutdown)]
#[case(CloseReason::Error)]
fn test_check_close_requested(manager: LifecycleManager, #[case] reason: CloseReason) {
    manager.set_state(LifecycleState::Active);
    manager.request_close(reason).unwrap();
    assert_eq!(manager.check_close_requested(), Some(reason));
}

#[rstest]
fn test_cleanup_handlers_execute(manager: LifecycleManager) {
    let counter = Arc::new(AtomicUsize::new(0));
    let c1 = counter.clone();
    let c2 = counter.clone();
    let c3 = counter.clone();

    manager.register_cleanup(move || {
        c1.fetch_add(1, Ordering::SeqCst);
    });
    manager.register_cleanup(move || {
        c2.fetch_add(10, Ordering::SeqCst);
    });
    manager.register_cleanup(move || {
        c3.fetch_add(100, Ordering::SeqCst);
    });

    manager.execute_cleanup();

    assert_eq!(counter.load(Ordering::SeqCst), 111);
    assert_eq!(manager.state(), LifecycleState::Destroyed);
}

#[rstest]
fn test_cleanup_transitions_state(manager: LifecycleManager) {
    manager.set_state(LifecycleState::Active);
    manager.execute_cleanup();
    assert_eq!(manager.state(), LifecycleState::Destroyed);
}

#[rstest]
fn test_wait_for_close_with_timeout(manager: LifecycleManager) {
    let tx = manager.close_sender();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        let _ = tx.send(CloseReason::UserRequest);
    });

    let reason = manager.wait_for_close(Duration::from_secs(1));
    assert_eq!(reason, Some(CloseReason::UserRequest));
}

#[rstest]
fn test_wait_for_close_timeout_expires(manager: LifecycleManager) {
    let reason = manager.wait_for_close(Duration::from_millis(10));
    assert_eq!(reason, None);
}

#[rstest]
fn test_close_sender_can_be_cloned(manager: LifecycleManager) {
    let tx1 = manager.close_sender();
    let tx2 = tx1.clone();

    thread::spawn(move || {
        let _ = tx2.send(CloseReason::AppRequest);
    });

    thread::sleep(Duration::from_millis(50));
    assert_eq!(
        manager.check_close_requested(),
        Some(CloseReason::AppRequest)
    );
}

#[rstest]
fn test_multiple_cleanup_handlers_execute_in_order(manager: LifecycleManager) {
    let order = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let o1 = order.clone();
    let o2 = order.clone();
    let o3 = order.clone();

    manager.register_cleanup(move || {
        o1.lock().push(1);
    });
    manager.register_cleanup(move || {
        o2.lock().push(2);
    });
    manager.register_cleanup(move || {
        o3.lock().push(3);
    });

    manager.execute_cleanup();

    let execution_order = order.lock();
    assert_eq!(*execution_order, vec![1, 2, 3]);
}
