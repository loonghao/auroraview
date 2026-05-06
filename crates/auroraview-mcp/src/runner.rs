//! Manages the lifecycle of the `AuroraView` MCP Server.
//!
//! This module provides `McpRunner` for starting/stopping the MCP server
//! with optional mDNS broadcast and AG-UI event streaming.

use crate::{
    agui::{AguiBus, AguiEvent},
    error::{McpError, Result},
    mcp_server::McpServer,
    mdns::MdnsBroadcaster,
    oauth::OAuthStore,
    types::{McpServerConfig, WebViewId},
    CdpAdapterConfig,
};
use axum::Router;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

/// Manages the lifecycle of the `AuroraView` MCP Server.
///
/// `McpRunner` starts an axum HTTP server that serves:
/// - `POST /mcp` — MCP Streamable HTTP transport (initialize + tool calls)
/// - `GET /mcp` — MCP SSE stream (stateful session reconnect)
/// - `DELETE /mcp` — terminate MCP session
/// - `GET /agui/events?run_id=<id>` — AG-UI SSE event stream
///
/// # Example
///
/// ```rust,ignore
/// let runner = McpRunner::new(McpServerConfig::default());
/// // Start server in background (non-blocking)
/// tokio::spawn(async move {
///     runner.start().await.expect("server start failed");
/// });
/// ```
pub struct McpRunner {
    /// Server configuration.
    config: McpServerConfig,
    /// MCP server implementation (handles tool calls).
    server: McpServer,
    /// OAuth 2.0 store (None if OAuth is disabled).
    oauth_store: Option<OAuthStore>,
    /// mDNS broadcaster (None if mDNS is disabled).
    broadcaster: Option<MdnsBroadcaster>,
    /// AG-UI event bus for streaming events to UI clients.
    agui_bus: AguiBus,
    /// Shutdown signal sender (used to stop the server).
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    /// Server task handle (used to wait for graceful shutdown).
    server_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl McpRunner {
    /// Create a new `McpRunner` with the given configuration.
    ///
    /// Initializes the AG-UI bus, CDP adapter, optional mDNS broadcaster,
    /// and optional OAuth store based on the configuration.
    #[must_use]
    pub fn new(config: McpServerConfig) -> Self {
        let agui_bus = AguiBus::new();
        let cdp_config = CdpAdapterConfig::localhost(config.port, env!("CARGO_PKG_VERSION"));
        let server = McpServer::new(cdp_config).with_agui_bus(agui_bus.clone());
        let broadcaster = if config.enable_mdns {
            MdnsBroadcaster::new()
                .map_err(|e| warn!("mDNS init failed: {e}"))
                .ok()
        } else {
            None
        };
        let oauth_store = if config.enable_oauth {
            Some(OAuthStore::new())
        } else {
            None
        };
        Self {
            config,
            server,
            oauth_store,
            broadcaster,
            agui_bus,
            shutdown_tx: Arc::default(),
            server_task: Arc::default(),
        }
    }

    /// Create a runner on the given port with a `WebView` capacity limit.
    ///
    /// Convenience constructor: equivalent to
    /// `McpRunner::new(McpServerConfig::default().with_port(port).with_max_webviews(max))`.
    ///
    /// mDNS is disabled (useful for tests and isolated DCC sessions).
    #[must_use]
    pub fn with_capacity(port: u16, max: usize) -> Self {
        let config = McpServerConfig::default()
            .with_port(port)
            .with_mdns(false)
            .with_max_webviews(max);
        Self::new(config)
    }

    /// Create a runner on the given port with mDNS broadcast enabled.
    ///
    /// Convenience constructor: equivalent to
    /// `McpRunner::new(McpServerConfig::default().with_port(port).with_mdns(true))`.
    ///
    /// Use this when you want `dcc-mcp-client` to auto-discover the server
    /// via mDNS without building a full [`McpServerConfig`] manually.
    #[must_use]
    pub fn with_mdns_port(port: u16) -> Self {
        let config = McpServerConfig::default().with_port(port).with_mdns(true);
        Self::new(config)
    }

    /// Return a reference to the inner `McpServer`.
    #[must_use]
    pub fn server(&self) -> &McpServer {
        &self.server
    }

    /// Return a reference to the server configuration.
    #[must_use]
    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Return a reference to the AG-UI event bus.
    /// Use `bus.emit(event)` to publish events to all SSE subscribers.
    #[must_use]
    pub fn agui_bus(&self) -> &AguiBus {
        &self.agui_bus
    }

    /// Start the MCP server in a background tokio task.
    ///
    /// Binds to `{host}:{port}` and serves:
    /// - `POST /mcp`  — MCP Streamable HTTP transport (initialize + tool calls)
    /// - `GET  /mcp`  — MCP SSE stream (stateful session reconnect)
    /// - `DELETE /mcp` — terminate MCP session
    /// - `GET  /agui/events?run_id=<id>` — AG-UI SSE event stream
    ///
    /// Returns immediately; the server runs until `stop()` is called.
    ///
    /// # Errors
    ///
    /// Returns [`McpError::InvalidConfig`] if the configuration is invalid
    /// (e.g. `port == 0`, empty `host`, or empty `service_name`).
    ///
    /// Returns [`McpError::AlreadyRunning`] if the server is already running.
    ///
    /// Returns [`McpError::Io`] if the TCP listener fails to bind.
    pub async fn start(&self) -> Result<()> {
        let start_time = std::time::Instant::now();

        self.config.validate().map_err(McpError::InvalidConfig)?;

        let mut lock = self.shutdown_tx.lock().await;
        if lock.is_some() {
            return Err(McpError::AlreadyRunning(self.config.port));
        }

        if let Some(broadcaster) = &self.broadcaster {
            broadcaster.start(&self.config).await?;
        }

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let cancel = CancellationToken::new();
        let (tx, rx) = oneshot::channel::<()>();
        *lock = Some(tx);
        drop(lock);

        let mcp_service = build_mcp_service(self.server.clone(), cancel.clone());
        let agui_bus = self.agui_bus.clone();

        let mut router = Router::new()
            .route("/health", axum::routing::get(health_handler))
            .nest_service("/mcp", mcp_service)
            .merge(agui_router(agui_bus));

        // Add OAuth routes if enabled
        if let Some(oauth_store) = &self.oauth_store {
            let oauth_store = oauth_store.clone();
            router = router.merge(oauth_router(oauth_store));
            info!(oauth_enabled = true, "OAuth 2.0 endpoints enabled");
        }

        info!(%addr, "AuroraView MCP Server starting");

        let tcp = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(McpError::Io)?;

        let elapsed = start_time.elapsed();
        info!(%addr, ?elapsed, "AuroraView MCP Server listening");

        let server_task = tokio::spawn({
            let cancel = cancel.clone();
            async move {
                let serve = axum::serve(tcp, router).with_graceful_shutdown(async move {
                    // shutdown when either the oneshot fires or token cancelled
                    let _ = rx.await;
                    cancel.cancel();
                });
                if let Err(e) = serve.await {
                    warn!(error = %e, "MCP server error");
                }
                info!("AuroraView MCP Server exited");
            }
        });

        // Store task handle for graceful shutdown
        *self.server_task.lock().await = Some(server_task);

        Ok(())
    }

    /// Stop the running server and unregister mDNS.
    ///
    /// Sends shutdown signal and waits for graceful shutdown with timeout.
    /// Cleans up resources (mDNS, registry, CDP connections) after shutdown.
    pub async fn stop(&self) {
        info!(port = self.config.port, "Stopping AuroraView MCP Server");

        // Send shutdown signal
        let task_handle = {
            let mut tx_lock = self.shutdown_tx.lock().await;
            if let Some(tx) = tx_lock.take() {
                if tx.send(()).is_err() {
                    warn!("Shutdown signal already consumed");
                }
            }

            // Take the server task handle
            let mut task_lock = self.server_task.lock().await;
            task_lock.take()
        };

        // Wait for server to stop (with timeout)
        if let Some(task) = task_handle {
            info!("Waiting for server to shut down gracefully");
            match tokio::time::timeout(std::time::Duration::from_secs(30), task).await {
                Ok(Ok(())) => {
                    info!(port = self.config.port, "Server stopped gracefully");
                }
                Ok(Err(e)) => {
                    warn!(error = %e, "Server task failed during shutdown");
                }
                Err(_) => {
                    warn!(timeout_secs = 30, "Server shutdown timed out, forcing exit");
                    // Task will be dropped, which cancels it
                }
            }
        } else {
            debug!("No active server task to stop");
        }

        // Stop mDNS broadcaster
        if let Some(broadcaster) = &self.broadcaster {
            broadcaster.stop().await;
            info!("mDNS broadcaster stopped");
        }

        // Cleanup: Clear registry and CDP client
        self.server.registry().clear();
        info!("WebView registry cleared");

        // Reset CDP client (will reconnect on next use)
        // Note: OnceCell doesn't have a clear() method, so we rely on
        // with_cdp_endpoint() to reset it when needed

        info!(
            port = self.config.port,
            "AuroraView MCP Server fully stopped"
        );
    }

    /// Check if the server is currently running.
    pub async fn is_running(&self) -> bool {
        self.shutdown_tx.lock().await.is_some()
    }

    /// Emit an AG-UI event to all active SSE subscribers.
    ///
    /// The event must be wrapped in an `Arc` for zero-copy broadcasting.
    pub fn emit_agui(&self, event: Arc<AguiEvent>) {
        self.agui_bus.emit(event);
    }

    /// Emit a `StepStarted` followed immediately by a `StepFinished` event.
    ///
    /// Useful for synchronous actions where the step has no meaningful
    /// intermediate state (e.g. a simple tool invocation).
    ///
    /// Both events share the same `run_id` and `step_id`.
    pub fn emit_agui_step(&self, run_id: &str, step_name: &str, step_id: &str) {
        self.agui_bus.emit(Arc::new(AguiEvent::StepStarted {
            run_id: run_id.to_string(),
            step_name: step_name.to_string(),
            step_id: step_id.to_string(),
        }));
        self.agui_bus.emit(Arc::new(AguiEvent::StepFinished {
            run_id: run_id.to_string(),
            step_id: step_id.to_string(),
        }));
    }

    /// Update the CDP endpoint for a registered `WebView`.
    ///
    /// Returns `Ok(())` if the `WebView` was found and updated.
    /// Returns `Err(...)` if no `WebView` with the given ID exists.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a string error message if no `WebView` with the
    /// given `id` exists in the registry.
    ///
    /// # Panics
    ///
    /// Panics if the `id` string cannot be parsed into a [`WebViewId`].
    /// In current implementation, `WebViewId` parsing is infallible,
    /// so this should not panic in practice.
    pub fn update_cdp_endpoint(&self, id: &str, endpoint: &str) -> std::result::Result<(), String> {
        let wid = id.parse::<WebViewId>().unwrap(); // Infallible
        if self
            .server
            .registry()
            .update_cdp_endpoint(&wid, endpoint.to_string())
        {
            Ok(())
        } else {
            Err(format!("WebView {id} not found"))
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn build_mcp_service(
    server: McpServer,
    _cancel: CancellationToken,
) -> StreamableHttpService<McpServer, LocalSessionManager> {
    let config = StreamableHttpServerConfig::default();
    StreamableHttpService::new(move || Ok(server.clone()), Arc::default(), config)
}

/// Build the AG-UI SSE router.
///
/// `GET /agui/events?run_id=<optional>` — streams `AguiEvent` as SSE lines.
/// The optional `run_id` query param filters events to a specific run.
fn agui_router(bus: AguiBus) -> Router {
    use axum::{
        extract::Query,
        response::{sse::Event, Sse},
    };
    use futures::StreamExt;
    use serde::Deserialize;
    use tokio_stream::wrappers::BroadcastStream;

    #[derive(Deserialize, Default)]
    struct AguiQuery {
        run_id: Option<String>,
    }

    Router::new().route(
        "/agui/events",
        axum::routing::get(move |Query(q): Query<AguiQuery>| {
            let bus = bus.clone();
            async move {
                let rx = bus.subscribe();
                let run_filter: Option<String> = q.run_id;
                let stream = BroadcastStream::new(rx).filter_map(move |msg| {
                    let run_filter = run_filter.clone();
                    async move {
                        let event = msg.ok()?;
                        // Apply optional run_id filter
                        if let Some(rid) = run_filter.as_deref() {
                            if event.run_id() != rid {
                                return None;
                            }
                        }
                        let data = serde_json::to_string(&event).ok()?;
                        Some(Ok::<Event, std::convert::Infallible>(
                            Event::default().data(data),
                        ))
                    }
                });
                Sse::new(stream).keep_alive(
                    axum::response::sse::KeepAlive::new()
                        .interval(std::time::Duration::from_secs(15))
                        .text("keep-alive"),
                )
            }
        }),
    )
}

/// Build the OAuth 2.0 router.
///
/// Endpoints:
/// - `GET  /.well-known/oauth-authorization-server` — server metadata
/// - `POST /oauth/register` — dynamic client registration
/// - `GET  /oauth/authorize` — authorization endpoint (simplified)
/// - `POST /oauth/token` — token endpoint
#[allow(clippy::too_many_lines)]
fn oauth_router(oauth_store: OAuthStore) -> Router {
    use axum::{
        extract::{Json, Query, State},
        http::StatusCode,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize)]
    struct OAuthRegisterRequest {
        client_name: String,
        redirect_uris: Vec<String>,
        scope: String,
    }

    #[derive(Debug, Serialize)]
    struct OAuthRegisterResponse {
        client_id: String,
        client_secret: String,
        client_name: String,
        redirect_uris: Vec<String>,
        scope: String,
    }

    #[derive(Debug, Deserialize)]
    struct OAuthAuthorizeQuery {
        client_id: String,
        redirect_uri: String,
        response_type: String,
        scope: String,
        code_challenge: String,
        code_challenge_method: String,
    }

    #[derive(Debug, Deserialize)]
    struct OAuthTokenRequest {
        grant_type: String,
        client_id: String,
        code: Option<String>,
        redirect_uri: Option<String>,
        code_verifier: Option<String>,
    }

    Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            axum::routing::get(|State(_store): State<OAuthStore>| async move {
                axum::Json(serde_json::json!({
                    "issuer": "auroraview-mcp",
                    "authorization_endpoint": "http://localhost:7890/oauth/authorize",
                    "token_endpoint": "http://localhost:7890/oauth/token",
                    "registration_endpoint": "http://localhost:7890/oauth/register",
                    "code_challenge_methods_supported": ["S256"],
                    "scopes_supported": ["mcp:tools", "mcp:resources"],
                    "response_types_supported": ["code"],
                    "grant_types_supported": ["authorization_code"],
                    "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post"]
                }))
            }),
        )
        .route(
            "/oauth/register",
            axum::routing::post(|State(store): State<OAuthStore>, Json(req): Json<OAuthRegisterRequest>| async move {
                let (client, secret) = store
                    .register_client(req.client_name, req.redirect_uris.clone(), req.scope.clone());

                Ok::<_, StatusCode>(axum::Json(OAuthRegisterResponse {
                    client_id: client.client_id,
                    client_secret: secret,
                    client_name: client.name,
                    redirect_uris: client.redirect_uris,
                    scope: client.scope,
                }))
            }),
        )
        .route(
            "/oauth/authorize",
            axum::routing::get(|State(store): State<OAuthStore>, Query(q): Query<OAuthAuthorizeQuery>| async move {
                if q.response_type != "code" {
                    return Err(StatusCode::BAD_REQUEST);
                }
                if q.code_challenge_method != "S256" {
                    return Err(StatusCode::BAD_REQUEST);
                }

                let code = store
                    .issue_code(q.client_id, q.redirect_uri.clone(), q.code_challenge, q.scope);

                let redirect_url = format!("{}?code={}&state=", q.redirect_uri, code);
                Ok::<_, StatusCode>(axum::response::Redirect::to(&redirect_url))
            }),
        )
        .route(
            "/oauth/token",
            axum::routing::post(|State(store): State<OAuthStore>, Json(req): Json<OAuthTokenRequest>| async move {
                if req.grant_type != "authorization_code" {
                    return Err(StatusCode::BAD_REQUEST);
                }

                let code = req.code.ok_or(StatusCode::BAD_REQUEST)?;
                let client_id = req.client_id;
                let redirect_uri = req.redirect_uri.ok_or(StatusCode::BAD_REQUEST)?;
                let code_verifier = req.code_verifier.ok_or(StatusCode::BAD_REQUEST)?;

                let token_resp = store
                    .exchange_code(&code, &client_id, &redirect_uri, &code_verifier)
                    .ok_or(StatusCode::BAD_REQUEST)?;

                Ok::<_, StatusCode>(axum::Json(serde_json::json!({
                    "access_token": token_resp.access_token,
                    "token_type": token_resp.token_type,
                    "expires_in": token_resp.expires_in,
                    "scope": token_resp.scope
                })))
            }),
        )
        .with_state(oauth_store)
}

// ---------------------------------------------------------------------------
// Health check handler
// ---------------------------------------------------------------------------

/// Health check handler for `GET /health`.
///
/// Returns a JSON response with server status:
/// ```json
/// {
///   "status": "ok",
///   "service": "auroraview-mcp",
///   "version": "<version>"
/// }
/// ```
async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "auroraview-mcp",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{McpServerConfig, WebViewConfig};

