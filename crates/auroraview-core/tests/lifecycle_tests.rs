//! Lifecycle state machine tests

use auroraview_core::backend::{
    AtomicLifecycle, LifecycleEvent, LifecycleObserver, LifecycleState, ObservableLifecycle,
    TransitionResult,
};
use rstest::rstest;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

// ============================================================================
// LifecycleState Display + From<u8>
// ============================================================================

#[rstest]
#[case(LifecycleState::Creating, "Creating")]
#[case(LifecycleState::Active, "Active")]
#[case(LifecycleState::CloseRequested, "CloseRequested")]
#[case(LifecycleState::Destroying, "Destroying")]
#[case(LifecycleState::Destroyed, "Destroyed")]
fn test_lifecycle_state_display(#[case] state: LifecycleState, #[case] expected: &str) {
    assert_eq!(format!("{}", state), expected);
}

#[rstest]
#[case(0u8, LifecycleState::Creating)]
#[case(1u8, LifecycleState::Active)]
#[case(2u8, LifecycleState::CloseRequested)]
#[case(3u8, LifecycleState::Destroying)]
#[case(4u8, LifecycleState::Destroyed)]
fn test_lifecycle_state_from_u8(#[case] value: u8, #[case] expected: LifecycleState) {
    assert_eq!(LifecycleState::from(value), expected);
}

#[test]
fn test_lifecycle_state_from_u8_invalid_defaults_to_destroyed() {
    assert_eq!(LifecycleState::from(5u8), LifecycleState::Destroyed);
    assert_eq!(LifecycleState::from(255u8), LifecycleState::Destroyed);
    assert_eq!(LifecycleState::from(100u8), LifecycleState::Destroyed);
}

// ============================================================================
// TransitionResult
// ============================================================================

#[test]
fn test_transition_result_is_success() {
    assert!(TransitionResult::Success.is_success());
    assert!(!TransitionResult::InvalidState.is_success());
    assert!(!TransitionResult::AlreadyInState.is_success());
}

#[test]
fn test_transition_result_eq() {
    assert_eq!(TransitionResult::Success, TransitionResult::Success);
    assert_eq!(TransitionResult::InvalidState, TransitionResult::InvalidState);
    assert_ne!(TransitionResult::Success, TransitionResult::InvalidState);
    assert_ne!(TransitionResult::Success, TransitionResult::AlreadyInState);
}

// ============================================================================
// AtomicLifecycle – initial state
// ============================================================================

#[test]
fn test_new_starts_creating() {
    let lc = AtomicLifecycle::new();
    assert_eq!(lc.state(), LifecycleState::Creating);
    assert!(!lc.is_active());
    assert!(!lc.is_closing());
    assert!(!lc.is_destroyed());
}

#[test]
fn test_new_active_starts_active() {
    let lc = AtomicLifecycle::new_active();
    assert_eq!(lc.state(), LifecycleState::Active);
    assert!(lc.is_active());
    assert!(!lc.is_closing());
    assert!(!lc.is_destroyed());
}

#[test]
fn test_default_is_same_as_new() {
    let lc = AtomicLifecycle::default();
    assert_eq!(lc.state(), LifecycleState::Creating);
}

// ============================================================================
// AtomicLifecycle – happy-path transitions
// ============================================================================

#[test]
fn test_full_lifecycle_sequence() {
    let lc = AtomicLifecycle::new();

    // Creating → Active
    assert_eq!(lc.activate(), TransitionResult::Success);
    assert_eq!(lc.state(), LifecycleState::Active);

    // Active → CloseRequested
    assert_eq!(lc.request_close(), TransitionResult::Success);
    assert_eq!(lc.state(), LifecycleState::CloseRequested);
    assert!(lc.is_closing());

    // CloseRequested → Destroying
    assert_eq!(lc.begin_destroy(), TransitionResult::Success);
    assert_eq!(lc.state(), LifecycleState::Destroying);
    assert!(lc.is_closing());

    // Destroying → Destroyed
    assert_eq!(lc.finish_destroy(), TransitionResult::Success);
    assert_eq!(lc.state(), LifecycleState::Destroyed);
    assert!(lc.is_destroyed());
    assert!(lc.is_closing());
}

// ============================================================================
// AtomicLifecycle – invalid transitions
// ============================================================================

#[test]
fn test_invalid_transitions_from_creating() {
    let lc = AtomicLifecycle::new();
    assert_eq!(lc.request_close(), TransitionResult::InvalidState);
    assert_eq!(lc.begin_destroy(), TransitionResult::InvalidState);
    assert_eq!(lc.finish_destroy(), TransitionResult::InvalidState);
}

#[test]
fn test_invalid_transitions_from_active() {
    let lc = AtomicLifecycle::new_active();
    assert_eq!(lc.begin_destroy(), TransitionResult::InvalidState);
    assert_eq!(lc.finish_destroy(), TransitionResult::InvalidState);
}

