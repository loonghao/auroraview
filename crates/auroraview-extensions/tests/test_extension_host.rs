use std::fs;

use auroraview_extensions::{error::ExtensionError, ExtensionConfig, ExtensionHost};
use rstest::*;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_manifest(
    name: &str,
    version: &str,
    permissions: &[&str],
    side_panel_path: Option<&str>,
    popup_path: Option<&str>,
) -> String {
    let perms = permissions
        .iter()
        .map(|p| format!("\"{}\"", p))
        .collect::<Vec<_>>()
        .join(", ");

    let side_panel = match side_panel_path {
        Some(p) => format!(r#","side_panel": {{"default_path": "{}"}}"#, p),
        None => String::new(),
    };

    let action = match popup_path {
        Some(p) => format!(r#","action": {{"default_popup": "{}"}}"#, p),
        None => String::new(),
    };

    format!(
        r#"{{
            "manifest_version": 3,
            "name": "{name}",
            "version": "{version}",
            "permissions": [{perms}]
            {side_panel}
            {action}
        }}"#,
    )
}

fn create_extension_dir(parent: &TempDir, dir_name: &str, manifest_json: &str) -> std::path::PathBuf {
    let ext_dir = parent.path().join(dir_name);
    fs::create_dir_all(&ext_dir).unwrap();
    fs::write(ext_dir.join("manifest.json"), manifest_json).unwrap();
    ext_dir
}

fn default_config(tmp: &TempDir) -> ExtensionConfig {
    ExtensionConfig {
        extensions_dir: tmp.path().to_path_buf(),
        storage_dir: tmp.path().join("storage"),
        developer_mode: true,
        enable_logging: false,
    }
}

// ---------------------------------------------------------------------------
// load_extension – success
// ---------------------------------------------------------------------------

#[tokio::test]
async fn load_extension_returns_dir_name_as_id() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("My Ext", "1.0.0", &[], None, None);
    create_extension_dir(&tmp, "my-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    let id = host
        .load_extension(&tmp.path().join("my-ext"))
        .await
        .unwrap();

    assert_eq!(id, "my-ext");
}

#[tokio::test]
async fn load_extension_stores_in_host() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("Stored", "2.0.0", &["storage"], None, None);
    create_extension_dir(&tmp, "stored-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&tmp.path().join("stored-ext"))
        .await
        .unwrap();

    let ext = host.get_extension("stored-ext");
    assert!(ext.is_some());
    assert_eq!(ext.unwrap().manifest.name, "Stored");
}

#[tokio::test]
async fn load_extension_sets_enabled_true() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("Enabled", "1.0.0", &[], None, None);
    create_extension_dir(&tmp, "enabled-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&tmp.path().join("enabled-ext"))
        .await
        .unwrap();

    let ext = host.get_extension("enabled-ext").unwrap();
    assert!(ext.enabled);
}

// ---------------------------------------------------------------------------
// load_extension – errors
// ---------------------------------------------------------------------------