    #[test]
    fn new_creates_runner_with_defaults() {
        let config = McpServerConfig::default();
        let runner = McpRunner::new(config);
        assert_eq!(runner.config().port, 7890);
        assert!(runner.server().registry().is_empty());
    }

    #[test]
    fn with_capacity_sets_port_and_limit() {
        let runner = McpRunner::with_capacity(9000, 5);
        assert_eq!(runner.config().port, 9000);
        assert_eq!(runner.config().max_webviews, Some(5));
        assert!(!runner.config().enable_mdns);
    }

    #[test]
    fn with_mdns_port_sets_port_and_mdns() {
        let runner = McpRunner::with_mdns_port(9001);
        assert_eq!(runner.config().port, 9001);
        assert!(runner.config().enable_mdns);
    }

    #[test]
    fn agui_bus_returns_valid_bus() {
        let runner = McpRunner::new(McpServerConfig::default());
        let bus = runner.agui_bus();
        assert_eq!(bus.receiver_count(), 0);
    }

    #[test]
    fn emit_agui_does_not_panic() {
        let runner = McpRunner::new(McpServerConfig::default());
        let event = crate::agui::AguiEvent::RunStarted {
            run_id: "test".to_string(),
            thread_id: "t1".to_string(),
        };
        runner.emit_agui(Arc::new(event));
    }

