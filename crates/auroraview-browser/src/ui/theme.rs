//! Theme system

use serde::{Deserialize, Serialize};

/// Theme selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Light theme (Edge-style)
    Light,
    /// Dark theme
    Dark,
    /// Follow system preference
    #[default]
    System,
    /// Custom theme
    Custom(Box<CustomTheme>),
}

impl Theme {
    /// Get theme colors
    pub fn colors(&self) -> ThemeColors {
        match self {
            Theme::Light => ThemeColors::light(),
            Theme::Dark => ThemeColors::dark(),
            Theme::System => {
                // Check system preference
                if is_dark_mode() {
                    ThemeColors::dark()
                } else {
                    ThemeColors::light()
                }
            }
            Theme::Custom(custom) => custom.colors.clone(),
        }
    }

    /// Get CSS for the theme
    pub fn css(&self) -> String {
        let colors = self.colors();
        format!(
            r#":root {{
    --bg-primary: {};
    --bg-secondary: {};
    --bg-tertiary: {};
    --bg-hover: {};
    --bg-active: {};
    --text-primary: {};
    --text-secondary: {};
    --text-disabled: {};
    --border-color: {};
    --accent-color: {};
    --accent-hover: {};
    --error-color: {};
    --success-color: {};
    --warning-color: {};
}}"#,
            colors.bg_primary,
            colors.bg_secondary,
            colors.bg_tertiary,
            colors.bg_hover,
            colors.bg_active,
            colors.text_primary,
            colors.text_secondary,
            colors.text_disabled,
            colors.border_color,
            colors.accent_color,
            colors.accent_hover,
            colors.error_color,
            colors.success_color,
            colors.warning_color,
        )
    }
}

/// Theme colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // Background colors
    pub bg_primary: String,
    pub bg_secondary: String,
    pub bg_tertiary: String,
    pub bg_hover: String,
    pub bg_active: String,

    // Text colors
    pub text_primary: String,
    pub text_secondary: String,
    pub text_disabled: String,

    // Border and accent
    pub border_color: String,
    pub accent_color: String,
    pub accent_hover: String,

    // Status colors
    pub error_color: String,
    pub success_color: String,
    pub warning_color: String,
}

impl ThemeColors {
    /// Light theme colors (Edge-style)
    pub fn light() -> Self {
        Self {
            bg_primary: "#ffffff".to_string(),
            bg_secondary: "#f3f3f3".to_string(),
            bg_tertiary: "#f9f9f9".to_string(),
            bg_hover: "rgba(0,0,0,0.04)".to_string(),
            bg_active: "rgba(0,0,0,0.08)".to_string(),
            text_primary: "#1a1a1a".to_string(),
            text_secondary: "#444444".to_string(),
            text_disabled: "#888888".to_string(),
            border_color: "#e5e5e5".to_string(),
            accent_color: "#0078d4".to_string(),
            accent_hover: "#106ebe".to_string(),
            error_color: "#d13438".to_string(),
            success_color: "#107c10".to_string(),
            warning_color: "#ffb900".to_string(),
        }
    }

    /// Dark theme colors
    pub fn dark() -> Self {
        Self {
            bg_primary: "#202020".to_string(),
            bg_secondary: "#2d2d2d".to_string(),
            bg_tertiary: "#383838".to_string(),
            bg_hover: "rgba(255,255,255,0.08)".to_string(),
            bg_active: "rgba(255,255,255,0.12)".to_string(),
            text_primary: "#ffffff".to_string(),
            text_secondary: "#b0b0b0".to_string(),
            text_disabled: "#666666".to_string(),
            border_color: "#3d3d3d".to_string(),
            accent_color: "#60cdff".to_string(),
            accent_hover: "#4cc2ff".to_string(),
            error_color: "#ff6b6b".to_string(),
            success_color: "#6ccb5f".to_string(),
            warning_color: "#ffc107".to_string(),
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::light()
    }
}

/// Custom theme definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomTheme {
    /// Theme name
    pub name: String,
    /// Theme colors
    pub colors: ThemeColors,
}

impl CustomTheme {
    /// Create a new custom theme
    pub fn new(name: impl Into<String>, colors: ThemeColors) -> Self {
        Self {
            name: name.into(),
            colors,
        }
    }
}

impl PartialEq for ThemeColors {
    fn eq(&self, other: &Self) -> bool {
        self.bg_primary == other.bg_primary
            && self.bg_secondary == other.bg_secondary
            && self.accent_color == other.accent_color
    }
}

impl Eq for ThemeColors {}

/// Check if system is in dark mode
#[cfg(target_os = "windows")]
fn is_dark_mode() -> bool {
    // Check Windows registry for dark mode setting
    use std::process::Command;

    let output = Command::new("reg")
        .args([
            "query",
            r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // If AppsUseLightTheme is 0, dark mode is enabled
        return stdout.contains("0x0");
    }

    false
}

#[cfg(not(target_os = "windows"))]
fn is_dark_mode() -> bool {
    // Default to light mode on non-Windows
    false
}
