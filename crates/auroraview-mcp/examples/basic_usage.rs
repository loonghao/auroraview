//!/bin/env rustc
// #[allow(dead_code)]

//! Basic usage example for AuroraView MCP Server.
//!
//! Run with:
//! ```bash
//! cd /path/to/auroraview
//! cargo run --example basic_usage -p auroraview-mcp
//! ```

use auroraview_mcp::{
    registry::WebViewRegistry,
    runner::McpRunner,
    types::{McpServerConfig, WebViewConfig, WebViewId},
};

fn main() {
    // 1. Create a WebViewRegistry and register some WebViews
    let reg = WebViewRegistry::new();
    let id1 = reg.register(&WebViewConfig {
        title: Some("Example 1".to_string()),
        url: Some("https://example.com".to_string()),
        ..Default::default()
    });
    let id2 = reg.register(&WebViewConfig {
        title: Some("Example 2".to_string()),
        url: Some("https://baidu.com".to_string()),
        ..Default::default()
    });

    println!("Registered WebViews: {}", reg.len());
    println!("ID1: {}", id1.0);
    println!("ID2: {}", id2.0);

    // 2. List all WebViews
    let views = reg.list();
    for v in &views {
        println!(
            "WebView: id={}, title={}, url={}",
            v.id.0, v.title, v.url
        );
    }

    // 3. Update a WebView URL
    reg.update_url(&id1, "https://google.com");
    let info = reg.get(&id1).unwrap();
    println!("Updated URL: {}", info.url);

    // 4. Create MCP Server config
    let config = McpServerConfig::default()
        .with_port(7890)
        .with_mdns(true)
        .with_oauth(false)
        .with_max_webviews(10);

    println!("MCP Server config: port={}, mdns={}", config.port, config.enable_mdns);

    // 5. Create MCP Runner (non-blocking server start)
    // Note: Actual server start requires tokio runtime
    // This example only shows configuration
    println!("MCP Server would start at http://127.0.0.1:7890/mcp");
    println!("AG-UI SSE endpoint: http://127.0.0.1:7890/agui/events");

    // 6. Demonstrate WebView removal
    let removed = reg.remove(&id1);
    println!("Removed ID1: {}", removed.is_some());
    println!("Remaining WebViews: {}", reg.len());
}
