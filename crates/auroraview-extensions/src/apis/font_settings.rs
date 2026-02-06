//! Chrome Font Settings API Implementation
//!
//! Provides functionality to manage Chrome's font settings.
//!
//! ## Features
//! - Get/set font family for different scripts
//! - Get/set default font size
//! - Get/set minimum font size
//! - Event notifications for changes

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Generic font family
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum GenericFamily {
    #[default]
    Standard,
    Serif,
    SansSerif,
    Fixed,
    Cursive,
    Fantasy,
    Math,
}

/// Script code (ISO 15924)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ScriptCode(pub String);

impl Default for ScriptCode {
    fn default() -> Self {
        ScriptCode("Zyyy".to_string()) // Common script
    }
}

/// Font name
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontName {
    /// Font family name
    pub font_id: String,
    /// Display name
    pub display_name: String,
}

/// Font details for get/set
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontDetails {
    /// Generic font family
    pub generic_family: GenericFamily,
    /// Script code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
}

/// Font settings state
#[derive(Debug, Clone)]
struct FontSettingsState {
    /// Font families per script and generic family
    fonts: HashMap<(String, GenericFamily), String>,
    /// Default font size
    default_font_size: i32,
    /// Default fixed font size
    default_fixed_font_size: i32,
    /// Minimum font size
    minimum_font_size: i32,
}

impl Default for FontSettingsState {
    fn default() -> Self {
        let mut fonts = HashMap::new();

        // Default fonts
        fonts.insert(
            ("Zyyy".to_string(), GenericFamily::Standard),
            "Times New Roman".to_string(),
        );
        fonts.insert(
            ("Zyyy".to_string(), GenericFamily::Serif),
            "Times New Roman".to_string(),
        );
        fonts.insert(
            ("Zyyy".to_string(), GenericFamily::SansSerif),
            "Arial".to_string(),
        );
        fonts.insert(
            ("Zyyy".to_string(), GenericFamily::Fixed),
            "Consolas".to_string(),
        );
        fonts.insert(
            ("Zyyy".to_string(), GenericFamily::Cursive),
            "Comic Sans MS".to_string(),
        );
        fonts.insert(
            ("Zyyy".to_string(), GenericFamily::Fantasy),
            "Impact".to_string(),
        );

        Self {
            fonts,
            default_font_size: 16,
            default_fixed_font_size: 13,
            minimum_font_size: 0,
        }
    }
}

/// Font Settings API handler
pub struct FontSettingsApi {
    /// Internal state
    state: Arc<RwLock<FontSettingsState>>,
}

impl Default for FontSettingsApi {
    fn default() -> Self {
        Self::new()
    }
}

