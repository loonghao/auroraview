//! Chrome Sessions API Implementation
//!
//! Provides session management functionality for extensions.
//!
//! ## Features
//! - Query recently closed tabs and windows
//! - Restore closed tabs and windows
//! - Get devices with synced sessions

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Maximum number of sessions to store
const MAX_SESSION_RESULTS: usize = 25;

/// Session object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// Time when session was last modified (ms since epoch)
    pub last_modified: i64,
    /// Tab info (if this session is a tab)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab: Option<SessionTab>,
    /// Window info (if this session is a window)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<SessionWindow>,
}

/// Session tab
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTab {
    /// Tab ID (for restoration)
    pub session_id: String,
    /// Window ID
    pub window_id: i32,
    /// Tab index
    pub index: i32,
    /// URL
    pub url: String,
    /// Title
    pub title: String,
    /// Favicon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fav_icon_url: Option<String>,
}

/// Session window
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionWindow {
    /// Window ID (for restoration)
    pub session_id: String,
    /// Tabs in this window
    pub tabs: Vec<SessionTab>,
}

/// Device with sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    /// Device name
    pub device_name: String,
    /// Sessions on this device
    pub sessions: Vec<Session>,
}

/// Filter for session queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    /// Maximum results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<usize>,
}

/// Sessions API handler
pub struct SessionsApi {
    /// Recently closed sessions
    sessions: Arc<RwLock<VecDeque<Session>>>,
    /// Next session ID
    #[allow(dead_code)]
    next_id: Arc<RwLock<u64>>,
}

impl Default for SessionsApi {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionsApi {
    /// Create a new SessionsApi instance
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(VecDeque::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Generate next session ID
    #[allow(dead_code)]
    fn next_id(&self) -> String {
        let mut id = self.next_id.write().unwrap();
        let current = *id;
        *id += 1;
        format!("session_{}", current)
    }

    /// Get recently closed sessions
    pub fn get_recently_closed(&self, filter: Filter) -> ExtensionResult<Value> {
        let sessions = self.sessions.read().unwrap();
        let max_results = filter.max_results.unwrap_or(MAX_SESSION_RESULTS);

        let results: Vec<&Session> = sessions.iter().take(max_results).collect();
        Ok(serde_json::to_value(results)?)
    }

    /// Get devices with synced sessions
    pub fn get_devices(&self, _filter: Filter) -> ExtensionResult<Value> {
        // In a real implementation, this would return synced sessions from other devices
        // For now, return empty list
        let devices: Vec<Device> = vec![];
        Ok(serde_json::to_value(devices)?)
    }

    /// Restore a session
    pub fn restore(&self, session_id: Option<String>) -> ExtensionResult<Value> {
        let mut sessions = self.sessions.write().unwrap();

        if let Some(id) = session_id {
            // Find and remove the session
            let idx = sessions.iter().position(|s| {
                s.tab.as_ref().map(|t| t.session_id == id).unwrap_or(false)
                    || s.window
                        .as_ref()
                        .map(|w| w.session_id == id)
                        .unwrap_or(false)
            });

            if let Some(idx) = idx {
                let session = sessions.remove(idx).unwrap();
                return Ok(serde_json::to_value(session)?);
            }

            return Err(ExtensionError::NotFound(format!(
                "Session {} not found",
                id
            )));
        }

        // Restore most recent session
        if let Some(session) = sessions.pop_front() {
            Ok(serde_json::to_value(session)?)
        } else {
            Err(ExtensionError::NotFound("No sessions to restore".into()))
        }
    }

    /// Record a closed tab (internal use)
    pub fn record_closed_tab(&self, tab: SessionTab) {
        let mut sessions = self.sessions.write().unwrap();

        let session = Session {
            last_modified: now_ms(),
            tab: Some(tab),
            window: None,
        };

        sessions.push_front(session);

        // Keep only MAX_SESSION_RESULTS
        while sessions.len() > MAX_SESSION_RESULTS {
            sessions.pop_back();
        }
    }

    /// Record a closed window (internal use)
    pub fn record_closed_window(&self, window: SessionWindow) {
        let mut sessions = self.sessions.write().unwrap();

        let session = Session {
            last_modified: now_ms(),
            tab: None,
            window: Some(window),
        };

        sessions.push_front(session);

        while sessions.len() > MAX_SESSION_RESULTS {
            sessions.pop_back();
        }
    }

    /// Handle API call
    pub fn handle(&self, method: &str, params: Value) -> ExtensionResult<Value> {
        match method {
            "getRecentlyClosed" => {
                let filter: Filter = serde_json::from_value(params).unwrap_or_default();
                self.get_recently_closed(filter)
            }
            "getDevices" => {
                let filter: Filter = serde_json::from_value(params).unwrap_or_default();
                self.get_devices(filter)
            }
            "restore" => {
                let session_id = params
                    .get("sessionId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                self.restore(session_id)
            }
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

/// Get current time in milliseconds
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_get_sessions() {
        let api = SessionsApi::new();

        api.record_closed_tab(SessionTab {
            session_id: "tab_1".to_string(),
            window_id: 1,
            index: 0,
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            fav_icon_url: None,
        });

        let result = api.get_recently_closed(Filter::default()).unwrap();
        let sessions: Vec<Session> = serde_json::from_value(result).unwrap();
        assert_eq!(sessions.len(), 1);
    }

    #[test]
    fn test_restore_session() {
        let api = SessionsApi::new();

        api.record_closed_tab(SessionTab {
            session_id: "tab_1".to_string(),
            window_id: 1,
            index: 0,
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            fav_icon_url: None,
        });

        let result = api.restore(Some("tab_1".to_string())).unwrap();
        let session: Session = serde_json::from_value(result).unwrap();
        assert!(session.tab.is_some());
    }
}
