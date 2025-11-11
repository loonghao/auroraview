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

    fn pump_windows_messages() {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, HWND_WIN(std::ptr::null_mut()), 0, 0, PM_REMOVE).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn recv_with_pump<T>(
        rx: mpsc::Receiver<std::result::Result<T, webview2::Error>>,
        what: &str,
    ) -> Result<T> {
        let start = Instant::now();
        loop {
            if let Ok(res) = rx.try_recv() {
                return res.map_err(|e| anyhow!(e.to_string()));
            }
            pump_windows_messages();
            std::thread::sleep(Duration::from_millis(1));
            if start.elapsed() > Duration::from_secs(10) {
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
