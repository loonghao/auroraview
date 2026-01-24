//! System tray support for desktop mode

mod icon;
mod menu;

use crate::config::TrayConfig;
use crate::error::{DesktopError, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

/// System tray manager
pub struct TrayManager {
    _tray_icon: TrayIcon,
    pub menu_ids: HashMap<tray_icon::menu::MenuId, String>,
}

impl TrayManager {
    /// Create a new tray manager
    pub fn new(config: &TrayConfig, window_icon: Option<&PathBuf>) -> Result<Self> {
        // Create menu
        let menu = Menu::new();
        let mut menu_ids = HashMap::new();

        for item in &config.menu {
            match item {
                crate::config::TrayMenuItem::Item { id, label, enabled } => {
                    let menu_item = MenuItem::new(label, *enabled, None);
                    menu_ids.insert(menu_item.id().clone(), id.clone());
                    menu.append(&menu_item)
                        .map_err(|e| DesktopError::Tray(e.to_string()))?;
                }
                crate::config::TrayMenuItem::Separator => {
                    menu.append(&PredefinedMenuItem::separator())
                        .map_err(|e| DesktopError::Tray(e.to_string()))?;
                }
            }
        }

        // Load icon
        let icon = icon::load_tray_icon(config.icon.as_ref(), window_icon)?;

        // Create tray icon
        let mut builder = TrayIconBuilder::new().with_menu(Box::new(menu));

        if let Some(ref tooltip) = config.tooltip {
            builder = builder.with_tooltip(tooltip);
        }

        builder = builder.with_icon(icon);

        let tray_icon = builder
            .build()
            .map_err(|e| DesktopError::Tray(e.to_string()))?;

        Ok(Self {
            _tray_icon: tray_icon,
            menu_ids,
        })
    }
}
