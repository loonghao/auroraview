//! Integration tests for auroraview-settings
//!
//! Covers SettingValue conversions, SettingsStore operations,
//! SchemaRegistry validation, and SettingsManager lifecycle.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use auroraview_settings::{SchemaRegistry, SettingValue, SettingsManager, SettingsStore, SettingSchema};
use rstest::rstest;

// ---------------------------------------------------------------------------
// SettingValue — type accessors and conversions
// ---------------------------------------------------------------------------

#[test]
fn setting_value_bool_roundtrip() {
    let v = SettingValue::Bool(true);
    assert_eq!(v.as_bool(), Some(true));
    assert_eq!(v.type_name(), "bool");
    assert!(!v.is_null());
}

#[test]
fn setting_value_integer_roundtrip() {
    let v: SettingValue = 42i64.into();
    assert_eq!(v.as_integer(), Some(42));
    assert_eq!(v.as_float(), Some(42.0));
    assert_eq!(v.type_name(), "integer");
}

#[test]
fn setting_value_i32_converts_to_integer() {
    let v: SettingValue = 7i32.into();
    assert_eq!(v.as_integer(), Some(7));
}

#[test]
fn setting_value_float_roundtrip() {
    let v: SettingValue = 1.5f64.into();
    assert!((v.as_float().unwrap() - 1.5).abs() < 1e-10);
    assert_eq!(v.type_name(), "float");
}

#[test]
fn setting_value_string_roundtrip() {
    let v: SettingValue = "hello".into();
    assert_eq!(v.as_str(), Some("hello"));
    assert_eq!(v.type_name(), "string");
}

#[test]
fn setting_value_string_owned() {
    let v: SettingValue = String::from("world").into();
    assert_eq!(v.as_str(), Some("world"));
}

#[test]
fn setting_value_null_is_null() {
    let v = SettingValue::Null;
    assert!(v.is_null());
    assert_eq!(v.type_name(), "null");
}

#[test]
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

#[test]
fn setting_value_array_from_vec() {
    let v: SettingValue = vec![1i64, 2, 3].into();
    let arr = v.as_array().expect("should be array");
    assert_eq!(arr.len(), 3);
}

// ---------------------------------------------------------------------------
// SettingsStore — CRUD and iteration
// ---------------------------------------------------------------------------

#[test]
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

#[test]
fn store_keys_with_prefix() {
    let mut store = SettingsStore::new();
    store.set("browser.homepage", SettingValue::String("https://example.com".into()));
    store.set("browser.timeout", SettingValue::Integer(30));
    store.set("appearance.theme", SettingValue::String("light".into()));

    let browser_keys: Vec<&str> = store.keys_with_prefix("browser.").collect();
    assert_eq!(browser_keys.len(), 2);
    assert!(browser_keys.contains(&"browser.homepage"));
    assert!(browser_keys.contains(&"browser.timeout"));
}

#[test]
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

#[test]
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

#[test]
fn store_clear() {
    let mut store = SettingsStore::new();
    store.set("x", SettingValue::Bool(true));
    store.set("y", SettingValue::Bool(false));
    store.clear();
    assert!(store.is_empty());
}

#[test]
fn store_into_iter() {
    let mut store = SettingsStore::new();
    store.set("a", SettingValue::Integer(1));
    store.set("b", SettingValue::Integer(2));

    let count = store.into_iter().count();
    assert_eq!(count, 2);
}

#[test]
fn store_ref_iter() {
    let mut store = SettingsStore::new();
    store.set("p", SettingValue::Bool(true));

    let count = store.into_iter().count();
    assert_eq!(count, 1);
}

#[test]
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

#[test]
fn schema_bool_validates_bool_value() {
    let schema = bool_schema("test.bool");
    assert!(schema.validate(&SettingValue::Bool(true)).is_ok());
    assert!(schema.validate(&SettingValue::Bool(false)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(1)).is_err());
}

#[test]
fn schema_integer_validates_in_bounds() {
    let schema = int_schema("test.int", Some(0), Some(100));
    assert!(schema.validate(&SettingValue::Integer(50)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(0)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(100)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(-1)).is_err());
    assert!(schema.validate(&SettingValue::Integer(101)).is_err());
}

#[test]
fn schema_integer_no_bounds() {
    let schema = int_schema("test.int_unbounded", None, None);
    assert!(schema.validate(&SettingValue::Integer(i64::MAX)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(i64::MIN)).is_ok());
}

#[test]
fn schema_enum_validates_allowed_value() {
    let schema = enum_schema("test.theme", vec!["light".into(), "dark".into()]);
    assert!(schema.validate(&SettingValue::String("dark".into())).is_ok());
    assert!(schema.validate(&SettingValue::String("light".into())).is_ok());
    assert!(schema.validate(&SettingValue::String("solarized".into())).is_err());
}

