//! WebView Lifecycle State Machine
//!
//! Provides a lock-free state machine for WebView lifecycle management.
//! This addresses the P0 finding about lock ordering violations in EventLoopState.
//!
//! ## State Transitions
//!
//! ```text
//! ┌──────────┐     create()     ┌──────────┐
//! │ Creating │ ───────────────► │  Active  │
//! └──────────┘                  └──────────┘
//!                                    │
//!                               request_close()
//!                                    │
//!                                    ▼
//!                              ┌──────────────┐
//!                              │CloseRequested│
//!                              └──────────────┘
//!                                    │
//!                               begin_destroy()
//!                                    │
//!                                    ▼
//!                              ┌──────────────┐
//!                              │  Destroying  │
//!                              └──────────────┘
//!                                    │
//!                               finish_destroy()
//!                                    │
//!                                    ▼
//!                              ┌──────────────┐
//!                              │  Destroyed   │
//!                              └──────────────┘
//! ```

use std::sync::atomic::{AtomicU8, Ordering};

/// WebView lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LifecycleState {
    /// WebView is being created
    Creating = 0,
    /// WebView is active and ready
    Active = 1,
    /// Close has been requested
    CloseRequested = 2,
    /// WebView is being destroyed
    Destroying = 3,
    /// WebView has been destroyed
    Destroyed = 4,
}

impl From<u8> for LifecycleState {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Creating,
            1 => Self::Active,
            2 => Self::CloseRequested,
            3 => Self::Destroying,
            4 => Self::Destroyed,
            _ => Self::Destroyed, // Invalid states default to Destroyed
        }
    }
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creating => write!(f, "Creating"),
            Self::Active => write!(f, "Active"),
            Self::CloseRequested => write!(f, "CloseRequested"),
            Self::Destroying => write!(f, "Destroying"),
            Self::Destroyed => write!(f, "Destroyed"),
        }
    }
}

/// Result of a state transition attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionResult {
    /// Transition succeeded
    Success,
    /// Transition failed - invalid state
    InvalidState,
    /// Transition failed - already in target state
    AlreadyInState,
}

impl TransitionResult {
    /// Check if transition succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
}

/// Lock-free lifecycle state machine
///
/// Uses atomic operations to manage state transitions without locks,
/// eliminating the risk of deadlocks identified in P0.
pub struct AtomicLifecycle {
    state: AtomicU8,
}

impl Default for AtomicLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicLifecycle {
    /// Create a new lifecycle in Creating state
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(LifecycleState::Creating as u8),
        }
    }

    /// Create a new lifecycle in Active state
    pub fn new_active() -> Self {
        Self {
            state: AtomicU8::new(LifecycleState::Active as u8),
        }
    }

    /// Get current state
    pub fn state(&self) -> LifecycleState {
        self.state.load(Ordering::Acquire).into()
    }

    /// Check if WebView is active
    pub fn is_active(&self) -> bool {
        self.state() == LifecycleState::Active
    }

    /// Check if close has been requested or WebView is closing/closed
    pub fn is_closing(&self) -> bool {
        matches!(
            self.state(),
            LifecycleState::CloseRequested | LifecycleState::Destroying | LifecycleState::Destroyed
        )
    }

    /// Check if WebView is destroyed
    pub fn is_destroyed(&self) -> bool {
        self.state() == LifecycleState::Destroyed
    }

    /// Transition from Creating to Active
    pub fn activate(&self) -> TransitionResult {
        self.try_transition(LifecycleState::Creating, LifecycleState::Active)
    }

    /// Request close (from Active to CloseRequested)
    pub fn request_close(&self) -> TransitionResult {
        self.try_transition(LifecycleState::Active, LifecycleState::CloseRequested)
    }

    /// Begin destruction (from CloseRequested to Destroying)
    pub fn begin_destroy(&self) -> TransitionResult {
        self.try_transition(LifecycleState::CloseRequested, LifecycleState::Destroying)
    }

    /// Finish destruction (from Destroying to Destroyed)
    pub fn finish_destroy(&self) -> TransitionResult {
        self.try_transition(LifecycleState::Destroying, LifecycleState::Destroyed)
    }

    /// Force transition to Destroyed state (emergency cleanup)
    pub fn force_destroy(&self) {
        self.state
            .store(LifecycleState::Destroyed as u8, Ordering::Release);
    }

    /// Try to transition from one state to another
    fn try_transition(&self, from: LifecycleState, to: LifecycleState) -> TransitionResult {
        match self
            .state
            .compare_exchange(from as u8, to as u8, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => TransitionResult::Success,
            Err(actual) => {
                if actual == to as u8 {
                    TransitionResult::AlreadyInState
                } else {
                    TransitionResult::InvalidState
                }
            }
        }
    }

    /// Execute a function only if in Active state
    ///
    /// This is useful for operations that should only run when the WebView is active.
    pub fn if_active<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce() -> R,
    {
        if self.is_active() {
            Some(f())
        } else {
            None
        }
    }

    /// Execute a function only if not closing/destroyed
    pub fn if_not_closing<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce() -> R,
    {
        if !self.is_closing() {
            Some(f())
        } else {
            None
        }
    }
}

/// Lifecycle event for callbacks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// WebView became active
    Activated,
    /// Close was requested
    CloseRequested,
    /// Destruction began
    DestroyStarted,
    /// Destruction completed
    Destroyed,
}

/// Lifecycle observer trait
pub trait LifecycleObserver: Send + Sync {
    /// Called when a lifecycle event occurs
    fn on_lifecycle_event(&self, event: LifecycleEvent);
}

