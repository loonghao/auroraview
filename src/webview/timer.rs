//! High-precision timer for WebView event processing.
//!
//! This module provides a cross-platform timer implementation that can be used
//! to periodically process WebView events. It supports multiple backends:
//!
//! - Windows: `SetTimer` API with message-based callbacks
//! - Cross-platform: Thread-based timer (fallback)
//!
//! The timer is designed to work in the main thread and integrates with the
//! host application's event loop.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
#[cfg(any(test, target_os = "windows", feature = "test-helpers"))]
use std::time::Duration;
#[cfg(any(test, target_os = "windows", feature = "test-helpers"))]
use std::time::Instant;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, KillTimer, PeekMessageW, SetTimer, MSG, PM_REMOVE, WM_TIMER,
};

/// Timer backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerBackend {
    /// Windows SetTimer API
    #[cfg(target_os = "windows")]
    WindowsSetTimer,
    /// Thread-based timer (fallback)
    ThreadBased,
}

/// High-precision timer for event processing
pub struct Timer {
    /// Timer interval in milliseconds
    interval_ms: u32,
    /// Whether the timer is running
    running: Arc<AtomicBool>,
    /// Timer backend
    backend: TimerBackend,
    /// Timer ID (for Windows SetTimer)
    #[cfg(target_os = "windows")]
    timer_id: Option<usize>,
    /// Window handle (for Windows SetTimer)
    #[cfg(target_os = "windows")]
    hwnd: Option<HWND>,
    /// Tick count
    tick_count: Arc<AtomicU64>,
    /// Last tick time (for throttling)
    #[cfg(any(test, target_os = "windows", feature = "test-helpers"))]
    last_tick: Arc<std::sync::Mutex<Option<Instant>>>,
}

impl Timer {
    /// Create a new timer with the specified interval
    ///
    /// # Arguments
    ///
    /// * `interval_ms` - Timer interval in milliseconds
    ///
    /// # Returns
    ///
    /// A new `Timer` instance
    pub fn new(interval_ms: u32) -> Self {
        Self {
            interval_ms,
            running: Arc::new(AtomicBool::new(false)),
            backend: TimerBackend::ThreadBased,
            #[cfg(target_os = "windows")]
            timer_id: None,
            #[cfg(target_os = "windows")]
            hwnd: None,
            tick_count: Arc::new(AtomicU64::new(0)),
            #[cfg(any(test, target_os = "windows", feature = "test-helpers"))]
            last_tick: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Start the timer with Windows SetTimer backend
    ///
    /// # Arguments
    ///
    /// * `hwnd` - Window handle to attach the timer to
    ///
    /// # Returns
    ///
    /// `Ok(())` if the timer started successfully, `Err` otherwise
    #[cfg(target_os = "windows")]
    pub fn start_windows(&mut self, hwnd: isize) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            return Err("Timer is already running".to_string());
        }

        let hwnd = HWND(hwnd as _);

        // Generate a unique timer ID based on the object address
        let timer_id = self as *const _ as usize;

        // Create the timer
        unsafe {
            let result = SetTimer(Some(hwnd), timer_id, self.interval_ms, None);
            if result == 0 {
                return Err("SetTimer failed".to_string());
            }
        }

        self.hwnd = Some(hwnd);
        self.timer_id = Some(timer_id);
        self.backend = TimerBackend::WindowsSetTimer;
        self.running.store(true, Ordering::Relaxed);

        Ok(())
    }

    /// Stop the timer
    pub fn stop(&mut self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        self.running.store(false, Ordering::Relaxed);

        #[cfg(target_os = "windows")]
        if self.backend == TimerBackend::WindowsSetTimer {
            if let (Some(hwnd), Some(timer_id)) = (self.hwnd, self.timer_id) {
                unsafe {
                    let _ = KillTimer(Some(hwnd), timer_id);
                }
            }
            self.hwnd = None;
            self.timer_id = None;
        }
    }

