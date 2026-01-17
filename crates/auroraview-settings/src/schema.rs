//! Setting schema definitions for validation and documentation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::value::SettingValue;

/// Type specification for a setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    /// Boolean type.
    Bool,
    /// Integer type with optional min/max.
    Integer {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<i64>,
    },
    /// Float type with optional min/max.
    Float {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },
    /// String type with optional pattern.
    String {
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
    },
    /// Enum type with allowed values.
    Enum { values: Vec<String> },
    /// Array type with item schema.
    Array {
        #[serde(skip_serializing_if = "Option::is_none")]
        items: Option<Box<SchemaType>>,
    },
    /// Object type.
    Object,
}

impl Default for SchemaType {
    fn default() -> Self {
        Self::String {
            pattern: None,
            max_length: None,
        }
    }
}

/// Schema definition for a setting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingSchema {
    /// The setting key (e.g., "appearance.theme").
    pub key: String,
    /// Human-readable title.
    pub title: String,
    /// Description of what this setting does.
    pub description: String,
    /// The type of this setting.
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    /// Default value.
    pub default: SettingValue,
    /// Category for grouping in UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Whether this setting requires restart.
    #[serde(default)]
    pub requires_restart: bool,
    /// Deprecation message if this setting is deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,
}

impl SettingSchema {
    /// Creates a new schema builder.
    pub fn builder(key: impl Into<String>) -> SchemaBuilder {
        SchemaBuilder::new(key)
    }

    /// Validates a value against this schema.
    pub fn validate(&self, value: &SettingValue) -> Result<(), String> {
        match (&self.schema_type, value) {
            (SchemaType::Bool, SettingValue::Bool(_)) => Ok(()),
            (SchemaType::Integer { min, max }, SettingValue::Integer(v)) => {
                if let Some(min) = min {
                    if v < min {
                        return Err(format!("Value {} is less than minimum {}", v, min));
                    }
                }
                if let Some(max) = max {
                    if v > max {
                        return Err(format!("Value {} is greater than maximum {}", v, max));
                    }
                }
                Ok(())
            }
            (SchemaType::Float { min, max }, SettingValue::Float(v)) => {
                if let Some(min) = min {
                    if v < min {
                        return Err(format!("Value {} is less than minimum {}", v, min));
                    }
                }
                if let Some(max) = max {
                    if v > max {
                        return Err(format!("Value {} is greater than maximum {}", v, max));
                    }
                }
                Ok(())
            }
            (SchemaType::String { max_length, .. }, SettingValue::String(v)) => {
                if let Some(max_len) = max_length {
                    if v.len() > *max_len {
                        return Err(format!(
                            "String length {} exceeds maximum {}",
                            v.len(),
                            max_len
                        ));
                    }
                }
                Ok(())
            }
            (SchemaType::Enum { values }, SettingValue::String(v)) => {
                if values.contains(v) {
                    Ok(())
                } else {
                    Err(format!(
                        "Value '{}' is not one of: {}",
                        v,
                        values.join(", ")
                    ))
                }
            }
            (SchemaType::Array { .. }, SettingValue::Array(_)) => Ok(()),
            (SchemaType::Object, SettingValue::Object(_)) => Ok(()),
            _ => Err(format!(
                "Type mismatch: expected {:?}, got {}",
                self.schema_type,
                value.type_name()
            )),
        }
    }
}

/// Builder for creating setting schemas.
pub struct SchemaBuilder {
    key: String,
    title: Option<String>,
    description: Option<String>,
    schema_type: SchemaType,
    default: SettingValue,
    category: Option<String>,
    requires_restart: bool,
    deprecated: Option<String>,
}

impl SchemaBuilder {
    /// Creates a new builder with the given key.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: None,
            description: None,
            schema_type: SchemaType::default(),
            default: SettingValue::Null,
            category: None,
            requires_restart: false,
            deprecated: None,
        }
    }

    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the type as boolean.
    pub fn bool_type(mut self) -> Self {
        self.schema_type = SchemaType::Bool;
        self
    }

    /// Sets the type as integer with optional bounds.
    pub fn integer_type(mut self, min: Option<i64>, max: Option<i64>) -> Self {
        self.schema_type = SchemaType::Integer { min, max };
        self
    }

    /// Sets the type as float with optional bounds.
    pub fn float_type(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.schema_type = SchemaType::Float { min, max };
        self
    }

    /// Sets the type as string.
    pub fn string_type(mut self) -> Self {
        self.schema_type = SchemaType::String {
            pattern: None,
            max_length: None,
        };
        self
    }

    /// Sets the type as enum with allowed values.
    pub fn enum_type(mut self, values: Vec<String>) -> Self {
        self.schema_type = SchemaType::Enum { values };
        self
    }

    /// Sets the default value.
    pub fn default(mut self, default: impl Into<SettingValue>) -> Self {
        self.default = default.into();
        self
    }

    /// Sets the category.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Marks this setting as requiring restart.
    pub fn requires_restart(mut self) -> Self {
        self.requires_restart = true;
        self
    }

    /// Marks this setting as deprecated.
    pub fn deprecated(mut self, message: impl Into<String>) -> Self {
        self.deprecated = Some(message.into());
        self
    }

    /// Builds the schema.
    pub fn build(self) -> SettingSchema {
        SettingSchema {
            key: self.key.clone(),
            title: self.title.unwrap_or_else(|| self.key.clone()),
            description: self.description.unwrap_or_default(),
            schema_type: self.schema_type,
            default: self.default,
            category: self.category,
            requires_restart: self.requires_restart,
            deprecated: self.deprecated,
        }
    }
}

/// Registry for setting schemas.
#[derive(Debug, Default)]
pub struct SchemaRegistry {
    schemas: HashMap<String, SettingSchema>,
}

impl SchemaRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a schema.
    pub fn register(&mut self, schema: SettingSchema) {
        self.schemas.insert(schema.key.clone(), schema);
    }

    /// Gets a schema by key.
    pub fn get(&self, key: &str) -> Option<&SettingSchema> {
        self.schemas.get(key)
    }

    /// Returns all schemas.
    pub fn all(&self) -> impl Iterator<Item = &SettingSchema> {
        self.schemas.values()
    }

    /// Returns schemas in a category.
    pub fn by_category<'a>(&'a self, category: &'a str) -> impl Iterator<Item = &'a SettingSchema> {
        self.schemas
            .values()
            .filter(move |s| s.category.as_deref() == Some(category))
    }

    /// Returns all unique categories.
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<_> = self
            .schemas
            .values()
            .filter_map(|s| s.category.clone())
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }
}
