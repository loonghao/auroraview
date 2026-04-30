//! AuroraView adapter for `dcc-mcp-core`.
//!
//! This crate exposes a running AuroraView instance as a
//! [`DccAdapter`](dcc_mcp_protocols::adapters::DccAdapter), so that
//! `dcc-mcp-server` (from `dcc-mcp-core v0.13+`) can discover it via
//! `FileRegistry` and call into it over the Chrome DevTools Protocol.
//!
//! # Status
//!
//! **Skeleton / Phase 2 of Epic #364.** The crate currently implements:
//!
//! - [`CdpAuroraViewAdapter::info`] — static [`DccInfo`] for `dcc_type = "auroraview"`.
//! - [`CdpAuroraViewAdapter::capabilities`] — advertises
//!   `snapshot = true`, `has_embedded_python = false`,
//!   [`BridgeKind::WebSocket`], and a WebSocket `bridge_endpoint`.
//! - [`DccConnection`] — `connect` / `disconnect` / `health_check` against
//!   the CDP browser-level WebSocket.
//! - [`DccSnapshot`] — `capture_viewport` via `Page.captureScreenshot`.
//!
//! Deliberately **not yet wired**:
//!
//! - [`DccScriptEngine`] — blocked on upstream
//!   [`dcc-mcp-core#222`](https://github.com/loonghao/dcc-mcp-core/issues/222)
//!   adding `ScriptLanguage::JavaScript`. Once that lands, `Runtime.evaluate`
//!   becomes a one-screen implementation.
//! - [`DccSceneInfo`] / scene-manager / hierarchy / transform — AuroraView is
//!   a web view, not a 3D DCC, so these stay `None` unless a skill explicitly
//!   opts in.
//!
//! # Wiring
//!
//! The `auroraview-cli run` path (Phase 3 of #364) is responsible for opening
//! a CDP debug port and constructing a [`CdpAuroraViewAdapter`] pointing at
//! it, then handing that adapter to `dcc-mcp-server`'s registry.

pub mod cdp;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use dcc_mcp_protocols::adapters::{
    BridgeKind, CaptureResult, DccAdapter, DccCapabilities, DccConnection, DccError, DccErrorCode,
    DccInfo, DccResult, DccSnapshot, ScriptLanguage,
};
use tokio::runtime::{Handle, Runtime};

use crate::cdp::{CdpClient, CdpError};

/// Default timeout for any single CDP call the adapter makes.
const DEFAULT_CDP_TIMEOUT: Duration = Duration::from_secs(10);

/// Configuration for wiring the adapter to a running AuroraView CDP port.
#[derive(Debug, Clone)]
pub struct CdpAdapterConfig {
    /// `http://host:port` of the AuroraView CDP devtools HTTP endpoint.
    pub http_endpoint: String,
    /// `ws://host:port` form, used purely as the `bridge_endpoint` value we
    /// advertise via [`DccCapabilities`]. This is what `dcc-mcp-server` will
    /// surface in its `ServiceEntry.extras` for other skills to consume.
    pub ws_endpoint: String,
    /// Optional window title for observability.
    pub window_title: Option<String>,
    /// Platform string (e.g. `"windows"`, `"linux"`, `"macos"`). Defaults to
    /// the value of `std::env::consts::OS` when [`CdpAuroraViewAdapter::new`]
    /// is called.
    pub platform: String,
    /// PID of the AuroraView process, if known; 0 otherwise.
    pub pid: u32,
    /// AuroraView version string (typically `CARGO_PKG_VERSION`).
    pub version: String,
}

impl CdpAdapterConfig {
    /// Build a config pointing at the standard AuroraView CDP layout:
    /// `http://127.0.0.1:<port>` and `ws://127.0.0.1:<port>`.
    #[must_use]
    pub fn localhost(port: u16, version: impl Into<String>) -> Self {
        Self {
            http_endpoint: format!("http://127.0.0.1:{port}"),
            ws_endpoint: format!("ws://127.0.0.1:{port}"),
            window_title: None,
            platform: std::env::consts::OS.to_owned(),
            pid: std::process::id(),
            version: version.into(),
        }
    }
}

/// Adapter that speaks to a running AuroraView instance over CDP.
pub struct CdpAuroraViewAdapter {
    config: CdpAdapterConfig,
    info: DccInfo,
    /// Tokio runtime used to block on async CDP calls from the sync trait
    /// methods. We lazily create an owned runtime iff there is no ambient
    /// runtime we can reuse.
    runtime: Arc<AdapterRuntime>,
    /// Live CDP client, populated by [`DccConnection::connect`].
    client: Mutex<Option<CdpClient>>,
}

