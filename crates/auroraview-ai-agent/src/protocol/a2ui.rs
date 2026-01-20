//! A2UI (Agent to UI) Protocol implementation
//!
//! A2UI enables AI agents to generate and manipulate UI components
//! dynamically, providing a structured way for agents to create
//! rich interactive interfaces.
//!
//! Reference: https://github.com/nicepkg/a2ui

use serde::{Deserialize, Serialize};

/// UI Component specification from A2UI protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIComponentSpec {
    /// Component type
    #[serde(rename = "type")]
    pub component_type: UIComponentType,

    /// Component ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Component props
    #[serde(default)]
    pub props: serde_json::Value,

    /// Child components
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<UIComponentSpec>,
}

impl UIComponentSpec {
    /// Create a new component spec
    pub fn new(component_type: UIComponentType) -> Self {
        Self {
            component_type,
            id: None,
            props: serde_json::Value::Object(Default::default()),
            children: Vec::new(),
        }
    }

    /// Set component ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set props
    pub fn with_props(mut self, props: serde_json::Value) -> Self {
        self.props = props;
        self
    }

    /// Add child component
    pub fn with_child(mut self, child: UIComponentSpec) -> Self {
        self.children.push(child);
        self
    }
}

/// Standard UI component types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UIComponentType {
    // Layout components
    /// Container/div
    Container,
    /// Flexbox row
    Row,
    /// Flexbox column
    Column,
    /// Grid layout
    Grid,
    /// Card component
    Card,

    // Text components
    /// Heading
    Heading,
    /// Paragraph
    Text,
    /// Code block
    Code,
    /// Markdown content
    Markdown,

    // Interactive components
    /// Button
    Button,
    /// Input field
    Input,
    /// Text area
    TextArea,
    /// Select/dropdown
    Select,
    /// Checkbox
    Checkbox,
    /// Radio button
    Radio,
    /// Slider
    Slider,
    /// Toggle/switch
    Toggle,

    // Data display
    /// Table
    Table,
    /// List
    List,
    /// Image
    Image,
    /// Chart/graph
    Chart,
    /// Progress bar
    Progress,

    // Feedback
    /// Alert/notification
    Alert,
    /// Tooltip
    Tooltip,
    /// Modal dialog
    Modal,
    /// Loading spinner
    Loading,

    // Custom
    /// Custom component type
    Custom(String),
}

/// A2UI Action that agent can perform on UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UIAction {
    /// Render new UI tree
    Render { root: UIComponentSpec },

    /// Update specific component
    Update {
        id: String,
        props: serde_json::Value,
    },

    /// Append child to component
    AppendChild {
        parent_id: String,
        child: UIComponentSpec,
    },

    /// Remove component
    Remove { id: String },

    /// Replace component
    Replace {
        id: String,
        component: UIComponentSpec,
    },

    /// Show modal
    ShowModal {
        content: UIComponentSpec,
        #[serde(default)]
        closable: bool,
    },

    /// Hide modal
    HideModal,

    /// Show notification
    Notify {
        message: String,
        #[serde(default)]
        level: NotifyLevel,
        #[serde(default)]
        duration_ms: Option<u64>,
    },
}

/// Notification level
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NotifyLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Helper functions to create common UI components
pub mod builders {
    use super::*;
    use serde_json::json;

    /// Create a text component
    pub fn text(content: &str) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Text).with_props(json!({
            "content": content
        }))
    }

    /// Create a heading component
    pub fn heading(content: &str, level: u8) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Heading).with_props(json!({
            "content": content,
            "level": level
        }))
    }

    /// Create a button component
    pub fn button(label: &str, action: &str) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Button).with_props(json!({
            "label": label,
            "action": action
        }))
    }

    /// Create an input component
    pub fn input(placeholder: &str, name: &str) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Input).with_props(json!({
            "placeholder": placeholder,
            "name": name
        }))
    }

    /// Create a card component
    pub fn card(title: &str) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Card).with_props(json!({
            "title": title
        }))
    }

    /// Create a container with children
    pub fn container(children: Vec<UIComponentSpec>) -> UIComponentSpec {
        let mut spec = UIComponentSpec::new(UIComponentType::Container);
        spec.children = children;
        spec
    }

    /// Create a row layout
    pub fn row(children: Vec<UIComponentSpec>) -> UIComponentSpec {
        let mut spec = UIComponentSpec::new(UIComponentType::Row);
        spec.children = children;
        spec
    }

    /// Create a column layout
    pub fn column(children: Vec<UIComponentSpec>) -> UIComponentSpec {
        let mut spec = UIComponentSpec::new(UIComponentType::Column);
        spec.children = children;
        spec
    }

    /// Create a code block
    pub fn code(content: &str, language: Option<&str>) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Code).with_props(json!({
            "content": content,
            "language": language
        }))
    }

    /// Create a markdown component
    pub fn markdown(content: &str) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Markdown).with_props(json!({
            "content": content
        }))
    }

    /// Create an alert component
    pub fn alert(message: &str, level: NotifyLevel) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Alert).with_props(json!({
            "message": message,
            "level": level
        }))
    }

    /// Create a progress bar
    pub fn progress(value: f64, max: f64) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Progress).with_props(json!({
            "value": value,
            "max": max
        }))
    }

    /// Create a table from data
    pub fn table(headers: Vec<&str>, rows: Vec<Vec<serde_json::Value>>) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Table).with_props(json!({
            "headers": headers,
            "rows": rows
        }))
    }

    /// Create an image component
    pub fn image(src: &str, alt: Option<&str>) -> UIComponentSpec {
        UIComponentSpec::new(UIComponentType::Image).with_props(json!({
            "src": src,
            "alt": alt
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use builders::*;

    #[test]
    fn test_component_builder() {
        let ui = container(vec![
            heading("Welcome", 1),
            text("This is a test."),
            button("Click me", "handle_click"),
        ]);

        assert_eq!(ui.component_type, UIComponentType::Container);
        assert_eq!(ui.children.len(), 3);
    }

    #[test]
    fn test_ui_action_serialization() {
        let action = UIAction::Notify {
            message: "Hello!".to_string(),
            level: NotifyLevel::Success,
            duration_ms: Some(3000),
        };

        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("notify"));
        assert!(json.contains("success"));
    }
}
