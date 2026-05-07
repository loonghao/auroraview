//! Event loop handling for desktop mode.
//!
//! This module contains the `run_desktop` function which runs the event loop
//! after creating the WebView. The event loop handles window events, tray icons,
//! and Ctrl+C signals.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tao::event_loop::ControlFlow;
use tao::platform::run_return::EventLoopExtRunReturn;

#[cfg(target_os = "windows")]
use auroraview_core::builder::apply_frameless_popup_window_style;

use super::create_desktop;
use crate::ipc::{IpcHandler, MessageQueue};
use crate::webview::config::WebViewConfig;
use crate::webview::tray::TrayManager;

/// Run desktop WebView with event_loop.run() (blocking until window closes).
///
/// This function is designed for desktop applications where the WebView owns
/// the event loop and the process should exit when the window closes.
/// It uses event_loop.run() which calls std::process::exit() on completion.
///
/// IMPORTANT: This will terminate the entire process when the window closes!
/// Only use this for desktop mode, NOT for DCC integration (embedded mode).
///
/// Use cases:
/// - Desktop Python scripts
/// - CLI applications
/// - Desktop applications
pub fn run_desktop(
    config: WebViewConfig,
    ipc_handler: Arc<IpcHandler>,
    message_queue: Arc<MessageQueue>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tray_icon::menu::MenuEvent;
    use tray_icon::TrayIconEvent;

    // Save config values before config is consumed
    let auto_show = config.auto_show;
    let headless = config.headless;
    let tray_config = config.tray.clone();
    let window_icon = config.icon.clone();
    #[cfg(target_os = "windows")]
    let decorations = config.decorations;

    // Create the WebView
    let mut webview_inner = create_desktop(config, ipc_handler, message_queue)?;

    // Cache HWND for post-show style enforcement (Windows only)
    #[cfg(target_os = "windows")]
    let cached_hwnd = webview_inner.cached_hwnd;

    // Take ownership of event loop and window using take()
    let mut event_loop = webview_inner
        .event_loop
        .take()
        .ok_or("Event loop is None")?;
    let window = webview_inner.window.take().ok_or("Window is None")?;
    let webview = webview_inner.webview.clone();

    // Create system tray if configured
    let tray_manager = if let Some(ref tray_cfg) = tray_config {
        match TrayManager::new(tray_cfg, window_icon.as_ref()) {
            Ok(manager) => Some(manager),
            Err(e) => {
                tracing::warn!("[Standalone] Failed to create system tray: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Get tray settings for event handling
    let hide_on_close = tray_config.as_ref().is_some_and(|t| t.hide_on_close);
    let show_on_click = tray_config.as_ref().is_some_and(|t| t.show_on_click);
    let show_on_double_click = tray_config.as_ref().is_some_and(|t| t.show_on_double_click);

    // Store menu IDs for event handling
    let menu_ids = tray_manager
        .as_ref()
        .map(|m| Arc::new(m.menu_ids.clone()))
        .unwrap_or_else(|| Arc::new(std::collections::HashMap::new()));
    let menu_ids_clone = menu_ids.clone();

    // Window starts hidden - will be shown after a short delay to let loading screen render
    // (only if auto_show is enabled and not in headless mode)
    if headless {
        tracing::info!("[Standalone] Headless mode enabled, window will remain hidden");
    } else if auto_show {
        tracing::info!(
            "[Standalone] Window created (hidden), will show after loading screen renders..."
        );
    } else {
        tracing::info!(
            "[Standalone] Window created (hidden), auto_show=false, window will stay hidden"
        );
    }

    // Use a simple delay to ensure loading screen is rendered before showing window
    // This avoids the white flash that occurs when showing window before WebView is ready
    let show_time = std::time::Instant::now() + std::time::Duration::from_millis(100);
    // Window should only be shown if: auto_show is true AND headless is false
    let mut window_shown = !auto_show || headless;
    let mut window_visible = false;

    // Set up Ctrl+C handler with atomic flag
    let ctrlc_pressed = Arc::new(AtomicBool::new(false));
    let ctrlc_flag = ctrlc_pressed.clone();

    // Try to set up Ctrl+C handler (may fail if already set)
    if let Err(e) = ctrlc::try_set_handler(move || {
        tracing::info!("[Standalone] Ctrl+C received, requesting exit...");
        ctrlc_flag.store(true, Ordering::SeqCst);
    }) {
        tracing::warn!("[Standalone] Could not set Ctrl+C handler: {}", e);
    }

    let tray_enabled = tray_manager.is_some();
    tracing::info!(
        "[Standalone] Starting event loop with run_return() (Ctrl+C enabled, tray={})",
        tray_enabled
    );

    // Run the event loop using run_return() to allow Ctrl+C handling
    // This returns normally instead of calling std::process::exit()
    event_loop.run_return(move |event, _, control_flow| {
        // Check for Ctrl+C signal
        if ctrlc_pressed.load(Ordering::SeqCst) {
            tracing::info!("[Standalone] Ctrl+C detected, exiting event loop");
            *control_flow = ControlFlow::Exit;
            return;
        }

        // Process tray icon events
        if tray_enabled {
            if let Ok(event) = TrayIconEvent::receiver().try_recv() {
                match event {
                    TrayIconEvent::Click {
                        button: tray_icon::MouseButton::Left,
                        ..
                    } if show_on_click && !window_visible => {
                        tracing::debug!("[Standalone] Tray click - showing window");
                        window.set_visible(true);
                        #[cfg(target_os = "windows")]
                        {
                            if !decorations {
                                if let Some(hwnd) = cached_hwnd {
                                    let _ = apply_frameless_popup_window_style(hwnd as isize);
                                }
                            }
                        }
                        window.set_focus();
                        window_visible = true;
                    }
                    TrayIconEvent::DoubleClick {
                        button: tray_icon::MouseButton::Left,
                        ..
                    } if show_on_double_click => {
                        tracing::debug!("[Standalone] Tray double-click - toggling window");
                        if window_visible {
                            window.set_visible(false);
                            window_visible = false;
                        } else {
                            window.set_visible(true);
                            #[cfg(target_os = "windows")]
                            {
                                if !decorations {
                                    if let Some(hwnd) = cached_hwnd {
                                        let _ = apply_frameless_popup_window_style(hwnd as isize);
                                    }
                                }
                            }
                            window.set_focus();
                            window_visible = true;
                        }
                    }
                    _ => {}
                }
            }

            // Process menu events
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if let Some(id) = menu_ids_clone.get(&event.id) {
                    tracing::info!("[Standalone] Tray menu clicked: {}", id);
                    // Handle built-in menu actions
                    match id.as_str() {
                        "quit" | "exit" => {
                            tracing::info!("[Standalone] Quit menu clicked, exiting");
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                        "show" => {
                            window.set_visible(true);
                            #[cfg(target_os = "windows")]
                            {
                                if !decorations {
                                    if let Some(hwnd) = cached_hwnd {
                                        let _ = apply_frameless_popup_window_style(hwnd as isize);
                                    }
                                }
                            }
                            window.set_focus();
                            window_visible = true;
                        }
                        "hide" => {
                            window.set_visible(false);
                            window_visible = false;
                        }
                        _ => {
                            // Custom menu item - emit event to JavaScript
                            if let Ok(wv) = webview.lock() {
                                let js = format!(
                                    r#"(function() {{
                                        if (window.auroraview && window.auroraview.trigger) {{
                                            window.auroraview.trigger('tray_menu_click', {{ id: '{}' }});
                                        }}
                                    }})()"#,
                                    id
                                );
                                let _ = wv.evaluate_script(&js);
                            }
                        }
                    }
                }
            }
        }

        // Poll frequently to check if we should show the window (only if auto_show)
        // Also poll to check for Ctrl+C signal and tray events
        if (auto_show && !window_shown) || tray_enabled {
            *control_flow = ControlFlow::Poll;
        } else {
            // Use WaitUntil with a timeout to periodically check for Ctrl+C
            *control_flow = ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(100),
            );
        }

        // Keep webview and tray alive
        let _ = &webview;
        let _ = &tray_manager;

        // Show window after delay (once) - only if auto_show is enabled and not headless
        if !headless && auto_show && !window_shown && std::time::Instant::now() >= show_time {
            tracing::info!("[Standalone] Loading screen should be rendered, showing window now!");
            window.set_visible(true);
            #[cfg(target_os = "windows")]
            {
                if !decorations {
                    if let Some(hwnd) = cached_hwnd {
                        let _ = apply_frameless_popup_window_style(hwnd as isize);
                    }
                }
            }
            window.request_redraw();
            window_shown = true;
            window_visible = true;
        }

        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            if hide_on_close && tray_enabled {
                // Hide to tray instead of closing
                tracing::info!("[Standalone] Window close requested, hiding to tray");
                window.set_visible(false);
                window_visible = false;
            } else {
                tracing::info!("[Standalone] Window close requested, exiting");
                // Set Exit control flow - WebView and Window will be dropped automatically
                // This helps avoid the Chrome_WidgetWin_0 unregister error
                *control_flow = ControlFlow::Exit;
            }
        }
    });

    tracing::info!("[Standalone] Event loop exited normally");
    Ok(())
}
