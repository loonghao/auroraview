//! Chrome API Polyfill Generator
//!
//! Generates JavaScript code that provides chrome.* API compatibility
//! by bridging to AuroraView's native extension host.
//!
//! ## Features
//!
//! - Complete Chrome Extension API polyfill (via SDK)
//! - WXT framework compatibility layer
//! - Service Worker simulation
//! - Event-driven messaging system

use crate::ExtensionId;

/// Generate a minimal polyfill for content scripts
pub fn generate_content_script_polyfill(extension_id: &ExtensionId) -> String {
    format!(
        r#"
// AuroraView Content Script API Polyfill
// Extension ID: {extension_id}
(function() {{
    'use strict';
    
    const EXTENSION_ID = '{extension_id}';
    
    // Event emitter helper
    class EventEmitter {{
        constructor() {{ this._listeners = []; }}
        addListener(callback) {{ this._listeners.push(callback); }}
        removeListener(callback) {{
            const idx = this._listeners.indexOf(callback);
            if (idx >= 0) this._listeners.splice(idx, 1);
        }}
        hasListener(callback) {{ return this._listeners.includes(callback); }}
        hasListeners() {{ return this._listeners.length > 0; }}
        _dispatch(...args) {{
            for (const listener of this._listeners) {{
                try {{ listener(...args); }} catch (e) {{ console.error(e); }}
            }}
        }}
    }}
    
    // Helper to call native API
    async function callNativeApi(api, method, params = {{}}) {{
        if (typeof window.auroraview?.invoke !== 'function') return null;
        try {{
            return await window.auroraview.invoke('plugin:extensions|api_call', {{
                extensionId: EXTENSION_ID,
                api, method, params
            }});
        }} catch (e) {{
            console.error(`[Content Script] ${{api}}.${{method}} failed:`, e);
            throw e;
        }}
    }}
    
    // Minimal chrome.runtime for content scripts
    if (typeof window.chrome === 'undefined') {{
        window.chrome = {{}};
    }}
    
    window.chrome.runtime = {{
        id: EXTENSION_ID,
        
        getURL(path) {{
            return `https://auroraview.localhost/extension/${{EXTENSION_ID}}/${{path.replace(/^\\//, '')}}`;
        }},
        
        async sendMessage(message, options) {{
            return callNativeApi('runtime', 'sendMessage', {{ message, options }});
        }},
        
        connect(connectInfo) {{
            const portId = `port_${{Date.now()}}`;
            return {{
                name: connectInfo?.name || '',
                sender: {{ id: EXTENSION_ID }},
                onMessage: new EventEmitter(),
                onDisconnect: new EventEmitter(),
                postMessage(message) {{
                    callNativeApi('runtime', 'portPostMessage', {{ portId, message }});
                }},
                disconnect() {{
                    callNativeApi('runtime', 'portDisconnect', {{ portId }});
                }}
            }};
        }},
        
        onMessage: new EventEmitter(),
        lastError: null
    }};
    
    // Listen for messages
    if (window.auroraview?.on) {{
        window.auroraview.on('runtime_message', (data) => {{
            if (data.targetExtensionId && data.targetExtensionId !== EXTENSION_ID) return;
            const {{ message, sender }} = data;
            window.chrome.runtime.onMessage._dispatch(message, sender, () => {{}});
        }});
    }}
    
    console.log('[AuroraView] Content script polyfill loaded');
}})();
"#,
        extension_id = extension_id
    )
}

/// Generate WXT-compatible module shim
pub fn generate_wxt_shim() -> String {
    r#"
// WXT Framework Compatibility Shim for AuroraView
(function() {
    'use strict';
    
    // Create module system for WXT imports
    const modules = {};
    
    // wxt/storage module
    modules['wxt/storage'] = {
        storage: window.__wxtStorage || {
            defineItem(key, options = {}) {
                const defaultValue = options.defaultValue;
                const area = options.storage || 'local';
                const storageArea = window.chrome?.storage?.[area];
                
                if (!storageArea) {
                    console.warn('[WXT] Storage not available');
                    return {
                        getValue: async () => defaultValue,
                        setValue: async () => {},
                        removeValue: async () => {},
                        watch: () => () => {}
                    };
                }
                
                return {
                    async getValue() {
                        const result = await storageArea.get(key);
                        return result[key] !== undefined ? result[key] : defaultValue;
                    },
                    async setValue(value) {
                        await storageArea.set({ [key]: value });
                    },
                    async removeValue() {
                        await storageArea.remove(key);
                    },
                    watch(callback) {
                        const listener = (changes, areaName) => {
                            if (areaName === area && key in changes) {
                                callback(changes[key].newValue, changes[key].oldValue);
                            }
                        };
                        window.chrome.storage.onChanged.addListener(listener);
                        return () => window.chrome.storage.onChanged.removeListener(listener);
                    }
                };
            }
        }
    };
    
    // wxt/browser module
    modules['wxt/browser'] = {
        browser: window.chrome
    };
    
    // Simple module resolver
    window.__wxtRequire = function(moduleName) {
        if (modules[moduleName]) {
            return modules[moduleName];
        }
        throw new Error(`Module not found: ${moduleName}`);
    };
    
    // ES module import map support
    if (typeof importShim === 'undefined') {
        // Basic import shim for WXT modules
        window.importShim = async function(specifier) {
            if (modules[specifier]) {
                return modules[specifier];
            }
            // Fallback to native import
            return import(specifier);
        };
    }
    
    console.log('[AuroraView] WXT compatibility shim loaded');
})();
"#
    .to_string()
}

/// Generate polyfill using SDK-generated JavaScript
///
/// This function uses the pre-built polyfill from `@auroraview/sdk` which provides
/// better maintainability and TypeScript support. The SDK polyfill is built during
/// the `npm run build:assets` step and embedded into this crate.
///
/// # Arguments
/// * `extension_id` - The extension ID
/// * `extension_path` - The file path to the extension directory
/// * `manifest` - Optional manifest JSON for the extension
/// * `messages` - Optional i18n messages JSON
///
/// # Returns
/// A JavaScript string that sets up the extension environment and installs the polyfill
pub fn generate_polyfill_from_sdk(
    extension_id: &ExtensionId,
    extension_path: &str,
    manifest: Option<&serde_json::Value>,
    messages: Option<&serde_json::Value>,
) -> String {
    let polyfill_script = crate::js_assets::chrome_polyfill();

    let manifest_json = manifest
        .map(|m| m.to_string())
        .unwrap_or_else(|| "{}".to_string());

    let messages_json = messages
        .map(|m| m.to_string())
        .unwrap_or_else(|| "{}".to_string());

    let escaped_path = extension_path.replace('\\', "/").replace('\'', "\\'");

    format!(
        r#"
// AuroraView Chrome Extension Environment Setup
// Extension ID: {extension_id}
// Generated at: {timestamp}
(function() {{
    'use strict';

    // Set up extension context
    window.__AURORAVIEW__ = true;
    window.__EXTENSION_ID__ = '{extension_id}';
    window.__EXTENSION_PATH__ = '{escaped_path}';
    window.__EXTENSION_MANIFEST__ = {manifest_json};
    window.__EXTENSION_MESSAGES__ = {messages_json};
}})();

{polyfill_script}
"#,
        extension_id = extension_id,
        escaped_path = escaped_path,
        manifest_json = manifest_json,
        messages_json = messages_json,
        polyfill_script = polyfill_script,
        timestamp = chrono::Utc::now().to_rfc3339()
    )
}
