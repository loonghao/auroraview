//! Tests for BrowserConfig

use auroraview_browser::devtools::DockSide;
use auroraview_browser::{BrowserConfig, BrowserFeatures, Theme};
use rstest::rstest;

// -------------------------------------------------------------------------
// Default values
// -------------------------------------------------------------------------

#[test]
fn browser_config_default() {
    let config = BrowserConfig::default();

    assert_eq!(config.title, "AuroraView Browser");
    assert_eq!(config.width, 1280);
    assert_eq!(config.height, 900);
    assert_eq!(config.home_url, "https://www.google.com");
    assert!(!config.debug);
    assert!(config.initial_urls.is_empty());
}

#[test]
fn browser_config_default_frameless() {
    // Default config should be frameless (no native title bar)
    let config = BrowserConfig::default();
    assert!(config.frameless);
}

#[test]
fn browser_config_default_remote_debugging_port_zero() {
    let config = BrowserConfig::default();
    assert_eq!(config.remote_debugging_port, 0);
}

#[test]
fn browser_config_default_user_data_dir_none() {
    let config = BrowserConfig::default();
    assert!(config.user_data_dir.is_none());
}

// -------------------------------------------------------------------------
// Builder — individual setters
// -------------------------------------------------------------------------

#[test]
fn browser_config_builder() {
    let config = BrowserConfig::builder()
        .title("Custom Browser")
        .size(1400, 1000)
        .home_url("https://github.com")
        .theme(Theme::Dark)
        .bookmarks_bar(true)
        .history(true)
        .extensions(false)
        .dev_tools(true)
        .debug(true)
        .build();

    assert_eq!(config.title, "Custom Browser");
    assert_eq!(config.width, 1400);
    assert_eq!(config.height, 1000);
    assert_eq!(config.home_url, "https://github.com");
    assert!(matches!(config.theme, Theme::Dark));
    assert!(config.features.bookmarks_bar);
    assert!(config.features.history);
    assert!(!config.features.extensions);
    assert!(config.features.dev_tools);
    assert!(config.debug);
}

#[test]
fn browser_config_builder_initial_urls() {
    let config = BrowserConfig::builder()
        .initial_urls(vec![
            "https://google.com".to_string(),
            "https://github.com".to_string(),
        ])
        .build();

    assert_eq!(config.initial_urls.len(), 2);
    assert_eq!(config.initial_urls[0], "https://google.com");
    assert_eq!(config.initial_urls[1], "https://github.com");
}

#[test]
fn browser_config_builder_user_data_dir() {
    let config = BrowserConfig::builder()
        .user_data_dir("/path/to/data")
        .build();

    assert_eq!(config.user_data_dir, Some("/path/to/data".to_string()));
}

#[test]
fn browser_features_default() {
    let config = BrowserConfig::default();

    assert!(!config.features.bookmarks_bar);
    assert!(config.features.history);
    assert!(config.features.extensions);
    assert!(config.features.downloads);
    assert!(!config.features.dev_tools);
    assert!(config.features.context_menu);
}

// -------------------------------------------------------------------------
// Builder — remote debugging / devtools
// -------------------------------------------------------------------------

#[test]
fn remote_debugging_port_enables_cdp() {
    let config = BrowserConfig::builder()
        .remote_debugging_port(9222)
        .build();

    assert_eq!(config.remote_debugging_port, 9222);
    assert!(config.features.cdp_enabled);
}

#[test]
fn remote_debugging_port_zero_no_cdp() {
    let config = BrowserConfig::builder()
        .remote_debugging_port(0)
        .build();

    assert_eq!(config.remote_debugging_port, 0);
    // Port 0 should not force cdp_enabled
    assert!(!config.features.cdp_enabled);
}

#[test]
fn devtools_auto_open() {
    let config = BrowserConfig::builder()
        .devtools_auto_open(true)
        .build();

    assert!(config.devtools.auto_open);
}