#[tokio::test]
async fn load_extension_missing_manifest_errors() {
    let tmp = TempDir::new().unwrap();
    let ext_dir = tmp.path().join("no-manifest");
    fs::create_dir_all(&ext_dir).unwrap();

    let host = ExtensionHost::new(default_config(&tmp));
    let result = host.load_extension(&ext_dir).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn load_extension_duplicate_returns_already_loaded() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("Dup", "1.0.0", &[], None, None);
    create_extension_dir(&tmp, "dup-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&tmp.path().join("dup-ext"))
        .await
        .unwrap();

    let result = host.load_extension(&tmp.path().join("dup-ext")).await;
    assert!(matches!(result, Err(ExtensionError::AlreadyLoaded(_))));
}

#[tokio::test]
async fn load_extension_invalid_manifest_errors() {
    let tmp = TempDir::new().unwrap();
    let ext_dir = tmp.path().join("bad-manifest");
    fs::create_dir_all(&ext_dir).unwrap();
    fs::write(ext_dir.join("manifest.json"), "{ not valid json").unwrap();

    let host = ExtensionHost::new(default_config(&tmp));
    let result = host.load_extension(&ext_dir).await;
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// load_extensions (batch)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn load_extensions_empty_dir_returns_empty_vec() {
    let tmp = TempDir::new().unwrap();
    let host = ExtensionHost::new(default_config(&tmp));
    let ids = host.load_extensions().await.unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn load_extensions_nonexistent_dir_returns_empty_vec() {
    let tmp = TempDir::new().unwrap();
    let config = ExtensionConfig {
        extensions_dir: tmp.path().join("does-not-exist"),
        storage_dir: tmp.path().join("storage"),
        developer_mode: true,
        enable_logging: false,
    };
    let host = ExtensionHost::new(config);
    let ids = host.load_extensions().await.unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn load_extensions_loads_all_valid_subdirs() {
    let tmp = TempDir::new().unwrap();
    for name in &["ext-a", "ext-b", "ext-c"] {
        let manifest = make_manifest(name, "1.0.0", &[], None, None);
        create_extension_dir(&tmp, name, &manifest);
    }

    let host = ExtensionHost::new(default_config(&tmp));
    let mut ids = host.load_extensions().await.unwrap();
    ids.sort();

    assert_eq!(ids, vec!["ext-a", "ext-b", "ext-c"]);
}

#[tokio::test]
async fn load_extensions_skips_dirs_without_manifest() {
    let tmp = TempDir::new().unwrap();
    // valid
    let manifest = make_manifest("Valid", "1.0.0", &[], None, None);
    create_extension_dir(&tmp, "valid-ext", &manifest);
    // no manifest.json
    fs::create_dir_all(tmp.path().join("no-manifest-dir")).unwrap();

    let host = ExtensionHost::new(default_config(&tmp));
    let ids = host.load_extensions().await.unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], "valid-ext");
}

// ---------------------------------------------------------------------------
// get_extension / get_all_extensions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_extension_returns_none_for_unknown() {
    let tmp = TempDir::new().unwrap();
    let host = ExtensionHost::new(default_config(&tmp));
    assert!(host.get_extension("ghost").is_none());
}

#[tokio::test]
async fn get_all_extensions_returns_all_loaded() {
    let tmp = TempDir::new().unwrap();
    for name in &["e1", "e2"] {
        let manifest = make_manifest(name, "1.0.0", &[], None, None);
        create_extension_dir(&tmp, name, &manifest);
    }
    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extensions().await.unwrap();

    let all = host.get_all_extensions();
    assert_eq!(all.len(), 2);
}

// ---------------------------------------------------------------------------
// unload_extension
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unload_extension_removes_it() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("ToUnload", "1.0.0", &[], None, None);
    create_extension_dir(&tmp, "to-unload", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&tmp.path().join("to-unload"))
        .await
        .unwrap();

    host.unload_extension("to-unload").unwrap();
    assert!(host.get_extension("to-unload").is_none());
}

#[tokio::test]
async fn unload_extension_not_found_errors() {
    let tmp = TempDir::new().unwrap();
    let host = ExtensionHost::new(default_config(&tmp));
    let result = host.unload_extension("ghost");
    assert!(matches!(result, Err(ExtensionError::NotFound(_))));
}

#[tokio::test]
async fn unload_extension_then_reload_succeeds() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("Reload", "1.0.0", &[], None, None);
    create_extension_dir(&tmp, "reload-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&tmp.path().join("reload-ext"))
        .await
        .unwrap();
    host.unload_extension("reload-ext").unwrap();
    // should succeed again
    host.load_extension(&tmp.path().join("reload-ext"))
        .await
        .unwrap();

    assert!(host.get_extension("reload-ext").is_some());
}

// ---------------------------------------------------------------------------
// enable / disable extension
// ---------------------------------------------------------------------------

#[tokio::test]
async fn disable_then_enable_extension() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("ToggleExt", "1.0.0", &[], None, None);
    create_extension_dir(&tmp, "toggle-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&tmp.path().join("toggle-ext"))
        .await
        .unwrap();

    host.disable_extension("toggle-ext").unwrap();
    assert!(!host.get_extension("toggle-ext").unwrap().enabled);

    host.enable_extension("toggle-ext").unwrap();
    assert!(host.get_extension("toggle-ext").unwrap().enabled);
}