    #[test]
    fn emit_agui_step_does_not_panic() {
        let runner = McpRunner::new(McpServerConfig::default());
        runner.emit_agui_step("run-1", "export", "step-1");
    }

    #[test]
    fn update_cdp_endpoint_updates_registered_view() {
        let runner = McpRunner::new(McpServerConfig::default());
        let registry = runner.server().registry();
        let id = registry.register(&WebViewConfig::default());

        let result = runner.update_cdp_endpoint(&id.0, "http://127.0.0.1:9222");
        assert!(result.is_ok());

        let info = registry.get(&id).unwrap();
        assert_eq!(info.cdp_endpoint, Some("http://127.0.0.1:9222".to_string()));
    }

    #[test]
    fn update_cdp_endpoint_returns_err_for_unknown_id() {
        let runner = McpRunner::new(McpServerConfig::default());
        let result = runner.update_cdp_endpoint("nonexistent", "http://127.0.0.1:9222");
        assert!(result.is_err());
    }

    #[test]
    fn config_returns_valid_config() {
        let config = McpServerConfig::default().with_port(9000);
        let runner = McpRunner::new(config.clone());
        let returned_config = runner.config();
        assert_eq!(returned_config.port, 9000);
    }

    #[test]
    fn server_returns_valid_server() {
        let runner = McpRunner::new(McpServerConfig::default());
        let server = runner.server();
        // Server should have an empty registry initially
        assert!(server.registry().is_empty());
    }

    #[tokio::test]
    async fn start_returns_err_for_invalid_config() {
        // Port 0 is invalid
        let config = McpServerConfig::default().with_port(0);
        let runner = McpRunner::new(config);
        let result = runner.start().await;
        assert!(result.is_err());
    }
}
