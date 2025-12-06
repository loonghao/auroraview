//! Dynamic Port Allocation
//!
//! Automatically finds available ports to avoid conflicts.

use std::net::{SocketAddr, TcpListener};
use tracing::{debug, info, warn};

/// Error type for port allocation
#[derive(Debug, Clone)]
pub enum PortError {
    /// No free port found in the specified range
    NoFreePort { start: u16, end: u16 },
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortError::NoFreePort { start, end } => {
                write!(f, "No free port found in range {}-{}", start, end)
            }
        }
    }
}

impl std::error::Error for PortError {}

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
    pub fn find_free_port(&self) -> Result<u16, PortError> {
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
                info!("Found free port: {}", port);
                return Ok(port);
            }
        }

        Err(PortError::NoFreePort {
            start: self.start_port,
            end: self.start_port.saturating_add(self.max_attempts),
        })
    }

    /// Check if a specific port is available
    pub fn is_port_available(port: u16) -> bool {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        TcpListener::bind(addr).is_ok()
    }

    /// Find any available port (let OS assign)
    pub fn find_any_port() -> Result<u16, PortError> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        match TcpListener::bind(addr) {
            Ok(listener) => {
                let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
                if port > 0 {
                    info!("OS assigned port: {}", port);
                    Ok(port)
                } else {
                    Err(PortError::NoFreePort { start: 0, end: 0 })
                }
            }
            Err(_) => Err(PortError::NoFreePort { start: 0, end: 0 }),
        }
    }
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new(9001, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_any_port() {
        let port = PortAllocator::find_any_port().unwrap();
        assert!(port > 0);
    }

    #[test]
    fn test_is_port_available() {
        // Bind a port
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        // Port should not be available
        assert!(!PortAllocator::is_port_available(port));

        // Drop listener
        drop(listener);

        // Port should be available now
        assert!(PortAllocator::is_port_available(port));
    }

    #[test]
    fn test_port_allocator_default() {
        let allocator = PortAllocator::default();
        assert_eq!(allocator.start_port, 9001);
        assert_eq!(allocator.max_attempts, 100);
    }
}
