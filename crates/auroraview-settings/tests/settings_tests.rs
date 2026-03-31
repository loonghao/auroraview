use auroraview_settings::{
    SchemaRegistry, SettingSchema, SettingValue, SettingsManager, SettingsStore,
};
use rstest::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// ========== SettingValue Tests ==========

#[test]
fn test_value_type_names() {
    assert_eq!(SettingValue::Null.type_name(), "null");
    assert_eq!(SettingValue::Bool(true).type_name(), "bool");
    assert_eq!(SettingValue::Integer(42).type_name(), "integer");
    assert_eq!(SettingValue::Float(2.78).type_name(), "float");
    assert_eq!(SettingValue::String("hello".into()).type_name(), "string");
    assert_eq!(SettingValue::Array(vec![]).type_name(), "array");
    assert_eq!(
        SettingValue::Object(Default::default()).type_name(),
        "object"
    );
}

#[test]
fn test_value_is_null() {
    assert!(SettingValue::Null.is_null());
    assert!(!SettingValue::Bool(false).is_null());
}

#[test]
fn test_value_accessors() {
    assert_eq!(SettingValue::Bool(true).as_bool(), Some(true));
    assert_eq!(SettingValue::Integer(42).as_integer(), Some(42));
    assert_eq!(SettingValue::Float(2.78).as_float(), Some(2.78));
    assert_eq!(SettingValue::Integer(10).as_float(), Some(10.0));
    assert_eq!(
        SettingValue::String("hi".into()).as_str(),
        Some("hi")
    );

    // Wrong type returns None
    assert_eq!(SettingValue::Bool(true).as_integer(), None);
    assert_eq!(SettingValue::Integer(1).as_str(), None);
}

#[test]
fn test_value_from_conversions() {
    assert_eq!(SettingValue::from(true), SettingValue::Bool(true));
    assert_eq!(SettingValue::from(42i32), SettingValue::Integer(42));
    assert_eq!(SettingValue::from(100i64), SettingValue::Integer(100));
    assert_eq!(SettingValue::from(2.78), SettingValue::Float(2.78));
    assert_eq!(
        SettingValue::from("hello"),
        SettingValue::String("hello".into())
    );
    assert_eq!(
        SettingValue::from(String::from("world")),
        SettingValue::String("world".into())
    );
}

#[test]
fn test_value_from_vec() {
    let arr = SettingValue::from(vec![1i32, 2, 3]);
    assert_eq!(
        arr.as_array().unwrap().len(),
        3
    );
}

#[test]
fn test_value_default() {
    assert_eq!(SettingValue::default(), SettingValue::Null);
}

// ========== SettingsStore Tests ==========

#[test]
fn test_store_basic_operations() {
    let mut store = SettingsStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);

    store.set("key1", SettingValue::String("value1".into()));
    assert_eq!(store.len(), 1);
    assert!(!store.is_empty());
    assert!(store.contains("key1"));
    assert!(!store.contains("key2"));

    assert_eq!(
        store.get("key1"),
        Some(&SettingValue::String("value1".into()))
    );
    assert_eq!(store.get("key2"), None);
}

#[test]
fn test_store_remove() {
    let mut store = SettingsStore::new();
    store.set("key", SettingValue::Bool(true));

    let removed = store.remove("key");
    assert_eq!(removed, Some(SettingValue::Bool(true)));
    assert!(store.is_empty());

    assert_eq!(store.remove("nonexistent"), None);
}

#[test]
fn test_store_clear() {
    let mut store = SettingsStore::new();
    store.set("a", SettingValue::Integer(1));
    store.set("b", SettingValue::Integer(2));
    assert_eq!(store.len(), 2);

    store.clear();
    assert!(store.is_empty());
}

#[test]
fn test_store_keys_with_prefix() {
    let mut store = SettingsStore::new();
    store.set("app.theme", SettingValue::String("dark".into()));
    store.set("app.font", SettingValue::String("mono".into()));
    store.set("browser.homepage", SettingValue::String("about:blank".into()));

    let app_keys: Vec<&str> = store.keys_with_prefix("app.").collect();
    assert_eq!(app_keys.len(), 2);
}

#[test]
fn test_store_merge() {
    let mut store1 = SettingsStore::new();
    store1.set("a", SettingValue::Integer(1));
    store1.set("b", SettingValue::Integer(2));

    let mut store2 = SettingsStore::new();
    store2.set("b", SettingValue::Integer(20));
    store2.set("c", SettingValue::Integer(3));

    store1.merge(store2);
    assert_eq!(store1.len(), 3);
    assert_eq!(store1.get("b"), Some(&SettingValue::Integer(20))); // Overwritten
    assert_eq!(store1.get("c"), Some(&SettingValue::Integer(3)));
}