#[test]
fn test_invalid_transitions_from_close_requested() {
    let lc = AtomicLifecycle::new_active();
    lc.request_close();
    assert_eq!(lc.activate(), TransitionResult::InvalidState);
    assert_eq!(lc.request_close(), TransitionResult::AlreadyInState);
    assert_eq!(lc.finish_destroy(), TransitionResult::InvalidState);
}

#[test]
fn test_invalid_transitions_from_destroying() {
    let lc = AtomicLifecycle::new_active();
    lc.request_close();
    lc.begin_destroy();
    assert_eq!(lc.activate(), TransitionResult::InvalidState);
    assert_eq!(lc.request_close(), TransitionResult::InvalidState);
    assert_eq!(lc.begin_destroy(), TransitionResult::AlreadyInState);
}

#[test]
fn test_invalid_transitions_from_destroyed() {
    let lc = AtomicLifecycle::new_active();
    lc.request_close();
    lc.begin_destroy();
    lc.finish_destroy();
    assert_eq!(lc.activate(), TransitionResult::InvalidState);
    assert_eq!(lc.request_close(), TransitionResult::InvalidState);
    assert_eq!(lc.begin_destroy(), TransitionResult::InvalidState);
    assert_eq!(lc.finish_destroy(), TransitionResult::AlreadyInState);
}

#[test]
fn test_already_in_state_activate_twice() {
    let lc = AtomicLifecycle::new_active();
    assert_eq!(lc.activate(), TransitionResult::AlreadyInState);
}

// ============================================================================
// AtomicLifecycle – force_destroy
// ============================================================================

#[test]
fn test_force_destroy_from_creating() {
    let lc = AtomicLifecycle::new();
    lc.force_destroy();
    assert_eq!(lc.state(), LifecycleState::Destroyed);
    assert!(lc.is_destroyed());
}

#[test]
fn test_force_destroy_from_active() {
    let lc = AtomicLifecycle::new_active();
    lc.force_destroy();
    assert_eq!(lc.state(), LifecycleState::Destroyed);
}

#[test]
fn test_force_destroy_from_close_requested() {
    let lc = AtomicLifecycle::new_active();
    lc.request_close();
    lc.force_destroy();
    assert_eq!(lc.state(), LifecycleState::Destroyed);
}

#[test]
fn test_force_destroy_idempotent() {
    let lc = AtomicLifecycle::new_active();
    lc.force_destroy();
    lc.force_destroy();
    assert_eq!(lc.state(), LifecycleState::Destroyed);
}

// ============================================================================
// AtomicLifecycle – if_active / if_not_closing
// ============================================================================

#[test]
fn test_if_active_returns_none_while_creating() {
    let lc = AtomicLifecycle::new();
    assert!(lc.if_active(|| 42).is_none());
}

#[test]
fn test_if_active_returns_some_while_active() {
    let lc = AtomicLifecycle::new_active();
    assert_eq!(lc.if_active(|| "hello"), Some("hello"));
}

#[test]
fn test_if_active_returns_none_after_close_requested() {
    let lc = AtomicLifecycle::new_active();
    lc.request_close();
    assert!(lc.if_active(|| 99).is_none());
}

#[test]
fn test_if_not_closing_returns_some_while_creating() {
    let lc = AtomicLifecycle::new();
    assert_eq!(lc.if_not_closing(|| 1), Some(1));
}

#[test]
fn test_if_not_closing_returns_some_while_active() {
    let lc = AtomicLifecycle::new_active();
    assert_eq!(lc.if_not_closing(|| "ok"), Some("ok"));
}

#[test]
fn test_if_not_closing_returns_none_after_close_requested() {
    let lc = AtomicLifecycle::new_active();
    lc.request_close();
    assert!(lc.if_not_closing(|| 0).is_none());
}

#[test]
fn test_if_not_closing_returns_none_while_destroying() {
    let lc = AtomicLifecycle::new_active();
    lc.request_close();
    lc.begin_destroy();
    assert!(lc.if_not_closing(|| 0).is_none());
}

#[test]
fn test_if_not_closing_returns_none_after_destroyed() {
    let lc = AtomicLifecycle::new_active();
    lc.force_destroy();
    assert!(lc.if_not_closing(|| 0).is_none());
}

// ============================================================================
// AtomicLifecycle – concurrent transitions
// ============================================================================

#[test]
fn test_concurrent_activate_only_one_succeeds() {
    use std::thread;

    let lc = Arc::new(AtomicLifecycle::new());
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let lc = Arc::clone(&lc);
            thread::spawn(move || lc.activate())
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let successes = results.iter().filter(|r| r.is_success()).count();
    assert_eq!(successes, 1);
    assert_eq!(lc.state(), LifecycleState::Active);
}

#[test]
fn test_concurrent_request_close_only_one_succeeds() {
    use std::thread;

    let lc = Arc::new(AtomicLifecycle::new_active());
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let lc = Arc::clone(&lc);
            thread::spawn(move || lc.request_close())
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let successes = results.iter().filter(|r| r.is_success()).count();
    assert_eq!(successes, 1);
    assert_eq!(lc.state(), LifecycleState::CloseRequested);
}

