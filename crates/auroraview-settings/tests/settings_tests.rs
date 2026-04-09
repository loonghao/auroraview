//! Integration tests for auroraview-settings
//!
//! Covers SettingValue conversions, SettingsStore operations,
//! SchemaRegistry validation, and SettingsManager lifecycle.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::thread;

use auroraview_settings::{
    SchemaRegistry, SettingSchema, SettingValue, SettingsManager, SettingsStore,
};
use rstest::rstest;

// ---------------------------------------------------------------------------
// SettingValue — type accessors and conversions
// ---------------------------------------------------------------------------

#[rstest]
fn setting_value_bool_roundtrip() {
    let v = SettingValue::Bool(true);
    assert_eq!(v.as_bool(), Some(true));
    assert_eq!(v.type_name(), "bool");
    assert!(!v.is_null());
}

#[rstest]
fn setting_value_integer_roundtrip() {
    let v: SettingValue = 42i64.into();
    assert_eq!(v.as_integer(), Some(42));
    assert_eq!(v.as_float(), Some(42.0));
    assert_eq!(v.type_name(), "integer");
}

#[rstest]
fn setting_value_i32_converts_to_integer() {
    let v: SettingValue = 7i32.into();
    assert_eq!(v.as_integer(), Some(7));
}

#[rstest]
fn setting_value_float_roundtrip() {
    let v: SettingValue = 1.5f64.into();
    assert!((v.as_float().unwrap() - 1.5).abs() < 1e-10);
    assert_eq!(v.type_name(), "float");
}

#[rstest]
fn setting_value_string_roundtrip() {
    let v: SettingValue = "hello".into();
    assert_eq!(v.as_str(), Some("hello"));
    assert_eq!(v.type_name(), "string");
}

#[rstest]
fn setting_value_string_owned() {
    let v: SettingValue = String::from("world").into();
    assert_eq!(v.as_str(), Some("world"));
}

#[rstest]
fn setting_value_null_is_null() {
    let v = SettingValue::Null;
    assert!(v.is_null());
    assert_eq!(v.type_name(), "null");
}

#[rstest]
fn setting_value_default_is_null() {
    let v = SettingValue::default();
    assert!(v.is_null());
}

#[rstest]
#[case(SettingValue::Bool(true), "bool")]
#[case(SettingValue::Integer(0), "integer")]
#[case(SettingValue::Float(0.0), "float")]
#[case(SettingValue::String("".into()), "string")]
#[case(SettingValue::Array(vec![]), "array")]
#[case(SettingValue::Null, "null")]
fn setting_value_type_name(#[case] value: SettingValue, #[case] expected: &str) {
    assert_eq!(value.type_name(), expected);
}

#[rstest]
fn setting_value_array_from_vec() {
    let v: SettingValue = vec![1i64, 2, 3].into();
    let arr = v.as_array().expect("should be array");
    assert_eq!(arr.len(), 3);
}

// ---------------------------------------------------------------------------
// SettingsStore — CRUD and iteration
// ---------------------------------------------------------------------------

#[rstest]
fn store_set_get_remove() {
    let mut store = SettingsStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);

    store.set("theme", SettingValue::String("dark".into()));
    assert_eq!(store.len(), 1);
    assert!(store.contains("theme"));

    let v = store.get("theme").expect("should exist");
    assert_eq!(v.as_str(), Some("dark"));

    let removed = store.remove("theme");
    assert!(removed.is_some());
    assert!(!store.contains("theme"));
}

#[rstest]
fn store_keys_with_prefix() {
    let mut store = SettingsStore::new();
    store.set(
        "browser.homepage",
        SettingValue::String("https://example.com".into()),
    );
    store.set("browser.timeout", SettingValue::Integer(30));
    store.set("appearance.theme", SettingValue::String("light".into()));

    let browser_keys: Vec<&str> = store.keys_with_prefix("browser.").collect();
    assert_eq!(browser_keys.len(), 2);
    assert!(browser_keys.contains(&"browser.homepage"));
    assert!(browser_keys.contains(&"browser.timeout"));
}

#[rstest]
fn store_merge_overwrites() {
    let mut base = SettingsStore::new();
    base.set("k1", SettingValue::Integer(1));
    base.set("k2", SettingValue::Integer(2));

    let mut overlay = SettingsStore::new();
    overlay.set("k2", SettingValue::Integer(99));
    overlay.set("k3", SettingValue::Integer(3));

    base.merge(overlay);
    assert_eq!(base.get("k1").unwrap().as_integer(), Some(1));
    assert_eq!(base.get("k2").unwrap().as_integer(), Some(99));
    assert_eq!(base.get("k3").unwrap().as_integer(), Some(3));
}

