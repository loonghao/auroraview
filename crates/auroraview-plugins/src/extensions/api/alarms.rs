use serde_json::Value;

use crate::extensions::AlarmInfo;
use crate::extensions::ExtensionsPlugin;
use auroraview_plugin_core::{PluginError, PluginResult};

impl ExtensionsPlugin {
    /// Handle alarms API calls
    pub fn handle_alarms_api(
        &self,
        extension_id: &str,
        method: &str,
        params: &Value,
    ) -> PluginResult<Value> {
        match method {
            "create" => {
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let alarm_info = params.get("alarmInfo").cloned().unwrap_or(params.clone());

                let delay_in_minutes = alarm_info
                    .get("delayInMinutes")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let period_in_minutes = alarm_info.get("periodInMinutes").and_then(|v| v.as_f64());
                let when = alarm_info.get("when").and_then(|v| v.as_f64());

                let scheduled_time = when.unwrap_or_else(|| {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as f64;
                    now + (delay_in_minutes * 60.0 * 1000.0)
                });

                let alarm = AlarmInfo {
                    name: name.clone(),
                    scheduled_time,
                    period_in_minutes,
                };

                let mut state = self.state.write();
                let ext_alarms = state.alarms.entry(extension_id.to_string()).or_default();

                ext_alarms.insert(name, alarm);

                Ok(serde_json::json!({}))
            }
            "get" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let state = self.state.read();
                let alarm = state.alarms.get(extension_id).and_then(|a| a.get(name));
                serde_json::to_value(alarm).map_err(PluginError::serialization_error)
            }
            "getAll" => {
                let state = self.state.read();
                let alarms: Vec<AlarmInfo> = state
                    .alarms
                    .get(extension_id)
                    .map(|a| a.values().cloned().collect())
                    .unwrap_or_default();
                serde_json::to_value(alarms).map_err(PluginError::serialization_error)
            }
            "clear" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let mut state = self.state.write();
                let cleared = if name.is_empty() {
                    // Clear all alarms
                    state
                        .alarms
                        .get_mut(extension_id)
                        .map(|a| {
                            let had_alarms = !a.is_empty();
                            a.clear();
                            had_alarms
                        })
                        .unwrap_or(false)
                } else {
                    // Clear only the alarm with given name
                    state
                        .alarms
                        .get_mut(extension_id)
                        .and_then(|a| a.remove(name))
                        .is_some()
                };
                Ok(serde_json::json!(cleared))
            }
            "clearAll" => {
                let mut state = self.state.write();
                let cleared = state
                    .alarms
                    .get_mut(extension_id)
                    .map(|a| {
                        let had_alarms = !a.is_empty();
                        a.clear();
                        had_alarms
                    })
                    .unwrap_or(false);
                Ok(serde_json::json!(cleared))
            }
            _ => Err(PluginError::command_not_found(&format!(
                "alarms.{}",
                method
            ))),
        }
    }
}