#[tokio::test]
async fn enable_nonexistent_extension_errors() {
    let tmp = TempDir::new().unwrap();
    let host = ExtensionHost::new(default_config(&tmp));
    let result = host.enable_extension("ghost");
    assert!(matches!(result, Err(ExtensionError::NotFound(_))));
}

#[tokio::test]
async fn disable_nonexistent_extension_errors() {
    let tmp = TempDir::new().unwrap();
    let host = ExtensionHost::new(default_config(&tmp));
    let result = host.disable_extension("ghost");
    assert!(matches!(result, Err(ExtensionError::NotFound(_))));
}

// ---------------------------------------------------------------------------
// get_side_panel_extensions / get_action_extensions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_side_panel_extensions_returns_only_panel_extensions() {
    let tmp = TempDir::new().unwrap();

    let manifest_with_panel = make_manifest(
        "WithPanel",
        "1.0.0",
        &["sidePanel"],
        Some("panel.html"),
        None,
    );
    create_extension_dir(&tmp, "has-panel", &manifest_with_panel);

    let manifest_no_panel = make_manifest("NoPanel", "1.0.0", &[], None, None);
    create_extension_dir(&tmp, "no-panel", &manifest_no_panel);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extensions().await.unwrap();

    let panel_exts = host.get_side_panel_extensions();
    assert_eq!(panel_exts.len(), 1);
    assert_eq!(panel_exts[0].id, "has-panel");
}

#[tokio::test]
async fn get_side_panel_extensions_excludes_disabled() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("Panel", "1.0.0", &["sidePanel"], Some("panel.html"), None);
    create_extension_dir(&tmp, "panel-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extensions().await.unwrap();
    host.disable_extension("panel-ext").unwrap();

    let panel_exts = host.get_side_panel_extensions();
    assert!(panel_exts.is_empty());
}

#[tokio::test]
async fn get_action_extensions_returns_only_action_extensions() {
    let tmp = TempDir::new().unwrap();

    let manifest_with_action = make_manifest("WithAction", "1.0.0", &[], None, Some("popup.html"));
    create_extension_dir(&tmp, "has-action", &manifest_with_action);

    let manifest_no_action = make_manifest("NoAction", "1.0.0", &[], None, None);
    create_extension_dir(&tmp, "no-action", &manifest_no_action);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extensions().await.unwrap();

    let action_exts = host.get_action_extensions();
    assert_eq!(action_exts.len(), 1);
    assert_eq!(action_exts[0].id, "has-action");
}

// ---------------------------------------------------------------------------
// get_side_panel_html / get_popup_html
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_side_panel_html_returns_file_contents() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("Panel", "1.0.0", &["sidePanel"], Some("panel.html"), None);
    let ext_dir = create_extension_dir(&tmp, "panel-ext", &manifest);
    fs::write(ext_dir.join("panel.html"), "<h1>Side Panel</h1>").unwrap();

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let html = host.get_side_panel_html("panel-ext").unwrap();
    assert_eq!(html, "<h1>Side Panel</h1>");
}

#[tokio::test]
async fn get_side_panel_html_not_found_errors() {
    let tmp = TempDir::new().unwrap();
    let host = ExtensionHost::new(default_config(&tmp));
    let result = host.get_side_panel_html("ghost");
    assert!(matches!(result, Err(ExtensionError::NotFound(_))));
}

#[tokio::test]
async fn get_side_panel_html_no_panel_configured_errors() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("NoPanelExt", "1.0.0", &[], None, None);
    let ext_dir = create_extension_dir(&tmp, "no-panel-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let result = host.get_side_panel_html("no-panel-ext");
    assert!(matches!(result, Err(ExtensionError::ApiNotSupported(_))));
}