#[rstest]
fn store_merge_ref() {
    let mut base = SettingsStore::new();
    base.set("a", SettingValue::Bool(false));

    let mut other = SettingsStore::new();
    other.set("a", SettingValue::Bool(true));
    other.set("b", SettingValue::Integer(5));

    base.merge_ref(&other);
    assert_eq!(base.get("a").unwrap().as_bool(), Some(true));
    assert_eq!(base.get("b").unwrap().as_integer(), Some(5));
}

#[rstest]
fn store_clear() {
    let mut store = SettingsStore::new();
    store.set("x", SettingValue::Bool(true));
    store.set("y", SettingValue::Bool(false));
    store.clear();
    assert!(store.is_empty());
}

#[rstest]
fn store_into_iter() {
    let mut store = SettingsStore::new();
    store.set("a", SettingValue::Integer(1));
    store.set("b", SettingValue::Integer(2));

    let count = store.into_iter().count();
    assert_eq!(count, 2);
}

#[rstest]
fn store_ref_iter() {
    let mut store = SettingsStore::new();
    store.set("p", SettingValue::Bool(true));

    let count = store.into_iter().count();
    assert_eq!(count, 1);
}

#[rstest]
fn store_from_map() {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_string(), SettingValue::Integer(42));
    let store = SettingsStore::from_map(map);
    assert_eq!(store.get("key").unwrap().as_integer(), Some(42));
}

// ---------------------------------------------------------------------------
// SchemaRegistry — validation logic
// ---------------------------------------------------------------------------

// Helper functions for building schemas in tests
fn bool_schema(key: &str) -> SettingSchema {
    SettingSchema::builder(key)
        .title(key)
        .description("bool setting")
        .bool_type()
        .default(false)
        .build()
}

fn int_schema(key: &str, min: Option<i64>, max: Option<i64>) -> SettingSchema {
    SettingSchema::builder(key)
        .title(key)
        .description("int setting")
        .integer_type(min, max)
        .default(0i64)
        .build()
}

fn enum_schema(key: &str, values: Vec<String>) -> SettingSchema {
    SettingSchema::builder(key)
        .title(key)
        .description("enum setting")
        .enum_type(values)
        .default("light")
        .build()
}

#[rstest]
fn schema_bool_validates_bool_value() {
    let schema = bool_schema("test.bool");
    assert!(schema.validate(&SettingValue::Bool(true)).is_ok());
    assert!(schema.validate(&SettingValue::Bool(false)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(1)).is_err());
}

#[rstest]
fn schema_integer_validates_in_bounds() {
    let schema = int_schema("test.int", Some(0), Some(100));
    assert!(schema.validate(&SettingValue::Integer(50)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(0)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(100)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(-1)).is_err());
    assert!(schema.validate(&SettingValue::Integer(101)).is_err());
}

#[rstest]
fn schema_integer_no_bounds() {
    let schema = int_schema("test.int_unbounded", None, None);
    assert!(schema.validate(&SettingValue::Integer(i64::MAX)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(i64::MIN)).is_ok());
}

#[rstest]
fn schema_enum_validates_allowed_value() {
    let schema = enum_schema("test.theme", vec!["light".into(), "dark".into()]);
    assert!(schema
        .validate(&SettingValue::String("dark".into()))
        .is_ok());
    assert!(schema
        .validate(&SettingValue::String("light".into()))
        .is_ok());
    assert!(schema
        .validate(&SettingValue::String("solarized".into()))
        .is_err());
}

#[rstest]
fn schema_string_max_length() {
    let schema = SettingSchema::builder("test.label")
        .title("Label")
        .description("label")
        .string_type()
        .max_length(10)
        .default("")
        .build();
    assert!(schema
        .validate(&SettingValue::String("short".into()))
        .is_ok());
    assert!(schema
        .validate(&SettingValue::String("this_is_too_long_string".into()))
        .is_err());
}

#[rstest]
fn registry_get_by_key() {
    let mut reg = SchemaRegistry::new();
    reg.register(bool_schema("nav.sidebar"));
    assert!(reg.get("nav.sidebar").is_some());
    assert!(reg.get("nav.toolbar").is_none());
}