impl std::fmt::Debug for CdpAuroraViewAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CdpAuroraViewAdapter")
            .field("config", &self.config)
            .field("info", &self.info)
            .field(
                "connected",
                &self.client.lock().map(|g| g.is_some()).unwrap_or(false),
            )
            .finish()
    }
}

enum AdapterRuntime {
    /// We are inside a running tokio runtime — reuse it.
    Borrowed(Handle),
    /// No ambient runtime; we own a multi-thread runtime.
    Owned(Runtime),
}

impl AdapterRuntime {
    fn current_or_owned() -> std::io::Result<Self> {
        if let Ok(handle) = Handle::try_current() {
            Ok(Self::Borrowed(handle))
        } else {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .worker_threads(1)
                .thread_name("auroraview-mcp")
                .build()?;
            Ok(Self::Owned(rt))
        }
    }

    fn block_on<F: std::future::Future>(&self, fut: F) -> F::Output {
        match self {
            Self::Borrowed(h) => {
                // `Handle::block_on` panics if we're currently on a runtime
                // thread, so we offload to a temporary blocking task.
                tokio::task::block_in_place(|| h.block_on(fut))
            }
            Self::Owned(rt) => rt.block_on(fut),
        }
    }
}

impl CdpAuroraViewAdapter {
    /// Create a new adapter. Does not yet establish a CDP connection —
    /// callers must invoke [`DccConnection::connect`] (or let the MCP
    /// framework do so) before calling snapshot/script-engine APIs.
    pub fn new(config: CdpAdapterConfig) -> std::io::Result<Self> {
        let info = DccInfo {
            dcc_type: "auroraview".to_owned(),
            version: config.version.clone(),
            // AuroraView itself does not embed Python; the host DCC might.
            python_version: None,
            platform: config.platform.clone(),
            pid: config.pid,
            metadata: {
                let mut m = std::collections::HashMap::new();
                if let Some(t) = &config.window_title {
                    m.insert("window_title".to_owned(), t.clone());
                }
                m.insert("cdp_http".to_owned(), config.http_endpoint.clone());
                m.insert("cdp_ws".to_owned(), config.ws_endpoint.clone());
                m
            },
        };
        Ok(Self {
            config,
            info,
            runtime: Arc::new(AdapterRuntime::current_or_owned()?),
            client: Mutex::new(None),
        })
    }

    fn map_cdp_err(kind: DccErrorCode, err: CdpError) -> DccError {
        DccError {
            code: kind,
            message: err.to_string(),
            details: None,
            recoverable: matches!(
                kind,
                DccErrorCode::ConnectionFailed
                    | DccErrorCode::Timeout
                    | DccErrorCode::NotResponding
            ),
        }
    }

    fn with_client<T>(&self, f: impl FnOnce(&mut CdpClient) -> DccResult<T>) -> DccResult<T> {
        let mut guard = self.client.lock().map_err(|_| DccError {
            code: DccErrorCode::Internal,
            message: "adapter mutex poisoned".to_owned(),
            details: None,
            recoverable: false,
        })?;
        let client = guard.as_mut().ok_or(DccError {
            code: DccErrorCode::ConnectionFailed,
            message: "CDP client is not connected".to_owned(),
            details: None,
            recoverable: true,
        })?;
        f(client)
    }
}

impl DccConnection for CdpAuroraViewAdapter {
    fn connect(&mut self) -> DccResult<()> {
        let endpoint = self.config.http_endpoint.clone();
        let fut = CdpClient::connect(&endpoint);
        let client = self
            .runtime
            .block_on(fut)
            .map_err(|e| Self::map_cdp_err(DccErrorCode::ConnectionFailed, e))?;
        *self.client.lock().map_err(|_| DccError {
            code: DccErrorCode::Internal,
            message: "adapter mutex poisoned".to_owned(),
            details: None,
            recoverable: false,
        })? = Some(client);
        Ok(())
    }

