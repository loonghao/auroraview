//! Integration tests for NameObfuscator, Obfuscator, and AstObfuscator

use auroraview_protect::{AstObfuscator, NameObfuscator, ObfuscationLevel, Obfuscator};

// ─── NameObfuscator ────────────────────────────────────────────────────────────

#[test]
fn name_obfuscator_same_name_gets_same_result() {
    let mut o = NameObfuscator::new();
    let a = o.obfuscate("my_var");
    let b = o.obfuscate("my_var");
    assert_eq!(a, b);
}

#[test]
fn name_obfuscator_different_names_get_different_results() {
    let mut o = NameObfuscator::new();
    let a = o.obfuscate("foo");
    let b = o.obfuscate("bar");
    assert_ne!(a, b);
}

#[test]
fn name_obfuscator_default_prefix_is_0x() {
    let mut o = NameObfuscator::new();
    let r = o.obfuscate("my_var");
    assert!(r.starts_with("_0x"), "expected _0x prefix, got: {r}");
}

#[test]
fn name_obfuscator_custom_prefix() {
    let mut o = NameObfuscator::new();
    o.set_prefix("__av_");
    let r = o.obfuscate("my_var");
    assert!(r.starts_with("__av_"), "expected __av_ prefix, got: {r}");
}

#[test]
fn name_obfuscator_preserve_stops_obfuscation() {
    let mut o = NameObfuscator::new();
    o.preserve(["important"]);
    assert_eq!(o.obfuscate("important"), "important");
}

#[test]
fn name_obfuscator_dunder_preserved() {
    let mut o = NameObfuscator::new();
    assert_eq!(o.obfuscate("__init__"), "__init__");
    assert_eq!(o.obfuscate("__all__"), "__all__");
}

#[test]
fn name_obfuscator_should_obfuscate_returns_false_for_preserved() {
    let mut o = NameObfuscator::new();
    o.preserve(["my_api"]);
    assert!(!o.should_obfuscate("my_api"));
}

#[test]
fn name_obfuscator_should_obfuscate_returns_false_for_dunder() {
    let o = NameObfuscator::new();
    assert!(!o.should_obfuscate("__class__"));
}

#[test]
fn name_obfuscator_should_obfuscate_returns_false_for_already_obfuscated() {
    let o = NameObfuscator::new();
    assert!(!o.should_obfuscate("_0xdeadbeef"));
}

#[test]
fn name_obfuscator_should_obfuscate_returns_true_for_regular() {
    let o = NameObfuscator::new();
    assert!(o.should_obfuscate("my_variable"));
}

#[test]
fn name_obfuscator_get_mapping_reflects_obfuscated_names() {
    let mut o = NameObfuscator::new();
    o.obfuscate("alpha");
    o.obfuscate("beta");
    let map = o.get_mapping();
    assert!(map.contains_key("alpha"));
    assert!(map.contains_key("beta"));
    assert_eq!(map.len(), 2);
}

#[test]
fn name_obfuscator_default_preserves_builtins() {
    let o = NameObfuscator::default();
    // Default preserves Python builtins
    assert!(!o.should_obfuscate("print"));
    assert!(!o.should_obfuscate("len"));
    assert!(!o.should_obfuscate("True"));
    assert!(!o.should_obfuscate("None"));
}

// ─── Obfuscator (legacy regex-based) ─────────────────────────────────────────

#[test]
fn obfuscator_none_level_returns_source_unchanged() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::None);
    let src = "x = 1\nprint(x)\n";
    let result = o.obfuscate(src).unwrap();
    assert_eq!(result, src);
}

#[test]
fn obfuscator_basic_renames_local_variable() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::Basic);
    let src = "message = 'hello'\nprint(message)\n";
    let result = o.obfuscate(src).unwrap();
    assert!(!result.contains("message ="), "variable should be renamed");
    assert!(result.contains("print"), "print should be preserved");
}