#[tokio::test]
async fn get_popup_html_returns_file_contents() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("Popup", "1.0.0", &[], None, Some("popup.html"));
    let ext_dir = create_extension_dir(&tmp, "popup-ext", &manifest);
    fs::write(ext_dir.join("popup.html"), "<h1>Popup</h1>").unwrap();

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let html = host.get_popup_html("popup-ext").unwrap();
    assert_eq!(html, "<h1>Popup</h1>");
}

#[tokio::test]
async fn get_popup_html_not_found_errors() {
    let tmp = TempDir::new().unwrap();
    let host = ExtensionHost::new(default_config(&tmp));
    let result = host.get_popup_html("ghost");
    assert!(matches!(result, Err(ExtensionError::NotFound(_))));
}

#[tokio::test]
async fn get_popup_html_no_action_configured_errors() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("NoPopup", "1.0.0", &[], None, None);
    let ext_dir = create_extension_dir(&tmp, "no-popup-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let result = host.get_popup_html("no-popup-ext");
    assert!(matches!(result, Err(ExtensionError::ApiNotSupported(_))));
}

// ---------------------------------------------------------------------------
// generate_api_polyfill
// ---------------------------------------------------------------------------

#[tokio::test]
async fn generate_api_polyfill_not_found_errors() {
    let tmp = TempDir::new().unwrap();
    let host = ExtensionHost::new(default_config(&tmp));
    let result = host.generate_api_polyfill("ghost");
    assert!(matches!(result, Err(ExtensionError::NotFound(_))));
}

#[tokio::test]
async fn generate_api_polyfill_contains_extension_id() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("PolyfillExt", "1.0.0", &[], None, None);
    let ext_dir = create_extension_dir(&tmp, "polyfill-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let polyfill = host.generate_api_polyfill("polyfill-ext").unwrap();
    assert!(polyfill.contains("polyfill-ext"));
}

#[tokio::test]
async fn generate_api_polyfill_with_storage_permission_includes_storage_api() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("StorageExt", "1.0.0", &["storage"], None, None);
    let ext_dir = create_extension_dir(&tmp, "storage-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let polyfill = host.generate_api_polyfill("storage-ext").unwrap();
    assert!(polyfill.contains("chrome.storage"));
}

#[tokio::test]
async fn generate_api_polyfill_without_storage_permission_excludes_storage_api() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("NoStorageExt", "1.0.0", &[], None, None);
    let ext_dir = create_extension_dir(&tmp, "no-storage-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let polyfill = host.generate_api_polyfill("no-storage-ext").unwrap();
    assert!(!polyfill.contains("chrome.storage = storage"));
}

#[tokio::test]
async fn generate_api_polyfill_with_tabs_permission_includes_tabs_api() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("TabsExt", "1.0.0", &["tabs"], None, None);
    let ext_dir = create_extension_dir(&tmp, "tabs-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let polyfill = host.generate_api_polyfill("tabs-ext").unwrap();
    assert!(polyfill.contains("chrome.tabs"));
}

// ---------------------------------------------------------------------------
// LoadedExtension: read_resource / read_resource_bytes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn loaded_ext_read_resource_returns_content() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("ReadRes", "1.0.0", &[], None, None);
    let ext_dir = create_extension_dir(&tmp, "read-res-ext", &manifest);
    fs::write(ext_dir.join("data.txt"), "hello resource").unwrap();

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let ext = host.get_extension("read-res-ext").unwrap();
    let content = ext.read_resource("data.txt").unwrap();
    assert_eq!(content, "hello resource");
}

#[tokio::test]
async fn loaded_ext_read_resource_missing_file_errors() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("ReadResMiss", "1.0.0", &[], None, None);
    let ext_dir = create_extension_dir(&tmp, "read-miss-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let ext = host.get_extension("read-miss-ext").unwrap();
    let result = ext.read_resource("nonexistent.txt");
    assert!(matches!(result, Err(ExtensionError::Io(_))));
}

#[tokio::test]
async fn loaded_ext_read_resource_bytes_returns_bytes() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("ReadBytes", "1.0.0", &[], None, None);
    let ext_dir = create_extension_dir(&tmp, "read-bytes-ext", &manifest);
    fs::write(ext_dir.join("icon.bin"), b"ICON_DATA" as &[u8]).unwrap();

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let ext = host.get_extension("read-bytes-ext").unwrap();
    let bytes = ext.read_resource_bytes("icon.bin").unwrap();
    assert_eq!(bytes, b"ICON_DATA");
}

