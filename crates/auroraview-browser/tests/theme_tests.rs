//! Tests for Theme, ThemeColors, and CustomTheme

use auroraview_browser::ui::{CustomTheme, Theme, ThemeColors};
use rstest::rstest;

// -------------------------------------------------------------------------
// Theme enum — basic
// -------------------------------------------------------------------------

#[test]
fn theme_default() {
    let theme = Theme::default();
    assert!(matches!(theme, Theme::System));
}

#[test]
fn theme_light_colors() {
    let colors = ThemeColors::light();

    assert_eq!(colors.bg_primary, "#ffffff");
    assert_eq!(colors.text_primary, "#1a1a1a");
    assert_eq!(colors.accent_color, "#0078d4");
}

#[test]
fn theme_dark_colors() {
    let colors = ThemeColors::dark();

    assert_eq!(colors.bg_primary, "#202020");
    assert_eq!(colors.text_primary, "#ffffff");
    assert_eq!(colors.accent_color, "#60cdff");
}

#[test]
fn theme_css_generation() {
    let theme = Theme::Light;
    let css = theme.css();

    assert!(css.contains("--bg-primary"));
    assert!(css.contains("--text-primary"));
    assert!(css.contains("--accent-color"));
    assert!(css.contains("#ffffff")); // Light theme bg
}

#[test]
fn custom_theme() {
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
fn theme_colors_equality() {
    let light1 = ThemeColors::light();
    let light2 = ThemeColors::light();
    let dark = ThemeColors::dark();

    assert_eq!(light1, light2);
    assert_ne!(light1, dark);
}

// -------------------------------------------------------------------------
// ThemeColors — full field coverage
// -------------------------------------------------------------------------

#[test]
fn light_colors_all_fields() {
    let c = ThemeColors::light();
    assert_eq!(c.bg_primary, "#ffffff");
    assert_eq!(c.bg_secondary, "#f3f3f3");
    assert_eq!(c.bg_tertiary, "#f9f9f9");
    assert_eq!(c.bg_hover, "rgba(0,0,0,0.04)");
    assert_eq!(c.bg_active, "rgba(0,0,0,0.08)");
    assert_eq!(c.text_primary, "#1a1a1a");
    assert_eq!(c.text_secondary, "#444444");
    assert_eq!(c.text_disabled, "#888888");
    assert_eq!(c.border_color, "#e5e5e5");
    assert_eq!(c.accent_color, "#0078d4");
    assert_eq!(c.accent_hover, "#106ebe");
    assert_eq!(c.error_color, "#d13438");
    assert_eq!(c.success_color, "#107c10");
    assert_eq!(c.warning_color, "#ffb900");
}

#[test]
fn dark_colors_all_fields() {
    let c = ThemeColors::dark();
    assert_eq!(c.bg_primary, "#202020");
    assert_eq!(c.bg_secondary, "#2d2d2d");
    assert_eq!(c.bg_tertiary, "#383838");
    assert_eq!(c.bg_hover, "rgba(255,255,255,0.08)");
    assert_eq!(c.bg_active, "rgba(255,255,255,0.12)");
    assert_eq!(c.text_primary, "#ffffff");
    assert_eq!(c.text_secondary, "#b0b0b0");
    assert_eq!(c.text_disabled, "#666666");
    assert_eq!(c.border_color, "#3d3d3d");
    assert_eq!(c.accent_color, "#60cdff");
    assert_eq!(c.accent_hover, "#4cc2ff");
    assert_eq!(c.error_color, "#ff6b6b");
    assert_eq!(c.success_color, "#6ccb5f");
    assert_eq!(c.warning_color, "#ffc107");
}

#[test]
fn theme_colors_default_is_light() {
    let default = ThemeColors::default();
    let light = ThemeColors::light();
    assert_eq!(default, light);
}

#[test]
fn theme_colors_clone() {
    let original = ThemeColors::dark();
    let cloned = original.clone();
    assert_eq!(original, cloned);
    assert_eq!(cloned.bg_primary, "#202020");
}

// -------------------------------------------------------------------------
// Theme Clone
// -------------------------------------------------------------------------

#[test]
fn theme_light_clone() {
    let t = Theme::Light;
    let c = t.clone();
    assert!(matches!(c, Theme::Light));
}

#[test]
fn theme_dark_clone() {
    let t = Theme::Dark;
    let c = t.clone();
    assert!(matches!(c, Theme::Dark));
}

#[test]
fn theme_system_clone() {
    let t = Theme::System;
    let c = t.clone();
    assert!(matches!(c, Theme::System));
}

#[test]
fn theme_custom_clone() {
    let colors = ThemeColors::dark();
    let custom = CustomTheme::new("MyTheme", colors);
    let theme = Theme::Custom(Box::new(custom));
    let cloned = theme.clone();
    if let Theme::Custom(ct) = cloned {
        assert_eq!(ct.name, "MyTheme");
    } else {
        panic!("Expected Custom theme after clone");
    }
}

// -------------------------------------------------------------------------
// Theme serde roundtrip
// -------------------------------------------------------------------------

#[test]
fn theme_light_serde_roundtrip() {
    let theme = Theme::Light;
    let json = serde_json::to_string(&theme).unwrap();
    let restored: Theme = serde_json::from_str(&json).unwrap();
    assert!(matches!(restored, Theme::Light));
}

#[test]
fn theme_dark_serde_roundtrip() {
    let theme = Theme::Dark;
    let json = serde_json::to_string(&theme).unwrap();
    let restored: Theme = serde_json::from_str(&json).unwrap();
    assert!(matches!(restored, Theme::Dark));
}

#[test]
fn theme_system_serde_roundtrip() {
    let theme = Theme::System;
    let json = serde_json::to_string(&theme).unwrap();
    let restored: Theme = serde_json::from_str(&json).unwrap();
    assert!(matches!(restored, Theme::System));
}

#[test]
fn theme_colors_serde_roundtrip() {
    let colors = ThemeColors::dark();
    let json = serde_json::to_string(&colors).unwrap();
    let restored: ThemeColors = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.bg_primary, "#202020");
    assert_eq!(restored.accent_color, "#60cdff");
}