/// Observable lifecycle that notifies observers of state changes
pub struct ObservableLifecycle {
    inner: AtomicLifecycle,
    observers: std::sync::RwLock<Vec<std::sync::Arc<dyn LifecycleObserver>>>,
}

impl Default for ObservableLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl ObservableLifecycle {
    /// Create a new observable lifecycle
    pub fn new() -> Self {
        Self {
            inner: AtomicLifecycle::new(),
            observers: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// Get current state
    pub fn state(&self) -> LifecycleState {
        self.inner.state()
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    /// Check if closing
    pub fn is_closing(&self) -> bool {
        self.inner.is_closing()
    }

    /// Add an observer
    pub fn add_observer(&self, observer: std::sync::Arc<dyn LifecycleObserver>) {
        if let Ok(mut observers) = self.observers.write() {
            observers.push(observer);
        }
    }

    /// Activate and notify observers
    pub fn activate(&self) -> TransitionResult {
        let result = self.inner.activate();
        if result.is_success() {
            self.notify(LifecycleEvent::Activated);
        }
        result
    }

    /// Request close and notify observers
    pub fn request_close(&self) -> TransitionResult {
        let result = self.inner.request_close();
        if result.is_success() {
            self.notify(LifecycleEvent::CloseRequested);
        }
        result
    }

    /// Begin destroy and notify observers
    pub fn begin_destroy(&self) -> TransitionResult {
        let result = self.inner.begin_destroy();
        if result.is_success() {
            self.notify(LifecycleEvent::DestroyStarted);
        }
        result
    }

    /// Finish destroy and notify observers
    pub fn finish_destroy(&self) -> TransitionResult {
        let result = self.inner.finish_destroy();
        if result.is_success() {
            self.notify(LifecycleEvent::Destroyed);
        }
        result
    }

    fn notify(&self, event: LifecycleEvent) {
        if let Ok(observers) = self.observers.read() {
            for observer in observers.iter() {
                observer.on_lifecycle_event(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;

    #[test]
    fn test_lifecycle_states() {
        let lifecycle = AtomicLifecycle::new();
        assert_eq!(lifecycle.state(), LifecycleState::Creating);
        assert!(!lifecycle.is_active());
        assert!(!lifecycle.is_closing());

        // Activate
        assert!(lifecycle.activate().is_success());
        assert_eq!(lifecycle.state(), LifecycleState::Active);
        assert!(lifecycle.is_active());

        // Request close
        assert!(lifecycle.request_close().is_success());
        assert_eq!(lifecycle.state(), LifecycleState::CloseRequested);
        assert!(lifecycle.is_closing());

        // Begin destroy
        assert!(lifecycle.begin_destroy().is_success());
        assert_eq!(lifecycle.state(), LifecycleState::Destroying);

        // Finish destroy
        assert!(lifecycle.finish_destroy().is_success());
        assert_eq!(lifecycle.state(), LifecycleState::Destroyed);
        assert!(lifecycle.is_destroyed());
    }

    #[test]
    fn test_invalid_transitions() {
        let lifecycle = AtomicLifecycle::new();

        // Can't request close from Creating
        assert_eq!(lifecycle.request_close(), TransitionResult::InvalidState);

        // Can't begin destroy from Creating
        assert_eq!(lifecycle.begin_destroy(), TransitionResult::InvalidState);

        // Activate first
        lifecycle.activate();

        // Can't activate again (already in Active state)
        assert_eq!(lifecycle.activate(), TransitionResult::AlreadyInState);

        // Can't begin destroy without requesting close
        assert_eq!(lifecycle.begin_destroy(), TransitionResult::InvalidState);
    }

    #[test]
    fn test_if_active() {
        let lifecycle = AtomicLifecycle::new();

        // Not active yet
        assert!(lifecycle.if_active(|| 42).is_none());

        lifecycle.activate();

        // Now active
        assert_eq!(lifecycle.if_active(|| 42), Some(42));

        lifecycle.request_close();

        // No longer active
        assert!(lifecycle.if_active(|| 42).is_none());
    }

    #[test]
    fn test_observable_lifecycle() {
        struct TestObserver {
            event_count: AtomicUsize,
        }

        impl LifecycleObserver for TestObserver {
            fn on_lifecycle_event(&self, _event: LifecycleEvent) {
                self.event_count.fetch_add(1, Ordering::SeqCst);
            }
        }

        let lifecycle = ObservableLifecycle::new();
        let observer = Arc::new(TestObserver {
            event_count: AtomicUsize::new(0),
        });

        lifecycle.add_observer(observer.clone());

        lifecycle.activate();
        assert_eq!(observer.event_count.load(Ordering::SeqCst), 1);

        lifecycle.request_close();
        assert_eq!(observer.event_count.load(Ordering::SeqCst), 2);

        lifecycle.begin_destroy();
        lifecycle.finish_destroy();
        assert_eq!(observer.event_count.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_concurrent_transitions() {
        use std::thread;

        let lifecycle = Arc::new(AtomicLifecycle::new_active());
        let mut handles = vec![];

        // Spawn multiple threads trying to request close
        for _ in 0..10 {
            let lc = Arc::clone(&lifecycle);
            handles.push(thread::spawn(move || lc.request_close()));
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Only one should succeed
        let successes = results
            .iter()
            .filter(|r| **r == TransitionResult::Success)
            .count();
        assert_eq!(successes, 1);

        // Final state should be CloseRequested
        assert_eq!(lifecycle.state(), LifecycleState::CloseRequested);
    }
}
