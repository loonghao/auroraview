//! Notification manager for displaying and tracking notifications.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use uuid::Uuid;

use crate::error::{NotificationError, Result};
use crate::notification::Notification;
use crate::permission::{Permission, PermissionState};

/// Callback type for notification events.
pub type NotificationCallback = Box<dyn Fn(&Notification) + Send + Sync>;

/// Callback type for action events.
pub type ActionCallback = Box<dyn Fn(&Notification, &str) + Send + Sync>;

/// Internal state for the notification manager.
struct NotificationState {
    /// Active notifications.
    active: HashMap<Uuid, Notification>,
    /// Notification history (dismissed notifications).
    history: Vec<Notification>,
    /// Permission state per origin.
    permissions: HashMap<String, Permission>,
    /// Maximum number of active notifications.
    max_active: usize,
    /// Maximum history size.
    max_history: usize,
    /// Show callback.
    on_show: Option<NotificationCallback>,
    /// Close callback.
    on_close: Option<NotificationCallback>,
    /// Action callback.
    on_action: Option<ActionCallback>,
}

/// Manager for notifications.
pub struct NotificationManager {
    inner: Arc<RwLock<NotificationState>>,
}

impl NotificationManager {
    /// Creates a new notification manager.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(NotificationState {
                active: HashMap::new(),
                history: Vec::new(),
                permissions: HashMap::new(),
                max_active: 5,
                max_history: 100,
                on_show: None,
                on_close: None,
                on_action: None,
            })),
        }
    }

    /// Sets the maximum number of active notifications.
    pub fn set_max_active(&self, max: usize) {
        self.inner.write().max_active = max;
    }

    /// Sets the maximum history size.
    pub fn set_max_history(&self, max: usize) {
        self.inner.write().max_history = max;
    }

    /// Registers a show callback.
    pub fn on_show<F>(&self, callback: F)
    where
        F: Fn(&Notification) + Send + Sync + 'static,
    {
        self.inner.write().on_show = Some(Box::new(callback));
    }

    /// Registers a close callback.
    pub fn on_close<F>(&self, callback: F)
    where
        F: Fn(&Notification) + Send + Sync + 'static,
    {
        self.inner.write().on_close = Some(Box::new(callback));
    }

    /// Registers an action callback.
    pub fn on_action<F>(&self, callback: F)
    where
        F: Fn(&Notification, &str) + Send + Sync + 'static,
    {
        self.inner.write().on_action = Some(Box::new(callback));
    }

    /// Gets the permission state for an origin.
    pub fn permission(&self, origin: &str) -> PermissionState {
        self.inner
            .read()
            .permissions
            .get(origin)
            .map(|p| p.state)
            .unwrap_or(PermissionState::Default)
    }

    /// Requests permission for an origin.
    pub fn request_permission(&self, origin: &str) -> PermissionState {
        let mut state = self.inner.write();

        if let Some(permission) = state.permissions.get(origin) {
            // Return existing permission state
            return permission.state;
        }

        // Create new permission (in a real app, this would show a UI prompt)
        // For now, auto-grant permission
        let permission = Permission::granted(origin);
        let result = permission.state;
        state.permissions.insert(origin.to_string(), permission);

        result
    }

    /// Sets permission for an origin.
    pub fn set_permission(&self, origin: &str, granted: bool) {
        let mut state = self.inner.write();
        let permission = if granted {
            Permission::granted(origin)
        } else {
            Permission::denied(origin)
        };
        state.permissions.insert(origin.to_string(), permission);
    }

    /// Shows a notification.
    pub fn notify(&self, notification: Notification) -> Result<Uuid> {
        self.notify_for_origin(notification, "default")
    }

    /// Shows a notification for a specific origin.
    pub fn notify_for_origin(&self, mut notification: Notification, origin: &str) -> Result<Uuid> {
        let mut state = self.inner.write();

        // Check permission
        let permission = state
            .permissions
            .get(origin)
            .map(|p| p.state)
            .unwrap_or(PermissionState::Granted); // Default to granted for standalone

        if permission == PermissionState::Denied {
            return Err(NotificationError::PermissionDenied);
        }

        // Check max active
        if state.active.len() >= state.max_active {
            // Remove oldest notification
            if let Some(oldest_id) = state
                .active
                .values()
                .min_by_key(|n| n.created_at)
                .map(|n| n.id)
            {
                if let Some(mut old) = state.active.remove(&oldest_id) {
                    old.mark_dismissed();
                    Self::add_to_history(&mut state, old);
                }
            }
        }

        // Handle tag-based replacement
        if let Some(tag) = &notification.tag {
            let existing_id = state
                .active
                .values()
                .find(|n| n.tag.as_ref() == Some(tag))
                .map(|n| n.id);

            if let Some(id) = existing_id {
                if let Some(mut old) = state.active.remove(&id) {
                    old.mark_dismissed();
                    Self::add_to_history(&mut state, old);
                }
            }
        }

        // Mark as shown
        notification.mark_shown();
        let id = notification.id;

        // Call show callback
        if let Some(ref callback) = state.on_show {
            callback(&notification);
        }

        // Add to active
        state.active.insert(id, notification);

        Ok(id)
    }

    /// Dismisses a notification.
    pub fn dismiss(&self, id: Uuid) -> Result<()> {
        let mut state = self.inner.write();

        let mut notification = state
            .active
            .remove(&id)
            .ok_or(NotificationError::NotFound(id))?;

        notification.mark_dismissed();

        // Call close callback
        if let Some(ref callback) = state.on_close {
            callback(&notification);
        }

        // Add to history
        Self::add_to_history(&mut state, notification);

        Ok(())
    }

    /// Dismisses all notifications.
    pub fn dismiss_all(&self) {
        let mut state = self.inner.write();

        let notifications: Vec<_> = state.active.drain().map(|(_, n)| n).collect();

        for mut notification in notifications {
            notification.mark_dismissed();

            if let Some(ref callback) = state.on_close {
                callback(&notification);
            }

            Self::add_to_history(&mut state, notification);
        }
    }

    /// Triggers an action on a notification.
    pub fn trigger_action(&self, id: Uuid, action_id: &str) -> Result<()> {
        let state = self.inner.read();

        let notification = state
            .active
            .get(&id)
            .ok_or(NotificationError::NotFound(id))?;

        // Verify action exists
        if !notification.actions.iter().any(|a| a.id == action_id) {
            return Err(NotificationError::InvalidNotification(format!(
                "Action '{}' not found",
                action_id
            )));
        }

        // Call action callback
        if let Some(ref callback) = state.on_action {
            callback(notification, action_id);
        }

        Ok(())
    }

    /// Gets a notification by ID.
    pub fn get(&self, id: Uuid) -> Option<Notification> {
        self.inner.read().active.get(&id).cloned()
    }

    /// Returns all active notifications.
    pub fn active(&self) -> Vec<Notification> {
        self.inner.read().active.values().cloned().collect()
    }

    /// Returns the notification history.
    pub fn history(&self) -> Vec<Notification> {
        self.inner.read().history.clone()
    }

    /// Clears the notification history.
    pub fn clear_history(&self) {
        self.inner.write().history.clear();
    }

    /// Returns the number of active notifications.
    pub fn active_count(&self) -> usize {
        self.inner.read().active.len()
    }

    /// Helper to add a notification to history.
    fn add_to_history(state: &mut NotificationState, notification: Notification) {
        state.history.push(notification);

        // Trim history if needed
        if state.history.len() > state.max_history {
            let excess = state.history.len() - state.max_history;
            state.history.drain(0..excess);
        }
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for NotificationManager {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
