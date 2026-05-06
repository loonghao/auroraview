//! Window and event loop creation for desktop mode.
//!
//! This module handles creating the window and event loop for desktop mode WebViews.
//! It extracts the window creation logic from `create_desktop` into a reusable function.

use std::sync::atomic::Ordering;

use tao::event_loop::{EventLoopBuilder, EventLoopProxy};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::window::WindowBuilder;
use wry::WebViewBuilder as WryWebViewBuilder;

use super::config::WebViewConfig;
use super::event_loop::UserEvent;

/// Creates the window and event loop for a desktop WebView.
///
/// Returns the `Window` and `EventLoop<UserEvent>` for further use in WebView creation.
pub fn create_window_and_event_loop(
    config: &WebViewConfig,
) -> Result<
    (
        tao::window::Window,
        tao::event_loop::EventLoop<UserEvent>,
    ),
    Box<dyn std::error::Error>,
> {
    // Initialize COM for WebView2 on Windows (using shared utility)
    #[cfg(target_os = "windows")]
    auroraview_core::builder::init_com_sta();

    // Allow event loop to be created on any thread (required for DCC integration)
    // Use UserEvent for custom events (wake-up for immediate message processing)
    #[cfg(target_os = "windows")]
    let event_loop = {
        use tao::platform::windows::EventLoopBuilderExtWindows;
        EventLoopBuilder::<UserEvent>::with_user_event()
            .with_any_thread(true)
            .build()
    };

    #[cfg(not(target_os = "windows"))]
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
    let mut window_builder = WindowBuilder::new()
        .with_title(&config.title)
        .with_resizable(config.resizable)
        .with_decorations(config.decorations)
        .with_transparent(config.transparent)
        .with_always_on_top(config.always_on_top)
        .with_visible(false); // Start hidden to avoid white flash

    // On Windows, apply platform-specific window styles for transparent windows
    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::WindowBuilderExtWindows;

        // If this window is intended to be a floating tool window, hide it from taskbar.
        if config.tool_window {
            window_builder = window_builder.with_skip_taskbar(true);
        }

        // For transparent frameless windows, disable shadow at creation time
        if config.transparent && !config.decorations && !config.undecorated_shadow {
            window_builder = window_builder.with_undecorated_shadow(false);
        }
    }

    // Set window icon (custom or default)
    if let Some(icon) = super::icon::load_window_icon(config.icon.as_ref()) {
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    // If width or height is 0, maximize the window; otherwise set the size
    if config.width == 0 || config.height == 0 {
        window_builder = window_builder.with_maximized(true);
    } else {
        window_builder =
            window_builder.with_inner_size(tao::dpi::LogicalSize::new(config.width, config.height));
    }

    // Parent/owner on Windows
    #[cfg(target_os = "windows")]
    {
        use crate::webview::config::EmbedMode;
        use tao::platform::windows::WindowBuilderExtWindows;

        if let Some(parent) = config.parent_hwnd {
            match config.embed_mode {
                EmbedMode::Child => {
                    window_builder = window_builder
                        .with_decorations(false)
                        .with_parent_window(parent as isize);
                }
                EmbedMode::Owner => {
                    // Owner relationship set after window creation
                }
                EmbedMode::None => {
                    // Standalone window mode - no parent relationship
                }
            }
        }
    }

    let window = window_builder.build(&event_loop)?;

    // Apply window styles based on embed mode (Windows only)
    #[cfg(target_os = "windows")]
    {
        use crate::webview::config::EmbedMode;
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};

        if let Ok(window_handle) = window.window_handle() {
            if let RawWindowHandle::Win32(handle) = window_handle.as_raw() {
                let hwnd = handle.hwnd.get();

                if let Some(parent) = config.parent_hwnd {
                    match config.embed_mode {
                        EmbedMode::Child => {
                            let _ = auroraview_core::builder::apply_child_window_style(
                                hwnd,
                                parent as isize,
                                auroraview_core::builder::ChildWindowStyleOptions::for_standalone(),
                            );
                        }
                        EmbedMode::Owner => {
                            // Set owner relationship using GWLP_HWNDPARENT
                            auroraview_core::builder::apply_owner_window_style(
                                hwnd, parent, config.tool_window,
                            );
                        }
                        EmbedMode::None => {}
                    }
                }
            }
        }
    }

    Ok((window, event_loop))
}