impl FontSettingsApi {
    /// Create a new FontSettingsApi instance
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(FontSettingsState::default())),
        }
    }

    /// Clear all font settings
    pub fn clear_all_fonts(&self) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        *state = FontSettingsState::default();
        Ok(json!(null))
    }

    /// Get font
    pub fn get_font(&self, details: FontDetails) -> ExtensionResult<Value> {
        let state = self.state.read().unwrap();
        let script = details.script.unwrap_or_else(|| "Zyyy".to_string());
        let key = (script, details.generic_family);

        let font_id = state.fonts.get(&key).cloned().unwrap_or_default();
        Ok(json!({
            "fontId": font_id,
            "levelOfControl": "controllable_by_this_extension"
        }))
    }

    /// Set font
    pub fn set_font(&self, details: FontDetails, font_id: &str) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        let script = details.script.unwrap_or_else(|| "Zyyy".to_string());
        let key = (script, details.generic_family);

        state.fonts.insert(key, font_id.to_string());
        Ok(json!(null))
    }

    /// Clear font
    pub fn clear_font(&self, details: FontDetails) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        let script = details.script.unwrap_or_else(|| "Zyyy".to_string());
        let key = (script.clone(), details.generic_family.clone());

        // Reset to default
        let default_font = match details.generic_family {
            GenericFamily::Standard | GenericFamily::Serif => "Times New Roman",
            GenericFamily::SansSerif => "Arial",
            GenericFamily::Fixed => "Consolas",
            GenericFamily::Cursive => "Comic Sans MS",
            GenericFamily::Fantasy => "Impact",
            GenericFamily::Math => "Cambria Math",
        };

        state.fonts.insert(key, default_font.to_string());
        Ok(json!(null))
    }

    /// Get font list
    pub fn get_font_list(&self) -> ExtensionResult<Value> {
        // Return a list of common fonts
        let fonts = vec![
            FontName {
                font_id: "Arial".to_string(),
                display_name: "Arial".to_string(),
            },
            FontName {
                font_id: "Consolas".to_string(),
                display_name: "Consolas".to_string(),
            },
            FontName {
                font_id: "Courier New".to_string(),
                display_name: "Courier New".to_string(),
            },
            FontName {
                font_id: "Georgia".to_string(),
                display_name: "Georgia".to_string(),
            },
            FontName {
                font_id: "Impact".to_string(),
                display_name: "Impact".to_string(),
            },
            FontName {
                font_id: "Times New Roman".to_string(),
                display_name: "Times New Roman".to_string(),
            },
            FontName {
                font_id: "Trebuchet MS".to_string(),
                display_name: "Trebuchet MS".to_string(),
            },
            FontName {
                font_id: "Verdana".to_string(),
                display_name: "Verdana".to_string(),
            },
        ];
        Ok(serde_json::to_value(fonts)?)
    }

    /// Get default font size
    pub fn get_default_font_size(&self) -> ExtensionResult<Value> {
        let state = self.state.read().unwrap();
        Ok(json!({
            "pixelSize": state.default_font_size,
            "levelOfControl": "controllable_by_this_extension"
        }))
    }

    /// Set default font size
    pub fn set_default_font_size(&self, pixel_size: i32) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        state.default_font_size = pixel_size;
        Ok(json!(null))
    }

    /// Clear default font size
    pub fn clear_default_font_size(&self) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        state.default_font_size = 16;
        Ok(json!(null))
    }

    /// Get default fixed font size
    pub fn get_default_fixed_font_size(&self) -> ExtensionResult<Value> {
        let state = self.state.read().unwrap();
        Ok(json!({
            "pixelSize": state.default_fixed_font_size,
            "levelOfControl": "controllable_by_this_extension"
        }))
    }

    /// Set default fixed font size
    pub fn set_default_fixed_font_size(&self, pixel_size: i32) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        state.default_fixed_font_size = pixel_size;
        Ok(json!(null))
    }

    /// Clear default fixed font size
    pub fn clear_default_fixed_font_size(&self) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        state.default_fixed_font_size = 13;
        Ok(json!(null))
    }

    /// Get minimum font size
    pub fn get_minimum_font_size(&self) -> ExtensionResult<Value> {
        let state = self.state.read().unwrap();
        Ok(json!({
            "pixelSize": state.minimum_font_size,
            "levelOfControl": "controllable_by_this_extension"
        }))
    }

    /// Set minimum font size
    pub fn set_minimum_font_size(&self, pixel_size: i32) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        state.minimum_font_size = pixel_size;
        Ok(json!(null))
    }

    /// Clear minimum font size
    pub fn clear_minimum_font_size(&self) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        state.minimum_font_size = 0;
        Ok(json!(null))
    }

    /// Handle API call
    pub fn handle(&self, method: &str, params: Value) -> ExtensionResult<Value> {
        match method {
            "clearAllFonts" => self.clear_all_fonts(),
            "getFont" => {
                let details: FontDetails = serde_json::from_value(params)?;
                self.get_font(details)
            }
            "setFont" => {
                let details: FontDetails =
                    serde_json::from_value(params.get("details").cloned().unwrap_or_default())?;
                let font_id = params
                    .get("fontId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing fontId".into()))?;
                self.set_font(details, font_id)
            }
            "clearFont" => {
                let details: FontDetails = serde_json::from_value(params)?;
                self.clear_font(details)
            }
            "getFontList" => self.get_font_list(),
            "getDefaultFontSize" => self.get_default_font_size(),
            "setDefaultFontSize" => {
                let pixel_size = params
                    .get("pixelSize")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing pixelSize".into()))?
                    as i32;
                self.set_default_font_size(pixel_size)
            }
            "clearDefaultFontSize" => self.clear_default_font_size(),
            "getDefaultFixedFontSize" => self.get_default_fixed_font_size(),
            "setDefaultFixedFontSize" => {
                let pixel_size = params
                    .get("pixelSize")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing pixelSize".into()))?
                    as i32;
                self.set_default_fixed_font_size(pixel_size)
            }
            "clearDefaultFixedFontSize" => self.clear_default_fixed_font_size(),
            "getMinimumFontSize" => self.get_minimum_font_size(),
            "setMinimumFontSize" => {
                let pixel_size = params
                    .get("pixelSize")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing pixelSize".into()))?
                    as i32;
                self.set_minimum_font_size(pixel_size)
            }
            "clearMinimumFontSize" => self.clear_minimum_font_size(),
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_font() {
        let api = FontSettingsApi::new();
        let result = api
            .get_font(FontDetails {
                generic_family: GenericFamily::SansSerif,
                script: None,
            })
            .unwrap();

        let font_id = result.get("fontId").and_then(|v| v.as_str()).unwrap();
        assert_eq!(font_id, "Arial");
    }

    #[test]
    fn test_set_font() {
        let api = FontSettingsApi::new();
        api.set_font(
            FontDetails {
                generic_family: GenericFamily::SansSerif,
                script: None,
            },
            "Helvetica",
        )
        .unwrap();

        let result = api
            .get_font(FontDetails {
                generic_family: GenericFamily::SansSerif,
                script: None,
            })
            .unwrap();

        let font_id = result.get("fontId").and_then(|v| v.as_str()).unwrap();
        assert_eq!(font_id, "Helvetica");
    }

    #[test]
    fn test_font_size() {
        let api = FontSettingsApi::new();

        api.set_default_font_size(20).unwrap();
        let result = api.get_default_font_size().unwrap();
        let size = result.get("pixelSize").and_then(|v| v.as_i64()).unwrap();
        assert_eq!(size, 20);

        api.clear_default_font_size().unwrap();
        let result = api.get_default_font_size().unwrap();
        let size = result.get("pixelSize").and_then(|v| v.as_i64()).unwrap();
        assert_eq!(size, 16);
    }

    #[test]
    fn test_get_font_list() {
        let api = FontSettingsApi::new();
        let result = api.get_font_list().unwrap();
        let fonts: Vec<FontName> = serde_json::from_value(result).unwrap();
        assert!(!fonts.is_empty());
    }
}
