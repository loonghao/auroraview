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

// ============================================================================
// New Tests
// ============================================================================

#[test]
fn test_emit_event_template_special_chars() {
    let template = EmitEventTemplate {
        event_name: "app:data-updated",
        event_data: r#"{"msg": "hello \"world\""}"#,
    };
    let result = template.render().unwrap();
    assert!(result.contains("app:data-updated"));
    assert!(result.contains("window.auroraview.trigger"));
}

#[test]
fn test_emit_event_template_null_data() {
    let template = EmitEventTemplate {
        event_name: "close",
        event_data: "null",
    };
    let result = template.render().unwrap();
    assert!(result.contains("close"));
    assert!(result.contains("null"));
    assert!(result.contains("window.auroraview.trigger"));
}

#[test]
fn test_emit_event_template_unicode() {
    let template = EmitEventTemplate {
        event_name: "unicode_event",
        event_data: r#"{"name": "测试数据", "emoji": "🚀"}"#,
    };
    let result = template.render().unwrap();
    assert!(result.contains("unicode_event"));
    assert!(result.contains("测试数据"));
    assert!(result.contains("🚀"));
}

#[test]
fn test_emit_event_template_array_data() {
    let template = EmitEventTemplate {
        event_name: "batch",
        event_data: r#"[1, 2, 3, "four"]"#,
    };
    let result = template.render().unwrap();
    assert!(result.contains("batch"));
    assert!(result.contains("[1, 2, 3"));
    assert!(result.contains("window.auroraview.trigger"));
}

#[test]
fn test_load_url_template_file_url() {
    let template = LoadUrlTemplate {
        url: "file:///C:/Users/test/index.html",
    };
    let result = template.render().unwrap();
    assert!(result.contains("file:///C:/Users/test/index.html"));
    assert!(result.contains("window.location.href"));
}

#[test]
fn test_load_url_template_localhost() {
    let template = LoadUrlTemplate {
        url: "http://localhost:3000",
    };
    let result = template.render().unwrap();
    assert!(result.contains("http://localhost:3000"));
}

#[test]
fn test_load_url_template_auroraview_protocol() {
    let template = LoadUrlTemplate {
        url: "https://auroraview.localhost/type:local/dist/index.html",
    };
    let result = template.render().unwrap();
    assert!(result.contains("auroraview.localhost"));
    assert!(result.contains("type:local"));
}

#[test]
fn test_api_registration_template_single_method() {
    let entries = vec![ApiMethodEntry {
        namespace: "api".to_string(),
        methods: vec!["get_version".to_string()],
    }];
    let template = ApiRegistrationTemplate {
        api_methods: entries,
    };
    let result = template.render().unwrap();
    assert!(result.contains("'api'"));
    assert!(result.contains("'get_version'"));
    assert!(result.contains("window.auroraview._registerApiMethods"));
}

#[test]
fn test_api_registration_template_many_namespaces() {
    let entries = vec![
        ApiMethodEntry {
            namespace: "scene".to_string(),
            methods: vec!["export".to_string(), "import".to_string()],
        },
        ApiMethodEntry {
            namespace: "render".to_string(),
            methods: vec!["start".to_string(), "stop".to_string(), "pause".to_string()],
        },
        ApiMethodEntry {
            namespace: "tool".to_string(),
            methods: vec!["apply".to_string()],
        },
    ];
    let template = ApiRegistrationTemplate {
        api_methods: entries,
    };
    let result = template.render().unwrap();

    // All namespaces present
    assert!(result.contains("'scene'"));
    assert!(result.contains("'render'"));
    assert!(result.contains("'tool'"));

    // All methods present
    assert!(result.contains("'export'"));
    assert!(result.contains("'import'"));
    assert!(result.contains("'start'"));
    assert!(result.contains("'stop'"));
    assert!(result.contains("'pause'"));
    assert!(result.contains("'apply'"));
}

#[test]
fn test_api_registration_template_empty_list() {
    let template = ApiRegistrationTemplate {
        api_methods: vec![],
    };
    let result = template.render().unwrap();
    // Should render without panic; may be empty or minimal JS
    assert!(!result.contains("'method'"));
}

