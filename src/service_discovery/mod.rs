//! Service Discovery Module - Python Bindings
//!
//! This module provides Python bindings for service discovery.
//! Core implementations are in auroraview-core.

// Re-export everything from core
pub use auroraview_core::service_discovery::{
    DiscoveryResponse, HttpDiscovery, MdnsService, PortAllocator, Result, ServiceDiscoveryError,
    ServiceInfo, SERVICE_TYPE,
};