#[test]
fn devtools_dock_side_bottom() {
    let config = BrowserConfig::builder()
        .devtools_dock_side(DockSide::Bottom)
        .build();

    assert!(matches!(config.devtools.dock_side, DockSide::Bottom));
}

#[test]
fn devtools_dock_side_left() {
    let config = BrowserConfig::builder()
        .devtools_dock_side(DockSide::Left)
        .build();

    assert!(matches!(config.devtools.dock_side, DockSide::Left));
}

#[test]
fn devtools_dock_side_undocked() {
    let config = BrowserConfig::builder()
        .devtools_dock_side(DockSide::Undocked)
        .build();

    assert!(matches!(config.devtools.dock_side, DockSide::Undocked));
}

// -------------------------------------------------------------------------
// Builder — frameless
// -------------------------------------------------------------------------

#[test]
fn frameless_can_be_disabled() {
    let config = BrowserConfig::builder()
        .frameless(false)
        .build();

    assert!(!config.frameless);
}

#[test]
fn frameless_default_is_true() {
    let config = BrowserConfig::builder().build();
    assert!(config.frameless);
}

// -------------------------------------------------------------------------
// Builder — feature toggles
// -------------------------------------------------------------------------

#[test]
fn all_features_disabled() {
    let config = BrowserConfig::builder()
        .bookmarks_bar(false)
        .history(false)
        .extensions(false)
        .downloads(false)
        .dev_tools(false)
        .build();

    assert!(!config.features.bookmarks_bar);
    assert!(!config.features.history);
    assert!(!config.features.extensions);
    assert!(!config.features.downloads);
    assert!(!config.features.dev_tools);
}

#[test]
fn all_features_enabled() {
    let config = BrowserConfig::builder()
        .bookmarks_bar(true)
        .history(true)
        .extensions(true)
        .downloads(true)
        .dev_tools(true)
        .build();

    assert!(config.features.bookmarks_bar);
    assert!(config.features.history);
    assert!(config.features.extensions);
    assert!(config.features.downloads);
    assert!(config.features.dev_tools);
}

// -------------------------------------------------------------------------
// Clone
// -------------------------------------------------------------------------

#[test]
fn browser_config_clone() {
    let config = BrowserConfig::builder()
        .title("Clone Test")
        .size(800, 600)
        .build();

    let cloned = config.clone();
    assert_eq!(cloned.title, config.title);
    assert_eq!(cloned.width, config.width);
    assert_eq!(cloned.height, config.height);
}

// -------------------------------------------------------------------------
// rstest parametrized — sizes
// -------------------------------------------------------------------------

#[rstest]
#[case(1280, 720)]
#[case(1920, 1080)]
#[case(2560, 1440)]
#[case(3840, 2160)]
#[case(800, 600)]
fn browser_config_size_variants(#[case] w: u32, #[case] h: u32) {
    let config = BrowserConfig::builder().size(w, h).build();
    assert_eq!(config.width, w);
    assert_eq!(config.height, h);
}

#[rstest]
#[case("https://google.com")]
#[case("https://github.com")]
#[case("https://rust-lang.org")]
#[case("file:///index.html")]
#[case("about:blank")]
fn browser_config_home_url_variants(#[case] url: &str) {
    let config = BrowserConfig::builder().home_url(url).build();
    assert_eq!(config.home_url, url);
}

#[rstest]
#[case("My Browser")]
#[case("AuroraView - Maya Panel")]
#[case("Tool v2.0")]
#[case("")]
fn browser_config_title_variants(#[case] title: &str) {
    let config = BrowserConfig::builder().title(title).build();
    assert_eq!(config.title, title);
}

// -------------------------------------------------------------------------
// rstest parametrized — initial URL counts
// -------------------------------------------------------------------------

#[rstest]
#[case(0)]
#[case(1)]
#[case(5)]
#[case(20)]
fn browser_config_initial_urls_count(#[case] count: usize) {
    let urls: Vec<String> = (0..count)
        .map(|i| format!("https://tab{}.example.com", i))
        .collect();

    let config = BrowserConfig::builder().initial_urls(urls.clone()).build();
    assert_eq!(config.initial_urls.len(), count);
}

