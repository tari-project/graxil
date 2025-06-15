// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/pool/client.rs
// Version: 1.0.0
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
use tokio::net::TcpStream;

/// Pool client for managing TCP connections to the mining pool
#[derive(Clone)]
pub struct PoolClient;

impl PoolClient {
    /// Create a new PoolClient instance
    pub fn new() -> Self {
        Self
    }

    /// Connect to the mining pool at the specified address
    pub async fn connect(&self, pool_address: SocketAddr) -> Result<TcpStream> {
        let stream = TcpStream::connect(pool_address).await?;
        stream.set_nodelay(true)?; // Disable Nagle's algorithm for low latency
        Ok(stream)
    }
}

// Changelog:
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Manages TCP connections to the mining pool, providing a client
//     interface for establishing reliable communication.
//   - Features: Implements a simple PoolClient struct with a connect method that
//     creates a TCP stream to the pool's address, with Nagle's algorithm disabled
//     for reduced latency in pool interactions.
//   - Note: This file is critical for initiating communication with the mining
//     pool, used by the miner module to send login requests, receive jobs, and
//     submit shares.