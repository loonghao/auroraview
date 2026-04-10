//! AuroraView MCP Server
//!
//! Exposes AuroraView WebView capabilities as a standard MCP (Model Context Protocol) server.
//! Designed to be embedded inside DCC applications (Maya, Houdini, Blender, Unreal Engine, etc.)
//! and discoverable by `dcc-mcp-client` via mDNS.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use auroraview_mcp::{McpRunner, McpServerConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = McpServerConfig::default(); // port 7890
//!     let runner = McpRunner::new(config);
//!     runner.start().await.expect("MCP server failed to start");
//!
//!     // ... run your DCC application event loop ...
//!
//!     runner.stop().await;
//! }
//! ```
//!
//! # Exposed MCP tools
//!
//! | Tool | Description |
//! |------|-------------|
//! | `screenshot` | Capture a WebView screenshot (PNG, base64) |
//! | `load_url` | Navigate a WebView to a URL |
//! | `load_html` | Load raw HTML into a WebView |
//! | `eval_js` | Execute JavaScript and return the result |
//! | `send_event` | Push a named event to the WebView JS context |
//! | `get_hwnd` | Get the native window handle (for UE embedding) |
//! | `list_webviews` | List all active WebView instances |
//! | `create_webview` | Create a new WebView |
//! | `close_webview` | Close a WebView |

pub mod error;
pub mod mdns;
pub mod registry;
pub mod runner;
pub mod server;
pub mod types;

pub use error::{McpError, Result};
pub use registry::WebViewRegistry;
pub use runner::McpRunner;
pub use server::AuroraViewMcpServer;
pub use types::{McpServerConfig, WebViewConfig, WebViewId, WebViewInfo};
