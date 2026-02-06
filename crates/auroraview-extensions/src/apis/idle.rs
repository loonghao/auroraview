//! Chrome Idle API Implementation
//!
//! Provides functionality to detect when the machine's idle state changes.
//!
//! ## Features
//! - Query idle state
//! - Set detection interval
//! - Event notifications for state changes

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Idle state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum IdleState {
    #[default]
    Active,
    Idle,
    Locked,
}

/// Idle API state
#[derive(Debug, Clone)]
struct IdleApiState {
    /// Current idle state
    state: IdleState,
    /// Detection interval in seconds
    detection_interval: u32,
}

impl Default for IdleApiState {
    fn default() -> Self {
        Self {
            state: IdleState::Active,
            detection_interval: 60,
        }
    }
}

/// Idle API handler
pub struct IdleApi {
    /// Internal state
    state: Arc<RwLock<IdleApiState>>,
}

impl Default for IdleApi {
    fn default() -> Self {
        Self::new()
    }
}

impl IdleApi {
    /// Create a new IdleApi instance
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(IdleApiState::default())),
        }
    }

    /// Query idle state
    pub fn query_state(&self, detection_interval_in_seconds: u32) -> ExtensionResult<Value> {
        let state = self.state.read().unwrap();

        // In a real implementation, this would check actual system idle time
        // For now, we return the current state
        let idle_state = if detection_interval_in_seconds > 0 {
            &state.state
        } else {
            &IdleState::Active
        };

        Ok(serde_json::to_value(idle_state)?)
    }

    /// Set detection interval
    pub fn set_detection_interval(&self, interval_in_seconds: u32) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        state.detection_interval = interval_in_seconds;
        Ok(json!(null))
    }

    /// Get auto-lock delay (ChromeOS only)
    pub fn get_auto_lock_delay(&self) -> ExtensionResult<Value> {
        // Return 0 to indicate no auto-lock (not ChromeOS)
        Ok(json!(0))
    }

    /// Set state for testing
    #[cfg(test)]
    pub fn set_state(&self, new_state: IdleState) {
        let mut state = self.state.write().unwrap();
        state.state = new_state;
    }

    /// Handle API call
    pub fn handle(&self, method: &str, params: Value) -> ExtensionResult<Value> {
        match method {
            "queryState" => {
                let interval = params
                    .get("detectionIntervalInSeconds")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(60) as u32;
                self.query_state(interval)
            }
            "setDetectionInterval" => {
                let interval = params
                    .get("intervalInSeconds")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        ExtensionError::InvalidParams("Missing intervalInSeconds".into())
                    })? as u32;
                self.set_detection_interval(interval)
            }
            "getAutoLockDelay" => self.get_auto_lock_delay(),
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_state() {
        let api = IdleApi::new();
        let result = api.query_state(60).unwrap();
        let state: IdleState = serde_json::from_value(result).unwrap();
        assert_eq!(state, IdleState::Active);
    }

    #[test]
    fn test_set_detection_interval() {
        let api = IdleApi::new();
        let result = api.set_detection_interval(120);
        assert!(result.is_ok());
    }

    #[test]
    fn test_idle_state() {
        let api = IdleApi::new();
        api.set_state(IdleState::Idle);
        let result = api.query_state(60).unwrap();
        let state: IdleState = serde_json::from_value(result).unwrap();
        assert_eq!(state, IdleState::Idle);
    }
}
