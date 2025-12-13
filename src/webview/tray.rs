//! System tray support for desktop mode
//!
//! This module provides system tray functionality using the `tray-icon` crate.
//! It allows creating tray icons with context menus and handling tray events.

use std::path::PathBuf;

// Use muda re-exported from tray-icon for version compatibility
use tray_icon::menu::{CheckMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use super::config::{TrayConfig, TrayMenuItem, TrayMenuItemType};

/// System tray manager
pub struct TrayManager {
    /// The tray icon instance
    tray_icon: Option<TrayIcon>,
    /// Menu item ID to callback mapping (public for event loop access)
    pub menu_ids: std::collections::HashMap<MenuId, String>,
}

impl TrayManager {
    /// Create a new tray manager from configuration
    pub fn new(config: &TrayConfig, window_icon: Option<&PathBuf>) -> Result<Self, TrayError> {
        if !config.enabled {
            return Ok(Self {
                tray_icon: None,
                menu_ids: std::collections::HashMap::new(),
            });
        }

        // Load tray icon
        let icon = Self::load_icon(config.icon.as_ref().or(window_icon))?;

        // Build context menu
        let (menu, menu_ids) = Self::build_menu(&config.menu_items)?;

        // Build tray icon
        let mut builder = TrayIconBuilder::new().with_icon(icon);

        if let Some(tooltip) = &config.tooltip {
            builder = builder.with_tooltip(tooltip);
        }

        builder = builder.with_menu(Box::new(menu));

        let tray_icon = builder
            .build()
            .map_err(|e| TrayError::BuildFailed(e.to_string()))?;

        tracing::info!("[Tray] System tray icon created");

        Ok(Self {
            tray_icon: Some(tray_icon),
            menu_ids,
        })
    }

    /// Load icon from path or use default
    fn load_icon(icon_path: Option<&PathBuf>) -> Result<Icon, TrayError> {
        use ::image::GenericImageView;

        // Try custom icon first
        if let Some(path) = icon_path {
            if path.exists() {
                let img = ::image::open(path).map_err(|e| {
                    TrayError::IconLoadFailed(format!("Failed to load icon: {}", e))
                })?;
                let (width, height) = img.dimensions();
                let rgba = img.into_rgba8().into_raw();
                return Icon::from_rgba(rgba, width, height)
                    .map_err(|e| TrayError::IconLoadFailed(e.to_string()));
            }
        }

        // Use embedded default icon
        const DEFAULT_ICON_BYTES: &[u8] = include_bytes!("../../assets/icons/auroraview-32.png");
        let img = ::image::load_from_memory(DEFAULT_ICON_BYTES).map_err(|e| {
            TrayError::IconLoadFailed(format!("Failed to load default icon: {}", e))
        })?;
        let (width, height) = img.dimensions();
        let rgba = img.into_rgba8().into_raw();
        Icon::from_rgba(rgba, width, height).map_err(|e| TrayError::IconLoadFailed(e.to_string()))
    }

    /// Build menu from configuration
    fn build_menu(
        items: &[TrayMenuItem],
    ) -> Result<(Menu, std::collections::HashMap<MenuId, String>), TrayError> {
        let menu = Menu::new();
        let mut menu_ids = std::collections::HashMap::new();

        for item in items {
            Self::add_menu_item(&menu, item, &mut menu_ids)?;
        }

        Ok((menu, menu_ids))
    }

    /// Add a menu item to the menu
    fn add_menu_item(
        menu: &Menu,
        item: &TrayMenuItem,
        menu_ids: &mut std::collections::HashMap<MenuId, String>,
    ) -> Result<(), TrayError> {
        match &item.item_type {
            TrayMenuItemType::Normal => {
                let menu_item = MenuItem::new(&item.text, item.enabled, None);
                menu_ids.insert(menu_item.id().clone(), item.id.clone());
                menu.append(&menu_item)
                    .map_err(|e| TrayError::MenuBuildFailed(e.to_string()))?;
            }
            TrayMenuItemType::Separator => {
                menu.append(&PredefinedMenuItem::separator())
                    .map_err(|e| TrayError::MenuBuildFailed(e.to_string()))?;
            }
            TrayMenuItemType::Checkbox { checked } => {
                let menu_item = CheckMenuItem::new(&item.text, item.enabled, *checked, None);
                menu_ids.insert(menu_item.id().clone(), item.id.clone());
                menu.append(&menu_item)
                    .map_err(|e| TrayError::MenuBuildFailed(e.to_string()))?;
            }
            TrayMenuItemType::Submenu { items: sub_items } => {
                let submenu = Submenu::new(&item.text, item.enabled);
                for sub_item in sub_items {
                    Self::add_submenu_item(&submenu, sub_item, menu_ids)?;
                }
                menu_ids.insert(submenu.id().clone(), item.id.clone());
                menu.append(&submenu)
                    .map_err(|e| TrayError::MenuBuildFailed(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Add a menu item to a submenu
    fn add_submenu_item(
        submenu: &Submenu,
        item: &TrayMenuItem,
        menu_ids: &mut std::collections::HashMap<MenuId, String>,
    ) -> Result<(), TrayError> {
        match &item.item_type {
            TrayMenuItemType::Normal => {
                let menu_item = MenuItem::new(&item.text, item.enabled, None);
                menu_ids.insert(menu_item.id().clone(), item.id.clone());
                submenu
                    .append(&menu_item)
                    .map_err(|e| TrayError::MenuBuildFailed(e.to_string()))?;
            }
            TrayMenuItemType::Separator => {
                submenu
                    .append(&PredefinedMenuItem::separator())
                    .map_err(|e| TrayError::MenuBuildFailed(e.to_string()))?;
            }
            TrayMenuItemType::Checkbox { checked } => {
                let menu_item = CheckMenuItem::new(&item.text, item.enabled, *checked, None);
                menu_ids.insert(menu_item.id().clone(), item.id.clone());
                submenu
                    .append(&menu_item)
                    .map_err(|e| TrayError::MenuBuildFailed(e.to_string()))?;
            }
            TrayMenuItemType::Submenu { items: sub_items } => {
                let nested_submenu = Submenu::new(&item.text, item.enabled);
                for sub_item in sub_items {
                    Self::add_submenu_item(&nested_submenu, sub_item, menu_ids)?;
                }
                menu_ids.insert(nested_submenu.id().clone(), item.id.clone());
                submenu
                    .append(&nested_submenu)
                    .map_err(|e| TrayError::MenuBuildFailed(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Check if tray is enabled
    pub fn is_enabled(&self) -> bool {
        self.tray_icon.is_some()
    }

    /// Get the custom menu item ID for a MenuId
    #[allow(dead_code)]
    pub fn get_menu_item_id(&self, menu_id: &MenuId) -> Option<&String> {
        self.menu_ids.get(menu_id)
    }

    /// Update tooltip
    #[allow(dead_code)]
    pub fn set_tooltip(&self, tooltip: &str) -> Result<(), TrayError> {
        if let Some(tray) = &self.tray_icon {
            tray.set_tooltip(Some(tooltip))
                .map_err(|e| TrayError::UpdateFailed(e.to_string()))?;
        }
        Ok(())
    }

    /// Update icon
    #[allow(dead_code)]
    pub fn set_icon(&self, icon_path: &PathBuf) -> Result<(), TrayError> {
        if let Some(tray) = &self.tray_icon {
            let icon = Self::load_icon(Some(icon_path))?;
            tray.set_icon(Some(icon))
                .map_err(|e| TrayError::UpdateFailed(e.to_string()))?;
        }
        Ok(())
    }
}

impl Drop for TrayManager {
    fn drop(&mut self) {
        if self.tray_icon.is_some() {
            tracing::info!("[Tray] System tray icon destroyed");
        }
    }
}

/// Tray event types
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TrayEvent {
    /// Tray icon was clicked
    Click,
    /// Tray icon was double-clicked
    DoubleClick,
    /// Tray icon was right-clicked
    RightClick,
    /// A menu item was clicked
    MenuClick { id: String },
}

/// Tray error types
#[derive(Debug, thiserror::Error)]
pub enum TrayError {
    #[error("Failed to load tray icon: {0}")]
    IconLoadFailed(String),
    #[error("Failed to build tray: {0}")]
    BuildFailed(String),
    #[error("Failed to build menu: {0}")]
    MenuBuildFailed(String),
    #[error("Failed to update tray: {0}")]
    UpdateFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_menu_item_new() {
        let item = TrayMenuItem::new("test_id", "Test Item");
        assert_eq!(item.id, "test_id");
        assert_eq!(item.text, "Test Item");
        assert!(item.enabled);
        assert!(matches!(item.item_type, TrayMenuItemType::Normal));
    }

    #[test]
    fn test_tray_menu_item_separator() {
        let item = TrayMenuItem::separator();
        assert!(matches!(item.item_type, TrayMenuItemType::Separator));
    }

    #[test]
    fn test_tray_menu_item_checkbox() {
        let item = TrayMenuItem::checkbox("check_id", "Check Item", true);
        assert_eq!(item.id, "check_id");
        assert!(matches!(
            item.item_type,
            TrayMenuItemType::Checkbox { checked: true }
        ));
    }

    #[test]
    fn test_tray_menu_item_submenu() {
        let sub_items = vec![TrayMenuItem::new("sub1", "Sub Item 1")];
        let item = TrayMenuItem::submenu("submenu_id", "Submenu", sub_items);
        assert_eq!(item.id, "submenu_id");
        assert!(matches!(item.item_type, TrayMenuItemType::Submenu { .. }));
    }

    #[test]
    fn test_tray_config_default() {
        let config = TrayConfig::default();
        assert!(!config.enabled);
        assert!(config.tooltip.is_none());
        assert!(config.menu_items.is_empty());
    }
}