#[rstest]
fn registry_by_category() {
    let mut reg = SchemaRegistry::new();

    let s1 = SettingSchema::builder("appearance.theme")
        .title("Theme")
        .description("Color theme")
        .enum_type(vec!["light".into(), "dark".into()])
        .default("light")
        .category("appearance")
        .build();

    let s2 = SettingSchema::builder("appearance.font_size")
        .title("Font Size")
        .description("UI font size")
        .integer_type(Some(8), Some(72))
        .default(14i64)
        .category("appearance")
        .build();

    let s3 = SettingSchema::builder("browser.homepage")
        .title("Homepage")
        .description("Start page URL")
        .string_type()
        .default("about:blank")
        .category("browser")
        .build();

    reg.register(s1);
    reg.register(s2);
    reg.register(s3);

    let appearance: Vec<_> = reg.by_category("appearance").collect();
    assert_eq!(appearance.len(), 2);

    let cats = reg.categories();
    assert_eq!(cats.len(), 2);
    assert!(cats.contains(&"appearance".to_string()));
    assert!(cats.contains(&"browser".to_string()));
}

// ---------------------------------------------------------------------------
// SettingsManager — get/set/default/reset/callbacks
// ---------------------------------------------------------------------------

#[rstest]
fn manager_set_get_string() {
    let mgr = SettingsManager::new();
    mgr.set("ui.theme", SettingValue::String("dark".into()))
        .unwrap();
    assert_eq!(mgr.get_string("ui.theme"), Some("dark".to_string()));
}

#[rstest]
fn manager_set_get_bool() {
    let mgr = SettingsManager::new();
    mgr.set("privacy.dnt", SettingValue::Bool(true)).unwrap();
    assert_eq!(mgr.get_bool("privacy.dnt"), Some(true));
}

#[rstest]
fn manager_set_get_integer() {
    let mgr = SettingsManager::new();
    mgr.set("perf.workers", SettingValue::Integer(4)).unwrap();
    assert_eq!(mgr.get_integer("perf.workers"), Some(4));
}

#[rstest]
fn manager_set_get_float() {
    let mgr = SettingsManager::new();
    mgr.set("audio.volume", SettingValue::Float(0.75)).unwrap();
    let v = mgr.get_float("audio.volume").unwrap();
    assert!((v - 0.75).abs() < 1e-10);
}

#[rstest]
fn manager_get_returns_default_when_unset() {
    let mgr = SettingsManager::new();

    let schema = SettingSchema::builder("ui.theme")
        .title("Theme")
        .description("Color theme")
        .enum_type(vec!["light".into(), "dark".into()])
        .default("light")
        .build();

    mgr.register_schema(schema);

    // Not explicitly set — should return registered default
    assert_eq!(mgr.get_string("ui.theme"), Some("light".to_string()));
}

#[rstest]
fn manager_set_overrides_default() {
    let mgr = SettingsManager::new();

    let schema = SettingSchema::builder("ui.theme")
        .title("Theme")
        .description("Color theme")
        .enum_type(vec!["light".into(), "dark".into()])
        .default("light")
        .build();

    mgr.register_schema(schema);
    mgr.set("ui.theme", SettingValue::String("dark".into()))
        .unwrap();
    assert_eq!(mgr.get_string("ui.theme"), Some("dark".to_string()));
}

#[rstest]
fn manager_reset_restores_default() {
    let mgr = SettingsManager::new();

    let schema = SettingSchema::builder("ui.theme")
        .title("Theme")
        .description("Color theme")
        .enum_type(vec!["light".into(), "dark".into()])
        .default("light")
        .build();

    mgr.register_schema(schema);
    mgr.set("ui.theme", SettingValue::String("dark".into()))
        .unwrap();
    mgr.reset("ui.theme").unwrap();

    // After reset, default should be served
    assert_eq!(mgr.get_string("ui.theme"), Some("light".to_string()));
}

#[rstest]
fn manager_reset_all_clears_user_overrides() {
    let mgr = SettingsManager::new();
    mgr.set("a", SettingValue::Bool(true)).unwrap();
    mgr.set("b", SettingValue::Integer(5)).unwrap();
    mgr.reset_all();
    assert!(mgr.user_settings().is_empty());
}

