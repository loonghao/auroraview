//! Tests for DCC configuration

use auroraview_dcc::{DccConfig, DccType};
use rstest::*;

// ===========================================================================
// Original tests
// ===========================================================================

#[test]
fn dcc_config_defaults() {
    let config = DccConfig::default();
    assert_eq!(config.title, "AuroraView");
    assert_eq!(config.width, 400);
    assert_eq!(config.height, 600);
    assert!(config.url.is_none());
    assert!(config.html.is_none());
    assert!(config.parent_hwnd.is_none());
}

#[test]
fn dcc_config_builder() {
    let config = DccConfig::new()
        .title("Maya Tool")
        .size(500, 700)
        .url("https://example.com")
        .parent_hwnd(0x12345)
        .dcc_type(DccType::Maya)
        .devtools(true);

    assert_eq!(config.title, "Maya Tool");
    assert_eq!(config.width, 500);
    assert_eq!(config.height, 700);
    assert_eq!(config.url, Some("https://example.com".to_string()));
    assert_eq!(config.parent_hwnd, Some(0x12345));
    assert_eq!(config.dcc_type, DccType::Maya);
    assert!(config.devtools);
}

#[test]
fn dcc_type_names() {
    assert_eq!(DccType::Maya.name(), "Maya");
    assert_eq!(DccType::Houdini.name(), "Houdini");
    assert_eq!(DccType::Nuke.name(), "Nuke");
    assert_eq!(DccType::Blender.name(), "Blender");
    assert_eq!(DccType::Max3ds.name(), "3ds Max");
    assert_eq!(DccType::Unreal.name(), "Unreal Engine");
    assert_eq!(DccType::Unknown.name(), "Unknown");
}

#[test]
fn dcc_type_default() {
    let dcc = DccType::default();
    assert_eq!(dcc, DccType::Unknown);
}

#[test]
fn dcc_config_panel_name() {
    let config = DccConfig::new().panel_name("MyToolPanel");
    assert_eq!(config.panel_name, Some("MyToolPanel".to_string()));
}

#[test]
fn dcc_config_debug_port() {
    let config = DccConfig::new().debug_port(9222);
    assert_eq!(config.debug_port, 9222);
}

#[test]
fn dcc_type_env_var() {
    assert_eq!(DccType::Maya.env_var(), Some("MAYA_LOCATION"));
    assert_eq!(DccType::Houdini.env_var(), Some("HFS"));
    assert_eq!(DccType::Nuke.env_var(), Some("NUKE_PATH"));
    assert_eq!(DccType::Blender.env_var(), Some("BLENDER_SYSTEM_SCRIPTS"));
    assert_eq!(DccType::Max3ds.env_var(), Some("ADSK_3DSMAX_X64_2025"));
    assert_eq!(DccType::Unreal.env_var(), Some("UE_ROOT"));
    assert_eq!(DccType::Unknown.env_var(), None);
}

#[test]
fn dcc_type_uses_qt() {
    assert!(DccType::Maya.uses_qt());
    assert!(DccType::Houdini.uses_qt());
    assert!(DccType::Nuke.uses_qt());
    assert!(DccType::Max3ds.uses_qt());
    assert!(!DccType::Blender.uses_qt());
    assert!(!DccType::Unreal.uses_qt());
    assert!(!DccType::Unknown.uses_qt());
}

#[test]
fn dcc_type_requires_main_thread() {
    assert!(DccType::Maya.requires_main_thread());
    assert!(DccType::Unreal.requires_main_thread());
    assert!(!DccType::Unknown.requires_main_thread());
}

#[test]
fn dcc_config_builder_extended() {
    let config = DccConfig::new()
        .dcc_version("2025.1")
        .data_dir("/tmp/auroraview")
        .background_color(30, 30, 30, 255);

    assert_eq!(config.dcc_version, Some("2025.1".to_string()));
    assert_eq!(
        config.data_dir,
        Some(std::path::PathBuf::from("/tmp/auroraview"))
    );
    assert_eq!(config.background_color, Some((30, 30, 30, 255)));
}

// ===========================================================================
// New: DccConfig clone
// ===========================================================================

#[test]
fn dcc_config_clone_is_independent() {
    let original = DccConfig::new()
        .title("Original")
        .size(800, 600)
        .url("https://original.com")
        .dcc_type(DccType::Houdini);

    let mut cloned = original.clone();
    cloned.title = "Cloned".to_string();
    cloned.width = 1920;

    assert_eq!(original.title, "Original");
    assert_eq!(original.width, 800);
    assert_eq!(cloned.title, "Cloned");
    assert_eq!(cloned.width, 1920);
}

