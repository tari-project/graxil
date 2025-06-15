// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/core/types.rs
// Version: 1.0.2
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file defines core data structures for the SHA3x miner, located in the
// core subdirectory. It includes types for command-line arguments, pool jobs,
// mining jobs, shares, and related protocol structures.
//
// Tree Location:
// - src/core/types.rs (core data structures)
// - Depends on: clap, serde

use clap::Parser;
use serde::{Deserialize, Serialize};

/// Command-line arguments for the SHA3x miner
#[derive(Parser, Debug)]
#[command(
    name = "sha3x-miner",
    author = "SHA3x Mining Team",
    version = "1.0.0",
    about = "High-performance SHA3x (Tari) CPU miner with benchmarking capabilities",
    long_about = "SHA3x Miner is a high-performance CPU miner for the Tari blockchain using the SHA3x algorithm.\n\
                  It supports both pool mining and standalone benchmarking for performance testing.\n\n\
                  MINING: Requires wallet address and pool connection\n\
                  BENCHMARK: Tests hardware performance without pool connection\n\n\
                  Examples:\n\
                    Mining:     sha3x-miner -u YOUR_WALLET -o pool.tari.com:4200 --threads 6\n\
                    Benchmark:  sha3x-miner --benchmark --threads 72 --benchmark-duration 60 --benchmark-difficulty 100000\n\n\
                  For detailed help, use: sha3x-miner --help"
)]
pub struct Args {
    /// Tari wallet address for receiving mining rewards (starts with '12' or '14')
    /// Example: 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW
    #[arg(
        short = 'u', 
        long = "wallet",
        value_name = "ADDRESS",
        help = "Tari wallet address for mining rewards"
    )]
    pub wallet: Option<String>,

    /// Mining pool address in format hostname:port or ip:port
    /// Examples: pool.tari.com:4200, tari-pool.com:4200, 192.168.1.100:4200
    #[arg(
        short = 'o', 
        long = "pool",
        value_name = "HOST:PORT",
        help = "Mining pool address (format: host:port)"
    )]
    pub pool: Option<String>,

    /// Pool password (often worker name or 'x' for no password)
    /// Examples: x, worker1, your-worker-name
    #[arg(
        short = 'p', 
        long = "password",
        value_name = "PASSWORD",
        default_value = "x",
        help = "Pool password (usually 'x' or worker identifier)"
    )]
    pub password: String,

    /// Worker name for pool identification (helps identify different mining rigs)
    /// Use descriptive names like: rig-01, office-pc, server-main, xeon-dual
    /// Avoid spaces and special characters
    #[arg(
        long, 
        default_value = "worker1",
        value_name = "NAME",
        help = "Worker identifier for pool (e.g., rig-01, office-pc)"
    )]
    pub worker: String,

    /// Number of CPU mining threads to use
    /// 0 = auto-detect (recommended), or specify exact count
    /// Typical values: 32 (high-end desktop), 64 (workstation), 72 (dual Xeon)
    #[arg(
        short, 
        long, 
        default_value = "0",
        value_name = "COUNT",
        help = "Number of CPU threads (0 = auto-detect)"
    )]
    pub threads: usize,

    /// Enable GPU mining (placeholder for future implementation)
    /// Currently not implemented - CPU mining only
    #[arg(
        short, 
        long, 
        default_value = "false",
        help = "Enable GPU mining (future feature)"
    )]
    pub gpu: bool,

    /// Enable TUI dashboard (requires 'tui' feature)
    /// Provides real-time visual dashboard with charts and statistics
    #[cfg(feature = "tui")]
    #[arg(
        long, 
        default_value = "false",
        help = "Enable TUI dashboard interface"
    )]
    pub tui: bool,

    /// Run in benchmark mode (no pool connection required)
    /// Tests hardware performance and finds optimal settings
    /// Useful for: hardware testing, optimization, comparison
    #[arg(
        long, 
        default_value = "false",
        help = "Run performance benchmark (no pool required)"
    )]
    pub benchmark: bool,

    /// Benchmark duration in seconds
    /// Recommended: 30s (quick test), 60s (standard), 300s (stability test)
    /// Longer tests provide more accurate results
    #[arg(
        long, 
        default_value = "30",
        value_name = "SECONDS",
        help = "Benchmark duration in seconds [30s=quick, 60s=standard, 300s=extended]"
    )]
    pub benchmark_duration: u64,

    /// Benchmark target difficulty (affects share finding frequency)
    /// 1,000 = very easy (many shares), 100,000 = medium, 1,000,000 = realistic, 10,000,000 = hard
    /// Higher values = easier to find shares for testing
    #[arg(
        long, 
        default_value = "1000000",
        value_name = "DIFFICULTY",
        help = "Benchmark difficulty [1K=easy, 100K=medium, 1M=realistic, 10M=hard]"
    )]
    pub benchmark_difficulty: u64,
}