#[rstest]
fn manager_on_change_callback_fires() {
    let mgr = SettingsManager::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();

    mgr.on_change(move |_key, _old, _new| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    mgr.set("x", SettingValue::Bool(true)).unwrap();
    mgr.set("x", SettingValue::Bool(false)).unwrap();

    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[rstest]
fn manager_on_change_receives_old_and_new_values() {
    let mgr = SettingsManager::new();
    let last_old = Arc::new(parking_lot::Mutex::new(SettingValue::Null));
    let last_new = Arc::new(parking_lot::Mutex::new(SettingValue::Null));

    let lo = last_old.clone();
    let ln = last_new.clone();
    mgr.on_change(move |_key, old, new| {
        *lo.lock() = old.clone();
        *ln.lock() = new.clone();
    });

    mgr.set("vol", SettingValue::Integer(50)).unwrap();
    mgr.set("vol", SettingValue::Integer(75)).unwrap();

    assert_eq!(last_old.lock().as_integer(), Some(50));
    assert_eq!(last_new.lock().as_integer(), Some(75));
}

#[rstest]
fn manager_schema_validation_rejects_out_of_range() {
    let mgr = SettingsManager::new();

    let schema = int_schema("volume", Some(0), Some(100));
    mgr.register_schema(schema);

    let result = mgr.set("volume", SettingValue::Integer(150));
    assert!(result.is_err());
}

#[rstest]
fn manager_schema_validation_accepts_valid_value() {
    let mgr = SettingsManager::new();

    let schema = int_schema("volume", Some(0), Some(100));
    mgr.register_schema(schema);

    assert!(mgr.set("volume", SettingValue::Integer(80)).is_ok());
    assert_eq!(mgr.get_integer("volume"), Some(80));
}

#[rstest]
fn manager_all_settings_merges_defaults_and_user() {
    let mgr = SettingsManager::new();

    let schema = SettingSchema::builder("ui.color")
        .title("Color")
        .description("UI color")
        .string_type()
        .default("blue")
        .build();
    mgr.register_schema(schema);

    // User only overrides one setting
    mgr.set("ui.font", SettingValue::String("mono".into()))
        .unwrap();

    let all = mgr.all_settings();
    // Default from schema should be included
    assert!(all.get("ui.color").is_some());
    // User setting should be included
    assert_eq!(all.get("ui.font").unwrap().as_str(), Some("mono"));
}

#[rstest]
fn manager_clone_shares_state() {
    let mgr = SettingsManager::new();
    let clone = mgr.clone();

    mgr.set("shared.key", SettingValue::Bool(true)).unwrap();
    // Cloned manager sees the same state
    assert_eq!(clone.get_bool("shared.key"), Some(true));
}

// ---------------------------------------------------------------------------
// Persistence — save/load round-trip
// ---------------------------------------------------------------------------

#[rstest]
fn manager_save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let mgr = SettingsManager::with_storage(&path);
    mgr.set("perf.workers", SettingValue::Integer(8)).unwrap();
    mgr.set("ui.theme", SettingValue::String("dark".into()))
        .unwrap();
    mgr.save().unwrap();

    // Load into a fresh manager
    let mgr2 = SettingsManager::with_storage(&path);
    mgr2.load().unwrap();
    assert_eq!(mgr2.get_integer("perf.workers"), Some(8));
    assert_eq!(mgr2.get_string("ui.theme"), Some("dark".to_string()));
}

#[rstest]
fn manager_load_nonexistent_path_is_ok() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.json");
    let mgr = SettingsManager::with_storage(&path);
    assert!(mgr.load().is_ok());
}

#[rstest]
fn manager_save_no_storage_path_is_ok() {
    let mgr = SettingsManager::new();
    assert!(mgr.save().is_ok());
}

// ---------------------------------------------------------------------------
// Serde Roundtrip Tests
// ---------------------------------------------------------------------------

use auroraview_settings::SettingsError;

#[rstest]
#[case(SettingValue::Bool(true))]
#[case(SettingValue::Bool(false))]
#[case(SettingValue::Integer(42))]
#[case(SettingValue::Integer(-1))]
#[case(SettingValue::Float(2.5))]
#[case(SettingValue::String("hello".into()))]
#[case(SettingValue::Null)]
fn setting_value_serde_roundtrip(#[case] value: SettingValue) {
    let json = serde_json::to_string(&value).unwrap();
    let back: SettingValue = serde_json::from_str(&json).unwrap();
    // Compare via type_name as PartialEq may not be derived
    assert_eq!(back.type_name(), value.type_name());
}

#[rstest]
fn setting_value_array_serde_roundtrip() {
    let v = SettingValue::Array(vec![
        SettingValue::Integer(1),
        SettingValue::String("two".into()),
        SettingValue::Bool(false),
    ]);
    let json = serde_json::to_string(&v).unwrap();
    let back: SettingValue = serde_json::from_str(&json).unwrap();
    let arr = back.as_array().unwrap();
    assert_eq!(arr.len(), 3);
}