#[test]
fn test_concurrent_force_destroy_is_safe() {
    use std::thread;

    let lc = Arc::new(AtomicLifecycle::new_active());
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let lc = Arc::clone(&lc);
            thread::spawn(move || lc.force_destroy())
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(lc.state(), LifecycleState::Destroyed);
}

// ============================================================================
// LifecycleEvent – Debug + Clone + PartialEq
// ============================================================================

#[test]
fn test_lifecycle_event_clone() {
    let ev = LifecycleEvent::Activated;
    let cloned = ev;
    assert_eq!(ev, cloned);
}

#[rstest]
#[case(LifecycleEvent::Activated)]
#[case(LifecycleEvent::CloseRequested)]
#[case(LifecycleEvent::DestroyStarted)]
#[case(LifecycleEvent::Destroyed)]
fn test_lifecycle_event_debug(#[case] ev: LifecycleEvent) {
    let s = format!("{:?}", ev);
    assert!(!s.is_empty());
}

// ============================================================================
// ObservableLifecycle
// ============================================================================

struct CountObserver {
    count: AtomicUsize,
    last_event: parking_lot::Mutex<Option<LifecycleEvent>>,
}

impl CountObserver {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            count: AtomicUsize::new(0),
            last_event: parking_lot::Mutex::new(None),
        })
    }

    fn event_count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

impl LifecycleObserver for CountObserver {
    fn on_lifecycle_event(&self, event: LifecycleEvent) {
        self.count.fetch_add(1, Ordering::SeqCst);
        *self.last_event.lock() = Some(event);
    }
}

#[test]
fn test_observable_lifecycle_default() {
    let lc = ObservableLifecycle::default();
    assert_eq!(lc.state(), LifecycleState::Creating);
    assert!(!lc.is_active());
    assert!(!lc.is_closing());
}

#[test]
fn test_observable_lifecycle_activate_notifies() {
    let lc = ObservableLifecycle::new();
    let obs = CountObserver::new();
    lc.add_observer(obs.clone());

    let result = lc.activate();
    assert!(result.is_success());
    assert_eq!(obs.event_count(), 1);
    assert_eq!(*obs.last_event.lock(), Some(LifecycleEvent::Activated));
}

#[test]
fn test_observable_lifecycle_request_close_notifies() {
    let lc = ObservableLifecycle::new();
    let obs = CountObserver::new();
    lc.add_observer(obs.clone());

    lc.activate();
    let result = lc.request_close();
    assert!(result.is_success());
    assert_eq!(obs.event_count(), 2);
    assert_eq!(*obs.last_event.lock(), Some(LifecycleEvent::CloseRequested));
}

#[test]
fn test_observable_lifecycle_begin_destroy_notifies() {
    let lc = ObservableLifecycle::new();
    let obs = CountObserver::new();
    lc.add_observer(obs.clone());

    lc.activate();
    lc.request_close();
    let result = lc.begin_destroy();
    assert!(result.is_success());
    assert_eq!(obs.event_count(), 3);
    assert_eq!(
        *obs.last_event.lock(),
        Some(LifecycleEvent::DestroyStarted)
    );
}

#[test]
fn test_observable_lifecycle_finish_destroy_notifies() {
    let lc = ObservableLifecycle::new();
    let obs = CountObserver::new();
    lc.add_observer(obs.clone());

    lc.activate();
    lc.request_close();
    lc.begin_destroy();
    let result = lc.finish_destroy();
    assert!(result.is_success());
    assert_eq!(obs.event_count(), 4);
    assert_eq!(*obs.last_event.lock(), Some(LifecycleEvent::Destroyed));
}

#[test]
fn test_observable_lifecycle_failed_transition_no_notify() {
    let lc = ObservableLifecycle::new();
    let obs = CountObserver::new();
    lc.add_observer(obs.clone());

    // activate first to get to Active
    lc.activate();
    // invalid: begin_destroy from Active
    let result = lc.begin_destroy();
    assert!(!result.is_success());
    // only the activate event was fired
    assert_eq!(obs.event_count(), 1);
}

#[test]
fn test_observable_lifecycle_multiple_observers() {
    let lc = ObservableLifecycle::new();
    let obs1 = CountObserver::new();
    let obs2 = CountObserver::new();
    lc.add_observer(obs1.clone());
    lc.add_observer(obs2.clone());

    lc.activate();
    lc.request_close();

    assert_eq!(obs1.event_count(), 2);
    assert_eq!(obs2.event_count(), 2);
}

#[test]
fn test_observable_lifecycle_is_closing_transitions() {
    let lc = ObservableLifecycle::new();
    lc.activate();
    assert!(!lc.is_closing());
    lc.request_close();
    assert!(lc.is_closing());
}

#[test]
fn test_observable_lifecycle_full_sequence_event_count() {
    let lc = ObservableLifecycle::new();
    let obs = CountObserver::new();
    lc.add_observer(obs.clone());

    lc.activate();
    lc.request_close();
    lc.begin_destroy();
    lc.finish_destroy();

    assert_eq!(obs.event_count(), 4);
}
