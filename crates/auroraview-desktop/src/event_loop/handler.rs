//! Event loop handler for desktop mode

use crate::config::DesktopConfig;
use crate::error::Result;
use crate::event_loop::UserEvent;
use crate::ipc::IpcRouter;
use crate::tray::TrayManager;
use crate::window::create_window_with_router;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::run_return::EventLoopExtRunReturn;
use tracing::info;

/// Run the desktop event loop (blocking)
///
/// This function creates a window and runs the event loop until the window is closed.
pub fn run(config: DesktopConfig) -> Result<()> {
    run_with_router(config, None)
}

/// Run the desktop event loop with shared IPC router (blocking)
///
/// This function creates a window with a shared IPC router and runs the event loop.
pub fn run_with_router(config: DesktopConfig, router: Option<Arc<IpcRouter>>) -> Result<()> {
    // Create event loop
    #[cfg(target_os = "windows")]
    let mut event_loop = {
        use tao::platform::windows::EventLoopBuilderExtWindows;
        EventLoopBuilder::<UserEvent>::with_user_event()
            .with_any_thread(true)
            .build()
    };

    #[cfg(not(target_os = "windows"))]
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // Save config for tray
    let tray_config = config.tray.clone();
    let window_icon = config.icon.clone();
    let auto_show = config.visible;

    // Create window with optional shared router
    let desktop_window = create_window_with_router(config, &event_loop, router)?;
    let window = &desktop_window.window;
    let webview = desktop_window.webview.clone();

    // Create system tray if configured
    let _tray_manager = if let Some(ref tray_cfg) = tray_config {
        TrayManager::new(tray_cfg, window_icon.as_ref()).ok()
    } else {
        None
    };

    // Window visibility tracking
    let show_time = std::time::Instant::now() + std::time::Duration::from_millis(100);
    let mut window_shown = !auto_show;

    // Ctrl+C handler
    let ctrlc_pressed = Arc::new(AtomicBool::new(false));
    let ctrlc_flag = ctrlc_pressed.clone();
    let _ = ctrlc::try_set_handler(move || {
        info!("[desktop] Ctrl+C received");
        ctrlc_flag.store(true, Ordering::SeqCst);
    });

    info!("[desktop] Starting event loop");

    // Run event loop
    event_loop.run_return(move |event, _, control_flow| {
        // Check Ctrl+C
        if ctrlc_pressed.load(Ordering::SeqCst) {
            info!("[desktop] Ctrl+C detected, exiting");
            *control_flow = ControlFlow::Exit;
            return;
        }

        // Default to wait with timeout
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(100),
        );

        // Keep webview alive
        let _ = &webview;

        // Show window after delay
        if !window_shown && std::time::Instant::now() >= show_time {
            info!("[desktop] Showing window");
            window.set_visible(true);
            window_shown = true;
        }

        // Handle events
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("[desktop] Window close requested");
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(user_event) => match user_event {
                UserEvent::CloseWindow => {
                    info!("[desktop] Close window event");
                    *control_flow = ControlFlow::Exit;
                }
                UserEvent::ShowWindow => {
                    window.set_visible(true);
                }
                UserEvent::HideWindow => {
                    window.set_visible(false);
                }
                UserEvent::DragWindow => {
                    let _ = window.drag_window();
                }
                UserEvent::EvalJs(script) => {
                    if let Ok(wv) = webview.lock() {
                        let _ = wv.evaluate_script(&script);
                    }
                }
                UserEvent::PluginEvent { event, data } => {
                    if let Ok(wv) = webview.lock() {
                        let js = format!(
                            r#"(function() {{
                                if (window.auroraview && window.auroraview.trigger) {{
                                    window.auroraview.trigger('{}', {});
                                }}
                            }})()"#,
                            event, data
                        );
                        let _ = wv.evaluate_script(&js);
                    }
                }
                UserEvent::WakeUp => {}
            },
            _ => {}
        }
    });

    info!("[desktop] Event loop exited");
    Ok(())
}