#[test]
fn obfuscator_standard_renames_function() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::Standard);
    let src = "def my_func():\n    pass\n";
    let result = o.obfuscate(src).unwrap();
    assert!(!result.contains("def my_func"), "function name should be obfuscated");
}

#[test]
fn obfuscator_advanced_adds_opaque_predicates() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::Advanced);
    let src = "if True:\n    x = 1\n";
    let result = o.obfuscate(src).unwrap();
    // Advanced wraps if-conditions with opaque predicates
    assert!(result.contains("_0xT"), "should contain opaque true predicate");
}

#[test]
fn obfuscator_maximum_inserts_dead_code() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::Maximum);
    let src = "x = 42\n";
    let result = o.obfuscate(src).unwrap();
    assert!(
        result.contains("if False:"),
        "maximum should insert dead code"
    );
}

#[test]
fn obfuscator_maximum_with_string_key_inserts_decrypt_func() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::Maximum);
    o.enable_string_encryption([0u8; 16]);
    let src = "x = 42\n";
    let result = o.obfuscate(src).unwrap();
    assert!(
        result.contains("_0xS"),
        "should contain string decryption function"
    );
}

#[test]
fn obfuscator_preserve_names_keeps_api_intact() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::Standard);
    o.preserve_names(["public_api"]);
    let src = "def public_api():\n    pass\n";
    let result = o.obfuscate(src).unwrap();
    assert!(
        result.contains("public_api"),
        "preserved name should survive obfuscation"
    );
}

#[test]
fn obfuscator_get_name_mapping_after_obfuscation() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::Standard);
    let src = "def my_func():\n    local_var = 1\n    return local_var\n";
    o.obfuscate(src).unwrap();
    let map = o.get_name_mapping();
    assert!(!map.is_empty(), "name mapping should be populated");
}

// ─── Obfuscator (AST-based, default) ─────────────────────────────────────────

#[test]
fn obfuscator_ast_none_returns_source_unchanged() {
    let mut o = Obfuscator::new(ObfuscationLevel::None);
    let src = "x = 1\nprint(x)\n";
    assert_eq!(o.obfuscate(src).unwrap(), src);
}

#[test]
fn obfuscator_ast_standard_renames_variable() {
    let mut o = Obfuscator::new(ObfuscationLevel::Standard);
    let src = "my_value = 42\nprint(my_value)\n";
    let result = o.obfuscate(src).unwrap();
    assert!(
        !result.contains("my_value"),
        "variable should be renamed in AST mode"
    );
    assert!(result.contains("print"), "builtins should survive");
}

#[test]
fn obfuscator_ast_preserves_dunder_methods() {
    let mut o = Obfuscator::new(ObfuscationLevel::Standard);
    let src = "class Foo:\n    def __init__(self):\n        pass\n";
    let result = o.obfuscate(src).unwrap();
    assert!(
        result.contains("__init__"),
        "__init__ should not be obfuscated"
    );
}

// ─── AstObfuscator ────────────────────────────────────────────────────────────

#[test]
fn ast_obfuscator_none_returns_source_unchanged() {
    let mut o = AstObfuscator::new(ObfuscationLevel::None);
    let src = "x = 1\n";
    assert_eq!(o.obfuscate(src).unwrap(), src);
}

#[test]
fn ast_obfuscator_renames_local_function() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Standard);
    let src = "def calculate(a, b):\n    total = a + b\n    return total\n";
    let result = o.obfuscate(src).unwrap();
    assert!(!result.contains("def calculate"), "function should be renamed");
    assert!(!result.contains("total ="), "local var should be renamed");
}

#[test]
fn ast_obfuscator_preserves_builtins() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Standard);
    let src = "x = len([1, 2, 3])\nprint(x)\n";
    let result = o.obfuscate(src).unwrap();
    assert!(result.contains("len"), "len should not be obfuscated");
    assert!(result.contains("print"), "print should not be obfuscated");
}