    /// Process pending timer messages (Windows only)
    ///
    /// This should be called periodically in the application's main loop
    /// when using the Windows SetTimer backend.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to call on each timer tick
    ///
    /// # Returns
    ///
    /// Number of timer messages processed
    #[cfg(target_os = "windows")]
    pub fn process_messages<F>(&mut self, mut callback: F) -> u32
    where
        F: FnMut(),
    {
        if !self.running.load(Ordering::Relaxed) {
            return 0;
        }

        if self.backend != TimerBackend::WindowsSetTimer {
            return 0;
        }

        let Some(hwnd) = self.hwnd else {
            return 0;
        };

        let Some(timer_id) = self.timer_id else {
            return 0;
        };

        let mut count = 0;
        let mut msg = MSG::default();

        // Process all pending WM_TIMER messages
        unsafe {
            while PeekMessageW(&mut msg, Some(hwnd), WM_TIMER, WM_TIMER, PM_REMOVE).as_bool() {
                if msg.wParam.0 == timer_id {
                    // This is our timer message
                    if self.should_tick() {
                        self.tick_count.fetch_add(1, Ordering::Relaxed);
                        callback();
                        count += 1;
                    }
                } else {
                    // Dispatch other timer messages
                    DispatchMessageW(&msg);
                }
            }
        }

        count
    }

    /// Check if enough time has passed since the last tick
    ///
    /// This is used for throttling to ensure we don't process ticks
    /// faster than the configured interval.
    #[cfg(any(test, target_os = "windows", feature = "test-helpers"))]
    pub fn should_tick(&self) -> bool {
        // Use ok() to avoid panic if mutex is poisoned during shutdown
        let mut last_tick = match self.last_tick.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let now = Instant::now();

        if let Some(last) = *last_tick {
            let elapsed = now.duration_since(last);
            if elapsed < Duration::from_millis(self.interval_ms as u64) {
                return false;
            }
        }

        *last_tick = Some(now);
        true
    }

    /// Get the current tick count
    pub fn tick_count(&self) -> u64 {
        self.tick_count.load(Ordering::Relaxed)
    }

    /// Check if the timer is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get the timer backend type
    pub fn backend(&self) -> TimerBackend {
        self.backend
    }

    /// Get the timer interval in milliseconds
    pub fn interval_ms(&self) -> u32 {
        self.interval_ms
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    /// Fixture: Create a timer with default interval (16ms)
    #[fixture]
    fn timer() -> Timer {
        Timer::new(16)
    }

    /// Unit test: Verify timer creation with correct initial state
    #[rstest]
    fn test_timer_creation(timer: Timer) {
        assert_eq!(timer.interval_ms(), 16);
        assert!(!timer.is_running());
        assert_eq!(timer.tick_count(), 0);
        assert_eq!(timer.backend(), TimerBackend::ThreadBased);
    }

    /// Unit test: Verify timer creation with different intervals
    #[rstest]
    #[case(1)]
    #[case(16)]
    #[case(33)]
    #[case(100)]
    #[case(1000)]
    fn test_timer_creation_with_different_intervals(#[case] interval: u32) {
        let timer = Timer::new(interval);
        assert_eq!(timer.interval_ms(), interval);
        assert!(!timer.is_running());
    }

    /// Unit test: Verify timer initial state
    #[rstest]
    fn test_timer_initial_state(timer: Timer) {
        assert!(!timer.is_running());
        assert_eq!(timer.tick_count(), 0);

        #[cfg(target_os = "windows")]
        {
            assert!(timer.timer_id.is_none());
            assert!(timer.hwnd.is_none());
        }
    }

    /// Unit test: Verify stop when not running doesn't panic
    #[rstest]
    fn test_timer_stop_when_not_running() {
        let mut timer = Timer::new(16);
        // Should not panic
        timer.stop();
        assert!(!timer.is_running());
    }

    /// Unit test: Verify default backend
    #[rstest]
    fn test_timer_backend_default(timer: Timer) {
        assert_eq!(timer.backend(), TimerBackend::ThreadBased);
    }

    /// Unit test: Verify Windows timer with invalid HWND
    #[cfg(target_os = "windows")]
    #[rstest]
    fn test_windows_timer_invalid_hwnd() {
        let mut timer = Timer::new(16);
        // HWND = 0 is allowed by WinAPI (thread-queue timers). Use -1 to force failure.
        let result = timer.start_windows(-1);
        assert!(result.is_err());
    }

    /// Unit test: Verify drop doesn't panic
    #[rstest]
    fn test_timer_drop() {
        let timer = Timer::new(16);
        // Ensure drop doesn't panic
        drop(timer);
    }
}

// Note: Integration tests have been moved to tests/timer_integration_tests.rs
// This includes tests for:
// - Timer throttling with actual time delays
// - Precise timer throttling
// - Multiple ticks over time
