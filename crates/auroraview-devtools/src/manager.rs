//! DevTools manager implementation

use crate::{ConsoleMessage, DevToolsConfig, DockSide, NetworkRequestInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DevTools state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevToolsState {
    /// Whether DevTools is currently open
    pub is_open: bool,
    /// Current dock side
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dock_side: Option<DockSide>,
    /// Selected panel (elements, console, network, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_panel: Option<String>,
}

/// DevTools manager
///
/// Manages DevTools state, console messages, and network requests.
pub struct DevToolsManager {
    /// Configuration
    config: DevToolsConfig,
    /// Current state
    state: DevToolsState,
    /// Console messages
    console_messages: Vec<ConsoleMessage>,
    /// Network requests (request_id -> info)
    network_requests: HashMap<String, NetworkRequestInfo>,
    /// Maximum console messages to keep
    max_console_messages: usize,
}

impl DevToolsManager {
    /// Default maximum console messages
    const DEFAULT_MAX_CONSOLE_MESSAGES: usize = 1000;

    /// Create a new DevTools manager
    pub fn new(config: DevToolsConfig) -> Self {
        let initial_dock_side = config.dock_side;
        Self {
            config,
            state: DevToolsState {
                is_open: false,
                dock_side: Some(initial_dock_side),
                selected_panel: None,
            },
            console_messages: Vec::new(),
            network_requests: HashMap::new(),
            max_console_messages: Self::DEFAULT_MAX_CONSOLE_MESSAGES,
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(DevToolsConfig::default())
    }

    /// Set maximum console messages
    pub fn with_max_console_messages(mut self, max: usize) -> Self {
        self.max_console_messages = max;
        self
    }

    // ========== State ==========

    /// Check if DevTools is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Check if DevTools is currently open
    pub fn is_open(&self) -> bool {
        self.state.is_open
    }

    /// Get remote debugging port
    pub fn remote_debugging_port(&self) -> u16 {
        self.config.remote_debugging_port
    }

    /// Check if remote debugging is enabled
    pub fn is_remote_debugging_enabled(&self) -> bool {
        self.config.is_remote_debugging_enabled()
    }

    /// Open DevTools
    pub fn open(&mut self) {
        self.state.is_open = true;
    }

    /// Close DevTools
    pub fn close(&mut self) {
        self.state.is_open = false;
    }

    /// Toggle DevTools
    pub fn toggle(&mut self) {
        self.state.is_open = !self.state.is_open;
    }

    /// Set dock side
    pub fn set_dock_side(&mut self, side: DockSide) {
        self.state.dock_side = Some(side);
    }

    /// Get current dock side
    pub fn dock_side(&self) -> Option<DockSide> {
        self.state.dock_side
    }

    /// Set selected panel
    pub fn set_selected_panel(&mut self, panel: impl Into<String>) {
        self.state.selected_panel = Some(panel.into());
    }

    /// Get selected panel
    pub fn selected_panel(&self) -> Option<&str> {
        self.state.selected_panel.as_deref()
    }

    /// Get current state
    pub fn state(&self) -> &DevToolsState {
        &self.state
    }

    /// Get config
    pub fn config(&self) -> &DevToolsConfig {
        &self.config
    }

    // ========== Console ==========

    /// Add console message
    pub fn add_console_message(&mut self, message: ConsoleMessage) {
        // Enforce max messages
        if self.console_messages.len() >= self.max_console_messages {
            self.console_messages.remove(0);
        }
        self.console_messages.push(message);
    }

    /// Get console messages
    pub fn console_messages(&self) -> &[ConsoleMessage] {
        &self.console_messages
    }

    /// Get error messages only
    pub fn error_messages(&self) -> Vec<&ConsoleMessage> {
        self.console_messages
            .iter()
            .filter(|m| m.is_error())
            .collect()
    }

    /// Get warning messages only
    pub fn warning_messages(&self) -> Vec<&ConsoleMessage> {
        self.console_messages
            .iter()
            .filter(|m| m.is_warning())
            .collect()
    }

    /// Clear console messages
    pub fn clear_console(&mut self) {
        self.console_messages.clear();
    }

    /// Get console message count
    pub fn console_message_count(&self) -> usize {
        self.console_messages.len()
    }

    // ========== Network ==========

    /// Track network request
    pub fn add_network_request(&mut self, request: NetworkRequestInfo) {
        self.network_requests
            .insert(request.request_id.clone(), request);
    }

    /// Get network request by ID
    pub fn get_network_request(&self, request_id: &str) -> Option<&NetworkRequestInfo> {
        self.network_requests.get(request_id)
    }

    /// Get all network requests
    pub fn network_requests(&self) -> &HashMap<String, NetworkRequestInfo> {
        &self.network_requests
    }

    /// Get network requests as list
    pub fn network_requests_list(&self) -> Vec<&NetworkRequestInfo> {
        self.network_requests.values().collect()
    }

    /// Clear network requests
    pub fn clear_network(&mut self) {
        self.network_requests.clear();
    }

    /// Get network request count
    pub fn network_request_count(&self) -> usize {
        self.network_requests.len()
    }

    // ========== Clear All ==========

    /// Clear all data
    pub fn clear_all(&mut self) {
        self.clear_console();
        self.clear_network();
    }
}

impl Default for DevToolsManager {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devtools_manager() {
        let config = DevToolsConfig {
            enabled: true,
            remote_debugging_port: 9222,
            ..Default::default()
        };
        let mut manager = DevToolsManager::new(config);

        assert!(!manager.is_open());
        manager.open();
        assert!(manager.is_open());
        manager.toggle();
        assert!(!manager.is_open());
        assert_eq!(manager.remote_debugging_port(), 9222);
    }

    #[test]
    fn test_console_messages() {
        let mut manager = DevToolsManager::default();

        manager.add_console_message(ConsoleMessage::log("Test message"));
        manager.add_console_message(ConsoleMessage::error("Error message"));

        assert_eq!(manager.console_message_count(), 2);
        assert_eq!(manager.error_messages().len(), 1);

        manager.clear_console();
        assert_eq!(manager.console_message_count(), 0);
    }

    #[test]
    fn test_console_message_limit() {
        let mut manager = DevToolsManager::default().with_max_console_messages(3);

        for i in 0..5 {
            manager.add_console_message(ConsoleMessage::log(format!("Message {}", i)));
        }

        assert_eq!(manager.console_message_count(), 3);
        // Should have messages 2, 3, 4 (oldest removed)
        assert!(manager.console_messages()[0].text.contains("2"));
    }

    #[test]
    fn test_network_requests() {
        let mut manager = DevToolsManager::default();

        let request = NetworkRequestInfo::new("req-1", "https://example.com", "GET");
        manager.add_network_request(request);

        assert_eq!(manager.network_request_count(), 1);
        assert!(manager.get_network_request("req-1").is_some());

        manager.clear_network();
        assert_eq!(manager.network_request_count(), 0);
    }

    #[test]
    fn test_dock_side() {
        let mut manager = DevToolsManager::default();

        assert_eq!(manager.dock_side(), Some(DockSide::Right));
        manager.set_dock_side(DockSide::Bottom);
        assert_eq!(manager.dock_side(), Some(DockSide::Bottom));
    }
}
