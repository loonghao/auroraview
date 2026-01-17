//! Tests for config module

use auroraview_browser::{BrowserConfig, Theme};

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
