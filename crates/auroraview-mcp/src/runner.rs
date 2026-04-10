use crate::{
    agui::{AguiBus, AguiEvent},
    error::{McpError, Result},
    mdns::MdnsBroadcaster,
    server::AuroraViewMcpServer,
    types::McpServerConfig,
};
use axum::Router;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::local::LocalSessionManager,
};
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

/// Manages the lifecycle of the AuroraView MCP Server.
///
/// Starts an axum HTTP server that serves the MCP Streamable HTTP transport
/// at `/mcp` and an AG-UI SSE event stream at `/agui/events`.
pub struct McpRunner {
    config: McpServerConfig,
    server: AuroraViewMcpServer,
    broadcaster: Option<MdnsBroadcaster>,
    agui_bus: AguiBus,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl McpRunner {
    pub fn new(config: McpServerConfig) -> Self {
        let agui_bus = AguiBus::new();
        let server = AuroraViewMcpServer::new(config.clone())
            .with_agui_bus(agui_bus.clone());
        let broadcaster = if config.enable_mdns {
            MdnsBroadcaster::new()
                .map_err(|e| warn!("mDNS init failed: {e}"))
                .ok()
        } else {
            None
        };
        Self {
            config,
            server,
            broadcaster,
            agui_bus,
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a runner on the given port with a WebView capacity limit.
    ///
    /// Convenience constructor: equivalent to
    /// `McpRunner::new(McpServerConfig::default().with_port(port).with_max_webviews(max))`.
    ///
    /// mDNS is disabled (useful for tests and isolated DCC sessions).
    pub fn with_capacity(port: u16, max: usize) -> Self {
        let config = McpServerConfig::default()
            .with_port(port)
            .with_mdns(false)
            .with_max_webviews(max);
        Self::new(config)
    }

    pub fn server(&self) -> &AuroraViewMcpServer {
        &self.server
    }

    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Return a reference to the AG-UI event bus.
    /// Use `bus.emit(event)` to publish events to all SSE subscribers.
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
    pub async fn start(&self) -> Result<()> {
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
        let router = Router::new()
            .nest_service("/mcp", mcp_service)
            .merge(agui_router(agui_bus));

        info!("AuroraView MCP Server starting on http://{addr}/mcp");

        let tcp = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(McpError::Io)?;

        info!("AuroraView MCP Server listening on http://{addr}");

        tokio::spawn({
            let cancel = cancel.clone();
            async move {
                let serve = axum::serve(tcp, router)
                    .with_graceful_shutdown(async move {
                        // shutdown when either the oneshot fires or token cancelled
                        let _ = rx.await;
                        cancel.cancel();
                    });
                if let Err(e) = serve.await {
                    warn!("MCP server error: {e}");
                }
                info!("AuroraView MCP Server exited");
            }
        });

        Ok(())
    }

    /// Stop the running server and unregister mDNS.
    pub async fn stop(&self) {
        let mut lock = self.shutdown_tx.lock().await;
        if let Some(tx) = lock.take() {
            let _ = tx.send(());
        }
        if let Some(broadcaster) = &self.broadcaster {
            broadcaster.stop().await;
        }
        info!("AuroraView MCP Server stopped");
    }

    /// Check if the server is currently running.
    pub async fn is_running(&self) -> bool {
        self.shutdown_tx.lock().await.is_some()
    }

    /// Emit an AG-UI event to all active SSE subscribers.
    pub fn emit_agui(&self, event: AguiEvent) {
        self.agui_bus.emit(event);
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn build_mcp_service(
    server: AuroraViewMcpServer,
    cancel: CancellationToken,
) -> StreamableHttpService<AuroraViewMcpServer, LocalSessionManager> {
    let config = StreamableHttpServerConfig::default()
        .with_cancellation_token(cancel);
    StreamableHttpService::new(
        move || Ok(server.clone()),
        Default::default(),
        config,
    )
}

/// Build the AG-UI SSE router.
///
/// `GET /agui/events?run_id=<optional>` — streams `AguiEvent` as SSE lines.
/// The optional `run_id` query param filters events to a specific run.
fn agui_router(bus: AguiBus) -> Router {
    use axum::{
        extract::Query,
        response::{
            Sse,
            sse::Event,
        },
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
        axum::routing::get(
            move |Query(q): Query<AguiQuery>| {
                let bus = bus.clone();
                async move {
                    let rx = bus.subscribe();
                    let run_filter: Option<String> = q.run_id;
                    let stream = BroadcastStream::new(rx)
                        .filter_map(move |msg| {
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
            },
        ),
    )
}