    fn disconnect(&mut self) -> DccResult<()> {
        if let Ok(mut guard) = self.client.lock() {
            guard.take();
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.client.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    fn health_check(&self) -> DccResult<u64> {
        self.with_client(|client| {
            let start = std::time::Instant::now();
            self.runtime
                .block_on(client.get_version(DEFAULT_CDP_TIMEOUT))
                .map_err(|e| Self::map_cdp_err(DccErrorCode::NotResponding, e))?;
            Ok(start.elapsed().as_millis() as u64)
        })
    }
}

impl DccSnapshot for CdpAuroraViewAdapter {
    fn capture_viewport(
        &self,
        _viewport: Option<&str>,
        _width: Option<u32>,
        _height: Option<u32>,
        format: &str,
    ) -> DccResult<CaptureResult> {
        match format {
            "png" | "jpeg" | "webp" => {}
            other => {
                return Err(DccError {
                    code: DccErrorCode::InvalidInput,
                    message: format!("unsupported capture format: {other}"),
                    details: None,
                    recoverable: false,
                });
            }
        }
        self.with_client(|client| {
            let bytes = self
                .runtime
                .block_on(client.capture_screenshot(format, DEFAULT_CDP_TIMEOUT))
                .map_err(|e| Self::map_cdp_err(DccErrorCode::Internal, e))?;
            // Resolution re-negotiation (Page.setDeviceMetricsOverride) is a
            // later-phase concern; for now we just report what CDP returned
            // and leave width/height as 0 ("unknown").
            Ok(CaptureResult {
                data: bytes,
                width: 0,
                height: 0,
                format: format.to_owned(),
                viewport: Some("auroraview".to_owned()),
            })
        })
    }
}

impl DccAdapter for CdpAuroraViewAdapter {
    fn info(&self) -> &DccInfo {
        &self.info
    }

    fn capabilities(&self) -> DccCapabilities {
        DccCapabilities {
            // AuroraView is JS-first. Until dcc-mcp-core#222 lands a
            // `ScriptLanguage::JavaScript` variant, we advertise the empty
            // set rather than lie about supporting Python/MEL.
            script_languages: Vec::<ScriptLanguage>::new(),
            scene_info: false,
            snapshot: true,
            undo_redo: false,
            progress_reporting: false,
            file_operations: false,
            selection: false,
            scene_manager: false,
            transform: false,
            render_capture: false,
            hierarchy: false,
            has_embedded_python: false,
            bridge_kind: Some(BridgeKind::WebSocket),
            bridge_endpoint: Some(self.config.ws_endpoint.clone()),
            extensions: Default::default(),
        }
    }

    fn as_connection(&mut self) -> Option<&mut dyn DccConnection> {
        Some(self)
    }

    fn as_script_engine(&self) -> Option<&dyn dcc_mcp_protocols::adapters::DccScriptEngine> {
        // See the top-of-file note on dcc-mcp-core#222.
        None
    }

    fn as_scene_info(&self) -> Option<&dyn dcc_mcp_protocols::adapters::DccSceneInfo> {
        None
    }

    fn as_snapshot(&self) -> Option<&dyn DccSnapshot> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_carries_cdp_metadata() {
        let cfg = CdpAdapterConfig::localhost(9222, "0.4.19");
        let adapter = CdpAuroraViewAdapter::new(cfg).expect("build adapter");
        let info = adapter.info();
        assert_eq!(info.dcc_type, "auroraview");
        assert_eq!(info.version, "0.4.19");
        assert_eq!(
            info.metadata.get("cdp_http").unwrap(),
            "http://127.0.0.1:9222"
        );
        assert_eq!(info.metadata.get("cdp_ws").unwrap(), "ws://127.0.0.1:9222");
    }

    #[test]
    fn capabilities_advertise_websocket_bridge() {
        let cfg = CdpAdapterConfig::localhost(9222, "0.4.19");
        let adapter = CdpAuroraViewAdapter::new(cfg).expect("build adapter");
        let caps = adapter.capabilities();
        assert!(caps.snapshot);
        assert!(!caps.has_embedded_python);
        assert!(matches!(caps.bridge_kind, Some(BridgeKind::WebSocket)));
        assert_eq!(caps.bridge_endpoint.as_deref(), Some("ws://127.0.0.1:9222"));
        assert!(
            caps.script_languages.is_empty(),
            "JavaScript variant still blocked on dcc-mcp-core#222"
        );
    }

    #[test]
    fn disconnected_snapshot_returns_connection_error() {
        let cfg = CdpAdapterConfig::localhost(9222, "0.4.19");
        let adapter = CdpAuroraViewAdapter::new(cfg).expect("build adapter");
        assert!(!adapter.is_connected());
        let err = adapter
            .capture_viewport(None, None, None, "png")
            .expect_err("must fail without a live connection");
        assert_eq!(err.code, DccErrorCode::ConnectionFailed);
    }

    #[test]
    fn snapshot_rejects_unknown_format() {
        let cfg = CdpAdapterConfig::localhost(9222, "0.4.19");
        let adapter = CdpAuroraViewAdapter::new(cfg).expect("build adapter");
        let err = adapter
            .capture_viewport(None, None, None, "bmp")
            .expect_err("bmp is not supported");
        assert_eq!(err.code, DccErrorCode::InvalidInput);
    }
}
