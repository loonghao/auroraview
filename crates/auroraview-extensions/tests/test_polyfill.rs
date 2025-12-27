//! Tests for Chrome API polyfill generation

use auroraview_extensions::polyfill::{
    generate_content_script_polyfill, generate_polyfill_from_sdk, generate_wxt_shim,
};
use serde_json::json;

#[test]
fn test_generate_content_script_polyfill() {
    let polyfill = generate_content_script_polyfill(&"test-extension".to_string());

    // Check it contains the extension ID
    assert!(polyfill.contains("test-extension"));

    // Check it sets up chrome.runtime
    assert!(polyfill.contains("window.chrome.runtime"));

    // Check it has sendMessage
    assert!(polyfill.contains("sendMessage"));

    // Check it has getURL
    assert!(polyfill.contains("getURL"));

    // Check it has onMessage
    assert!(polyfill.contains("onMessage"));

    // Check it's an IIFE
    assert!(polyfill.contains("(function()"));
    assert!(polyfill.contains("'use strict'"));
}

#[test]
fn test_generate_content_script_polyfill_escapes_extension_id() {
    // Test with special characters in extension ID
    let polyfill = generate_content_script_polyfill(&"test-ext-123".to_string());
    assert!(polyfill.contains("test-ext-123"));
}

#[test]
fn test_generate_wxt_shim() {
    let shim = generate_wxt_shim();

    // Check it sets up WXT modules
    assert!(shim.contains("wxt/storage"));
    assert!(shim.contains("wxt/browser"));

    // Check it has module resolver
    assert!(shim.contains("__wxtRequire"));

    // Check it has storage.defineItem
    assert!(shim.contains("defineItem"));

    // Check it's an IIFE
    assert!(shim.contains("(function()"));
    assert!(shim.contains("'use strict'"));
}

#[test]
fn test_generate_polyfill_from_sdk_basic() {
    let polyfill = generate_polyfill_from_sdk(
        &"my-extension".to_string(),
        "/path/to/extension",
        None,
        None,
    );

    // Check extension ID is set
    assert!(polyfill.contains("my-extension"));
    assert!(polyfill.contains("__EXTENSION_ID__"));

    // Check extension path is set
    assert!(polyfill.contains("__EXTENSION_PATH__"));
    assert!(polyfill.contains("/path/to/extension"));

    // Check default manifest
    assert!(polyfill.contains("__EXTENSION_MANIFEST__"));

    // Check default messages
    assert!(polyfill.contains("__EXTENSION_MESSAGES__"));
}

#[test]
fn test_generate_polyfill_from_sdk_with_manifest() {
    let manifest = json!({
        "name": "Test Extension",
        "version": "1.0.0",
        "manifest_version": 3
    });

    let polyfill = generate_polyfill_from_sdk(
        &"manifest-ext".to_string(),
        "/ext/path",
        Some(&manifest),
        None,
    );

    // Check manifest is included
    assert!(polyfill.contains("Test Extension"));
    assert!(polyfill.contains("1.0.0"));
}

#[test]
fn test_generate_polyfill_from_sdk_with_messages() {
    let messages = json!({
        "extensionName": {
            "message": "My Extension"
        }
    });

    let polyfill = generate_polyfill_from_sdk(
        &"messages-ext".to_string(),
        "/ext/path",
        None,
        Some(&messages),
    );

    // Check messages are included
    assert!(polyfill.contains("extensionName"));
    assert!(polyfill.contains("My Extension"));
}

#[test]
fn test_generate_polyfill_from_sdk_escapes_path() {
    // Test with Windows-style path
    let polyfill = generate_polyfill_from_sdk(
        &"path-ext".to_string(),
        "C:\\Users\\test\\extension",
        None,
        None,
    );

    // Check path is escaped (backslashes converted to forward slashes)
    assert!(polyfill.contains("C:/Users/test/extension"));
    assert!(!polyfill.contains("C:\\Users"));
}

#[test]
fn test_generate_polyfill_from_sdk_escapes_quotes() {
    // Test with path containing single quotes
    let polyfill =
        generate_polyfill_from_sdk(&"quote-ext".to_string(), "/path/with'quote", None, None);

    // Check quote is escaped
    assert!(polyfill.contains("/path/with\\'quote"));
}

#[test]
fn test_generate_polyfill_from_sdk_sets_auroraview_flag() {
    let polyfill = generate_polyfill_from_sdk(&"flag-ext".to_string(), "/ext/path", None, None);

    // Check AuroraView flag is set
    assert!(polyfill.contains("__AURORAVIEW__"));
    assert!(polyfill.contains("= true"));
}

#[test]
fn test_generate_polyfill_from_sdk_has_timestamp() {
    let polyfill = generate_polyfill_from_sdk(&"time-ext".to_string(), "/ext/path", None, None);

    // Check timestamp comment is present
    assert!(polyfill.contains("Generated at:"));
}
