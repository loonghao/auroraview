//! Tests for theme module

use auroraview_browser::ui::{CustomTheme, Theme, ThemeColors};

#[test]
fn test_theme_default() {
    let theme = Theme::default();
    assert!(matches!(theme, Theme::System));
}

#[test]
fn test_theme_light_colors() {
    let colors = ThemeColors::light();

    assert_eq!(colors.bg_primary, "#ffffff");
    assert_eq!(colors.text_primary, "#1a1a1a");
    assert_eq!(colors.accent_color, "#0078d4");
}

#[test]
fn test_theme_dark_colors() {
    let colors = ThemeColors::dark();

    assert_eq!(colors.bg_primary, "#202020");
    assert_eq!(colors.text_primary, "#ffffff");
    assert_eq!(colors.accent_color, "#60cdff");
}

#[test]
fn test_theme_css_generation() {
    let theme = Theme::Light;
    let css = theme.css();

    assert!(css.contains("--bg-primary"));
    assert!(css.contains("--text-primary"));
    assert!(css.contains("--accent-color"));
    assert!(css.contains("#ffffff")); // Light theme bg
}

#[test]
fn test_custom_theme() {
    let colors = ThemeColors {
        bg_primary: "#1a1a2e".to_string(),
        bg_secondary: "#16213e".to_string(),
        bg_tertiary: "#0f3460".to_string(),
        bg_hover: "rgba(255,255,255,0.1)".to_string(),
        bg_active: "rgba(255,255,255,0.15)".to_string(),
        text_primary: "#eaeaea".to_string(),
        text_secondary: "#a0a0a0".to_string(),
        text_disabled: "#606060".to_string(),
        border_color: "#2a2a4e".to_string(),
        accent_color: "#e94560".to_string(),
        accent_hover: "#ff6b6b".to_string(),
        error_color: "#ff4444".to_string(),
        success_color: "#00c853".to_string(),
        warning_color: "#ffab00".to_string(),
    };

    let custom = CustomTheme::new("Aurora Night", colors.clone());
    let theme = Theme::Custom(Box::new(custom));

    let css = theme.css();
    assert!(css.contains("#1a1a2e")); // Custom bg
    assert!(css.contains("#e94560")); // Custom accent
}

#[test]
fn test_theme_colors_equality() {
    let light1 = ThemeColors::light();
    let light2 = ThemeColors::light();
    let dark = ThemeColors::dark();

    assert_eq!(light1, light2);
    assert_ne!(light1, dark);
}