// -------------------------------------------------------------------------
// rstest parametrized — debug flag
// -------------------------------------------------------------------------

#[rstest]
#[case(true)]
#[case(false)]
fn browser_config_debug_flag(#[case] debug: bool) {
    let config = BrowserConfig::builder().debug(debug).build();
    assert_eq!(config.debug, debug);
}

// -------------------------------------------------------------------------
// Send + Sync bounds
// -------------------------------------------------------------------------

#[test]
fn browser_config_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<BrowserConfig>();
    assert_sync::<BrowserConfig>();
}

#[test]
fn browser_features_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<BrowserFeatures>();
    assert_sync::<BrowserFeatures>();
}

// -------------------------------------------------------------------------
// BrowserFeatures serde roundtrip
// -------------------------------------------------------------------------

#[test]
fn browser_features_serde_roundtrip_default() {
    let features = BrowserFeatures::default();
    let json = serde_json::to_string(&features).unwrap();
    let restored: BrowserFeatures = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.bookmarks_bar, features.bookmarks_bar);
    assert_eq!(restored.history, features.history);
    assert_eq!(restored.extensions, features.extensions);
    assert_eq!(restored.downloads, features.downloads);
    assert_eq!(restored.dev_tools, features.dev_tools);
    assert_eq!(restored.context_menu, features.context_menu);
    assert_eq!(restored.cdp_enabled, features.cdp_enabled);
}

#[test]
fn browser_features_serde_roundtrip_all_enabled() {
    let features = BrowserFeatures {
        bookmarks_bar: true,
        history: true,
        extensions: true,
        downloads: true,
        dev_tools: true,
        context_menu: true,
        cdp_enabled: true,
    };
    let json = serde_json::to_string(&features).unwrap();
    let restored: BrowserFeatures = serde_json::from_str(&json).unwrap();
    assert!(restored.bookmarks_bar);
    assert!(restored.history);
    assert!(restored.extensions);
    assert!(restored.downloads);
    assert!(restored.dev_tools);
    assert!(restored.context_menu);
    assert!(restored.cdp_enabled);
}

// -------------------------------------------------------------------------
// Theme variant tests
// -------------------------------------------------------------------------

#[test]
fn browser_config_theme_light() {
    let config = BrowserConfig::builder().theme(Theme::Light).build();
    assert!(matches!(config.theme, Theme::Light));
}

#[test]
fn browser_config_theme_system() {
    let config = BrowserConfig::builder().theme(Theme::System).build();
    assert!(matches!(config.theme, Theme::System));
}

#[test]
fn browser_config_default_theme_is_system() {
    let config = BrowserConfig::default();
    assert!(matches!(config.theme, Theme::System));
}

#[rstest]
#[case(Theme::Light)]
#[case(Theme::Dark)]
#[case(Theme::System)]
fn browser_config_theme_variants(#[case] theme: Theme) {
    let config = BrowserConfig::builder().theme(theme.clone()).build();
    assert!(std::mem::discriminant(&config.theme) == std::mem::discriminant(&theme));
}

// -------------------------------------------------------------------------
// DevToolsConfig defaults
// -------------------------------------------------------------------------

#[test]
fn devtools_config_default_enabled() {
    let config = BrowserConfig::default();
    assert!(config.devtools.enabled);
}

#[test]
fn devtools_config_default_dock_side_right() {
    let config = BrowserConfig::default();
    assert!(matches!(config.devtools.dock_side, DockSide::Right));
}

#[test]
fn devtools_config_default_auto_open_false() {
    let config = BrowserConfig::default();
    assert!(!config.devtools.auto_open);
}

// -------------------------------------------------------------------------
// context_menu feature toggle
// -------------------------------------------------------------------------

#[test]
fn browser_features_context_menu_default_true() {
    let config = BrowserConfig::default();
    assert!(config.features.context_menu);
}

