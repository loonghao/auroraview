//! Dynamic Port Allocation
//!
//! Automatically finds available ports to avoid conflicts.

use super::{Result, ServiceDiscoveryError};
use std::net::{SocketAddr, TcpListener};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Port allocator for finding free ports
pub struct PortAllocator {
    /// Starting port for search
    start_port: u16,

    /// Maximum number of ports to try
    max_attempts: u16,
}

impl PortAllocator {
    /// Create a new port allocator
    ///
    /// # Arguments
    /// * `start_port` - Starting port (default: 9001)
    /// * `max_attempts` - Maximum ports to try (default: 100)
    pub fn new(start_port: u16, max_attempts: u16) -> Self {
        Self {
            start_port,
            max_attempts,
        }
    }

    /// Find a free port in the configured range
    ///
    /// # Returns
    /// * `Ok(port)` - Available port number
    /// * `Err` - No free port found
    pub fn find_free_port(&self) -> Result<u16> {
        info!(
            "Searching for free port starting from {} (max attempts: {})",
            self.start_port, self.max_attempts
        );

        for offset in 0..self.max_attempts {
            let port = self.start_port.saturating_add(offset);

            if port == 0 {
                warn!("Port {} out of valid range, skipping", port);
                continue;
            }

            debug!("Checking port {}", port);

            if Self::is_port_available(port) {
                info!("✅ Found free port: {}", port);
                return Ok(port);
            }
        }

        Err(ServiceDiscoveryError::NoFreePort {
            start: self.start_port,
            end: self.start_port.saturating_add(self.max_attempts),
        })
    }

    /// Check if a specific port is available
    ///
    /// # Arguments
    /// * `port` - Port number to check
    ///
    /// # Returns
    /// * `true` - Port is available
    /// * `false` - Port is in use
    pub fn is_port_available(port: u16) -> bool {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        match TcpListener::bind(addr) {
            Ok(_) => {
                debug!("Port {} is available", port);
                true
            }
            Err(e) => {
                debug!("Port {} is in use: {}", port, e);
                false
            }
        }
    }

    /// Find a free port with custom timeout
    ///
    /// This is useful for quick checks without blocking
    #[allow(dead_code)]
    pub fn find_free_port_with_timeout(&self, _timeout: Duration) -> Result<u16> {
        // For now, just use the standard method
        // In the future, we could implement async port checking with timeout
        self.find_free_port()
    }

    /// Get the start port
    pub fn start_port(&self) -> u16 {
        self.start_port
    }

    /// Get the max attempts
    pub fn max_attempts(&self) -> u16 {
        self.max_attempts
    }
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new(9001, 100)
    }
}
