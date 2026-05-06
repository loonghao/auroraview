//! `dcc-mcp-core` adapter implementation backed by AuroraView CDP.

mod config;
mod runtime;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dcc_mcp_protocols::adapters::{
    BridgeKind, CaptureResult, DccAdapter, DccCapabilities, DccConnection, DccError, DccErrorCode,
    DccInfo, DccResult, DccSnapshot, ScriptLanguage,
};

pub use config::CdpAdapterConfig;
use runtime::AdapterRuntime;

use crate::cdp::{CdpClient, CdpError};

/// Default timeout for any single CDP call the adapter makes.
pub const DEFAULT_CDP_TIMEOUT: Duration = Duration::from_secs(5);

/// Adapter that speaks to a running `AuroraView` instance over CDP.
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
            .field("runtime", &self.runtime)
            .field("connected", &self.client.lock().is_ok_and(|g| g.is_some()))
            .finish()
    }
}

impl CdpAuroraViewAdapter {
    /// Create a new adapter. Does not yet establish a CDP connection — callers
    /// must invoke [`DccConnection::connect`] before calling snapshot APIs.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the tokio runtime cannot be created.
    pub fn new(config: CdpAdapterConfig) -> std::io::Result<Self> {
        let info = DccInfo {
            dcc_type: "auroraview".to_owned(),
            version: config.version.clone(),
            // AuroraView itself does not embed Python; the host DCC might.
            python_version: None,
            platform: config.platform.clone(),
            pid: config.pid,
            metadata: {
                let mut m = HashMap::new();
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

    fn map_cdp_err(kind: DccErrorCode, err: &CdpError) -> DccError {
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

    /// Return the adapter configuration.
    #[must_use]
    pub fn config(&self) -> &CdpAdapterConfig {
        &self.config
    }

    /// Return a connected CDP client, establishing the connection if needed.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] when the CDP endpoint cannot be reached.
    pub async fn get_or_connect_client(&self) -> Result<CdpClient, CdpError> {
        if let Some(client) = self.client.lock().ok().and_then(|guard| guard.clone()) {
            return Ok(client);
        }

        let client = CdpClient::connect(&self.config.http_endpoint).await?;
        let mut guard = self
            .client
            .lock()
            .map_err(|_| CdpError::MalformedResponse("adapter".to_owned(), "client mutex"))?;
        if let Some(existing) = guard.as_ref() {
            return Ok(existing.clone());
        }
        *guard = Some(client.clone());
        Ok(client)
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
            .map_err(|e| Self::map_cdp_err(DccErrorCode::ConnectionFailed, &e))?;
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
        self.client.lock().is_ok_and(|g| g.is_some())
    }

    fn health_check(&self) -> DccResult<u64> {
        self.with_client(|client| {
            let start = std::time::Instant::now();
            self.runtime
                .block_on(client.get_version(DEFAULT_CDP_TIMEOUT))
                .map_err(|e| Self::map_cdp_err(DccErrorCode::NotResponding, &e))?;
            #[allow(clippy::cast_possible_truncation)]
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
                .map_err(|e| Self::map_cdp_err(DccErrorCode::Internal, &e))?;
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
            // AuroraView is JS-first. Until dcc-mcp-core exposes a stable
            // JavaScript script engine contract, we advertise the empty set
            // rather than lie about supporting Python/MEL.
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
            extensions: HashMap::default(),
        }
    }

    fn as_connection(&mut self) -> Option<&mut dyn DccConnection> {
        Some(self)
    }

    fn as_script_engine(&self) -> Option<&dyn dcc_mcp_protocols::adapters::DccScriptEngine> {
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
        assert!(caps.script_languages.is_empty());
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
