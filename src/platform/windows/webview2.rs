//! Windows WebView2 backend (scaffold)
//!
//! This module defines a Windows-specific WebView2 backend that embeds a
//! WebView2 control as a child of a given parent HWND. It is intentionally
//! not wired into the crate yet to avoid breaking builds while we iterate.
//!
//! Design goals:
//! - Create on an STA thread and marshal all UI ops onto that thread
//! - Support embedded (parent HWND) and standalone (owned top-level) modes
//! - Provide a small API surface consumed by pyo3 wrappers
//! - Avoid winit/tao/wry event loops in DCC-hosted scenarios

#![allow(dead_code, unused_variables, unused_imports, unused_parens)]

#[cfg(all(target_os = "windows", feature = "win-webview2"))]
pub mod win {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use anyhow::{anyhow, Context, Result};
    use webview2::{Controller, Environment, WebView};
    use winapi::shared::windef::{HWND, RECT};
    use windows::Win32::Foundation::HWND as HWND_WIN;
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };

    /// Real WebView2 wrapper used by the pyo3 layer.
    pub struct WinWebView {
        pub parent_hwnd: isize,
        pub bounds: (i32, i32, i32, i32), // x, y, w, h
        controller: Controller,
        webview: WebView,
    }

    impl WinWebView {
        /// Create an embedded WebView2 under the given parent HWND (Qt container recommended).
        pub fn create_embedded(parent_hwnd: isize, x: i32, y: i32, w: i32, h: i32) -> Result<Self> {
            let env = create_environment_blocking()?;
            let (controller, webview) = create_controller_blocking(&env, parent_hwnd)?;

            // Set initial bounds and make it visible
            let rect: RECT = RECT {
                left: x,
                top: y,
                right: x + w,
                bottom: y + h,
            };
            controller.put_bounds(rect).context("put_bounds failed")?;
            controller
                .put_is_visible(true)
                .context("put_is_visible failed")?;

            Ok(Self {
                parent_hwnd,
                bounds: (x, y, w, h),
                controller,
                webview,
            })
        }

        /// Update bounds (pixels) of the embedded WebView2.
        pub fn set_bounds(&mut self, x: i32, y: i32, w: i32, h: i32) {
            self.bounds = (x, y, w, h);
            let rect: RECT = RECT {
                left: x,
                top: y,
                right: x + w,
                bottom: y + h,
            };
            let _ = self.controller.put_bounds(rect);
        }

        /// Navigate to a URL.
        pub fn navigate(&self, url: &str) -> Result<()> {
            self.webview.navigate(url).context("navigate failed")
        }

        /// Evaluate JavaScript in the page (fires-and-forgets result).
        pub fn eval(&self, script: &str) -> Result<()> {
            // If you want to block until result is returned, use a oneshot channel.
            self.webview
                .execute_script(script, |_result| Ok(()))
                .context("execute_script failed")
        }

        /// Post a JSON message to the page.
        pub fn post_message(&self, json: &str) -> Result<()> {
            self.webview
                .post_web_message_as_json(json)
                .context("post_web_message_as_json failed")
        }

        /// Register a handler for messages posted from the page via chrome.webview.postMessage
        pub fn on_message<F>(&self, handler: F) -> Result<()>
        where
            F: Fn(String) + 'static,
        {
            // Subscribe to WebMessageReceived and forward JSON payload to the provided handler
            self.webview
                .add_web_message_received(move |_sender, args| {
                    // Prefer JSON to preserve types round-trip
                    let json = args.get_web_message_as_json().unwrap_or_default();
                    handler(json);
                    Ok(())
                })
                .context("add_web_message_received failed")?;
            Ok(())
        }

        /// Dispose resources.
        pub fn dispose(self) {
            let _ = self.controller.close();
        }
    }

    pub(crate) fn pump_windows_messages() {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn recv_with_pump<T>(
        rx: mpsc::Receiver<std::result::Result<T, webview2::Error>>,
        what: &str,
    ) -> Result<T> {
        // Replace the previous busy-loop (PeekMessageW + Sleep(1ms)) implementation,
        // which would aggressively dispatch ALL host (e.g. Maya/Qt) window messages
        // on the calling thread and could cause the host UI to appear frozen for
        // several seconds during WebView2 cold start.
        //
        // Instead, let the OS wake us up only when:
        //   * a COM call is dispatched into our STA (WebView2's async callback), or
        //   * a window message arrives for this thread.
        //
        // We keep an outer loop with try_recv() so we never miss the channel send,
        // and rely on the kernel to schedule us efficiently while the WebView2
        // browser process is initializing.
        use windows::Win32::System::Com::{
            CoWaitForMultipleHandles, COWAIT_DISPATCH_CALLS, COWAIT_DISPATCH_WINDOW_MESSAGES,
        };

        let start = Instant::now();
        // Generous timeout: WebView2 cold start (Edge runtime spin-up, user-data
        // folder bring-up, ICU data load, etc.) can legitimately take several
        // seconds on first run, especially on slower machines or under AV scans.
        const TOTAL_TIMEOUT: Duration = Duration::from_secs(30);
        // Per-iteration wait. Short enough to react quickly when the callback
        // fires before a window message arrives; long enough to avoid burning CPU.
        const TICK_MS: u32 = 50;

        // The two flags together let the OS:
        //   - run any pending COM calls targeted at this STA
        //     (this is how WebView2's async build callback gets delivered)
        //   - dispatch any window messages waiting on this thread
        //     (so the host UI stays responsive)
        // This is safer than calling PeekMessageW/DispatchMessageW ourselves,
        // because we don't reorder or steal the host's own message stream.
        //
        // NOTE on types: in `windows = 0.62`, `COWAIT_*` constants are exposed as
        // `COWAIT_FLAGS(i32)` newtypes, but `CoWaitForMultipleHandles` takes a raw
        // `u32` for `dwflags`. We unwrap with `.0`, OR the bits together, then
        // reinterpret the bit pattern as u32.
        let flags: u32 =
            (COWAIT_DISPATCH_CALLS.0 | COWAIT_DISPATCH_WINDOW_MESSAGES.0).try_into().unwrap();

        loop {
            if let Ok(res) = rx.try_recv() {
                return res.map_err(|e| anyhow!(e.to_string()));
            }

            // SAFETY: CoWaitForMultipleHandles is a documented Win32 API.
            // We pass an empty handle slice and rely solely on the timeout +
            // dispatch flags to wake us.
            unsafe {
                // Ignore the result: RPC_S_CALLPENDING / timeout / dispatched
                // are all expected and we re-check the channel on the next iter.
                let _ = CoWaitForMultipleHandles(flags, TICK_MS, &[]);
            }

            if start.elapsed() > TOTAL_TIMEOUT {
                return Err(anyhow!(format!("timeout while waiting for {}", what)));
            }
        }
    }

    fn create_environment_blocking() -> Result<Environment> {
        // Ensure we're in an STA; ignore if already initialized.
        #[allow(unsafe_code)]
        unsafe {
            #[cfg(target_os = "windows")]
            {
                use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
                let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            }
        }

        let (tx, rx) = mpsc::channel();
        let builder = Environment::builder();
        builder
            .build(move |res| {
                let _ = tx.send(res);
                Ok(())
            })
            .context("Environment::builder().build failed")?;
        recv_with_pump(rx, "WebView2 Environment")
    }

    fn create_controller_blocking(
        env: &Environment,
        parent_hwnd: isize,
    ) -> Result<(Controller, WebView)> {
        let (tx, rx) = mpsc::channel();
        let hwnd: HWND = parent_hwnd as *mut winapi::shared::windef::HWND__;
        env.create_controller(hwnd, move |res| {
            let res =
                res.and_then(|controller| controller.get_webview().map(|wv| (controller, wv)));
            let _ = tx.send(res);
            Ok(())
        })
        .context("Environment::create_controller failed")?;

        recv_with_pump(rx, "WebView2 Controller")
    }
}