#[test]
fn test_store_from_map() {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_string(), SettingValue::Bool(true));

    let store = SettingsStore::from_map(map);
    assert_eq!(store.get("key"), Some(&SettingValue::Bool(true)));

    let back = store.into_map();
    assert_eq!(back.len(), 1);
}

#[test]
fn test_store_iterators() {
    let mut store = SettingsStore::new();
    store.set("a", SettingValue::Integer(1));
    store.set("b", SettingValue::Integer(2));

    let keys: Vec<&str> = store.keys().collect();
    assert_eq!(keys.len(), 2);

    let items: Vec<_> = (&store).into_iter().collect();
    assert_eq!(items.len(), 2);

    let owned_items: Vec<_> = store.into_iter().collect();
    assert_eq!(owned_items.len(), 2);
}

// ========== Schema Tests ==========

#[test]
fn test_schema_builder() {
    let schema = SettingSchema::builder("app.theme")
        .title("Theme")
        .description("Application theme")
        .enum_type(vec!["light".into(), "dark".into(), "auto".into()])
        .default(SettingValue::String("auto".into()))
        .category("Appearance")
        .requires_restart()
        .build();

    assert_eq!(schema.key, "app.theme");
    assert_eq!(schema.title, "Theme");
    assert!(schema.requires_restart);
    assert_eq!(schema.category, Some("Appearance".to_string()));
}

#[test]
fn test_schema_validation_bool() {
    let schema = SettingSchema::builder("flag")
        .bool_type()
        .build();

    assert!(schema.validate(&SettingValue::Bool(true)).is_ok());
    assert!(schema.validate(&SettingValue::Integer(1)).is_err());
}

#[rstest]
#[case(5, true)]
#[case(0, true)]
#[case(10, true)]
#[case(-1, false)]
#[case(11, false)]
fn test_schema_validation_integer_bounds(#[case] value: i64, #[case] valid: bool) {
    let schema = SettingSchema::builder("count")
        .integer_type(Some(0), Some(10))
        .build();

    assert_eq!(
        schema.validate(&SettingValue::Integer(value)).is_ok(),
        valid
    );
}

#[test]
fn test_schema_validation_float() {
    let schema = SettingSchema::builder("opacity")
        .float_type(Some(0.0), Some(1.0))
        .build();

    assert!(schema.validate(&SettingValue::Float(0.5)).is_ok());
    assert!(schema.validate(&SettingValue::Float(-0.1)).is_err());
    assert!(schema.validate(&SettingValue::Float(1.1)).is_err());
}

#[test]
fn test_schema_validation_string_max_length() {
    let schema = SettingSchema::builder("name")
        .string_type()
        .build();

    // Default string schema has no max_length
    assert!(schema.validate(&SettingValue::String("hello".into())).is_ok());
}

#[test]
fn test_schema_validation_enum() {
    let schema = SettingSchema::builder("mode")
        .enum_type(vec!["fast".into(), "slow".into()])
        .build();

    assert!(schema.validate(&SettingValue::String("fast".into())).is_ok());
    assert!(schema.validate(&SettingValue::String("invalid".into())).is_err());
}

// ========== SchemaRegistry Tests ==========

#[test]
fn test_registry() {
    let mut registry = SchemaRegistry::new();

    registry.register(
        SettingSchema::builder("app.theme")
            .category("Appearance")
            .build(),
    );
    registry.register(
        SettingSchema::builder("app.font")
            .category("Appearance")
            .build(),
    );
    registry.register(
        SettingSchema::builder("browser.homepage")
            .category("Browser")
            .build(),
    );

    assert!(registry.get("app.theme").is_some());
    assert!(registry.get("nonexistent").is_none());

    let appearance: Vec<_> = registry.by_category("Appearance").collect();
    assert_eq!(appearance.len(), 2);

    let categories = registry.categories();
    assert_eq!(categories.len(), 2);
    assert!(categories.contains(&"Appearance".to_string()));
    assert!(categories.contains(&"Browser".to_string()));
}

// ========== SettingsManager Tests ==========

#[fixture]
fn manager() -> SettingsManager {
    SettingsManager::new()
}

#[rstest]
fn test_manager_set_and_get(manager: SettingsManager) {
    manager
        .set("theme", SettingValue::String("dark".into()))
        .unwrap();

    assert_eq!(manager.get_string("theme"), Some("dark".to_string()));
    assert_eq!(manager.get_bool("theme"), None); // Wrong type
}

#[rstest]
fn test_manager_typed_getters(manager: SettingsManager) {
    manager.set("flag", SettingValue::Bool(true)).unwrap();
    manager.set("count", SettingValue::Integer(42)).unwrap();
    manager.set("ratio", SettingValue::Float(0.75)).unwrap();
    manager
        .set("name", SettingValue::String("test".into()))
        .unwrap();

    assert_eq!(manager.get_bool("flag"), Some(true));
    assert_eq!(manager.get_integer("count"), Some(42));
    assert_eq!(manager.get_float("ratio"), Some(0.75));
    assert_eq!(manager.get_string("name"), Some("test".to_string()));
    assert_eq!(manager.get("nonexistent"), None);
}

