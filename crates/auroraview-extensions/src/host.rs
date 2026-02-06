//! Extension Host - Manages loaded extensions
//!
//! The ExtensionHost is responsible for:
//! - Loading extensions from disk
//! - Managing extension lifecycle
//! - Routing API calls to appropriate handlers
//! - Providing extension resources (HTML, JS, CSS)

use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

use crate::error::{ExtensionError, ExtensionResult};
use crate::manifest::Manifest;
use crate::runtime::ExtensionRuntime;
use crate::storage::StorageBackend;
use crate::ExtensionId;

/// Extension host configuration
#[derive(Debug, Clone)]
pub struct ExtensionConfig {
    /// Directory containing extensions
    pub extensions_dir: PathBuf,
    /// Directory for extension storage data
    pub storage_dir: PathBuf,
    /// Enable developer mode (allows unsigned extensions)
    pub developer_mode: bool,
    /// Enable logging
    pub enable_logging: bool,
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self {
            extensions_dir: PathBuf::from("extensions"),
            storage_dir: PathBuf::from("extension_storage"),
            developer_mode: true,
            enable_logging: true,
        }
    }
}

/// A loaded extension
#[derive(Debug, Clone)]
pub struct LoadedExtension {
    /// Extension ID (directory name or generated)
    pub id: ExtensionId,
    /// Extension manifest
    pub manifest: Manifest,
    /// Extension root directory
    pub root_dir: PathBuf,
    /// Whether the extension is enabled
    pub enabled: bool,
}

impl LoadedExtension {
    /// Get the full path to a resource
    pub fn get_resource_path(&self, relative_path: &str) -> PathBuf {
        self.root_dir.join(relative_path)
    }

    /// Get the side panel HTML path
    pub fn get_side_panel_path(&self) -> Option<PathBuf> {
        self.manifest
            .get_side_panel_path()
            .map(|p| self.get_resource_path(p))
    }

    /// Get the popup HTML path
    pub fn get_popup_path(&self) -> Option<PathBuf> {
        self.manifest
            .get_popup_path()
            .map(|p| self.get_resource_path(p))
    }

    /// Get the background service worker path
    pub fn get_service_worker_path(&self) -> Option<PathBuf> {
        self.manifest
            .get_service_worker_path()
            .map(|p| self.get_resource_path(p))
    }

    /// Read a resource file
    pub fn read_resource(&self, relative_path: &str) -> ExtensionResult<String> {
        let path = self.get_resource_path(relative_path);
        std::fs::read_to_string(&path).map_err(|e| {
            ExtensionError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Resource not found: {} ({})", relative_path, e),
            ))
        })
    }

    /// Read a resource file as bytes
    pub fn read_resource_bytes(&self, relative_path: &str) -> ExtensionResult<Vec<u8>> {
        let path = self.get_resource_path(relative_path);
        std::fs::read(&path).map_err(|e| {
            ExtensionError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Resource not found: {} ({})", relative_path, e),
            ))
        })
    }
}

/// Extension host - manages all loaded extensions
pub struct ExtensionHost {
    /// Configuration
    config: ExtensionConfig,
    /// Loaded extensions
    extensions: Arc<RwLock<HashMap<ExtensionId, LoadedExtension>>>,
    /// Storage backend
    storage: Arc<StorageBackend>,
    /// Extension runtimes (for background scripts)
    runtimes: Arc<RwLock<HashMap<ExtensionId, ExtensionRuntime>>>,
}