/// Raw job data received from the mining pool
/// This matches the Tari Stratum protocol format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolJob {
    /// Hex-encoded block header template (32 bytes when decoded)
    pub blob: String,
    
    /// Unique job identifier from the pool
    pub job_id: String,
    
    /// Hex-encoded target difficulty (8 bytes, little-endian)
    pub target: String,
    
    /// Mining algorithm (should be "sha3x" for Tari)
    pub algo: String,
    
    /// Current blockchain height
    pub height: u64,
    
    /// Optional seed hash (used for algorithm verification)
    pub seed_hash: Option<String>,
    
    /// Optional difficulty value (some pools send this directly)
    #[serde(default)]
    pub difficulty: Option<u64>,
}

/// Internal representation of a mining job
/// This is the processed version of PoolJob ready for mining threads
#[derive(Debug, Clone)]
pub struct MiningJob {
    /// Job identifier (matches PoolJob.job_id)
    pub job_id: String,
    
    /// Decoded mining hash/header template (32 bytes)
    pub mining_hash: Vec<u8>,
    
    /// Target difficulty as u64 (converted from hex target)
    pub target_difficulty: u64,
    
    /// Blockchain height for this job
    pub height: u64,
}

/// Represents a found share ready for submission
#[derive(Debug, Clone)]
pub struct Share {
    /// Job ID this share is for
    pub job_id: String,
    
    /// Nonce value that produced valid hash (little-endian)
    pub nonce: u64,
    
    /// The resulting hash (32 bytes)
    pub hash: Vec<u8>,
    
    /// Calculated difficulty of this share
    pub difficulty: u64,
    
    /// Thread ID that found this share
    pub thread_id: usize,
    
    /// Timestamp when share was found
    pub found_at: std::time::Instant,
}

/// Pool response to a submitted share
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareResponse {
    /// Response ID (matches submission ID)
    pub id: u64,
    
    /// Result of share submission
    #[serde(default)]
    pub result: Option<ShareResult>,
    
    /// Error information if share was rejected
    #[serde(default)]
    pub error: Option<ShareError>,
}

/// Possible share submission results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShareResult {
    /// Simple boolean result (some pools)
    Bool(bool),
    
    /// Status object result (Tari pools)
    Status { status: String },
    
    /// Null result (often means accepted)
    Null,
}

/// Share rejection error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareError {
    /// Error code
    pub code: i32,
    
    /// Error message
    pub message: String,
    
    /// Additional error data
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// Represents a mining target/difficulty
#[derive(Debug, Clone, Copy)]
pub struct Target {
    /// Difficulty as u64
    pub difficulty: u64,
    
    /// Raw target bytes (32 bytes for SHA3x)
    pub bits: [u8; 32],
}

/// Benchmark results for performance testing
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Total hashes computed
    pub total_hashes: u64,
    
    /// Duration of benchmark
    pub duration: std::time::Duration,
    
    /// Average hashrate (H/s)
    pub hashrate: f64,
    
    /// Peak hashrate (H/s)
    pub peak_hashrate: f64,
    
    /// Shares found during benchmark
    pub shares_found: u64,
    
    /// Thread count used
    pub thread_count: usize,
    
    /// Memory allocations (if tracked)
    pub allocations: Option<u64>,
}