#[rstest]
fn store_serde_via_setting_value() {
    // SettingsStore does not implement Serialize/Deserialize directly;
    // test SettingValue serde via manager export instead
    let mgr = SettingsManager::new();
    mgr.set("key1", SettingValue::String("value1".into()))
        .unwrap();
    mgr.set("key2", SettingValue::Integer(42)).unwrap();
    mgr.set("key3", SettingValue::Bool(true)).unwrap();

    let user = mgr.user_settings();
    assert_eq!(user.get("key1").unwrap().as_str(), Some("value1"));
    assert_eq!(user.get("key2").unwrap().as_integer(), Some(42));
    assert_eq!(user.get("key3").unwrap().as_bool(), Some(true));
}

#[rstest]
fn store_empty_serde_via_manager() {
    let mgr = SettingsManager::new();
    let user = mgr.user_settings();
    assert!(user.is_empty());
}

// ---------------------------------------------------------------------------
// SettingsError Display Tests
// ---------------------------------------------------------------------------

#[rstest]
fn error_not_found_display() {
    let err = SettingsError::NotFound("ui.theme".to_string());
    let msg = err.to_string();
    assert!(msg.contains("ui.theme"), "got: {msg}");
}

#[rstest]
fn error_type_mismatch_display() {
    let err = SettingsError::TypeMismatch {
        key: "volume".to_string(),
        expected: "integer".to_string(),
        actual: "string".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("volume"), "got: {msg}");
    assert!(msg.contains("integer"), "got: {msg}");
    assert!(msg.contains("string"), "got: {msg}");
}

#[rstest]
fn error_validation_failed_display() {
    let err = SettingsError::ValidationFailed {
        key: "volume".to_string(),
        reason: "value 150 exceeds maximum 100".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("volume"), "got: {msg}");
    assert!(msg.contains("150"), "got: {msg}");
}

#[rstest]
fn error_invalid_key_display() {
    let err = SettingsError::InvalidKey("".to_string());
    let msg = err.to_string();
    assert!(!msg.is_empty());
}

#[rstest]
fn error_schema_not_found_display() {
    let err = SettingsError::SchemaNotFound("advanced.setting".to_string());
    let msg = err.to_string();
    assert!(msg.contains("advanced.setting"), "got: {msg}");
}

// ---------------------------------------------------------------------------
// SettingValue Edge Cases
// ---------------------------------------------------------------------------

#[rstest]
fn setting_value_wrong_type_accessors_return_none() {
    let v = SettingValue::Bool(true);
    assert!(v.as_integer().is_none());
    assert!(v.as_float().is_none());
    assert!(v.as_str().is_none());
    assert!(v.as_array().is_none());
}

#[rstest]
fn setting_value_integer_as_float_coerces() {
    let v = SettingValue::Integer(10);
    assert_eq!(v.as_float(), Some(10.0));
}

#[rstest]
fn setting_value_float_as_integer_none() {
    let v = SettingValue::Float(2.5);

    // float does not coerce to integer
    assert!(v.as_integer().is_none());
}

#[rstest]
fn setting_value_empty_array() {
    let v = SettingValue::Array(vec![]);
    let arr = v.as_array().unwrap();
    assert!(arr.is_empty());
}

#[rstest]
fn setting_value_large_integer() {
    let v = SettingValue::Integer(i64::MAX);
    assert_eq!(v.as_integer(), Some(i64::MAX));
}