impl ExtensionHost {
    /// Create a new extension host
    pub fn new(config: ExtensionConfig) -> Self {
        let storage = Arc::new(StorageBackend::new(config.storage_dir.clone()));

        Self {
            config,
            extensions: Arc::new(RwLock::new(HashMap::new())),
            storage,
            runtimes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the storage backend
    pub fn storage(&self) -> &Arc<StorageBackend> {
        &self.storage
    }

    /// Load all extensions from the extensions directory
    pub async fn load_extensions(&self) -> ExtensionResult<Vec<ExtensionId>> {
        let mut loaded_ids = Vec::new();

        if !self.config.extensions_dir.exists() {
            tracing::info!(
                "Extensions directory does not exist: {:?}",
                self.config.extensions_dir
            );
            return Ok(loaded_ids);
        }

        // Iterate through extension directories
        for entry in WalkDir::new(&self.config.extensions_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() {
                let manifest_path = entry.path().join("manifest.json");
                if manifest_path.exists() {
                    match self.load_extension(entry.path()).await {
                        Ok(id) => {
                            tracing::info!("Loaded extension: {}", id);
                            loaded_ids.push(id);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to load extension from {:?}: {}",
                                entry.path(),
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(loaded_ids)
    }

    /// Load a single extension from a directory
    pub async fn load_extension(&self, path: &Path) -> ExtensionResult<ExtensionId> {
        let manifest_path = path.join("manifest.json");
        let manifest = Manifest::from_file(&manifest_path)?;
        manifest.validate()?;

        // Use directory name as extension ID
        let id = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Check if already loaded
        {
            let extensions = self.extensions.read();
            if extensions.contains_key(&id) {
                return Err(ExtensionError::AlreadyLoaded(id));
            }
        }

        let extension = LoadedExtension {
            id: id.clone(),
            manifest,
            root_dir: dunce::canonicalize(path)?,
            enabled: true,
        };

        // Store extension
        {
            let mut extensions = self.extensions.write();
            extensions.insert(id.clone(), extension);
        }

        // Initialize runtime if extension has background script
        // (This will be implemented later)

        Ok(id)
    }

    /// Unload an extension
    pub fn unload_extension(&self, id: &str) -> ExtensionResult<()> {
        let mut extensions = self.extensions.write();
        extensions
            .remove(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;

        // Clean up runtime
        let mut runtimes = self.runtimes.write();
        runtimes.remove(id);

        Ok(())
    }

    /// Get a loaded extension
    pub fn get_extension(&self, id: &str) -> Option<LoadedExtension> {
        let extensions = self.extensions.read();
        extensions.get(id).cloned()
    }

    /// Get all loaded extensions
    pub fn get_all_extensions(&self) -> Vec<LoadedExtension> {
        let extensions = self.extensions.read();
        extensions.values().cloned().collect()
    }

    /// Get extensions with side panel
    pub fn get_side_panel_extensions(&self) -> Vec<LoadedExtension> {
        let extensions = self.extensions.read();
        extensions
            .values()
            .filter(|ext| ext.manifest.has_side_panel() && ext.enabled)
            .cloned()
            .collect()
    }

    /// Get extensions with action (toolbar button)
    pub fn get_action_extensions(&self) -> Vec<LoadedExtension> {
        let extensions = self.extensions.read();
        extensions
            .values()
            .filter(|ext| ext.manifest.has_action() && ext.enabled)
            .cloned()
            .collect()
    }

    /// Enable an extension
    pub fn enable_extension(&self, id: &str) -> ExtensionResult<()> {
        let mut extensions = self.extensions.write();
        let ext = extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;
        ext.enabled = true;
        Ok(())
    }

    /// Disable an extension
    pub fn disable_extension(&self, id: &str) -> ExtensionResult<()> {
        let mut extensions = self.extensions.write();
        let ext = extensions
            .get_mut(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;
        ext.enabled = false;
        Ok(())
    }

    /// Get the side panel HTML content for an extension
    pub fn get_side_panel_html(&self, id: &str) -> ExtensionResult<String> {
        let ext = self
            .get_extension(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;

        let path = ext.manifest.get_side_panel_path().ok_or_else(|| {
            ExtensionError::ApiNotSupported("Extension has no side panel".to_string())
        })?;

        ext.read_resource(path)
    }

    /// Get the popup HTML content for an extension
    pub fn get_popup_html(&self, id: &str) -> ExtensionResult<String> {
        let ext = self
            .get_extension(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;

        let path = ext
            .manifest
            .get_popup_path()
            .ok_or_else(|| ExtensionError::ApiNotSupported("Extension has no popup".to_string()))?;

        ext.read_resource(path)
    }

    /// Generate the chrome.* API polyfill script for an extension
    pub fn generate_api_polyfill(&self, id: &str) -> ExtensionResult<String> {
        let ext = self
            .get_extension(id)
            .ok_or_else(|| ExtensionError::NotFound(id.to_string()))?;

        Ok(generate_chrome_api_polyfill(&ext))
    }
}

/// Generate Chrome API polyfill JavaScript
fn generate_chrome_api_polyfill(extension: &LoadedExtension) -> String {
    let ext_id = &extension.id;
    let has_storage = extension.manifest.has_permission("storage");
    let has_tabs = extension.manifest.has_permission("tabs");
    let has_side_panel = extension.manifest.has_permission("sidePanel");
    let has_scripting = extension.manifest.has_permission("scripting");

    format!(
        r#"
// AuroraView Chrome Extension API Polyfill
// Extension: {name} ({id})
(function() {{
    'use strict';

    const EXTENSION_ID = '{id}';

    // Helper to call native API
    async function callNative(api, method, params) {{
        return await window.auroraview.invoke('plugin:extensions|api_call', {{
            extensionId: EXTENSION_ID,
            api: api,
            method: method,
            params: params || {{}}
        }});
    }}

    // chrome.runtime API
    const runtime = {{
        id: EXTENSION_ID,
        getManifest: function() {{
            return {manifest};
        }},
        getURL: function(path) {{
            return 'auroraview-extension://' + EXTENSION_ID + '/' + path;
        }},
        sendMessage: async function(message, options) {{
            return await callNative('runtime', 'sendMessage', {{ message, options }});
        }},
        onMessage: {{
            _listeners: [],
            addListener: function(callback) {{
                this._listeners.push(callback);
            }},
            removeListener: function(callback) {{
                const idx = this._listeners.indexOf(callback);
                if (idx >= 0) this._listeners.splice(idx, 1);
            }},
            hasListener: function(callback) {{
                return this._listeners.includes(callback);
            }}
        }},
        onInstalled: {{
            _listeners: [],
            addListener: function(callback) {{
                this._listeners.push(callback);
            }},
            removeListener: function(callback) {{
                const idx = this._listeners.indexOf(callback);
                if (idx >= 0) this._listeners.splice(idx, 1);
            }}
        }},
        connect: function(extensionId, connectInfo) {{
            console.warn('[AuroraView] chrome.runtime.connect is not fully supported');
            return {{
                postMessage: function() {{}},
                disconnect: function() {{}},
                onMessage: {{ addListener: function() {{}} }},
                onDisconnect: {{ addListener: function() {{}} }}
            }};
        }},
        lastError: null
    }};

    {storage_api}

    {tabs_api}

    {side_panel_api}

    {scripting_api}

    // Build chrome object
    window.chrome = window.chrome || {{}};
    window.chrome.runtime = runtime;
    {chrome_storage}
    {chrome_tabs}
    {chrome_side_panel}
    {chrome_scripting}

    console.log('[AuroraView] Chrome API polyfill loaded for extension:', EXTENSION_ID);
}})();
"#,
        name = extension.manifest.name,
        id = ext_id,
        manifest = serde_json::to_string(&extension.manifest).unwrap_or_default(),
        storage_api = if has_storage {
            generate_storage_api()
        } else {
            String::new()
        },
        tabs_api = if has_tabs {
            generate_tabs_api()
        } else {
            String::new()
        },
        side_panel_api = if has_side_panel {
            generate_side_panel_api()
        } else {
            String::new()
        },
        scripting_api = if has_scripting {
            generate_scripting_api()
        } else {
            String::new()
        },
        chrome_storage = if has_storage {
            "window.chrome.storage = storage;"
        } else {
            ""
        },
        chrome_tabs = if has_tabs {
            "window.chrome.tabs = tabs;"
        } else {
            ""
        },
        chrome_side_panel = if has_side_panel {
            "window.chrome.sidePanel = sidePanel;"
        } else {
            ""
        },
        chrome_scripting = if has_scripting {
            "window.chrome.scripting = scripting;"
        } else {
            ""
        },
    )
}

fn generate_storage_api() -> String {
    r#"
    // chrome.storage API
    function createStorageArea(areaName) {
        return {
            get: async function(keys) {
                const keyList = keys === null ? null : (Array.isArray(keys) ? keys : (typeof keys === 'string' ? [keys] : Object.keys(keys || {})));
                const result = await callNative('storage', 'get', { area: areaName, keys: keyList });
                // Merge with defaults if keys was an object
                if (keys && typeof keys === 'object' && !Array.isArray(keys)) {
                    return { ...keys, ...result };
                }
                return result;
            },
            set: async function(items) {
                return await callNative('storage', 'set', { area: areaName, items });
            },
            remove: async function(keys) {
                const keyList = Array.isArray(keys) ? keys : [keys];
                return await callNative('storage', 'remove', { area: areaName, keys: keyList });
            },
            clear: async function() {
                return await callNative('storage', 'clear', { area: areaName });
            },
            getBytesInUse: async function(keys) {
                const keyList = keys ? (Array.isArray(keys) ? keys : [keys]) : null;
                return await callNative('storage', 'getBytesInUse', { area: areaName, keys: keyList });
            },
            onChanged: {
                _listeners: [],
                addListener: function(callback) {
                    this._listeners.push(callback);
                },
                removeListener: function(callback) {
                    const idx = this._listeners.indexOf(callback);
                    if (idx >= 0) this._listeners.splice(idx, 1);
                }
            }
        };
    }

    const storage = {
        local: createStorageArea('local'),
        sync: createStorageArea('sync'),
        session: createStorageArea('session'),
        managed: createStorageArea('managed'),
        onChanged: {
            _listeners: [],
            addListener: function(callback) {
                this._listeners.push(callback);
            },
            removeListener: function(callback) {
                const idx = this._listeners.indexOf(callback);
                if (idx >= 0) this._listeners.splice(idx, 1);
            }
        }
    };
"#.to_string()
}

fn generate_tabs_api() -> String {
    r#"
    // chrome.tabs API
    const tabs = {
        query: async function(queryInfo) {
            return await callNative('tabs', 'query', queryInfo);
        },
        get: async function(tabId) {
            return await callNative('tabs', 'get', { tabId });
        },
        getCurrent: async function() {
            return await callNative('tabs', 'getCurrent', {});
        },
        create: async function(createProperties) {
            return await callNative('tabs', 'create', createProperties);
        },
        update: async function(tabId, updateProperties) {
            if (typeof tabId === 'object') {
                updateProperties = tabId;
                tabId = null;
            }
            return await callNative('tabs', 'update', { tabId, ...updateProperties });
        },
        remove: async function(tabIds) {
            const ids = Array.isArray(tabIds) ? tabIds : [tabIds];
            return await callNative('tabs', 'remove', { tabIds: ids });
        },
        sendMessage: async function(tabId, message, options) {
            return await callNative('tabs', 'sendMessage', { tabId, message, options });
        },
        onUpdated: {
            _listeners: [],
            addListener: function(callback) {
                this._listeners.push(callback);
            },
            removeListener: function(callback) {
                const idx = this._listeners.indexOf(callback);
                if (idx >= 0) this._listeners.splice(idx, 1);
            }
        },
        onActivated: {
            _listeners: [],
            addListener: function(callback) {
                this._listeners.push(callback);
            },
            removeListener: function(callback) {
                const idx = this._listeners.indexOf(callback);
                if (idx >= 0) this._listeners.splice(idx, 1);
            }
        }
    };
"#
    .to_string()
}

fn generate_side_panel_api() -> String {
    r#"
    // chrome.sidePanel API
    const sidePanel = {
        open: async function(options) {
            return await callNative('sidePanel', 'open', options || {});
        },
        close: async function(options) {
            return await callNative('sidePanel', 'close', options || {});
        },
        setOptions: async function(options) {
            return await callNative('sidePanel', 'setOptions', options);
        },
        getOptions: async function(options) {
            return await callNative('sidePanel', 'getOptions', options || {});
        },
        setPanelBehavior: async function(behavior) {
            return await callNative('sidePanel', 'setPanelBehavior', behavior);
        },
        getPanelBehavior: async function() {
            return await callNative('sidePanel', 'getPanelBehavior', {});
        }
    };
"#
    .to_string()
}

fn generate_scripting_api() -> String {
    r#"
    // chrome.scripting API
    const scripting = {
        executeScript: async function(injection) {
            return await callNative('scripting', 'executeScript', injection);
        },
        insertCSS: async function(injection) {
            return await callNative('scripting', 'insertCSS', injection);
        },
        removeCSS: async function(injection) {
            return await callNative('scripting', 'removeCSS', injection);
        },
        registerContentScripts: async function(scripts) {
            return await callNative('scripting', 'registerContentScripts', { scripts });
        },
        unregisterContentScripts: async function(filter) {
            return await callNative('scripting', 'unregisterContentScripts', filter || {});
        },
        getRegisteredContentScripts: async function(filter) {
            return await callNative('scripting', 'getRegisteredContentScripts', filter || {});
        }
    };
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_load_extension() {
        let dir = tempdir().unwrap();
        let ext_dir = dir.path().join("test-extension");
        fs::create_dir_all(&ext_dir).unwrap();

        let manifest = r#"{
            "manifest_version": 3,
            "name": "Test Extension",
            "version": "1.0.0",
            "permissions": ["storage"],
            "side_panel": {
                "default_path": "sidepanel.html"
            }
        }"#;

        fs::write(ext_dir.join("manifest.json"), manifest).unwrap();
        fs::write(ext_dir.join("sidepanel.html"), "<html></html>").unwrap();

        let config = ExtensionConfig {
            extensions_dir: dir.path().to_path_buf(),
            storage_dir: dir.path().join("storage"),
            developer_mode: true,
            enable_logging: false,
        };

        let host = ExtensionHost::new(config);
        let ids = host.load_extensions().await.unwrap();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "test-extension");

        let ext = host.get_extension("test-extension").unwrap();
        assert_eq!(ext.manifest.name, "Test Extension");
        assert!(ext.manifest.has_permission("storage"));
    }
}
