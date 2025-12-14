//! Template tests

use askama::Template;
use auroraview_core::templates::{
    ApiMethodEntry, ApiRegistrationTemplate, EmitEventTemplate, LoadUrlTemplate,
};

#[test]
fn test_emit_event_template() {
    let template = EmitEventTemplate {
        event_name: "test_event",
        event_data: r#"{"key": "value"}"#,
    };
    let result = template.render().unwrap();

    assert!(result.contains("test_event"));
    assert!(result.contains(r#"{"key": "value"}"#));
    assert!(result.contains("window.auroraview.trigger"));
}

#[test]
fn test_load_url_template() {
    let template = LoadUrlTemplate {
        url: "https://example.com/path",
    };
    let result = template.render().unwrap();

    assert!(result.contains("https://example.com/path"));
    assert!(result.contains("window.location.href"));
}

#[test]
fn test_api_registration_template() {
    let entries = vec![
        ApiMethodEntry {
            namespace: "test".to_string(),
            methods: vec!["method1".to_string(), "method2".to_string()],
        },
        ApiMethodEntry {
            namespace: "other".to_string(),
            methods: vec!["foo".to_string()],
        },
    ];
    let template = ApiRegistrationTemplate {
        api_methods: entries,
    };
    let result = template.render().unwrap();

    assert!(result.contains("window.auroraview._registerApiMethods"));
    assert!(result.contains("'test'"));
    assert!(result.contains("'method1'"));
    assert!(result.contains("'method2'"));
    assert!(result.contains("'other'"));
    assert!(result.contains("'foo'"));
}

#[test]
fn test_api_registration_template_empty_methods() {
    let entries = vec![ApiMethodEntry {
        namespace: "empty".to_string(),
        methods: vec![],
    }];
    let template = ApiRegistrationTemplate {
        api_methods: entries,
    };
    let result = template.render().unwrap();

    // Empty methods should not generate registration call
    assert!(!result.contains("'empty'"));
}