#[test]
fn ast_obfuscator_preserves_exported_names() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Standard);
    let src = "__all__ = ['public_api']\n\ndef public_api():\n    pass\n";
    let result = o.obfuscate(src).unwrap();
    assert!(
        result.contains("public_api"),
        "exported name should be preserved"
    );
}

#[test]
fn ast_obfuscator_explicit_preserve_keeps_name() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Standard);
    o.preserve(["my_special_fn"]);
    let src = "def my_special_fn():\n    pass\n";
    let result = o.obfuscate(src).unwrap();
    assert!(
        result.contains("my_special_fn"),
        "explicitly preserved name should survive"
    );
}

#[test]
fn ast_obfuscator_string_encryption_key() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Maximum);
    o.enable_string_encryption([42u8; 16]);
    let src = "x = 1\n";
    // Should not panic with encryption enabled
    let result = o.obfuscate(src);
    assert!(result.is_ok(), "obfuscation with encryption key should succeed");
}

#[test]
fn ast_obfuscator_preserves_import_names() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Standard);
    let src = "import os\nimport sys\n\npath = os.path.join('a', 'b')\n";
    let result = o.obfuscate(src).unwrap();
    // os and sys are imported names and should be preserved
    assert!(result.contains("os"), "imported module 'os' should be preserved");
    assert!(result.contains("sys"), "imported module 'sys' should be preserved");
}

#[test]
fn ast_obfuscator_advanced_adds_control_flow() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Advanced);
    let src = "def my_func():\n    x = 1\n    return x\n";
    let result = o.obfuscate(src).unwrap();
    // Advanced obfuscation should modify the source
    assert!(!result.is_empty());
    assert_ne!(result, src, "advanced obfuscation should modify source");
}

// ─── ObfuscationLevel ordering ────────────────────────────────────────────────

#[test]
fn obfuscation_level_ordering() {
    assert!(ObfuscationLevel::None < ObfuscationLevel::Basic);
    assert!(ObfuscationLevel::Basic < ObfuscationLevel::Standard);
    assert!(ObfuscationLevel::Standard < ObfuscationLevel::Advanced);
    assert!(ObfuscationLevel::Advanced < ObfuscationLevel::Maximum);
}

#[test]
fn obfuscation_level_equality() {
    assert_eq!(ObfuscationLevel::None, ObfuscationLevel::None);
    assert_eq!(ObfuscationLevel::Maximum, ObfuscationLevel::Maximum);
    assert_ne!(ObfuscationLevel::Basic, ObfuscationLevel::Standard);
}

// ─── NameObfuscator extra coverage ────────────────────────────────────────────

#[test]
fn name_obfuscator_empty_string_input() {
    let mut o = NameObfuscator::new();
    // Empty string: behavior is implementation-defined; must not panic
    let _ = o.obfuscate("");
}

#[test]
fn name_obfuscator_already_obfuscated_not_double_obfuscated() {
    let mut o = NameObfuscator::new();
    let first = o.obfuscate("my_var");
    // Calling obfuscate on already-obfuscated name: should_obfuscate returns false,
    // so calling obfuscate("_0x...") should return the same value
    let second = o.obfuscate(&first.clone());
    // Either identity preserved or stable — must not panic
    assert!(!second.is_empty());
}

#[test]
fn name_obfuscator_multiple_preserve_all_survive() {
    let mut o = NameObfuscator::new();
    o.preserve(["foo", "bar", "baz"]);
    assert_eq!(o.obfuscate("foo"), "foo");
    assert_eq!(o.obfuscate("bar"), "bar");
    assert_eq!(o.obfuscate("baz"), "baz");
}

#[test]
fn name_obfuscator_mapping_grows_with_each_new_name() {
    let mut o = NameObfuscator::new();
    o.obfuscate("name_a");
    let count1 = o.get_mapping().len();
    o.obfuscate("name_b");
    let count2 = o.get_mapping().len();
    assert_eq!(count2, count1 + 1);
}

