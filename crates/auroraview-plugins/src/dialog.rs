//! Dialog Plugin
//!
//! Provides native file/folder dialog capabilities.
//!
//! ## Commands
//!
//! - `open_file` - Open file picker dialog
//! - `open_files` - Open multiple file picker dialog
//! - `open_folder` - Open folder picker dialog
//! - `save_file` - Open save file dialog
//! - `message` - Show message dialog
//! - `confirm` - Show confirmation dialog
//!
//! ## Example
//!
//! ```javascript
//! // Open single file
//! const file = await auroraview.invoke("plugin:dialog|open_file", {
//!     title: "Select a file",
//!     filters: [{ name: "Images", extensions: ["png", "jpg"] }]
//! });
//!
//! // Open folder
//! const folder = await auroraview.invoke("plugin:dialog|open_folder", {
//!     title: "Select a folder"
//! });
//!
//! // Save file
//! const savePath = await auroraview.invoke("plugin:dialog|save_file", {
//!     title: "Save as",
//!     defaultPath: "document.txt"
//! });
//! ```

use crate::{PluginError, PluginHandler, PluginResult, ScopeConfig};
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Dialog plugin
pub struct DialogPlugin {
    name: String,
}

impl DialogPlugin {
    /// Create a new dialog plugin
    pub fn new() -> Self {
        Self {
            name: "dialog".to_string(),
        }
    }
}

impl Default for DialogPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// File filter for dialogs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileFilter {
    /// Filter name (e.g., "Images")
    pub name: String,
    /// File extensions (e.g., ["png", "jpg"])
    pub extensions: Vec<String>,
}

/// Options for file dialogs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDialogOptions {
    /// Dialog title
    #[serde(default)]
    pub title: Option<String>,
    /// Default path/directory
    #[serde(default)]
    pub default_path: Option<String>,
    /// File filters
    #[serde(default)]
    pub filters: Vec<FileFilter>,
    /// Default file name (for save dialog)
    #[serde(default)]
    pub default_name: Option<String>,
}

/// Options for message dialogs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageDialogOptions {
    /// Dialog title
    #[serde(default)]
    pub title: Option<String>,
    /// Message content
    pub message: String,
    /// Message level (info, warning, error)
    #[serde(default)]
    pub level: Option<String>,
    /// Button type (ok, ok_cancel, yes_no)
    #[serde(default)]
    pub buttons: Option<String>,
}