#[rstest]
fn test_manager_defaults(manager: SettingsManager) {
    let schema = SettingSchema::builder("theme")
        .enum_type(vec!["light".into(), "dark".into()])
        .default(SettingValue::String("light".into()))
        .build();

    manager.register_schema(schema);

    // Should return default when not explicitly set
    assert_eq!(manager.get_string("theme"), Some("light".to_string()));

    // Set explicitly
    manager
        .set("theme", SettingValue::String("dark".into()))
        .unwrap();
    assert_eq!(manager.get_string("theme"), Some("dark".to_string()));
}

#[rstest]
fn test_manager_validation(manager: SettingsManager) {
    let schema = SettingSchema::builder("count")
        .integer_type(Some(0), Some(100))
        .default(SettingValue::Integer(50))
        .build();

    manager.register_schema(schema);

    // Valid
    assert!(manager.set("count", SettingValue::Integer(75)).is_ok());

    // Invalid - out of bounds
    assert!(manager.set("count", SettingValue::Integer(200)).is_err());

    // Invalid - wrong type
    assert!(manager
        .set("count", SettingValue::String("hello".into()))
        .is_err());
}

#[rstest]
fn test_manager_reset(manager: SettingsManager) {
    let schema = SettingSchema::builder("theme")
        .string_type()
        .default(SettingValue::String("light".into()))
        .build();

    manager.register_schema(schema);
    manager
        .set("theme", SettingValue::String("dark".into()))
        .unwrap();

    assert_eq!(manager.get_string("theme"), Some("dark".to_string()));

    manager.reset("theme").unwrap();
    assert_eq!(manager.get_string("theme"), Some("light".to_string()));
}

#[rstest]
fn test_manager_reset_all(manager: SettingsManager) {
    manager.set("a", SettingValue::Integer(1)).unwrap();
    manager.set("b", SettingValue::Integer(2)).unwrap();

    manager.reset_all();
    assert!(manager.user_settings().is_empty());
}

#[rstest]
fn test_manager_change_callback(manager: SettingsManager) {
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    manager.on_change(move |_key, _old, _new| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    manager.set("key", SettingValue::Bool(true)).unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);

    manager.set("key", SettingValue::Bool(false)).unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[rstest]
fn test_manager_user_vs_all_settings(manager: SettingsManager) {
    let schema = SettingSchema::builder("default_key")
        .string_type()
        .default(SettingValue::String("default_val".into()))
        .build();
    manager.register_schema(schema);

    manager.set("user_key", SettingValue::Integer(1)).unwrap();

    let user = manager.user_settings();
    assert_eq!(user.len(), 1);
    assert!(user.get("user_key").is_some());
    assert!(user.get("default_key").is_none());

    let all = manager.all_settings();
    assert!(all.get("user_key").is_some());
    assert!(all.get("default_key").is_some());
}

#[rstest]
fn test_manager_persistence() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("settings.json");

    // Save
    {
        let manager = SettingsManager::with_storage(&path);
        manager
            .set("theme", SettingValue::String("dark".into()))
            .unwrap();
        manager.set("count", SettingValue::Integer(42)).unwrap();
        manager.save().unwrap();
    }

    // Load
    {
        let manager = SettingsManager::with_storage(&path);
        manager.load().unwrap();
        assert_eq!(manager.get_string("theme"), Some("dark".to_string()));
        assert_eq!(manager.get_integer("count"), Some(42));
    }
}

#[rstest]
fn test_manager_load_nonexistent_path() {
    let manager = SettingsManager::with_storage("/nonexistent/path/settings.json");
    // Loading from nonexistent path should return Ok (empty)
    assert!(manager.load().is_ok());
}

#[rstest]
fn test_manager_save_no_storage_path(manager: SettingsManager) {
    // Saving without a storage path should return Ok (noop)
    assert!(manager.save().is_ok());
}

#[rstest]
fn test_manager_clone_shares_state(manager: SettingsManager) {
    let manager2 = manager.clone();

    manager.set("key", SettingValue::Bool(true)).unwrap();
    assert_eq!(manager2.get_bool("key"), Some(true));
}

#[test]
fn test_manager_default() {
    let manager = SettingsManager::default();
    assert!(manager.user_settings().is_empty());
}

#[rstest]
fn test_manager_register_schemas(manager: SettingsManager) {
    let schemas = vec![
        SettingSchema::builder("a").bool_type().default(true).build(),
        SettingSchema::builder("b")
            .integer_type(None, None)
            .default(0i32)
            .build(),
    ];

    manager.register_schemas(schemas);

    let registry = manager.registry();
    assert!(registry.get("a").is_some());
    assert!(registry.get("b").is_some());
}