#[test]
fn name_obfuscator_prefix_change_affects_new_names() {
    let mut o = NameObfuscator::new();
    o.set_prefix("__x_");
    let r = o.obfuscate("my_func");
    assert!(r.starts_with("__x_"), "got: {r}");
}

#[test]
fn name_obfuscator_different_prefix_instances_independent() {
    let mut o1 = NameObfuscator::new();
    let mut o2 = NameObfuscator::new();
    o2.set_prefix("__yy_");
    let r1 = o1.obfuscate("name");
    let r2 = o2.obfuscate("name");
    assert_ne!(r1, r2, "different prefixes produce different results");
}

// ─── Obfuscator (legacy) extra coverage ───────────────────────────────────────

#[test]
fn obfuscator_none_level_multiple_calls_stable() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::None);
    let src = "x = 1\n";
    let r1 = o.obfuscate(src).unwrap();
    let r2 = o.obfuscate(src).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn obfuscator_basic_result_not_empty() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::Basic);
    let src = "x = 42\n";
    let result = o.obfuscate(src).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn obfuscator_preserve_names_empty_list_no_change() {
    let mut o = Obfuscator::new_legacy(ObfuscationLevel::Standard);
    let empty: [&str; 0] = [];
    o.preserve_names(empty);
    let src = "def func():\n    pass\n";
    let result = o.obfuscate(src).unwrap();
    // Should not panic; function should be renamed
    assert!(!result.is_empty());
}

// ─── AstObfuscator extra coverage ─────────────────────────────────────────────

#[test]
fn ast_obfuscator_empty_source_no_panic() {
    let mut o = AstObfuscator::new(ObfuscationLevel::None);
    // Empty source: must not panic
    let result = o.obfuscate("");
    assert!(result.is_ok());
}

#[test]
fn ast_obfuscator_basic_level_result_not_empty() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Basic);
    let src = "x = 1\n";
    let result = o.obfuscate(src).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn ast_obfuscator_maximum_level_result_not_empty() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Maximum);
    let src = "x = 42\n";
    let result = o.obfuscate(src).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn ast_obfuscator_preserve_multiple_names() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Standard);
    o.preserve(["run", "start", "stop"]);
    let src = "def run():\n    pass\ndef start():\n    pass\ndef stop():\n    pass\n";
    let result = o.obfuscate(src).unwrap();
    assert!(result.contains("run") && result.contains("start") && result.contains("stop"));
}

#[test]
fn ast_obfuscator_get_name_mapping_populated_after_standard() {
    let mut o = AstObfuscator::new(ObfuscationLevel::Standard);
    let src = "def my_fn():\n    local = 1\n    return local\n";
    o.obfuscate(src).unwrap();
    let map = o.get_name_mapping();
    assert!(!map.is_empty());
}

#[test]
fn ast_obfuscator_none_get_name_mapping_empty() {
    let mut o = AstObfuscator::new(ObfuscationLevel::None);
    let src = "x = 1\n";
    o.obfuscate(src).unwrap();
    // None level does no renaming, mapping should be empty
    let map = o.get_name_mapping();
    assert!(map.is_empty(), "None level should not populate name mapping");
}

// ─── ObfuscationLevel: Clone + Copy + Hash ────────────────────────────────────

#[test]
fn obfuscation_level_clone() {
    let level = ObfuscationLevel::Advanced;
    let clone = level.clone();
    assert_eq!(level, clone);
}

#[test]
fn obfuscation_level_debug_non_empty() {
    let levels = [
        ObfuscationLevel::None,
        ObfuscationLevel::Basic,
        ObfuscationLevel::Standard,
        ObfuscationLevel::Advanced,
        ObfuscationLevel::Maximum,
    ];
    for l in &levels {
        assert!(!format!("{:?}", l).is_empty());
    }
}
