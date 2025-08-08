// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/pool/client.rs
// Version: 1.2.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements the TCP client for communicating with the mining pool
// in the SHA3x miner, located in the pool subdirectory. It handles establishing
// and maintaining connections to the pool server with connection tracking.
//
// Tree Location:
// - src/pool/client.rs (pool TCP client logic)
// - Depends on: tokio, std

use crate::Result;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::{TcpStream, lookup_host};

/// Connection information for tracking pool connectivity and performance
#[derive(Clone, Debug)]
pub struct ConnectionInfo {
    pub pool_address: Option<String>,
    pub resolved_address: Option<SocketAddr>,
    pub connection_latency: Option<Duration>,
    pub connected_at: Option<Instant>,
    pub is_connected: bool,
    pub connection_attempts: u32,
    pub last_successful_connect: Option<Instant>,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            pool_address: None,
            resolved_address: None,
            connection_latency: None,
            connected_at: None,
            is_connected: false,
            connection_attempts: 0,
            last_successful_connect: None,
        }
    }
}

impl ConnectionInfo {
    /// Get connection latency in milliseconds
    pub fn latency_ms(&self) -> Option<u64> {
        self.connection_latency.map(|d| d.as_millis() as u64)
    }

    /// Get uptime since connection established
    pub fn uptime(&self) -> Option<Duration> {
        self.connected_at.map(|connected| connected.elapsed())
    }

    /// Get formatted pool address for display
    pub fn display_address(&self) -> String {
        self.pool_address
            .clone()
            .unwrap_or_else(|| "Not connected".to_string())
    }
}

/// Pool client for managing TCP connections to the mining pool
#[derive(Clone)]
pub struct PoolClient {
    connection_info: Arc<Mutex<ConnectionInfo>>,
}

impl PoolClient {
    /// Create a new PoolClient instance
    pub fn new() -> Self {
        Self {
            connection_info: Arc::new(Mutex::new(ConnectionInfo::default())),
        }
    }

    /// Resolve pool address from either IP:port or domain:port format
    async fn resolve_pool_address(pool_str: &str) -> Result<SocketAddr> {
        // Try parsing as direct IP:port first
        if let Ok(addr) = pool_str.parse::<SocketAddr>() {
            return Ok(addr);
        }

        // If that fails, try DNS resolution
        let mut addrs = lookup_host(pool_str).await?;
        addrs
            .next()
            .ok_or_else(|| "No addresses found for hostname".into())
    }

    /// Connect to the mining pool at the specified SocketAddr (backward compatibility)
    pub async fn connect(&self, pool_address: SocketAddr) -> Result<TcpStream> {
        let pool_str = pool_address.to_string();
        self.connect_with_tracking(&pool_str, pool_address).await
    }

    /// Connect to the mining pool at the specified address (supports both IP and domain)
    pub async fn connect_str(&self, pool_address: &str) -> Result<TcpStream> {
        let start_time = Instant::now();

        // Update connection attempt counter
        {
            let mut info = self.connection_info.lock().unwrap();
            info.connection_attempts += 1;
            info.pool_address = Some(pool_address.to_string());
        }

        // Resolve address and measure connection time
        let resolved_addr = Self::resolve_pool_address(pool_address).await?;
        let stream = TcpStream::connect(resolved_addr).await?;
        let connection_latency = start_time.elapsed();

        // Update connection info on successful connection
        {
            let mut info = self.connection_info.lock().unwrap();
            info.resolved_address = Some(resolved_addr);
            info.connection_latency = Some(connection_latency);
            info.connected_at = Some(Instant::now());
            info.is_connected = true;
            info.last_successful_connect = Some(Instant::now());
        }

        stream.set_nodelay(true)?; // Disable Nagle's algorithm for low latency
        Ok(stream)
    }

    /// Internal method for connecting with tracking (used by backward compatible method)
    async fn connect_with_tracking(
        &self,
        pool_str: &str,
        pool_address: SocketAddr,
    ) -> Result<TcpStream> {
        let start_time = Instant::now();

        // Update connection attempt counter
        {
            let mut info = self.connection_info.lock().unwrap();
            info.connection_attempts += 1;
            info.pool_address = Some(pool_str.to_string());
        }

        let stream = TcpStream::connect(pool_address).await?;
        let connection_latency = start_time.elapsed();

        // Update connection info on successful connection
        {
            let mut info = self.connection_info.lock().unwrap();
            info.resolved_address = Some(pool_address);
            info.connection_latency = Some(connection_latency);
            info.connected_at = Some(Instant::now());
            info.is_connected = true;
            info.last_successful_connect = Some(Instant::now());
        }

        stream.set_nodelay(true)?;
        Ok(stream)
    }

    /// Mark connection as disconnected (should be called when connection is lost)
    pub fn mark_disconnected(&self) {
        let mut info = self.connection_info.lock().unwrap();
        info.is_connected = false;
        info.connected_at = None;
    }

    /// Update connection latency (for periodic ping measurements)
    pub fn update_latency(&self, latency: Duration) {
        let mut info = self.connection_info.lock().unwrap();
        info.connection_latency = Some(latency);
    }

    /// Get current connection information
    pub fn get_connection_info(&self) -> ConnectionInfo {
        self.connection_info.lock().unwrap().clone()
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        self.connection_info.lock().unwrap().is_connected
    }

    /// Get connection latency in milliseconds (convenience method)
    pub fn get_latency_ms(&self) -> Option<u64> {
        self.connection_info.lock().unwrap().latency_ms()
    }

    /// Get pool address for display (convenience method)
    pub fn get_pool_address(&self) -> String {
        self.connection_info.lock().unwrap().display_address()
    }
}

// Changelog:
// - v1.2.0 (2025-06-24): Added connection tracking and performance monitoring
//   - Added ConnectionInfo struct to track pool connectivity, latency, and statistics
//   - Enhanced PoolClient with connection_info Arc<Mutex<ConnectionInfo>> field
//   - Added connection latency measurement during connect operations
//   - Added methods for connection state management (mark_disconnected, update_latency)
//   - Added convenience methods for getting connection status and metrics
//   - Tracks connection attempts, successful connections, and uptime
//   - Provides serializable data for dashboard integration
//   - Maintains backward compatibility with existing connect methods
// - v1.1.0 (2025-06-23): Added DNS resolution support
//   - Added resolve_pool_address method for handling both IP addresses and domain names
//   - Modified connect method to accept &str instead of SocketAddr for flexibility
//   - Added connect_addr method for backward compatibility with SocketAddr
//   - Now supports connecting to pools using domain names like pool.sha3x.supportxtm.com:6118
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Manages TCP connections to the mining pool, providing a client
//     interface for establishing reliable communication.
//   - Features: Implements a simple PoolClient struct with a connect method that
//     creates a TCP stream to the pool's address, with Nagle's algorithm disabled
//     for reduced latency in pool interactions.
//   - Note: This file is critical for initiating communication with the mining
//     pool, used by the miner module to send login requests, receive jobs, and
//     submit shares.