#[test]
fn custom_theme_serde_roundtrip() {
    let colors = ThemeColors::light();
    let custom = CustomTheme::new("Roundtrip Theme", colors);
    let json = serde_json::to_string(&custom).unwrap();
    let restored: CustomTheme = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.name, "Roundtrip Theme");
    assert_eq!(restored.colors.bg_primary, "#ffffff");
}

// -------------------------------------------------------------------------
// CSS output — dark and system
// -------------------------------------------------------------------------

#[test]
fn dark_theme_css_generation() {
    let theme = Theme::Dark;
    let css = theme.css();

    assert!(css.contains("--bg-primary"));
    assert!(css.contains("--accent-color"));
    assert!(css.contains("#202020")); // Dark theme bg
    assert!(css.contains("#60cdff")); // Dark theme accent
}

#[test]
fn css_contains_all_variables() {
    let css = Theme::Light.css();

    let expected_vars = [
        "--bg-primary",
        "--bg-secondary",
        "--bg-tertiary",
        "--bg-hover",
        "--bg-active",
        "--text-primary",
        "--text-secondary",
        "--text-disabled",
        "--border-color",
        "--accent-color",
        "--accent-hover",
        "--error-color",
        "--success-color",
        "--warning-color",
    ];

    for var in expected_vars {
        assert!(
            css.contains(var),
            "CSS should contain variable: {}",
            var
        );
    }
}

#[test]
fn css_starts_with_root() {
    let css = Theme::Light.css();
    assert!(css.trim_start().starts_with(":root"));
}

// -------------------------------------------------------------------------
// CustomTheme
// -------------------------------------------------------------------------

#[test]
fn custom_theme_new() {
    let colors = ThemeColors::dark();
    let custom = CustomTheme::new("Test Theme", colors.clone());
    assert_eq!(custom.name, "Test Theme");
    assert_eq!(custom.colors, colors);
}

#[test]
fn custom_theme_clone() {
    let colors = ThemeColors::light();
    let custom = CustomTheme::new("Cloneable", colors);
    let cloned = custom.clone();
    assert_eq!(cloned.name, "Cloneable");
}

#[test]
fn custom_theme_equality() {
    let colors = ThemeColors::light();
    let ct1 = CustomTheme::new("Same", colors.clone());
    let ct2 = CustomTheme::new("Same", colors);
    assert_eq!(ct1, ct2);
}

