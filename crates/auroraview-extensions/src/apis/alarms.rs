//! chrome.alarms API handler
//!
//! Provides scheduled task functionality for extensions.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::apis::ApiHandler;
use crate::error::{ExtensionError, ExtensionResult};
use crate::ExtensionId;

/// Alarm creation info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmCreateInfo {
    /// Time when alarm should fire (ms since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<f64>,
    /// Delay in minutes before alarm fires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_in_minutes: Option<f64>,
    /// Period in minutes for repeating alarm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_in_minutes: Option<f64>,
}

/// Alarm info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alarm {
    /// Alarm name
    pub name: String,
    /// Scheduled time (ms since epoch)
    pub scheduled_time: f64,
    /// Period in minutes (for repeating alarms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_in_minutes: Option<f64>,
}

/// Stored alarm with metadata
#[derive(Debug, Clone)]
struct StoredAlarm {
    /// Alarm info
    alarm: Alarm,
    /// Extension ID
    extension_id: ExtensionId,
}

/// Alarms manager
pub struct AlarmsManager {
    /// Active alarms
    alarms: RwLock<HashMap<(ExtensionId, String), StoredAlarm>>,
}

impl AlarmsManager {
    /// Create a new alarms manager
    pub fn new() -> Self {
        Self {
            alarms: RwLock::new(HashMap::new()),
        }
    }

    /// Create an alarm
    pub fn create(
        &self,
        extension_id: &str,
        name: &str,
        info: AlarmCreateInfo,
    ) -> ExtensionResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0;

        let scheduled_time = if let Some(when) = info.when {
            when
        } else if let Some(delay) = info.delay_in_minutes {
            now + (delay * 60.0 * 1000.0)
        } else {
            // Default to 1 minute from now
            now + 60000.0
        };

        let alarm = Alarm {
            name: name.to_string(),
            scheduled_time,
            period_in_minutes: info.period_in_minutes,
        };

        let stored = StoredAlarm {
            alarm,
            extension_id: extension_id.to_string(),
        };

        let key = (extension_id.to_string(), name.to_string());
        self.alarms.write().insert(key, stored);

        // TODO: Actually schedule the alarm using a timer
        // For now, we just store it

        Ok(())
    }

    /// Get an alarm
    pub fn get(&self, extension_id: &str, name: &str) -> Option<Alarm> {
        let key = (extension_id.to_string(), name.to_string());
        self.alarms.read().get(&key).map(|s| s.alarm.clone())
    }

    /// Get all alarms for an extension
    pub fn get_all(&self, extension_id: &str) -> Vec<Alarm> {
        self.alarms
            .read()
            .iter()
            .filter(|(_, s)| s.extension_id == extension_id)
            .map(|(_, s)| s.alarm.clone())
            .collect()
    }

    /// Clear an alarm
    pub fn clear(&self, extension_id: &str, name: &str) -> bool {
        let key = (extension_id.to_string(), name.to_string());
        self.alarms.write().remove(&key).is_some()
    }

    /// Clear all alarms for an extension
    pub fn clear_all(&self, extension_id: &str) {
        self.alarms
            .write()
            .retain(|(ext_id, _), _| ext_id != extension_id);
    }
}

impl Default for AlarmsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Alarms API handler
pub struct AlarmsApiHandler {
    manager: Arc<AlarmsManager>,
}

impl AlarmsApiHandler {
    /// Create a new alarms API handler
    pub fn new(manager: Arc<AlarmsManager>) -> Self {
        Self { manager }
    }
}

impl ApiHandler for AlarmsApiHandler {
    fn namespace(&self) -> &str {
        "alarms"
    }

    fn handle(&self, method: &str, params: Value, extension_id: &str) -> ExtensionResult<Value> {
        match method {
            "create" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

                let info: AlarmCreateInfo = params
                    .get("alarmInfo")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or(AlarmCreateInfo {
                        when: None,
                        delay_in_minutes: Some(1.0),
                        period_in_minutes: None,
                    });

                self.manager.create(extension_id, name, info)?;
                Ok(serde_json::json!({}))
            }
            "get" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

                let alarm = self.manager.get(extension_id, name);
                Ok(serde_json::to_value(alarm)?)
            }
            "getAll" => {
                let alarms = self.manager.get_all(extension_id);
                Ok(serde_json::to_value(alarms)?)
            }
            "clear" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

                let was_cleared = self.manager.clear(extension_id, name);
                Ok(serde_json::json!(was_cleared))
            }
            "clearAll" => {
                self.manager.clear_all(extension_id);
                Ok(serde_json::json!(true))
            }
            _ => Err(ExtensionError::ApiNotSupported(format!(
                "alarms.{} is not supported",
                method
            ))),
        }
    }

    fn methods(&self) -> Vec<&str> {
        vec!["create", "get", "getAll", "clear", "clearAll"]
    }
}