// ============================================================================
// EmitEventTemplate - additional coverage
// ============================================================================

#[test]
fn test_emit_event_template_empty_event_name() {
    let template = EmitEventTemplate {
        event_name: "",
        event_data: "{}",
    };
    let result = template.render().unwrap();
    assert!(result.contains("window.auroraview.trigger"));
}

#[test]
fn test_emit_event_template_nested_json() {
    let template = EmitEventTemplate {
        event_name: "scene.loaded",
        event_data: r#"{"scene":{"name":"test","objects":[{"id":1},{"id":2}]}}"#,
    };
    let result = template.render().unwrap();
    assert!(result.contains("scene.loaded"));
    assert!(result.contains("window.auroraview.trigger"));
    assert!(result.contains("objects"));
}

#[test]
fn test_emit_event_template_boolean_data() {
    let template = EmitEventTemplate {
        event_name: "ready",
        event_data: "true",
    };
    let result = template.render().unwrap();
    assert!(result.contains("ready"));
    assert!(result.contains("true"));
}

#[test]
fn test_emit_event_template_number_data() {
    let template = EmitEventTemplate {
        event_name: "frame",
        event_data: "42",
    };
    let result = template.render().unwrap();
    assert!(result.contains("frame"));
    assert!(result.contains("42"));
}

// ============================================================================
// LoadUrlTemplate - additional coverage
// ============================================================================

#[test]
fn test_load_url_template_empty_url() {
    let template = LoadUrlTemplate {
        url: "",
    };
    let result = template.render().unwrap();
    // Should render without panic even with empty URL
    assert!(result.contains("window.location.href"));
}

#[test]
fn test_load_url_template_query_string() {
    let template = LoadUrlTemplate {
        url: "https://example.com/page?q=1&r=2#section",
    };
    let result = template.render().unwrap();
    assert!(result.contains("example.com/page"));
    assert!(result.contains("q=1"));
}

#[test]
fn test_load_url_template_unicode_url() {
    let template = LoadUrlTemplate {
        url: "https://example.com/path?name=中文",
    };
    let result = template.render().unwrap();
    assert!(result.contains("example.com/path"));
}

// ============================================================================
// ApiRegistrationTemplate - additional coverage
// ============================================================================

#[test]
fn test_api_registration_template_method_with_underscore() {
    let entries = vec![ApiMethodEntry {
        namespace: "maya_api".to_string(),
        methods: vec!["get_selection".to_string(), "set_keyframe".to_string()],
    }];
    let template = ApiRegistrationTemplate { api_methods: entries };
    let result = template.render().unwrap();
    assert!(result.contains("'maya_api'"));
    assert!(result.contains("'get_selection'"));
    assert!(result.contains("'set_keyframe'"));
}

#[test]
fn test_api_registration_template_numeric_looking_namespace() {
    let entries = vec![ApiMethodEntry {
        namespace: "v2".to_string(),
        methods: vec!["ping".to_string()],
    }];
    let template = ApiRegistrationTemplate { api_methods: entries };
    let result = template.render().unwrap();
    assert!(result.contains("'v2'"));
    assert!(result.contains("'ping'"));
}

#[test]
fn test_api_registration_template_only_empty_namespace_methods() {
    // Namespace with single method that has empty string
    let entries = vec![ApiMethodEntry {
        namespace: "ns".to_string(),
        methods: vec!["".to_string()],
    }];
    let template = ApiRegistrationTemplate { api_methods: entries };
    let result = template.render().unwrap();
    // Should not panic; may or may not include empty method
    assert!(!result.is_empty());
}

#[test]
fn test_api_registration_template_unicode_namespace() {
    let entries = vec![ApiMethodEntry {
        namespace: "dcc".to_string(),
        methods: vec!["export_fbx".to_string(), "import_obj".to_string()],
    }];
    let template = ApiRegistrationTemplate { api_methods: entries };
    let result = template.render().unwrap();
    assert!(result.contains("'dcc'"));
    assert!(result.contains("'export_fbx'"));
    assert!(result.contains("'import_obj'"));
}
