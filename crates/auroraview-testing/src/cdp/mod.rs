//! CDP (Chrome DevTools Protocol) client implementation

mod client;
mod websocket;

pub use client::{CdpClient, TargetInfo};
pub use websocket::WebSocketCdpClient;