#[test]
fn dcc_config_clone_preserves_all_fields() {
    let config = DccConfig::new()
        .title("Full Config")
        .size(1280, 720)
        .url("https://full.com")
        .parent_hwnd(0xDEAD)
        .dcc_type(DccType::Maya)
        .panel_name("Panel")
        .devtools(true)
        .debug_port(9229)
        .dcc_version("2025")
        .data_dir("/data")
        .background_color(0, 0, 0, 255);

    let cloned = config.clone();
    assert_eq!(cloned.title, "Full Config");
    assert_eq!(cloned.width, 1280);
    assert_eq!(cloned.height, 720);
    assert_eq!(cloned.url, Some("https://full.com".to_string()));
    assert_eq!(cloned.parent_hwnd, Some(0xDEAD));
    assert_eq!(cloned.dcc_type, DccType::Maya);
    assert_eq!(cloned.panel_name, Some("Panel".to_string()));
    assert!(cloned.devtools);
    assert_eq!(cloned.debug_port, 9229);
    assert_eq!(cloned.dcc_version, Some("2025".to_string()));
    assert_eq!(cloned.background_color, Some((0, 0, 0, 255)));
}

// ===========================================================================
// New: serde roundtrip
// ===========================================================================

#[test]
fn dcc_config_serde_roundtrip_default() {
    let config = DccConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let restored: DccConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.title, config.title);
    assert_eq!(restored.width, config.width);
    assert_eq!(restored.height, config.height);
    assert!(restored.url.is_none());
}

#[test]
fn dcc_config_serde_roundtrip_full() {
    let config = DccConfig::new()
        .title("Serde Test")
        .size(640, 480)
        .url("https://serde.com")
        .html("<h1>hi</h1>")
        .parent_hwnd(42)
        .dcc_type(DccType::Blender)
        .panel_name("BlenderPanel")
        .devtools(false)
        .debug_port(8080)
        .dcc_version("3.6")
        .data_dir("/tmp/serde")
        .background_color(128, 64, 32, 200);

    let json = serde_json::to_string(&config).unwrap();
    let restored: DccConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.title, "Serde Test");
    assert_eq!(restored.width, 640);
    assert_eq!(restored.height, 480);
    assert_eq!(restored.url, Some("https://serde.com".to_string()));
    assert_eq!(restored.html, Some("<h1>hi</h1>".to_string()));
    assert_eq!(restored.parent_hwnd, Some(42));
    assert_eq!(restored.dcc_type, DccType::Blender);
    assert_eq!(restored.panel_name, Some("BlenderPanel".to_string()));
    assert!(!restored.devtools);
    assert_eq!(restored.debug_port, 8080);
    assert_eq!(restored.dcc_version, Some("3.6".to_string()));
    assert_eq!(restored.background_color, Some((128, 64, 32, 200)));
}

#[test]
fn dcc_type_serde_roundtrip_all_variants() {
    for variant in [
        DccType::Maya,
        DccType::Houdini,
        DccType::Nuke,
        DccType::Blender,
        DccType::Max3ds,
        DccType::Unreal,
        DccType::Unknown,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let restored: DccType = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, variant, "Serde roundtrip failed for {:?}", variant);
    }
}

// ===========================================================================
// New: html builder sets html field
// ===========================================================================

#[test]
fn dcc_config_html_field() {
    let config = DccConfig::new().html("<h1>Hello AuroraView</h1>");
    assert_eq!(
        config.html,
        Some("<h1>Hello AuroraView</h1>".to_string())
    );
    assert!(config.url.is_none());
}

// url takes precedence convention: just check both can be set independently
#[test]
fn dcc_config_url_and_html_are_independent() {
    let config_url = DccConfig::new().url("https://url.com");
    let config_html = DccConfig::new().html("<html/>");
    assert!(config_url.url.is_some());
    assert!(config_url.html.is_none());
    assert!(config_html.html.is_some());
    assert!(config_html.url.is_none());
}

// ===========================================================================
// New: rstest – DccType.name() parametric
// ===========================================================================

#[rstest]
#[case(DccType::Maya, "Maya")]
#[case(DccType::Houdini, "Houdini")]
#[case(DccType::Nuke, "Nuke")]
#[case(DccType::Blender, "Blender")]
#[case(DccType::Max3ds, "3ds Max")]
#[case(DccType::Unreal, "Unreal Engine")]
#[case(DccType::Unknown, "Unknown")]
fn dcc_type_name_parametric(#[case] dcc: DccType, #[case] expected: &str) {
    assert_eq!(dcc.name(), expected);
}

// ===========================================================================
// New: rstest – uses_qt parametric
// ===========================================================================