#[test]
fn custom_theme_inequality_by_name() {
    let colors = ThemeColors::light();
    let ct1 = CustomTheme::new("A", colors.clone());
    let ct2 = CustomTheme::new("B", colors);
    assert_ne!(ct1, ct2);
}

#[test]
fn custom_theme_css_reflects_colors() {
    let mut colors = ThemeColors::light();
    colors.accent_color = "#deadbe".to_string();
    let custom = CustomTheme::new("Custom Accent", colors);
    let theme = Theme::Custom(Box::new(custom));
    let css = theme.css();
    assert!(css.contains("#deadbe"));
}

// -------------------------------------------------------------------------
// Theme::colors() for all variants
// -------------------------------------------------------------------------

#[rstest]
#[case(Theme::Light, "#ffffff")]
#[case(Theme::Dark, "#202020")]
fn theme_colors_bg_primary(#[case] theme: Theme, #[case] expected_bg: &str) {
    let colors = theme.colors();
    assert_eq!(colors.bg_primary, expected_bg);
}

// -------------------------------------------------------------------------
// rstest parametrized
// -------------------------------------------------------------------------

#[rstest]
#[case("Midnight Blue", "#0a0a2e")]
#[case("Ocean", "#003366")]
#[case("Forest", "#1a3a1a")]
#[case("Warm Sand", "#f5e6d3")]
fn custom_theme_names_and_bg(#[case] name: &str, #[case] bg: &str) {
    let mut colors = ThemeColors::light();
    colors.bg_primary = bg.to_string();
    let custom = CustomTheme::new(name, colors);
    assert_eq!(custom.name, name);
    assert_eq!(custom.colors.bg_primary, bg);

    let theme = Theme::Custom(Box::new(custom));
    let css = theme.css();
    assert!(css.contains(bg));
}

// ─── Additional coverage R9 ──────────────────────────────────────────────────

#[test]
fn theme_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Theme>();
}

#[test]
fn theme_colors_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ThemeColors>();
}

#[test]
fn theme_light_debug() {
    let theme = Theme::Light;
    let debug_str = format!("{:?}", theme);
    assert!(debug_str.contains("Light"));
}

#[test]
fn theme_dark_debug() {
    let theme = Theme::Dark;
    let debug_str = format!("{:?}", theme);
    assert!(debug_str.contains("Dark"));
}

#[test]
fn theme_system_debug() {
    let theme = Theme::System;
    let debug_str = format!("{:?}", theme);
    assert!(debug_str.contains("System"));
}

#[test]
fn theme_colors_light_text_primary() {
    let colors = ThemeColors::light();
    assert_eq!(colors.text_primary, "#1a1a1a");
}

#[test]
fn theme_colors_dark_text_primary() {
    let colors = ThemeColors::dark();
    assert_eq!(colors.text_primary, "#ffffff");
}

#[test]
fn theme_colors_clone_independent() {
    let colors = ThemeColors::light();
    let mut cloned = colors.clone();
    cloned.bg_primary = "#abcdef".to_string();
    assert_eq!(colors.bg_primary, "#ffffff");
    assert_eq!(cloned.bg_primary, "#abcdef");
}

#[test]
fn custom_theme_with_dark_colors() {
    let colors = ThemeColors::dark();
    let custom = CustomTheme::new("Night", colors.clone());
    assert_eq!(custom.colors.bg_primary, colors.bg_primary);
}

#[test]
fn theme_css_not_empty() {
    assert!(!Theme::Light.css().is_empty());
    assert!(!Theme::Dark.css().is_empty());
}

#[test]
fn theme_colors_light_equal_to_another_light() {
    let c1 = ThemeColors::light();
    let c2 = ThemeColors::light();
    assert_eq!(c1, c2);
}

#[test]
fn theme_colors_inequality_light_dark() {
    let light = ThemeColors::light();
    let dark = ThemeColors::dark();
    assert_ne!(light, dark);
}

#[test]
fn theme_system_css_not_empty() {
    // System theme should resolve to a non-empty CSS (either light or dark)
    let css = Theme::System.css();
    assert!(!css.is_empty());
    assert!(css.contains("--bg-primary"));
}

#[rstest]
#[case(Theme::Light)]
#[case(Theme::Dark)]
fn theme_css_contains_colon(#[case] theme: Theme) {
    // CSS variable declarations always use ':' for assignment
    let css = theme.css();
    assert!(css.contains(':'));
}
