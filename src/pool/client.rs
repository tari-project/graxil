// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/pool/client.rs
// Version: 1.1.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements the TCP client for communicating with the mining pool
// in the SHA3x miner, located in the pool subdirectory. It handles establishing
// and maintaining connections to the pool server.
//
// Tree Location:
// - src/pool/client.rs (pool TCP client logic)
// - Depends on: tokio, std

use crate::Result;
use std::net::SocketAddr;
use tokio::net::{TcpStream, lookup_host};

/// Pool client for managing TCP connections to the mining pool
#[derive(Clone)]
pub struct PoolClient;

impl PoolClient {
    /// Create a new PoolClient instance
    pub fn new() -> Self {
        Self
    }

    /// Resolve pool address from either IP:port or domain:port format
    async fn resolve_pool_address(pool_str: &str) -> Result<SocketAddr> {
        // Try parsing as direct IP:port first
        if let Ok(addr) = pool_str.parse::<SocketAddr>() {
            return Ok(addr);
        }
        
        // If that fails, try DNS resolution
        let mut addrs = lookup_host(pool_str).await?;
        addrs.next()
            .ok_or_else(|| "No addresses found for hostname".into())
    }

    /// Connect to the mining pool at the specified SocketAddr (backward compatibility)
    pub async fn connect(&self, pool_address: SocketAddr) -> Result<TcpStream> {
        let stream = TcpStream::connect(pool_address).await?;
        stream.set_nodelay(true)?; // Disable Nagle's algorithm for low latency
        Ok(stream)
    }

    /// Connect to the mining pool at the specified address (supports both IP and domain)
    pub async fn connect_str(&self, pool_address: &str) -> Result<TcpStream> {
        let resolved_addr = Self::resolve_pool_address(pool_address).await?;
        let stream = TcpStream::connect(resolved_addr).await?;
        stream.set_nodelay(true)?; // Disable Nagle's algorithm for low latency
        Ok(stream)
    }
}

// Changelog:
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