#[rstest]
#[case(DccType::Maya, true)]
#[case(DccType::Houdini, true)]
#[case(DccType::Nuke, true)]
#[case(DccType::Max3ds, true)]
#[case(DccType::Blender, false)]
#[case(DccType::Unreal, false)]
#[case(DccType::Unknown, false)]
fn dcc_type_uses_qt_parametric(#[case] dcc: DccType, #[case] uses_qt: bool) {
    assert_eq!(dcc.uses_qt(), uses_qt);
}

// ===========================================================================
// New: rstest – requires_main_thread for all variants
// ===========================================================================

#[rstest]
#[case(DccType::Maya, true)]
#[case(DccType::Houdini, true)]
#[case(DccType::Nuke, true)]
#[case(DccType::Max3ds, true)]
#[case(DccType::Blender, true)]
#[case(DccType::Unreal, true)]
#[case(DccType::Unknown, false)]
fn dcc_type_requires_main_thread_parametric(#[case] dcc: DccType, #[case] expected: bool) {
    assert_eq!(dcc.requires_main_thread(), expected);
}

// ===========================================================================
// New: builder – devtools disabled by default in non-debug check
// ===========================================================================

#[test]
fn dcc_config_devtools_field_explicit_false() {
    let config = DccConfig::new().devtools(false);
    assert!(!config.devtools);
}

#[test]
fn dcc_config_devtools_field_explicit_true() {
    let config = DccConfig::new().devtools(true);
    assert!(config.devtools);
}

// ===========================================================================
// New: builder – overwrite fields multiple times
// ===========================================================================

#[test]
fn dcc_config_builder_overwrite_title() {
    let config = DccConfig::new().title("First").title("Second");
    assert_eq!(config.title, "Second");
}

#[test]
fn dcc_config_builder_overwrite_size() {
    let config = DccConfig::new().size(100, 200).size(1920, 1080);
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
}

// ===========================================================================
// New: builder – zero size is accepted
// ===========================================================================

#[test]
fn dcc_config_size_zero() {
    let config = DccConfig::new().size(0, 0);
    assert_eq!(config.width, 0);
    assert_eq!(config.height, 0);
}

// ===========================================================================
// New: builder – large HWND value
// ===========================================================================

#[test]
fn dcc_config_large_hwnd() {
    let large_hwnd: isize = isize::MAX;
    let config = DccConfig::new().parent_hwnd(large_hwnd);
    assert_eq!(config.parent_hwnd, Some(large_hwnd));
}

// ===========================================================================
// New: builder – background_color edge cases
// ===========================================================================

#[rstest]
#[case(0, 0, 0, 0)]       // fully transparent black
#[case(255, 255, 255, 255)] // fully opaque white
#[case(128, 128, 128, 128)] // 50% gray
fn dcc_config_background_color_parametric(#[case] r: u8, #[case] g: u8, #[case] b: u8, #[case] a: u8) {
    let config = DccConfig::new().background_color(r, g, b, a);
    assert_eq!(config.background_color, Some((r, g, b, a)));
}

// ===========================================================================
// New: debug_port 0 means auto-select
// ===========================================================================

#[test]
fn dcc_config_debug_port_zero() {
    let config = DccConfig::new().debug_port(0);
    assert_eq!(config.debug_port, 0);
}

// ===========================================================================
// New: DccType Debug format check
// ===========================================================================

#[test]
fn dcc_type_debug_format() {
    let s = format!("{:?}", DccType::Maya);
    assert!(!s.is_empty());
    let s = format!("{:?}", DccType::Unknown);
    assert!(!s.is_empty());
}

// ===========================================================================
// New: DccConfig new() equals default()
// ===========================================================================

#[test]
fn dcc_config_new_equals_default() {
    let a = DccConfig::new();
    let b = DccConfig::default();
    // Compare serialized JSON as a structural equality proxy
    let json_a = serde_json::to_string(&a).unwrap();
    let json_b = serde_json::to_string(&b).unwrap();
    assert_eq!(json_a, json_b);
}

// ===========================================================================
// New: DccType::detect() uses env variables
// ===========================================================================

#[test]
fn dcc_type_detect_fallback_to_unknown_when_no_env() {
    // Remove all known DCC env vars to get Unknown
    let vars = [
        "MAYA_LOCATION",
        "HFS",
        "NUKE_PATH",
        "BLENDER_SYSTEM_SCRIPTS",
        "ADSK_3DSMAX_X64_2025",
        "3DSMAX_LOCATION",
        "UE_ROOT",
        "UE4_ROOT",
    ];
    for v in &vars {
        std::env::remove_var(v);
    }
    // If none set, should detect Unknown
    let detected = DccType::detect();
    assert_eq!(detected, DccType::Unknown);
}

// ===========================================================================
// New: DccType env_var parametric (all variants)
// ===========================================================================

