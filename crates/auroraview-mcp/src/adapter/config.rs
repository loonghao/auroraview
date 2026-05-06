//! CDP adapter configuration.

/// Configuration for wiring the adapter to a running `AuroraView` CDP port.
#[derive(Debug, Clone)]
pub struct CdpAdapterConfig {
    /// `http://host:port` of the `AuroraView` CDP devtools HTTP endpoint.
    pub http_endpoint: String,
    /// `ws://host:port` form, used purely as the `bridge_endpoint` value we
    /// advertise via [`dcc_mcp_protocols::adapters::DccCapabilities`].
    pub ws_endpoint: String,
    /// Optional window title for observability.
    pub window_title: Option<String>,
    /// Platform string (e.g. `"windows"`, `"linux"`, `"macos"`).
    pub platform: String,
    /// PID of the `AuroraView` process, if known; 0 otherwise.
    pub pid: u32,
    /// `AuroraView` version string (typically `CARGO_PKG_VERSION`).
    pub version: String,
}

impl CdpAdapterConfig {
    /// Build a config pointing at the standard `AuroraView` CDP layout:
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