impl PluginHandler for DialogPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, command: &str, args: Value, _scope: &ScopeConfig) -> PluginResult<Value> {
        match command {
            "open_file" => {
                let opts: FileDialogOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut dialog = FileDialog::new();

                if let Some(title) = &opts.title {
                    dialog = dialog.set_title(title);
                }

                if let Some(path) = &opts.default_path {
                    dialog = dialog.set_directory(path);
                }

                for filter in &opts.filters {
                    let extensions: Vec<&str> =
                        filter.extensions.iter().map(|s| s.as_str()).collect();
                    dialog = dialog.add_filter(&filter.name, &extensions);
                }

                let result = dialog.pick_file();

                match result {
                    Some(path) => Ok(serde_json::json!({
                        "path": path.to_string_lossy().to_string(),
                        "cancelled": false
                    })),
                    None => Ok(serde_json::json!({
                        "path": null,
                        "cancelled": true
                    })),
                }
            }
            "open_files" => {
                let opts: FileDialogOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut dialog = FileDialog::new();

                if let Some(title) = &opts.title {
                    dialog = dialog.set_title(title);
                }

                if let Some(path) = &opts.default_path {
                    dialog = dialog.set_directory(path);
                }

                for filter in &opts.filters {
                    let extensions: Vec<&str> =
                        filter.extensions.iter().map(|s| s.as_str()).collect();
                    dialog = dialog.add_filter(&filter.name, &extensions);
                }

                let result = dialog.pick_files();

                match result {
                    Some(paths) => {
                        let paths: Vec<String> = paths
                            .iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();
                        Ok(serde_json::json!({
                            "paths": paths,
                            "cancelled": false
                        }))
                    }
                    None => Ok(serde_json::json!({
                        "paths": [],
                        "cancelled": true
                    })),
                }
            }
            "open_folder" => {
                let opts: FileDialogOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut dialog = FileDialog::new();

                if let Some(title) = &opts.title {
                    dialog = dialog.set_title(title);
                }

                if let Some(path) = &opts.default_path {
                    dialog = dialog.set_directory(path);
                }

                let result = dialog.pick_folder();

                match result {
                    Some(path) => Ok(serde_json::json!({
                        "path": path.to_string_lossy().to_string(),
                        "cancelled": false
                    })),
                    None => Ok(serde_json::json!({
                        "path": null,
                        "cancelled": true
                    })),
                }
            }
            "open_folders" => {
                let opts: FileDialogOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut dialog = FileDialog::new();

                if let Some(title) = &opts.title {
                    dialog = dialog.set_title(title);
                }

                if let Some(path) = &opts.default_path {
                    dialog = dialog.set_directory(path);
                }

                let result = dialog.pick_folders();

                match result {
                    Some(paths) => {
                        let paths: Vec<String> = paths
                            .iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();
                        Ok(serde_json::json!({
                            "paths": paths,
                            "cancelled": false
                        }))
                    }
                    None => Ok(serde_json::json!({
                        "paths": [],
                        "cancelled": true
                    })),
                }
            }
            "save_file" => {
                let opts: FileDialogOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut dialog = FileDialog::new();

                if let Some(title) = &opts.title {
                    dialog = dialog.set_title(title);
                }

                if let Some(path) = &opts.default_path {
                    dialog = dialog.set_directory(path);
                }

                if let Some(name) = &opts.default_name {
                    dialog = dialog.set_file_name(name);
                }

                for filter in &opts.filters {
                    let extensions: Vec<&str> =
                        filter.extensions.iter().map(|s| s.as_str()).collect();
                    dialog = dialog.add_filter(&filter.name, &extensions);
                }

                let result = dialog.save_file();

                match result {
                    Some(path) => Ok(serde_json::json!({
                        "path": path.to_string_lossy().to_string(),
                        "cancelled": false
                    })),
                    None => Ok(serde_json::json!({
                        "path": null,
                        "cancelled": true
                    })),
                }
            }
            "message" => {
                let opts: MessageDialogOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let level = match opts.level.as_deref() {
                    Some("warning") => MessageLevel::Warning,
                    Some("error") => MessageLevel::Error,
                    _ => MessageLevel::Info,
                };

                let buttons = match opts.buttons.as_deref() {
                    Some("ok_cancel") => MessageButtons::OkCancel,
                    Some("yes_no") => MessageButtons::YesNo,
                    Some("yes_no_cancel") => MessageButtons::YesNoCancelCustom(
                        "Yes".to_string(),
                        "No".to_string(),
                        "Cancel".to_string(),
                    ),
                    _ => MessageButtons::Ok,
                };

                let mut dialog = MessageDialog::new()
                    .set_level(level)
                    .set_buttons(buttons)
                    .set_description(&opts.message);

                if let Some(title) = &opts.title {
                    dialog = dialog.set_title(title);
                }

                let result = dialog.show();

                let response = match result {
                    MessageDialogResult::Ok => "ok",
                    MessageDialogResult::Cancel => "cancel",
                    MessageDialogResult::Yes => "yes",
                    MessageDialogResult::No => "no",
                    MessageDialogResult::Custom(s) => {
                        return Ok(serde_json::json!({ "response": s }));
                    }
                };

                Ok(serde_json::json!({ "response": response }))
            }
            "confirm" => {
                let opts: MessageDialogOptions = serde_json::from_value(args)
                    .map_err(|e| PluginError::invalid_args(e.to_string()))?;

                let mut dialog = MessageDialog::new()
                    .set_level(MessageLevel::Info)
                    .set_buttons(MessageButtons::YesNo)
                    .set_description(&opts.message);

                if let Some(title) = &opts.title {
                    dialog = dialog.set_title(title);
                }

                let result = dialog.show();
                let confirmed = matches!(result, MessageDialogResult::Yes);

                Ok(serde_json::json!({ "confirmed": confirmed }))
            }
            _ => Err(PluginError::command_not_found(command)),
        }
    }

    fn commands(&self) -> Vec<&str> {
        vec![
            "open_file",
            "open_files",
            "open_folder",
            "open_folders",
            "save_file",
            "message",
            "confirm",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_plugin_commands() {
        let plugin = DialogPlugin::new();
        let commands = plugin.commands();
        assert!(commands.contains(&"open_file"));
        assert!(commands.contains(&"open_files"));
        assert!(commands.contains(&"open_folder"));
        assert!(commands.contains(&"save_file"));
        assert!(commands.contains(&"message"));
        assert!(commands.contains(&"confirm"));
    }

    // Note: Dialog tests require user interaction and can't be automated
    // These tests just verify the plugin structure
}