#[test]
fn schema_string_max_length() {
    let schema = SettingSchema::builder("test.label")
        .title("Label")
        .description("label")
        .string_type()
        .max_length(10)
        .default("")
        .build();
    assert!(schema.validate(&SettingValue::String("short".into())).is_ok());
    assert!(schema.validate(&SettingValue::String("this_is_too_long_string".into())).is_err());
}

#[test]
fn registry_get_by_key() {
    let mut reg = SchemaRegistry::new();
    reg.register(bool_schema("nav.sidebar"));
    assert!(reg.get("nav.sidebar").is_some());
    assert!(reg.get("nav.toolbar").is_none());
}

#[test]
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

#[test]
fn manager_set_get_string() {
    let mgr = SettingsManager::new();
    mgr.set("ui.theme", SettingValue::String("dark".into())).unwrap();
    assert_eq!(mgr.get_string("ui.theme"), Some("dark".to_string()));
}

#[test]
fn manager_set_get_bool() {
    let mgr = SettingsManager::new();
    mgr.set("privacy.dnt", SettingValue::Bool(true)).unwrap();
    assert_eq!(mgr.get_bool("privacy.dnt"), Some(true));
}

#[test]
fn manager_set_get_integer() {
    let mgr = SettingsManager::new();
    mgr.set("perf.workers", SettingValue::Integer(4)).unwrap();
    assert_eq!(mgr.get_integer("perf.workers"), Some(4));
}

#[test]
fn manager_set_get_float() {
    let mgr = SettingsManager::new();
    mgr.set("audio.volume", SettingValue::Float(0.75)).unwrap();
    let v = mgr.get_float("audio.volume").unwrap();
    assert!((v - 0.75).abs() < 1e-10);
}

#[test]
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

#[test]
fn manager_set_overrides_default() {
    let mgr = SettingsManager::new();

    let schema = SettingSchema::builder("ui.theme")
        .title("Theme")
        .description("Color theme")
        .enum_type(vec!["light".into(), "dark".into()])
        .default("light")
        .build();

    mgr.register_schema(schema);
    mgr.set("ui.theme", SettingValue::String("dark".into())).unwrap();
    assert_eq!(mgr.get_string("ui.theme"), Some("dark".to_string()));
}

#[test]
fn manager_reset_restores_default() {
    let mgr = SettingsManager::new();

    let schema = SettingSchema::builder("ui.theme")
        .title("Theme")
        .description("Color theme")
        .enum_type(vec!["light".into(), "dark".into()])
        .default("light")
        .build();

    mgr.register_schema(schema);
    mgr.set("ui.theme", SettingValue::String("dark".into())).unwrap();
    mgr.reset("ui.theme").unwrap();

    // After reset, default should be served
    assert_eq!(mgr.get_string("ui.theme"), Some("light".to_string()));
}

#[test]
fn manager_reset_all_clears_user_overrides() {
    let mgr = SettingsManager::new();
    mgr.set("a", SettingValue::Bool(true)).unwrap();
    mgr.set("b", SettingValue::Integer(5)).unwrap();
    mgr.reset_all();
    assert!(mgr.user_settings().is_empty());
}

#[test]
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

#[test]
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

#[test]
fn manager_schema_validation_rejects_out_of_range() {
    let mgr = SettingsManager::new();

    let schema = int_schema("volume", Some(0), Some(100));
    mgr.register_schema(schema);

    let result = mgr.set("volume", SettingValue::Integer(150));
    assert!(result.is_err());
}

#[test]
fn manager_schema_validation_accepts_valid_value() {
    let mgr = SettingsManager::new();

    let schema = int_schema("volume", Some(0), Some(100));
    mgr.register_schema(schema);

    assert!(mgr.set("volume", SettingValue::Integer(80)).is_ok());
    assert_eq!(mgr.get_integer("volume"), Some(80));
}

#[test]
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
    mgr.set("ui.font", SettingValue::String("mono".into())).unwrap();

    let all = mgr.all_settings();
    // Default from schema should be included
    assert!(all.get("ui.color").is_some());
    // User setting should be included
    assert_eq!(all.get("ui.font").unwrap().as_str(), Some("mono"));
}

#[test]
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

#[test]
fn manager_save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let mgr = SettingsManager::with_storage(&path);
    mgr.set("perf.workers", SettingValue::Integer(8)).unwrap();
    mgr.set("ui.theme", SettingValue::String("dark".into())).unwrap();
    mgr.save().unwrap();

    // Load into a fresh manager
    let mgr2 = SettingsManager::with_storage(&path);
    mgr2.load().unwrap();
    assert_eq!(mgr2.get_integer("perf.workers"), Some(8));
    assert_eq!(mgr2.get_string("ui.theme"), Some("dark".to_string()));
}

#[test]
fn manager_load_nonexistent_path_is_ok() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.json");
    let mgr = SettingsManager::with_storage(&path);
    assert!(mgr.load().is_ok());
}

#[test]
fn manager_save_no_storage_path_is_ok() {
    let mgr = SettingsManager::new();
    assert!(mgr.save().is_ok());
}