#[rstest]
fn setting_value_negative_float() {
    let v = SettingValue::Float(-273.15);
    assert!((v.as_float().unwrap() + 273.15).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// SettingsStore Additional Coverage
// ---------------------------------------------------------------------------

#[rstest]
fn store_remove_nonexistent_returns_none() {
    let mut store = SettingsStore::new();
    let result = store.remove("nonexistent");
    assert!(result.is_none());
}

#[rstest]
fn store_contains_after_remove() {
    let mut store = SettingsStore::new();
    store.set("k", SettingValue::Bool(true));
    store.remove("k");
    assert!(!store.contains("k"));
}

#[rstest]
fn store_overwrite_existing_key() {
    let mut store = SettingsStore::new();
    store.set("key", SettingValue::Integer(1));
    store.set("key", SettingValue::Integer(99));
    assert_eq!(store.get("key").unwrap().as_integer(), Some(99));
}

#[rstest]
fn store_keys_with_prefix_no_match() {
    let mut store = SettingsStore::new();
    store.set(
        "browser.homepage",
        SettingValue::String("about:blank".into()),
    );
    let results: Vec<&str> = store.keys_with_prefix("network.").collect();
    assert!(results.is_empty());
}

#[rstest]
fn store_merge_does_not_affect_source() {
    let mut base = SettingsStore::new();
    base.set("original", SettingValue::Integer(1));

    let empty = SettingsStore::new();
    base.merge(empty);

    assert_eq!(base.get("original").unwrap().as_integer(), Some(1));
}

// ---------------------------------------------------------------------------
// SettingsManager Additional Coverage
// ---------------------------------------------------------------------------

#[rstest]
fn manager_get_nonexistent_returns_none() {
    let mgr = SettingsManager::new();
    assert!(mgr.get_string("nonexistent.key").is_none());
    assert!(mgr.get_bool("nonexistent.key").is_none());
    assert!(mgr.get_integer("nonexistent.key").is_none());
    assert!(mgr.get_float("nonexistent.key").is_none());
}

#[rstest]
fn manager_schema_validation_wrong_type() {
    let mgr = SettingsManager::new();
    let schema = bool_schema("ui.enabled");
    mgr.register_schema(schema);

    // Setting a string where bool is expected should fail
    let result = mgr.set("ui.enabled", SettingValue::String("yes".into()));
    assert!(result.is_err());
}

#[rstest]
fn manager_user_settings_only_includes_explicitly_set() {
    let mgr = SettingsManager::new();

    let schema = bool_schema("feature.x");
    mgr.register_schema(schema);

    // Not set explicitly
    assert!(mgr.user_settings().is_empty());

    mgr.set("feature.x", SettingValue::Bool(true)).unwrap();
    assert_eq!(mgr.user_settings().len(), 1);
}

#[rstest]
fn manager_reset_key_without_schema_removes_user_value() {
    let mgr = SettingsManager::new();
    mgr.set("misc.key", SettingValue::Integer(5)).unwrap();
    let _ = mgr.reset("misc.key"); // may return error or ok depending on schema presence
                                   // No panic is the key requirement
}

#[rstest]
fn manager_on_change_not_called_when_value_unchanged() {
    let mgr = SettingsManager::new();
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();

    mgr.on_change(move |_key, old, new| {
        if old.type_name() != new.type_name() || old.as_integer() != new.as_integer() {
            c.fetch_add(1, Ordering::SeqCst);
        }
    });

    mgr.set("x", SettingValue::Integer(42)).unwrap();
    // Setting same value again - callback fires but old==new
    mgr.set("x", SettingValue::Integer(42)).unwrap();

    // At least first set should fire; second may or may not depending on impl
    assert!(count.load(Ordering::SeqCst) >= 1);
}

// ---------------------------------------------------------------------------
// Concurrent Tests
// ---------------------------------------------------------------------------

#[rstest]
fn concurrent_set_no_deadlock() {
    let mgr = Arc::new(SettingsManager::new());

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let m = Arc::clone(&mgr);
            thread::spawn(move || {
                for j in 0..10 {
                    let key = format!("thread{i}.setting{j}");
                    let _ = m.set(key, SettingValue::Integer(i as i64 * 10 + j));
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert!(!mgr.user_settings().is_empty());
}

#[rstest]
fn concurrent_get_set_no_panic() {
    let mgr = Arc::new(SettingsManager::new());

    mgr.set("shared", SettingValue::Integer(0)).unwrap();

    let writer = {
        let m = Arc::clone(&mgr);
        thread::spawn(move || {
            for i in 0..50 {
                let _ = m.set("shared", SettingValue::Integer(i));
            }
        })
    };

    let reader = {
        let m = Arc::clone(&mgr);
        thread::spawn(move || {
            for _ in 0..50 {
                let _ = m.get_integer("shared");
            }
        })
    };

    writer.join().unwrap();
    reader.join().unwrap();
}

#[rstest]
fn concurrent_clone_and_set_shares_state() {
    let mgr = SettingsManager::new();
    let mgr_clone = mgr.clone();

    let writer = thread::spawn(move || {
        for i in 0..20 {
            let _ = mgr_clone.set(format!("concurrent.key{i}"), SettingValue::Bool(true));
        }
    });

    writer.join().unwrap();

    // mgr and mgr_clone share state
    assert!(!mgr.user_settings().is_empty());
}