// -------------------------------------------------------------------------
// CDP feature — multiple port values
// -------------------------------------------------------------------------

#[rstest]
#[case(9222, true)]
#[case(9229, true)]
#[case(8080, true)]
#[case(1, true)]
#[case(0, false)]
fn remote_debugging_port_cdp_state(#[case] port: u16, #[case] expected_cdp: bool) {
    let config = BrowserConfig::builder()
        .remote_debugging_port(port)
        .build();
    assert_eq!(config.features.cdp_enabled, expected_cdp);
    assert_eq!(config.remote_debugging_port, port);
}

// -------------------------------------------------------------------------
// Builder — overwrite fields
// -------------------------------------------------------------------------

#[test]
fn builder_overwrite_title() {
    let config = BrowserConfig::builder()
        .title("First")
        .title("Second")
        .build();
    assert_eq!(config.title, "Second");
}

#[test]
fn builder_overwrite_size() {
    let config = BrowserConfig::builder()
        .size(640, 480)
        .size(1920, 1080)
        .build();
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
}

#[test]
fn builder_size_zero() {
    let config = BrowserConfig::builder().size(0, 0).build();
    assert_eq!(config.width, 0);
    assert_eq!(config.height, 0);
}

// -------------------------------------------------------------------------
// Builder — user_data_dir unicode path
// -------------------------------------------------------------------------

#[test]
fn builder_user_data_dir_unicode() {
    let path = "/tmp/アuroraview/テスト";
    let config = BrowserConfig::builder().user_data_dir(path).build();
    assert_eq!(config.user_data_dir, Some(path.to_string()));
}

// -------------------------------------------------------------------------
// Clone independence
// -------------------------------------------------------------------------

#[test]
fn browser_config_clone_independence() {
    let config = BrowserConfig::builder()
        .title("Original")
        .size(800, 600)
        .build();
    let mut cloned = config.clone();
    cloned.title = "Cloned".to_string();
    cloned.width = 1920;
    assert_eq!(config.title, "Original");
    assert_eq!(config.width, 800);
    assert_eq!(cloned.title, "Cloned");
    assert_eq!(cloned.width, 1920);
}

#[test]
fn browser_features_clone_independence() {
    let mut features = BrowserFeatures {
        history: false,
        ..BrowserFeatures::default()
    };
    let cloned = features.clone();
    features.history = true;
    assert!(!cloned.history);
    assert!(features.history);
}

// -------------------------------------------------------------------------
// DockSide serde roundtrip
// -------------------------------------------------------------------------

#[rstest]
#[case(DockSide::Right)]
#[case(DockSide::Bottom)]
#[case(DockSide::Left)]
#[case(DockSide::Undocked)]
fn dock_side_serde_roundtrip(#[case] side: DockSide) {
    let json = serde_json::to_string(&side).unwrap();
    let restored: DockSide = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, side);
}

// ============================================================================
// R15 Extensions
// ============================================================================

#[test]
fn browser_config_default_title_is_not_empty() {
    let cfg = BrowserConfig::default();
    assert!(!cfg.title.is_empty());
}

#[test]
fn browser_config_default_width_positive() {
    let cfg = BrowserConfig::default();
    assert!(cfg.width > 0);
}

#[test]
fn browser_config_default_height_positive() {
    let cfg = BrowserConfig::default();
    assert!(cfg.height > 0);
}

#[test]
fn browser_config_builder_title_custom() {
    let cfg = BrowserConfig::builder()
        .title("MyBrowser")
        .build();
    assert_eq!(cfg.title, "MyBrowser");
}

#[test]
fn browser_features_default_has_history_true() {
    let features = BrowserFeatures::default();
    assert!(features.history, "History should be enabled by default");
}

#[test]
fn dock_side_all_variants_not_equal_undocked() {
    let docked = [DockSide::Left, DockSide::Right, DockSide::Bottom];
    for s in &docked {
        assert_ne!(*s, DockSide::Undocked);
    }
}
