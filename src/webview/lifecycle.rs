//! Cross-platform window lifecycle management
//!
//! This module provides a unified interface for managing window lifecycle
//! across different platforms (Windows, macOS, Linux) and modes (standalone, embedded).

use flume::{Receiver, Sender};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{debug, info, trace, warn};

/// Type alias for cleanup handlers to reduce complexity
type CleanupHandlers = Vec<Box<dyn FnOnce() + Send + 'static>>;

/// Window lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// Window is being created
    Creating,
    /// Window is active and running
    Active,
    /// Close has been requested but not yet processed
    #[allow(dead_code)]
    CloseRequested,
    /// Window is being destroyed
    Destroying,
    /// Window has been destroyed
    Destroyed,
}

/// Cross-platform window lifecycle manager
pub struct LifecycleManager {
    /// Current lifecycle state
    state: Arc<Mutex<LifecycleState>>,
    /// Close signal channel (sender)
    close_tx: Sender<CloseReason>,
    /// Close signal channel (receiver)
    close_rx: Receiver<CloseReason>,
    /// Platform-specific cleanup handlers
    cleanup_handlers: Arc<Mutex<CleanupHandlers>>,
}

/// Reason for window closure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CloseReason {
    /// User clicked the close button
    UserRequest,
    /// Application requested close
    AppRequest,
    /// Parent window closed (embedded mode)
    ParentClosed,
    /// System shutdown
    SystemShutdown,
    /// Error occurred
    Error,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        let (close_tx, close_rx) = flume::bounded(1);

        Self {
            state: Arc::new(Mutex::new(LifecycleState::Creating)),
            close_tx,
            close_rx,
            cleanup_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the current lifecycle state
    pub fn state(&self) -> LifecycleState {
        *self.state.lock()
    }

    /// Set the lifecycle state
    pub fn set_state(&self, new_state: LifecycleState) {
        let mut state = self.state.lock();
        let old_state = *state;
        *state = new_state;

        info!(
            "[LifecycleManager] State transition: {:?} -> {:?}",
            old_state, new_state
        );
    }

    /// Request window close
    #[allow(dead_code)]
    pub fn request_close(&self, reason: CloseReason) -> Result<(), String> {
        let current_state = self.state();

        match current_state {
            LifecycleState::Destroyed => {
                warn!("[LifecycleManager] Close requested on already destroyed window");
                return Err("Window already destroyed".to_string());
            }
            LifecycleState::Destroying => {
                debug!("[LifecycleManager] Close already in progress");
                return Ok(());
            }
            _ => {}
        }

        self.set_state(LifecycleState::CloseRequested);

        self.close_tx
            .send(reason)
            .map_err(|e| format!("Failed to send close signal: {}", e))?;

        info!("[LifecycleManager] Close requested: {:?}", reason);
        Ok(())
    }

    /// Check if close has been requested (non-blocking)
    pub fn check_close_requested(&self) -> Option<CloseReason> {
        self.close_rx.try_recv().ok()
    }

    /// Wait for close request (blocking with timeout)
    #[allow(dead_code)]
    pub fn wait_for_close(&self, timeout: std::time::Duration) -> Option<CloseReason> {
        self.close_rx.recv_timeout(timeout).ok()
    }

    /// Register a cleanup handler
    #[allow(dead_code)]
    pub fn register_cleanup<F>(&self, handler: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.cleanup_handlers.lock().push(Box::new(handler));
        trace!("[LifecycleManager] Cleanup handler registered");
    }

    /// Execute all cleanup handlers
    pub fn execute_cleanup(&self) {
        self.set_state(LifecycleState::Destroying);

        let handlers = {
            let mut handlers_lock = self.cleanup_handlers.lock();
            std::mem::take(&mut *handlers_lock)
        };

        info!(
            "[LifecycleManager] Executing {} cleanup handlers",
            handlers.len()
        );

        for (idx, handler) in handlers.into_iter().enumerate() {
            trace!("[LifecycleManager] Executing cleanup handler {}", idx);
            handler();
        }

        self.set_state(LifecycleState::Destroyed);
        info!("[LifecycleManager] All cleanup handlers executed");
    }

    /// Clone the close sender for sharing across threads
    #[allow(dead_code)]
    pub fn close_sender(&self) -> Sender<CloseReason> {
        self.close_tx.clone()
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_lifecycle_cleanup_executes_handlers_and_transitions_state() {
        let manager = LifecycleManager::new();
        assert_eq!(manager.state(), LifecycleState::Creating);

        manager.set_state(LifecycleState::Active);
        assert_eq!(manager.state(), LifecycleState::Active);

        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();

        manager.register_cleanup(move || {
            c1.fetch_add(1, Ordering::SeqCst);
        });
        manager.register_cleanup(move || {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        manager.execute_cleanup();

        assert_eq!(counter.load(Ordering::SeqCst), 2);
        assert_eq!(manager.state(), LifecycleState::Destroyed);
    }

    #[test]
    fn test_wait_for_close_and_close_sender() {
        let manager = LifecycleManager::new();
        let tx = manager.close_sender();

        // Send a close reason from another thread shortly after
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            let _ = tx.send(CloseReason::UserRequest);
        });

        let reason = manager.wait_for_close(Duration::from_secs(1));
        assert_eq!(reason, Some(CloseReason::UserRequest));
    }
}

#[test]
fn test_request_close_sets_state_and_sends_reason() {
    let manager = LifecycleManager::new();
    manager.set_state(LifecycleState::Active);
    assert!(manager.request_close(CloseReason::AppRequest).is_ok());
    assert_eq!(manager.state(), LifecycleState::CloseRequested);
    assert_eq!(
        manager.check_close_requested(),
        Some(CloseReason::AppRequest)
    );
}

#[test]
fn test_request_close_on_destroyed_errors() {
    let manager = LifecycleManager::new();
    manager.set_state(LifecycleState::Destroyed);
    let err = manager.request_close(CloseReason::UserRequest).unwrap_err();
    assert!(err.contains("already destroyed"));
}

#[test]
fn test_request_close_when_destroying_is_noop_ok() {
    let manager = LifecycleManager::new();
    manager.set_state(LifecycleState::Destroying);
    // Should be Ok and not enqueue a reason
    assert!(manager.request_close(CloseReason::SystemShutdown).is_ok());
    assert_eq!(manager.check_close_requested(), None);
    // state should remain Destroying
    assert_eq!(manager.state(), LifecycleState::Destroying);
}
