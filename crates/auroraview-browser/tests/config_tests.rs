//! Tests for BrowserConfig

use auroraview_browser::{BrowserConfig, Theme};
use auroraview_browser::devtools::DockSide;
use rstest::rstest;

// -------------------------------------------------------------------------
// Default values
// -------------------------------------------------------------------------

#[test]
fn test_browser_config_default() {
    let config = BrowserConfig::default();

    assert_eq!(config.title, "AuroraView Browser");
    assert_eq!(config.width, 1280);
    assert_eq!(config.height, 900);
    assert_eq!(config.home_url, "https://www.google.com");
    assert!(!config.debug);
    assert!(config.initial_urls.is_empty());
}

#[test]
fn test_browser_config_default_frameless() {
    // Default config should be frameless (no native title bar)
    let config = BrowserConfig::default();
    assert!(config.frameless);
}

#[test]
fn test_browser_config_default_remote_debugging_port_zero() {
    let config = BrowserConfig::default();
    assert_eq!(config.remote_debugging_port, 0);
}

#[test]
fn test_browser_config_default_user_data_dir_none() {
    let config = BrowserConfig::default();
    assert!(config.user_data_dir.is_none());
}

// -------------------------------------------------------------------------
// Builder — individual setters
// -------------------------------------------------------------------------

#[test]
fn test_browser_config_builder() {
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
fn test_browser_config_builder_initial_urls() {
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
fn test_browser_config_builder_user_data_dir() {
    let config = BrowserConfig::builder()
        .user_data_dir("/path/to/data")
        .build();

    assert_eq!(config.user_data_dir, Some("/path/to/data".to_string()));
}

#[test]
fn test_browser_features_default() {
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
fn test_remote_debugging_port_enables_cdp() {
    let config = BrowserConfig::builder()
        .remote_debugging_port(9222)
        .build();

    assert_eq!(config.remote_debugging_port, 9222);
    assert!(config.features.cdp_enabled);
}

#[test]
fn test_remote_debugging_port_zero_no_cdp() {
    let config = BrowserConfig::builder()
        .remote_debugging_port(0)
        .build();

    assert_eq!(config.remote_debugging_port, 0);
    // Port 0 should not force cdp_enabled
    assert!(!config.features.cdp_enabled);
}

#[test]
fn test_devtools_auto_open() {
    let config = BrowserConfig::builder()
        .devtools_auto_open(true)
        .build();

    assert!(config.devtools.auto_open);
}

#[test]
fn test_devtools_dock_side_bottom() {
    let config = BrowserConfig::builder()
        .devtools_dock_side(DockSide::Bottom)
        .build();

    assert!(matches!(config.devtools.dock_side, DockSide::Bottom));
}

#[test]
fn test_devtools_dock_side_left() {
    let config = BrowserConfig::builder()
        .devtools_dock_side(DockSide::Left)
        .build();

    assert!(matches!(config.devtools.dock_side, DockSide::Left));
}

#[test]
fn test_devtools_dock_side_undocked() {
    let config = BrowserConfig::builder()
        .devtools_dock_side(DockSide::Undocked)
        .build();

    assert!(matches!(config.devtools.dock_side, DockSide::Undocked));
}

// -------------------------------------------------------------------------
// Builder — frameless
// -------------------------------------------------------------------------

#[test]
fn test_frameless_can_be_disabled() {
    let config = BrowserConfig::builder()
        .frameless(false)
        .build();

    assert!(!config.frameless);
}

#[test]
fn test_frameless_default_is_true() {
    let config = BrowserConfig::builder().build();
    assert!(config.frameless);
}

// -------------------------------------------------------------------------
// Builder — feature toggles
// -------------------------------------------------------------------------

#[test]
fn test_all_features_disabled() {
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
fn test_all_features_enabled() {
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
fn test_browser_config_clone() {
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
fn test_browser_config_size_variants(#[case] w: u32, #[case] h: u32) {
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
fn test_browser_config_home_url_variants(#[case] url: &str) {
    let config = BrowserConfig::builder().home_url(url).build();
    assert_eq!(config.home_url, url);
}

#[rstest]
#[case("My Browser")]
#[case("AuroraView - Maya Panel")]
#[case("Tool v2.0")]
#[case("")]
fn test_browser_config_title_variants(#[case] title: &str) {
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
fn test_browser_config_initial_urls_count(#[case] count: usize) {
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
fn test_browser_config_debug_flag(#[case] debug: bool) {
    let config = BrowserConfig::builder().debug(debug).build();
    assert_eq!(config.debug, debug);
}