#[tokio::test]
async fn loaded_ext_read_resource_bytes_missing_file_errors() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("BytesMiss", "1.0.0", &[], None, None);
    let ext_dir = create_extension_dir(&tmp, "bytes-miss-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let ext = host.get_extension("bytes-miss-ext").unwrap();
    let result = ext.read_resource_bytes("missing.bin");
    assert!(matches!(result, Err(ExtensionError::Io(_))));
}

// ---------------------------------------------------------------------------
// LoadedExtension: get_resource_path / get_side_panel_path / get_popup_path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn loaded_ext_get_resource_path_appends_relative_path() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("PathExt", "1.0.0", &[], None, None);
    let ext_dir = create_extension_dir(&tmp, "path-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let ext = host.get_extension("path-ext").unwrap();
    let path = ext.get_resource_path("assets/icon.png");
    assert!(path.ends_with("assets/icon.png"));
}

#[tokio::test]
async fn loaded_ext_get_side_panel_path_some_when_configured() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest(
        "PanelPath",
        "1.0.0",
        &["sidePanel"],
        Some("panel.html"),
        None,
    );
    let ext_dir = create_extension_dir(&tmp, "panel-path-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let ext = host.get_extension("panel-path-ext").unwrap();
    let path = ext.get_side_panel_path();
    assert!(path.is_some());
    assert!(path.unwrap().ends_with("panel.html"));
}

#[tokio::test]
async fn loaded_ext_get_popup_path_some_when_configured() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("PopupPath", "1.0.0", &[], None, Some("popup.html"));
    let ext_dir = create_extension_dir(&tmp, "popup-path-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let ext = host.get_extension("popup-path-ext").unwrap();
    let path = ext.get_popup_path();
    assert!(path.is_some());
    assert!(path.unwrap().ends_with("popup.html"));
}

#[tokio::test]
async fn loaded_ext_get_side_panel_path_none_when_not_configured() {
    let tmp = TempDir::new().unwrap();
    let manifest = make_manifest("NoPanelPath", "1.0.0", &[], None, None);
    let ext_dir = create_extension_dir(&tmp, "no-panel-path-ext", &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let ext = host.get_extension("no-panel-path-ext").unwrap();
    assert!(ext.get_side_panel_path().is_none());
}

// ---------------------------------------------------------------------------
// storage accessor
// ---------------------------------------------------------------------------

#[tokio::test]
async fn host_storage_returns_arc() {
    let tmp = TempDir::new().unwrap();
    let host = ExtensionHost::new(default_config(&tmp));
    // Just verify it's accessible; StorageBackend doesn't expose meaningful ops without a key
    let _storage = host.storage();
}

// ---------------------------------------------------------------------------
// rstest parametric: permission polyfill inclusion
// ---------------------------------------------------------------------------

#[rstest]
#[case("storage", "chrome.storage")]
#[case("tabs", "chrome.tabs")]
#[case("sidePanel", "chrome.sidePanel")]
#[case("scripting", "chrome.scripting")]
#[tokio::test]
async fn generate_api_polyfill_permission_includes_api(
    #[case] permission: &str,
    #[case] expected_snippet: &str,
) {
    let tmp = TempDir::new().unwrap();
    let dir_name = format!("perm-{}", permission.to_lowercase().replace("sidepanel", "sp"));
    let manifest = make_manifest("PermExt", "1.0.0", &[permission], None, None);
    let ext_dir = create_extension_dir(&tmp, &dir_name, &manifest);

    let host = ExtensionHost::new(default_config(&tmp));
    host.load_extension(&ext_dir).await.unwrap();

    let polyfill = host.generate_api_polyfill(&dir_name).unwrap();
    assert!(
        polyfill.contains(expected_snippet),
        "Expected polyfill to contain '{}' for permission '{}'",
        expected_snippet,
        permission
    );
}
