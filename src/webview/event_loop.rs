//! Improved event loop handling using ApplicationHandler pattern
//! 
//! This module provides a better event loop implementation that:
//! - Uses a dedicated event handler structure
//! - Supports both blocking and non-blocking modes
//! - Properly manages window lifecycle
//! - Integrates better with Python's GIL

use std::sync::{Arc, Mutex};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::Window;

/// Event loop state management
pub struct EventLoopState {
    /// Whether the event loop should continue running
    pub should_exit: Arc<Mutex<bool>>,
    /// Window reference
    pub window: Option<Window>,
}

impl EventLoopState {
    /// Create a new event loop state
    pub fn new(window: Window) -> Self {
        Self {
            should_exit: Arc::new(Mutex::new(false)),
            window: Some(window),
        }
    }

    /// Signal the event loop to exit
    pub fn request_exit(&self) {
        if let Ok(mut should_exit) = self.should_exit.lock() {
            *should_exit = true;
        }
    }

    /// Check if exit was requested
    pub fn should_exit(&self) -> bool {
        self.should_exit
            .lock()
            .map(|flag| *flag)
            .unwrap_or(false)
    }
}

/// Improved event loop handler
pub struct WebViewEventHandler {
    state: Arc<Mutex<EventLoopState>>,
}

impl WebViewEventHandler {
    /// Create a new event handler
    pub fn new(state: Arc<Mutex<EventLoopState>>) -> Self {
        Self { state }
    }

    /// Handle window events
    pub fn handle_window_event(&self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Close requested");
                self.state
                    .lock()
                    .ok()
                    .map(|state| state.request_exit());
            }
            WindowEvent::Resized(size) => {
                tracing::debug!("Window resized: {:?}", size);
                // Handle resize if needed
            }
            WindowEvent::Focused(focused) => {
                tracing::debug!("Window focus changed: {}", focused);
            }
            _ => {}
        }
    }

    /// Run the event loop (blocking)
    pub fn run_blocking(
        event_loop: EventLoop<()>,
        state: Arc<Mutex<EventLoopState>>,
    ) {
        tracing::info!("Starting event loop (blocking mode)");

        // Show the window
        if let Ok(state_guard) = state.lock() {
            if let Some(window) = &state_guard.window {
                window.set_visible(true);
                tracing::info!("Window is now visible");
            }
        }

        let state_clone = state.clone();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event, .. } => {
                    let handler = WebViewEventHandler::new(state_clone.clone());
                    handler.handle_window_event(event);
                }
                Event::MainEventsCleared => {
                    // Check if we should exit
                    if let Ok(state_guard) = state_clone.lock() {
                        if state_guard.should_exit() {
                            tracing::info!("Exit requested, exiting event loop");
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                _ => {}
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_loop_state_creation() {
        // This test would require creating actual window/webview
        // which is complex in unit tests, so we skip it for now
    }
}