#[rstest]
#[case(DccType::Maya, Some("MAYA_LOCATION"))]
#[case(DccType::Houdini, Some("HFS"))]
#[case(DccType::Nuke, Some("NUKE_PATH"))]
#[case(DccType::Blender, Some("BLENDER_SYSTEM_SCRIPTS"))]
#[case(DccType::Max3ds, Some("ADSK_3DSMAX_X64_2025"))]
#[case(DccType::Unreal, Some("UE_ROOT"))]
#[case(DccType::Unknown, None)]
fn dcc_type_env_var_parametric(#[case] dcc: DccType, #[case] expected: Option<&str>) {
    assert_eq!(dcc.env_var(), expected);
}

// ===========================================================================
// New: DccType Copy semantics
// ===========================================================================

#[test]
fn dcc_type_copy() {
    let a = DccType::Maya;
    let b = a; // Copy
    assert_eq!(a, b);
}

// ===========================================================================
// New: DccType::detect() with Maya env var set
// ===========================================================================

#[test]
fn dcc_type_detect_maya_via_env() {
    std::env::set_var("MAYA_LOCATION", "/usr/autodesk/maya2025");
    let detected = DccType::detect();
    std::env::remove_var("MAYA_LOCATION");
    assert_eq!(detected, DccType::Maya);
}

// ===========================================================================
// New: DccType::detect() with UE env var set
// ===========================================================================

#[test]
fn dcc_type_detect_unreal_via_env() {
    // Ensure Maya not set (it takes priority)
    std::env::remove_var("MAYA_LOCATION");
    std::env::remove_var("HFS");
    std::env::remove_var("NUKE_PATH");
    std::env::remove_var("BLENDER_SYSTEM_SCRIPTS");
    std::env::remove_var("ADSK_3DSMAX_X64_2025");
    std::env::remove_var("3DSMAX_LOCATION");
    std::env::set_var("UE_ROOT", "/opt/ue5");
    let detected = DccType::detect();
    std::env::remove_var("UE_ROOT");
    assert_eq!(detected, DccType::Unreal);
}

// ===========================================================================
// New: DccConfig Send+Sync
// ===========================================================================

#[test]
fn dcc_config_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<DccConfig>();
    assert_sync::<DccConfig>();
}

#[test]
fn dcc_type_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<DccType>();
    assert_sync::<DccType>();
}

// ===========================================================================
// New: DccConfig data_dir as PathBuf
// ===========================================================================

#[test]
fn dcc_config_data_dir_pathbuf() {
    let path = std::path::Path::new("/var/data/auroraview");
    let config = DccConfig::new().data_dir(path);
    assert_eq!(config.data_dir, Some(path.to_path_buf()));
}

#[test]
fn dcc_config_data_dir_unicode_path() {
    let path = "/tmp/アuroraview/データ";
    let config = DccConfig::new().data_dir(path);
    let expected = std::path::PathBuf::from(path);
    assert_eq!(config.data_dir, Some(expected));
}

// ===========================================================================
// New: DccConfig builder unicode title/panel_name
// ===========================================================================

#[test]
fn dcc_config_unicode_title() {
    let title = "AuroraView - Maya ツール";
    let config = DccConfig::new().title(title);
    assert_eq!(config.title, title);
}

#[test]
fn dcc_config_unicode_panel_name() {
    let name = "パネル - Houdini";
    let config = DccConfig::new().panel_name(name);
    assert_eq!(config.panel_name, Some(name.to_string()));
}

// ===========================================================================
// New: DccConfig url with special chars
// ===========================================================================

#[rstest]
#[case("https://example.com/path?q=hello+world&lang=ja")]
#[case("file:///C:/Users/user/tool.html")]
#[case("http://localhost:8080/panel")]
#[case("about:blank")]
fn dcc_config_url_variants(#[case] url: &str) {
    let config = DccConfig::new().url(url);
    assert_eq!(config.url, Some(url.to_string()));
}

// ===========================================================================
// New: builder default_debug_port is 0
// ===========================================================================

#[test]
fn dcc_config_default_debug_port_is_zero() {
    let config = DccConfig::default();
    assert_eq!(config.debug_port, 0);
}

// ===========================================================================
// New: DccConfig debug_port high value
// ===========================================================================

#[test]
fn dcc_config_debug_port_max() {
    let config = DccConfig::new().debug_port(u16::MAX);
    assert_eq!(config.debug_port, u16::MAX);
}

// ===========================================================================
// New: DccType PartialEq
// ===========================================================================

#[rstest]
#[case(DccType::Maya, DccType::Maya, true)]
#[case(DccType::Maya, DccType::Houdini, false)]
#[case(DccType::Unknown, DccType::Unknown, true)]
#[case(DccType::Blender, DccType::Unreal, false)]
fn dcc_type_partial_eq(#[case] a: DccType, #[case] b: DccType, #[case] equal: bool) {
    assert_eq!(a == b, equal);
}