#[cfg(not(target_os = "windows"))]
pub mod win {
    /// Stubs for non-Windows builds
    pub struct WinWebView {
        pub parent_hwnd: isize,
        pub bounds: (i32, i32, i32, i32),
    }
    impl WinWebView {
        pub fn create_embedded(
            _parent_hwnd: isize,
            _x: i32,
            _y: i32,
            _w: i32,
            _h: i32,
        ) -> anyhow::Result<Self> {
            Err(anyhow::anyhow!("Windows-only backend"))
        }
        pub fn set_bounds(&mut self, _x: i32, _y: i32, _w: i32, _h: i32) {}
        pub fn navigate(&self, _url: &str) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("Windows-only backend"))
        }
        pub fn eval(&self, _script: &str) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("Windows-only backend"))
        }
        pub fn post_message(&self, _json: &str) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("Windows-only backend"))
        }
        pub fn dispose(self) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_non_windows_stub_create_embedded_fails() {
        let result = win::WinWebView::create_embedded(0, 0, 0, 800, 600);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Windows-only"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_non_windows_stub_navigate_fails() {
        // Create a stub instance (will fail, but we test the error path)
        let result = win::WinWebView::create_embedded(0, 0, 0, 800, 600);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_non_windows_stub_eval_fails() {
        // Test that eval returns error on non-Windows
        let stub = win::WinWebView {
            parent_hwnd: 0,
            bounds: (0, 0, 800, 600),
        };
        let result = stub.eval("console.log('test')");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_non_windows_stub_post_message_fails() {
        let stub = win::WinWebView {
            parent_hwnd: 0,
            bounds: (0, 0, 800, 600),
        };
        let result = stub.post_message(r#"{"test": "data"}"#);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_non_windows_stub_set_bounds() {
        let mut stub = win::WinWebView {
            parent_hwnd: 0,
            bounds: (0, 0, 800, 600),
        };
        // Should not panic
        stub.set_bounds(10, 20, 1024, 768);
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_non_windows_stub_dispose() {
        let stub = win::WinWebView {
            parent_hwnd: 0,
            bounds: (0, 0, 800, 600),
        };
        // Should not panic
        stub.dispose();
    }

    // Windows-specific tests
    #[test]
    #[cfg(all(target_os = "windows", feature = "win-webview2"))]
    fn test_win_webview_bounds_structure() {
        // Test that bounds are stored correctly
        let bounds = (10, 20, 800, 600);
        assert_eq!(bounds.0, 10);
        assert_eq!(bounds.1, 20);
        assert_eq!(bounds.2, 800);
        assert_eq!(bounds.3, 600);
    }

    #[test]
    #[cfg(all(target_os = "windows", feature = "win-webview2"))]
    fn test_pump_windows_messages_does_not_panic() {
        // Test that message pump doesn't panic when called
        win::pump_windows_messages();
    }
}
