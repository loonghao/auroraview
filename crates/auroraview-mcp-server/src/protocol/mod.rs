//! Protocol module for JSON-RPC 2.0 implementation.
//!
//! This module provides the protocol layer for communication between
//! the MCP Sidecar and the main AuroraView process.

mod jsonrpc;

pub use jsonrpc::*;
