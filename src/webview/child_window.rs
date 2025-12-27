//! Child WebView window creation
//!
//! This module handles creating independent child WebView windows.
//! Child windows are created on the main thread to avoid WebView2's threading restrictions.

#[cfg(target_os = "windows")]
use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
use tao::dpi::LogicalSize;
#[cfg(target_os = "windows")]
use tao::event::{Event, WindowEvent};
#[cfg(target_os = "windows")]
use tao::event_loop::{ControlFlow, EventLoopBuilder};
#[cfg(target_os = "windows")]
use tao::platform::run_return::EventLoopExtRunReturn;
#[cfg(target_os = "windows")]
use tao::window::WindowBuilder;
#[cfg(target_os = "windows")]
use wry::WebViewBuilder;

/// Child window state for tracking open windows
#[cfg(target_os = "windows")]
struct ChildWindowState {
    should_close: bool,
}

/// Create a new child WebView window
///
/// This function creates an independent WebView window that runs its own event loop
/// in a separate thread. The window is fully independent from the parent window.
#[cfg(target_os = "windows")]
pub fn create_child_webview_window(
    url: &str,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tao::platform::windows::EventLoopBuilderExtWindows;

    let url = url.to_string();
    eprintln!(
        "[ChildWindow] Creating child window for: {} ({}x{})",
        url, width, height
    );

    // Spawn a new thread for the child window's event loop
    std::thread::spawn(move || {
        eprintln!("[ChildWindow] Thread started for: {}", url);

        // Create event loop for this child window
        let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

        // Create window
        let window = match WindowBuilder::new()
            .with_title(format!("AuroraView - {}", url))
            .with_inner_size(LogicalSize::new(width, height))
            .with_resizable(true)
            .build(&event_loop)
        {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[ChildWindow] Failed to create window: {}", e);
                return;
            }
        };

        eprintln!("[ChildWindow] Window created, creating WebView...");

        // Create WebView
        let webview = match WebViewBuilder::new()
            .with_url(&url)
            .with_devtools(true)
            .build(&window)
        {
            Ok(wv) => wv,
            Err(e) => {
                eprintln!("[ChildWindow] Failed to create WebView: {}", e);
                return;
            }
        };

        eprintln!("[ChildWindow] WebView created successfully!");

        // State for tracking window
        let state = Arc::new(Mutex::new(ChildWindowState {
            should_close: false,
        }));
        let state_clone = state.clone();

        // Keep webview alive
        let _webview = webview;

        // Run event loop
        let mut event_loop = event_loop;
        event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    eprintln!("[ChildWindow] Close requested");
                    if let Ok(mut s) = state_clone.lock() {
                        s.should_close = true;
                    }
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Destroyed,
                    ..
                } => {
                    eprintln!("[ChildWindow] Window destroyed");
                    *control_flow = ControlFlow::Exit;
                }
                Event::MainEventsCleared => {
                    // Check if we should close
                    if let Ok(s) = state_clone.lock() {
                        if s.should_close {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                _ => {}
            }
        });

        eprintln!("[ChildWindow] Event loop exited for: {}", url);
    });

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn create_child_webview_window(
    url: &str,
    _width: u32,
    _height: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // On non-Windows platforms, open in system browser
    tracing::warn!(
        "[ChildWindow] Child window not supported on this platform, opening in browser: {}",
        url
    );
    open::that(url)?;
    Ok(())
}
