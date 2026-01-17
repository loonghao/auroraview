//! Session management for tab persistence

use crate::{Result, TabError, TabGroup, TabId, TabState};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Session data - serializable snapshot of tab state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// Session version
    pub version: u32,
    /// All tabs
    pub tabs: Vec<TabState>,
    /// Active tab ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_tab_id: Option<TabId>,
    /// Tab groups
    #[serde(default)]
    pub groups: Vec<TabGroup>,
    /// Timestamp
    pub timestamp: i64,
}

impl Session {
    /// Current session version
    const CURRENT_VERSION: u32 = 1;

    /// Create a new session
    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            tabs: Vec::new(),
            active_tab_id: None,
            groups: Vec::new(),
            timestamp: current_timestamp(),
        }
    }

    /// Create session from tabs and groups
    pub fn from_state(
        tabs: Vec<TabState>,
        active_tab_id: Option<TabId>,
        groups: Vec<TabGroup>,
    ) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            tabs,
            active_tab_id,
            groups,
            timestamp: current_timestamp(),
        }
    }

    /// Check if session is empty
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Get tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Session manager - handles session persistence
pub struct SessionManager {
    /// Storage path
    storage_path: PathBuf,
    /// Auto-save enabled
    auto_save: bool,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(data_dir: &Path) -> Self {
        Self {
            storage_path: data_dir.join("session.json"),
            auto_save: true,
        }
    }

    /// Create session manager with custom path
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            storage_path: path,
            auto_save: true,
        }
    }

    /// Enable/disable auto-save
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save = enabled;
    }

    /// Check if auto-save is enabled
    pub fn auto_save(&self) -> bool {
        self.auto_save
    }

    /// Save session
    pub fn save(&self, session: &Session) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(session)?;
        std::fs::write(&self.storage_path, json)?;

        Ok(())
    }

    /// Load session
    pub fn load(&self) -> Result<Session> {
        if !self.storage_path.exists() {
            return Ok(Session::new());
        }

        let json = std::fs::read_to_string(&self.storage_path)?;
        let session: Session = serde_json::from_str(&json)?;

        // Version check/migration could go here
        Ok(session)
    }

    /// Check if session file exists
    pub fn exists(&self) -> bool {
        self.storage_path.exists()
    }

    /// Delete session file
    pub fn delete(&self) -> Result<()> {
        if self.storage_path.exists() {
            std::fs::remove_file(&self.storage_path)?;
        }
        Ok(())
    }

    /// Get storage path
    pub fn path(&self) -> &Path {
        &self.storage_path
    }

    /// Create backup of current session
    pub fn backup(&self) -> Result<PathBuf> {
        if !self.storage_path.exists() {
            return Err(TabError::Session("No session to backup".to_string()));
        }

        let backup_path = self.storage_path.with_extension("json.bak");
        std::fs::copy(&self.storage_path, &backup_path)?;

        Ok(backup_path)
    }

    /// Restore from backup
    pub fn restore_backup(&self) -> Result<Session> {
        let backup_path = self.storage_path.with_extension("json.bak");

        if !backup_path.exists() {
            return Err(TabError::Session("No backup found".to_string()));
        }

        let json = std::fs::read_to_string(&backup_path)?;
        let session: Session = serde_json::from_str(&json)?;

        Ok(session)
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_creation() {
        let session = Session::new();

        assert!(session.is_empty());
        assert_eq!(session.version, Session::CURRENT_VERSION);
    }

    #[test]
    fn test_session_from_state() {
        let tabs = vec![
            TabState::new("tab-1".to_string(), "https://github.com"),
            TabState::new("tab-2".to_string(), "https://gitlab.com"),
        ];

        let session = Session::from_state(tabs, Some("tab-1".to_string()), Vec::new());

        assert_eq!(session.tab_count(), 2);
        assert_eq!(session.active_tab_id, Some("tab-1".to_string()));
    }

    #[test]
    fn test_session_manager_save_load() {
        let dir = TempDir::new().unwrap();
        let manager = SessionManager::new(dir.path());

        let tabs = vec![TabState::new("tab-1".to_string(), "https://github.com")];
        let session = Session::from_state(tabs, Some("tab-1".to_string()), Vec::new());

        manager.save(&session).unwrap();
        assert!(manager.exists());

        let loaded = manager.load().unwrap();
        assert_eq!(loaded.tab_count(), 1);
        assert_eq!(loaded.active_tab_id, Some("tab-1".to_string()));
    }

    #[test]
    fn test_session_manager_backup() {
        let dir = TempDir::new().unwrap();
        let manager = SessionManager::new(dir.path());

        let session = Session::from_state(
            vec![TabState::new("tab-1".to_string(), "https://github.com")],
            None,
            Vec::new(),
        );

        manager.save(&session).unwrap();
        let backup_path = manager.backup().unwrap();

        assert!(backup_path.exists());

        let restored = manager.restore_backup().unwrap();
        assert_eq!(restored.tab_count(), 1);
    }
}