impl Args {
    /// Validate arguments and return helpful errors
    pub fn validate(&self) -> Result<(), String> {
        if !self.benchmark {
            if self.wallet.is_none() {
                return Err("Wallet address is required for mining mode. Use --wallet YOUR_ADDRESS".to_string());
            }
            if self.pool.is_none() {
                return Err("Pool address is required for mining mode. Use --pool HOST:PORT".to_string());
            }
            
            // Validate wallet address format
            if let Some(ref wallet) = self.wallet {
                if wallet.len() < 80 {
                    return Err("Tari wallet address is too short (minimum 80 characters)".to_string());
                }
                if !wallet.starts_with("12") && !wallet.starts_with("14") {
                    return Err("Tari wallet address must start with '12' (one-sided) or '14' (interactive)".to_string());
                }
                // Basic character validation - Tari addresses use Base58
                if !wallet.chars().all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)) {
                    return Err("Tari wallet address contains invalid characters (must be Base58)".to_string());
                }
            }
            
            // Validate pool address format
            if let Some(ref pool) = self.pool {
                if !pool.contains(':') {
                    return Err("Pool address must be in format HOST:PORT (e.g., pool.tari.com:4200)".to_string());
                }
                let parts: Vec<&str> = pool.split(':').collect();
                if parts.len() != 2 {
                    return Err("Pool address must contain exactly one colon (HOST:PORT)".to_string());
                }
                if parts[1].parse::<u16>().is_err() {
                    return Err("Pool port must be a valid number (1-65535)".to_string());
                }
            }
        }
        
        if self.benchmark_duration == 0 {
            return Err("Benchmark duration must be greater than 0 seconds".to_string());
        }
        
        if self.benchmark_duration > 3600 {
            return Err("Benchmark duration cannot exceed 1 hour (3600 seconds)".to_string());
        }
        
        if self.threads > 1024 {
            return Err("Thread count cannot exceed 1024".to_string());
        }
        
        Ok(())
    }
}

impl Share {
    /// Create a new share
    pub fn new(
        job_id: String,
        nonce: u64,
        hash: Vec<u8>,
        difficulty: u64,
        thread_id: usize,
    ) -> Self {
        Self {
            job_id,
            nonce,
            hash,
            difficulty,
            thread_id,
            found_at: std::time::Instant::now(),
        }
    }
    
    /// Get the age of this share
    pub fn age(&self) -> std::time::Duration {
        self.found_at.elapsed()
    }
}

impl BenchmarkResult {
    /// Calculate hashrate from totals
    pub fn calculate_hashrate(total_hashes: u64, duration: std::time::Duration) -> f64 {
        total_hashes as f64 / duration.as_secs_f64()
    }
    
    /// Format hashrate for display
    pub fn format_hashrate(&self) -> String {
        if self.hashrate >= 1_000_000.0 {
            format!("{:.2} MH/s", self.hashrate / 1_000_000.0)
        } else if self.hashrate >= 1_000.0 {
            format!("{:.2} KH/s", self.hashrate / 1_000.0)
        } else {
            format!("{:.2} H/s", self.hashrate)
        }
    }
}

// Changelog:
// - v1.0.2 (2025-06-15): Enhanced help descriptions and validation.
//   - Added comprehensive help text for all command-line arguments.
//   - Enhanced Args::validate() with detailed wallet and pool validation.
//   - Added value names and examples for better user experience.
//   - Added long_about with usage examples and mode descriptions.
// - v1.0.1 (2025-06-14): Added benchmark support.
//   - Added benchmark, benchmark_duration, and benchmark_difficulty to Args.
//   - Made wallet and pool optional for benchmark mode.
//   - Added BenchmarkResult struct for performance metrics.
//   - Added Args::validate() method for argument validation.
//   - Added BenchmarkResult helper methods for calculations and formatting.
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Defines core data structures for the SHA3x miner, including
//     command-line arguments, pool jobs, mining jobs, shares, and protocol
//     responses, used across the project.
//   - Features: Provides types like Args for CLI parsing, PoolJob for pool
//     communication, MiningJob for thread processing, and Share-related types
//     for submission tracking. Supports serialization and TUI feature flags.
//   - Note: This file is central to the miner's data model, ensuring consistent
//     type usage for mining operations and pool interactions.