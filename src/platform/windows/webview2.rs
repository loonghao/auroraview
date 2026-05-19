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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};

    use anyhow::{anyhow, Context, Result};
    use webview2::{Controller, Environment, WebView};
    use winapi::shared::windef::{HWND, RECT};
    use windows::Win32::Foundation::{HANDLE, HWND as HWND_WIN};

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

    /// Default total timeout used by `recv_with_pump` when the
    /// `AURORAVIEW_WEBVIEW2_TIMEOUT_SECS` environment variable is unset or
    /// holds a non-positive / non-numeric value.
    const DEFAULT_WEBVIEW2_TIMEOUT_SECS: u64 = 30;

    /// Environment variable name used to override the WebView2 cold-start
    /// timeout in [`recv_with_pump`]. Value is interpreted as **seconds, as a
    /// plain unsigned integer with no unit suffix** (e.g. `"60"`, not `"60s"`
    /// or `"60 seconds"`). Anything that fails to parse, is zero, or is
    /// negative falls back to [`DEFAULT_WEBVIEW2_TIMEOUT_SECS`].
    const WEBVIEW2_TIMEOUT_ENV: &str = "AURORAVIEW_WEBVIEW2_TIMEOUT_SECS";

    /// Resolve the WebView2 cold-start timeout for [`recv_with_pump`].
    ///
    /// Reads `AURORAVIEW_WEBVIEW2_TIMEOUT_SECS` once per process and caches
    /// the result. If the variable is missing, empty, not a valid `u64`,
    /// zero, or carries a unit suffix, falls back to the 30s default and
    /// emits a single `tracing::warn!` (subsequent calls are silent because
    /// the parsed value is memoized).
    ///
    /// Caching is a deliberate trade-off: every WebView2 cold start invokes
    /// this twice (Environment + Controller) and the env value cannot
    /// meaningfully change at runtime. Re-parsing each call would also
    /// double-emit the warning for invalid values, which is just noise.
    fn webview2_total_timeout() -> Duration {
        // `OnceLock` is `Send + Sync` and resolves on first access; later
        // callers see the cached value with no syscall and no logging.
        static CACHED: OnceLock<Duration> = OnceLock::new();
        *CACHED.get_or_init(parse_webview2_total_timeout)
    }

    /// One-shot parse of `AURORAVIEW_WEBVIEW2_TIMEOUT_SECS`. Pulled out of
    /// [`webview2_total_timeout`] so the unit tests can exercise the parser
    /// directly without going through the process-wide cache (which would
    /// otherwise pin the very first observed value for the lifetime of the
    /// test binary).
    fn parse_webview2_total_timeout() -> Duration {
        match std::env::var(WEBVIEW2_TIMEOUT_ENV) {
            Ok(raw) => match raw.trim().parse::<u64>() {
                Ok(secs) if secs > 0 => Duration::from_secs(secs),
                _ => {
                    tracing::warn!(
                        "{} = {:?} is not a positive integer; falling back to {}s",
                        WEBVIEW2_TIMEOUT_ENV,
                        raw,
                        DEFAULT_WEBVIEW2_TIMEOUT_SECS
                    );
                    Duration::from_secs(DEFAULT_WEBVIEW2_TIMEOUT_SECS)
                }
            },
            Err(_) => Duration::from_secs(DEFAULT_WEBVIEW2_TIMEOUT_SECS),
        }
    }

    fn recv_with_pump<T>(
        rx: mpsc::Receiver<std::result::Result<T, webview2::Error>>,
        signal: &SignalEvent,
        what: &str,
    ) -> Result<T> {
        // Replace the previous busy-loop (PeekMessageW + Sleep(1ms)) implementation,
        // which would aggressively dispatch ALL host (e.g. Maya/Qt) window messages
        // on the calling thread and could cause the host UI to appear frozen for
        // several seconds during WebView2 cold start.
        //
        // Instead, let the OS wake us up only when:
        //   * a COM call is dispatched into our STA (WebView2's async callback), or
        //   * a window message arrives for this thread, or
        //   * `signal` becomes signaled — set by the caller's `tx.send` side
        //     immediately after the channel push (see callers below). This is
        //     the on-demand wake-up that lets us drop the previous 50ms polling
        //     tick: we only re-check the channel when something actually
        //     happened.
        //
        // The `&SignalEvent` borrow makes the "handle outlives this call"
        // contract a compile-time guarantee — the caller cannot drop (and
        // therefore `CloseHandle`) the event while we are still parked on
        // `CoWaitForMultipleHandles`.
        //
        // We keep an outer loop with try_recv() so we never miss the channel send
        // (the `tx.send → SetEvent` pair is not atomic; the thread can be
        // pre-empted between the two), and rely on the kernel to schedule us
        // efficiently while the WebView2 browser process is initializing.
        use windows::Win32::Foundation::{CO_E_NOTINITIALIZED, RPC_E_WRONG_THREAD};
        use windows::Win32::System::Com::{
            CoWaitForMultipleHandles, COWAIT_DISPATCH_CALLS, COWAIT_DISPATCH_WINDOW_MESSAGES,
        };

        let start = Instant::now();
        // Generous timeout: WebView2 cold start (Edge runtime spin-up, user-data
        // folder bring-up, ICU data load, etc.) can legitimately take several
        // seconds on first run, especially on slower machines or under AV scans.
        //
        // Default is 30s, which covers the vast majority of real-world setups.
        // For unusually slow hosts (CI containers, AV-heavy enterprise machines,
        // first-run Edge install paths) the default can be overridden via the
        // `AURORAVIEW_WEBVIEW2_TIMEOUT_SECS` environment variable (positive
        // integer, in seconds). A non-parseable / zero / negative value is
        // ignored and the default is used.
        let total_timeout = webview2_total_timeout();
        // Per-iteration backstop for the `CoWaitForMultipleHandles` wait.
        //
        // **This is NOT the response-latency knob.** With the auto-reset
        // `signal` event, the typical wake-up latency is whatever the
        // kernel's IPI / scheduler costs (sub-millisecond). The waiter
        // only sleeps for `BACKSTOP_TICK_MS` when *every* wake source
        // (signal event, COM dispatch, window message) is silent — at
        // which point we just want to re-check `start.elapsed()` against
        // `total_timeout` and loop back.
        //
        // **Do not "tune" this down to 50ms** to make `recv_with_pump`
        // feel snappier — that is exactly the busy-loop behaviour the
        // signal-event design replaced. A lower value only wastes CPU
        // (and Maya/Houdini paint cycles) without changing the success
        // path: the channel send already wakes us via `SetEvent`. The
        // value is intentionally large (1s) so the cost of "did the
        // host go completely silent?" is bounded but cheap.
        const BACKSTOP_TICK_MS: u32 = 1_000;

        // The two flags together let the OS:
        //   - run any pending COM calls targeted at this STA
        //     (this is how WebView2's async build callback gets delivered)
        //   - dispatch any window messages waiting on this thread
        //     (so the host UI stays responsive)
        // This is safer than calling PeekMessageW/DispatchMessageW ourselves,
        // because we don't reorder or steal the host's own message stream.
        //
        // `CoWaitForMultipleHandles`'s `dwflags` parameter is a `u32`, while
        // `COWAIT_*` constants in `windows = 0.62` are `COWAIT_FLAGS(i32)`
        // newtypes. A direct `as u32` bit-pattern cast is the simplest stable
        // mapping and matches the API signature exactly.
        let flags: u32 =
            (COWAIT_DISPATCH_CALLS.0 as u32) | (COWAIT_DISPATCH_WINDOW_MESSAGES.0 as u32);

        let handles = [signal.handle()];

        loop {
            if let Ok(res) = rx.try_recv() {
                return res.map_err(|e| anyhow!(e.to_string()));
            }

            // Bail out *before* re-entering the wait. Putting the timeout
            // check here (as opposed to after `match wait_result`) means:
            //   * we never spend an extra BACKSTOP_TICK_MS sleeping past the
            //     deadline just to log one more `trace!` line, and
            //   * the timeout path is symmetric with the `try_recv` check
            //     above — both are evaluated on every iteration before any
            //     blocking call.
            if start.elapsed() > total_timeout {
                return Err(anyhow!(format!("timeout while waiting for {}", what)));
            }

            // SAFETY: CoWaitForMultipleHandles is a documented Win32 API.
            // The `&SignalEvent` borrow guarantees the underlying kernel
            // handle is valid for the entire duration of this wait —
            // `CloseHandle` cannot run until the borrow ends, which
            // happens no earlier than this function's return. The
            // auto-reset semantics mean the event is consumed by exactly
            // one waiter, but we always re-check `try_recv` on every
            // wake so a missed wake-up only delays us by at most
            // `BACKSTOP_TICK_MS`.
            //
            // Signature in `windows = 0.62`:
            //   fn CoWaitForMultipleHandles(...) -> Result<u32, windows::core::Error>
            // The `Ok(u32)` payload is the WAIT_* index that signaled — useless
            // to us in this configuration, but we still check `Err` to detect
            // fatal apartment / initialization mistakes.
            let wait_result =
                unsafe { CoWaitForMultipleHandles(flags, BACKSTOP_TICK_MS, &handles) };

            // Map the result into one of three categories:
            //   * "fatal caller error" — STA not initialized or wrong thread.
            //     Return immediately so the caller sees the real cause instead
            //     of a 30s timeout.
            //   * other Err  — RPC_S_CALLPENDING / RPC_S_TIMEOUT / etc. Expected
            //     during normal operation; trace and keep looping.
            //   * Ok(_)      — a wait was satisfied; just loop and re-check
            //     the channel.
            match wait_result {
                Err(ref e) if e.code() == CO_E_NOTINITIALIZED => {
                    return Err(anyhow!(
                        "{}: COM not initialized on this thread (CO_E_NOTINITIALIZED). \
                         CoInitializeEx(COINIT_APARTMENTTHREADED) must be called first.",
                        what
                    ));
                }
                Err(ref e) if e.code() == RPC_E_WRONG_THREAD => {
                    return Err(anyhow!(
                        "{}: called from the wrong COM apartment (RPC_E_WRONG_THREAD). \
                         This function must run on the STA that owns the WebView2 controller.",
                        what
                    ));
                }
                Err(e) => {
                    tracing::trace!(
                        "[recv_with_pump:{}] CoWaitForMultipleHandles hr=0x{:08X}, elapsed={:?}",
                        what,
                        e.code().0 as u32,
                        start.elapsed()
                    );
                }
                Ok(idx) => {
                    tracing::trace!(
                        "[recv_with_pump:{}] CoWaitForMultipleHandles signaled idx={}, elapsed={:?}",
                        what,
                        idx,
                        start.elapsed()
                    );
                }
            }

            // Loop back: the next iteration's `try_recv` + `start.elapsed()`
            // checks at the top of the loop drive both the success and the
            // timeout exits.
        }
    }

    /// Shared inner state for [`SignalEvent`] / [`SignalToken`].
    ///
    /// The kernel handle is owned here and `CloseHandle` runs in
    /// `Drop` — which only fires when **both** the waiter side
    /// (`SignalEvent`) and every closure-side ([`SignalToken`])
    /// have been dropped. That's the entire point of using `Arc`:
    /// the WebView2 worker can no longer race with `CloseHandle`,
    /// regardless of whether `recv_with_pump` returned via the
    /// success or the timeout path.
    ///
    /// `Send + Sync` is sound because:
    ///   * `HANDLE` is just an opaque kernel index; `SetEvent` /
    ///     `CloseHandle` are documented thread-safe.
    ///   * `signaled` is an `AtomicBool`.
    ///   * The `Drop` impl only ever runs once, on the last
    ///     `Arc::drop` — single-threaded by `Arc` semantics.
    struct SignalInner {
        handle: HANDLE,
        /// Flipped by [`SignalToken::signal`] right after `SetEvent`.
        /// Read by [`SignalEvent::Drop`] to surface the "callback
        /// never fired" diagnostic.
        signaled: AtomicBool,
    }

    impl Drop for SignalInner {
        fn drop(&mut self) {
            use windows::Win32::Foundation::CloseHandle;
            // SAFETY: handle was created by CreateEventW and ownership is
            // exclusive to this Arc — by the time the last reference drops,
            // no SignalToken can exist that might still call SetEvent. This
            // is the property `Arc<SignalInner>` buys us over the previous
            // raw-HANDLE-copy design.
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }

    // SAFETY: `HANDLE` is `!Send + !Sync` in the `windows` crate purely
    // because it's a raw pointer newtype, but the underlying kernel
    // object is process-wide and the operations we perform on it
    // (`SetEvent`, `CloseHandle`) are explicitly documented thread-safe
    // by MSDN. The only mutable state besides the handle is
    // `AtomicBool`, which is `Sync` by definition. Wrapping in `Arc`
    // requires both bounds.
    unsafe impl Send for SignalInner {}
    unsafe impl Sync for SignalInner {}

    /// RAII guard wrapping a `CreateEventW` handle.
    ///
    /// Used by `recv_with_pump` callers to signal channel completion.
    ///
    /// # Ownership model
    ///
    /// Both the waiter (`SignalEvent`, on the STA stack) and every
    /// closure-side wake-up token (`SignalToken`, captured into the
    /// WebView2 builder callback) hold an `Arc<SignalInner>`. The
    /// kernel `HANDLE` is closed exactly once — when the last `Arc`
    /// reference drops. This is what eliminates the closed-handle
    /// race the previous raw-`HANDLE`-copy design accepted: a
    /// pre-empted closure can no longer invoke `SetEvent` on a
    /// recycled kernel handle, because the handle simply cannot be
    /// closed while the closure (and therefore its `SignalToken`)
    /// is still alive.
    ///
    /// # Drop-time diagnostics
    ///
    /// `Drop` emits a `tracing::warn!` if the channel was never
    /// signaled before *the waiter side* released its `Arc`. That
    /// still flags real problems (timeout while the WebView2
    /// callback never fired) without depending on the closure
    /// having outlived the waiter.
    struct SignalEvent {
        inner: std::sync::Arc<SignalInner>,
    }

    /// Wake-up token captured by the WebView2 callback closure.
    ///
    /// Holds an `Arc<SignalInner>` so the kernel handle stays alive
    /// for as long as any closure copy exists. Cheap to clone.
    #[derive(Clone)]
    struct SignalToken {
        inner: std::sync::Arc<SignalInner>,
    }

    impl SignalToken {
        /// Signal the event so any `recv_with_pump` waiter wakes up.
        ///
        /// Safe to call from any thread; `SetEvent` on a kernel
        /// `HANDLE` is documented thread-safe and the `Arc` keeps
        /// the handle alive for the entire call.
        fn signal(&self) {
            use windows::Win32::System::Threading::SetEvent;
            // Mark *before* `SetEvent` so the `Drop` warning path
            // never fires when the closure has clearly run, even
            // if `SetEvent` itself happens to fail.
            self.inner.signaled.store(true, Ordering::Release);
            // SAFETY: handle is kept alive by `self.inner`; cannot be
            // closed until every Arc ref (including this one) is dropped.
            if let Err(e) = unsafe { SetEvent(self.inner.handle) } {
                tracing::trace!("SetEvent on recv_with_pump signal failed: {}", e);
            }
        }
    }

    impl SignalEvent {
        fn new() -> Result<Self> {
            use windows::Win32::System::Threading::CreateEventW;
            // Auto-reset, initially non-signaled. Anonymous (no name) so we
            // don't pollute the global namespace.
            // SAFETY: CreateEventW with all defaults is documented to either
            // return a valid kernel handle or an Err; we propagate the Err.
            let h = unsafe { CreateEventW(None, false, false, None) }
                .context("CreateEventW for recv_with_pump signal handle failed")?;
            Ok(SignalEvent {
                inner: std::sync::Arc::new(SignalInner {
                    handle: h,
                    signaled: AtomicBool::new(false),
                }),
            })
        }

        fn handle(&self) -> HANDLE {
            self.inner.handle
        }

        /// Build a wake-up token for the WebView2 callback closure.
        ///
        /// The token shares the same kernel `HANDLE` and the same
        /// `signaled` flag as the owning `SignalEvent`. Because both
        /// hold an `Arc<SignalInner>`, the underlying handle is
        /// closed exactly once, after every reference is dropped —
        /// the WebView2 worker can never observe a closed handle.
        fn token(&self) -> SignalToken {
            SignalToken {
                inner: std::sync::Arc::clone(&self.inner),
            }
        }
    }

    impl Drop for SignalEvent {
        fn drop(&mut self) {
            // Surface the "callback never ran" path. We hit this when
            // `recv_with_pump` returns via the timeout branch *and* the
            // WebView2 callback never managed to push a result before
            // we tore the channel down. That's an actual diagnostic
            // signal — operators want to see it instead of having
            // to instrument the call site.
            //
            // The actual `CloseHandle` happens in `SignalInner::drop`,
            // which only fires once the WebView2 closure has also
            // released its Arc — eliminating the previous design's
            // closed-handle race entirely.
            if !self.inner.signaled.load(Ordering::Acquire) {
                tracing::warn!(
                    "SignalEvent dropped without the WebView2 callback ever firing. \
                     This usually means: \
                     (a) WebView2 cold start exceeded AURORAVIEW_WEBVIEW2_TIMEOUT_SECS, or \
                     (b) the COM apartment was torn down while a callback was queued. \
                     If this fires repeatedly, raise the timeout and check Edge runtime health."
                );
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

        let signal = SignalEvent::new()?;
        // Closures captured by the WebView2 builder run on a worker thread
        // and require `'static`, so they cannot borrow `signal`. Capture a
        // `SignalToken` (Copy of the raw `HANDLE` plus a clone of the
        // shared `signaled` flag) instead — the owning `SignalEvent`
        // outlives `recv_with_pump` (we pass it by reference), which in
        // turn outlives every closure invocation.
        let token = signal.token();
        let (tx, rx) = mpsc::channel();
        let builder = Environment::builder();
        builder
            .build(move |res| {
                // Push the result first so a wake-up always observes a ready
                // channel. Then signal — even if the waiter is pre-empted
                // between try_recv and CoWaitForMultipleHandles, the auto-reset
                // event keeps the wake-up pending until the next wait.
                let _ = tx.send(res);
                token.signal();
                Ok(())
            })
            .context("Environment::builder().build failed")?;
        recv_with_pump(rx, &signal, "WebView2 Environment")
    }

    fn create_controller_blocking(
        env: &Environment,
        parent_hwnd: isize,
    ) -> Result<(Controller, WebView)> {
        let signal = SignalEvent::new()?;
        // See `create_environment_blocking` for why we capture a token
        // instead of borrowing `signal` directly.
        let token = signal.token();
        let (tx, rx) = mpsc::channel();
        let hwnd: HWND = parent_hwnd as *mut winapi::shared::windef::HWND__;
        env.create_controller(hwnd, move |res| {
            let res =
                res.and_then(|controller| controller.get_webview().map(|wv| (controller, wv)));
            let _ = tx.send(res);
            token.signal();
            Ok(())
        })
        .context("Environment::create_controller failed")?;

        recv_with_pump(rx, &signal, "WebView2 Controller")
    }

    #[cfg(test)]
    mod timeout_env_tests {
        //! Unit tests for the timeout env var parsing.
        //!
        //! These tests intentionally exercise [`parse_webview2_total_timeout`]
        //! rather than [`webview2_total_timeout`], because the latter caches
        //! its result in a process-wide `OnceLock` — once any test resolves
        //! it, every later test would observe the same pinned value
        //! regardless of how the env var is mutated.
        //!
        //! Tests still mutate the process-wide
        //! `AURORAVIEW_WEBVIEW2_TIMEOUT_SECS` environment variable and must
        //! therefore run serially — `serial_test::serial` is required.
        use super::{
            parse_webview2_total_timeout, DEFAULT_WEBVIEW2_TIMEOUT_SECS, WEBVIEW2_TIMEOUT_ENV,
        };
        use serial_test::serial;
        use std::time::Duration;

        fn default_duration() -> Duration {
            Duration::from_secs(DEFAULT_WEBVIEW2_TIMEOUT_SECS)
        }

        #[test]
        #[serial]
        fn falls_back_to_default_when_env_unset() {
            std::env::remove_var(WEBVIEW2_TIMEOUT_ENV);
            assert_eq!(parse_webview2_total_timeout(), default_duration());
        }

        #[test]
        #[serial]
        fn parses_positive_integer() {
            std::env::set_var(WEBVIEW2_TIMEOUT_ENV, "60");
            assert_eq!(parse_webview2_total_timeout(), Duration::from_secs(60));
            std::env::remove_var(WEBVIEW2_TIMEOUT_ENV);
        }

        #[test]
        #[serial]
        fn trims_surrounding_whitespace() {
            std::env::set_var(WEBVIEW2_TIMEOUT_ENV, "  45  ");
            assert_eq!(parse_webview2_total_timeout(), Duration::from_secs(45));
            std::env::remove_var(WEBVIEW2_TIMEOUT_ENV);
        }

        #[test]
        #[serial]
        fn falls_back_on_zero_or_non_numeric() {
            // Each invalid input must fall back to the default and emit a
            // `tracing::warn!` (we only assert the value here).
            for raw in ["0", "-5", "abc", "", " ", "30s", "30 seconds"] {
                std::env::set_var(WEBVIEW2_TIMEOUT_ENV, raw);
                assert_eq!(
                    parse_webview2_total_timeout(),
                    default_duration(),
                    "input {:?} should fall back to default",
                    raw
                );
            }
            std::env::remove_var(WEBVIEW2_TIMEOUT_ENV);
        }
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
}
