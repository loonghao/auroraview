//! Advanced event loop implementation with better state management
//! This is an alternative implementation that can be used without Python bindings

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::Window;

/// Application event types
#[derive(Debug, Clone)]
pub enum AppEvent {
    WindowEvent(String), // Simplified for now
    CustomEvent(String),
    Exit,
}

/// Event queue for managing events
pub struct EventQueue {
    events: Arc<Mutex<VecDeque<AppEvent>>>,
}

impl EventQueue {
    /// Create a new event queue
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Push an event to the queue
    pub fn push(&self, event: AppEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push_back(event);
        }
    }

    /// Pop an event from the queue
    pub fn pop(&self) -> Option<AppEvent> {
        self.events
            .lock()
            .ok()
            .and_then(|mut events| events.pop_front())
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.events
            .lock()
            .map(|events| events.is_empty())
            .unwrap_or(true)
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.events
            .lock()
            .map(|events| events.len())
            .unwrap_or(0)
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced event loop state
pub struct AdvancedEventLoopState {
    /// Whether the event loop should continue running
    pub should_exit: Arc<Mutex<bool>>,
    /// Window reference
    pub window: Option<Window>,
    /// Event queue
    pub event_queue: EventQueue,
    /// Last event timestamp (for deduplication)
    pub last_event_time: Arc<Mutex<std::time::Instant>>,
}

impl AdvancedEventLoopState {
    /// Create a new state
    pub fn new(window: Window) -> Self {
        Self {
            should_exit: Arc::new(Mutex::new(false)),
            window: Some(window),
            event_queue: EventQueue::new(),
            last_event_time: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }

    /// Request exit
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

    /// Queue an event
    pub fn queue_event(&self, event: AppEvent) {
        self.event_queue.push(event);
    }

    /// Process queued events
    pub fn process_events(&self) {
        while let Some(event) = self.event_queue.pop() {
            match event {
                AppEvent::Exit => {
                    self.request_exit();
                }
                AppEvent::WindowEvent(msg) => {
                    tracing::debug!("Processing window event: {}", msg);
                }
                AppEvent::CustomEvent(msg) => {
                    tracing::debug!("Processing custom event: {}", msg);
                }
            }
        }
    }
}

/// Advanced event loop handler
pub struct AdvancedEventLoopHandler;

impl AdvancedEventLoopHandler {
    /// Run the event loop with advanced state management
    pub fn run_blocking(
        event_loop: EventLoop<()>,
        state: Arc<Mutex<AdvancedEventLoopState>>,
    ) {
        tracing::info!("Starting advanced event loop (blocking mode)");

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
                    if let Ok(state_guard) = state_clone.lock() {
                        match event {
                            WindowEvent::CloseRequested => {
                                tracing::info!("Close requested");
                                state_guard.queue_event(AppEvent::Exit);
                            }
                            WindowEvent::Resized(size) => {
                                tracing::debug!("Window resized: {:?}", size);
                                state_guard.queue_event(AppEvent::WindowEvent(
                                    format!("Resized: {:?}", size),
                                ));
                            }
                            WindowEvent::Focused(focused) => {
                                tracing::debug!("Window focus: {}", focused);
                            }
                            _ => {}
                        }
                    }
                }
                Event::MainEventsCleared => {
                    // Process queued events
                    if let Ok(state_guard) = state_clone.lock() {
                        state_guard.process_events();

                        // Check if we should exit
                        if state_guard.should_exit() {
                            tracing::info!("Exit requested, exiting event loop");
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                _ => {}
            }
        });

        tracing::info!("Advanced event loop exited");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_queue() {
        let queue = EventQueue::new();
        assert!(queue.is_empty());

        queue.push(AppEvent::CustomEvent("test".to_string()));
        assert_eq!(queue.len(), 1);

        let event = queue.pop();
        assert!(event.is_some());
        assert!(queue.is_empty());
    }

    #[test]
    fn test_event_queue_fifo() {
        let queue = EventQueue::new();
        queue.push(AppEvent::CustomEvent("first".to_string()));
        queue.push(AppEvent::CustomEvent("second".to_string()));

        if let Some(AppEvent::CustomEvent(msg)) = queue.pop() {
            assert_eq!(msg, "first");
        } else {
            panic!("Expected first event");
        }

        if let Some(AppEvent::CustomEvent(msg)) = queue.pop() {
            assert_eq!(msg, "second");
        } else {
            panic!("Expected second event");
        }
    }
}

