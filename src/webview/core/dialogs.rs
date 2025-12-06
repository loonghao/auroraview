//! AuroraView Core - File and Message Dialogs
//!
//! This module contains dialog-related methods:
//! - File open/save dialogs
//! - Folder selection dialogs
//! - Message dialogs (alert, confirm, error)

use pyo3::prelude::*;

use super::AuroraView;

#[pymethods]
impl AuroraView {
    // ========================================
    // File Dialog Methods
    // ========================================

    /// Create a file dialog for opening files
    ///
    /// Args:
    ///     title (str, optional): Dialog title
    ///     directory (str, optional): Initial directory
    ///     allow_multiple (bool, optional): Allow multiple file selection
    ///     file_types (list[tuple[str, list[str]]], optional): File type filters
    ///
    /// Returns:
    ///     list[str] or None: Selected file path(s), or None if cancelled
    #[pyo3(signature = (title=None, directory=None, allow_multiple=false, file_types=None))]
    fn open_file_dialog(
        &self,
        title: Option<&str>,
        directory: Option<&str>,
        allow_multiple: bool,
        file_types: Option<Vec<(String, Vec<String>)>>,
    ) -> PyResult<Option<Vec<String>>> {
        let mut dialog = rfd::FileDialog::new();

        if let Some(t) = title {
            dialog = dialog.set_title(t);
        }

        if let Some(dir) = directory {
            dialog = dialog.set_directory(dir);
        }

        if let Some(types) = file_types {
            for (name, extensions) in types {
                let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
                dialog = dialog.add_filter(&name, &ext_refs);
            }
        }

        if allow_multiple {
            let result = dialog.pick_files();
            Ok(result.map(|paths| {
                paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect()
            }))
        } else {
            let result = dialog.pick_file();
            Ok(result.map(|path| vec![path.to_string_lossy().to_string()]))
        }
    }

    /// Create a file dialog for saving files
    ///
    /// Args:
    ///     title (str, optional): Dialog title
    ///     directory (str, optional): Initial directory
    ///     default_name (str, optional): Default file name
    ///     file_types (list[tuple[str, list[str]]], optional): File type filters
    ///
    /// Returns:
    ///     str or None: Selected file path, or None if cancelled
    #[pyo3(signature = (title=None, directory=None, default_name=None, file_types=None))]
    fn save_file_dialog(
        &self,
        title: Option<&str>,
        directory: Option<&str>,
        default_name: Option<&str>,
        file_types: Option<Vec<(String, Vec<String>)>>,
    ) -> PyResult<Option<String>> {
        let mut dialog = rfd::FileDialog::new();

        if let Some(t) = title {
            dialog = dialog.set_title(t);
        }

        if let Some(dir) = directory {
            dialog = dialog.set_directory(dir);
        }

        if let Some(name) = default_name {
            dialog = dialog.set_file_name(name);
        }

        if let Some(types) = file_types {
            for (name, extensions) in types {
                let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
                dialog = dialog.add_filter(&name, &ext_refs);
            }
        }

        let result = dialog.save_file();
        Ok(result.map(|path| path.to_string_lossy().to_string()))
    }

    /// Create a folder selection dialog
    ///
    /// Args:
    ///     title (str, optional): Dialog title
    ///     directory (str, optional): Initial directory
    ///
    /// Returns:
    ///     str or None: Selected folder path, or None if cancelled
    #[pyo3(signature = (title=None, directory=None))]
    fn select_folder_dialog(
        &self,
        title: Option<&str>,
        directory: Option<&str>,
    ) -> PyResult<Option<String>> {
        let mut dialog = rfd::FileDialog::new();

        if let Some(t) = title {
            dialog = dialog.set_title(t);
        }

        if let Some(dir) = directory {
            dialog = dialog.set_directory(dir);
        }

        let result = dialog.pick_folder();
        Ok(result.map(|path| path.to_string_lossy().to_string()))
    }

    /// Create a multi-folder selection dialog
    #[pyo3(signature = (title=None, directory=None))]
    fn select_folders_dialog(
        &self,
        title: Option<&str>,
        directory: Option<&str>,
    ) -> PyResult<Option<Vec<String>>> {
        let mut dialog = rfd::FileDialog::new();

        if let Some(t) = title {
            dialog = dialog.set_title(t);
        }

        if let Some(dir) = directory {
            dialog = dialog.set_directory(dir);
        }

        let result = dialog.pick_folders();
        Ok(result.map(|paths| {
            paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect()
        }))
    }

    // ========================================
    // Message Dialog Methods
    // ========================================

    /// Show a confirmation dialog with Yes/No buttons
    ///
    /// Returns:
    ///     bool: True if user clicked Yes, False if clicked No
    fn confirm_dialog(&self, title: &str, message: &str) -> PyResult<bool> {
        let result = rfd::MessageDialog::new()
            .set_title(title)
            .set_description(message)
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();
        Ok(result == rfd::MessageDialogResult::Yes)
    }

    /// Show an alert/info dialog with OK button
    fn alert_dialog(&self, title: &str, message: &str) -> PyResult<()> {
        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(message)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
        Ok(())
    }

    /// Show an error dialog with OK button
    fn error_dialog(&self, title: &str, message: &str) -> PyResult<()> {
        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(message)
            .set_level(rfd::MessageLevel::Error)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
        Ok(())
    }

    /// Show a dialog with OK/Cancel buttons
    ///
    /// Returns:
    ///     bool: True if user clicked OK, False if clicked Cancel
    fn ok_cancel_dialog(&self, title: &str, message: &str) -> PyResult<bool> {
        let result = rfd::MessageDialog::new()
            .set_title(title)
            .set_description(message)
            .set_buttons(rfd::MessageButtons::OkCancel)
            .show();
        Ok(result == rfd::MessageDialogResult::Ok)
    }
}
