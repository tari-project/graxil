// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/lib.rs
// Version: 1.0.2
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file serves as the main library entry point for the SHA3x miner,
// located at the root of the source tree. It exports all public modules
// and types that other crates or binaries can use.
//
// Tree Location:
// - src/lib.rs (root library file)
// - Exports modules: core, miner, pool, utils, benchmark, help, tui (optional)

pub mod benchmark;
pub mod core;
pub mod help;
pub mod miner;
pub mod pool;
pub mod utils;

// Re-export commonly used types at the crate root for convenience
pub use crate::benchmark::runner::BenchmarkRunner;
pub use crate::core::{difficulty, sha3x};
pub use crate::help::{display_full_help, display_quick_help, display_version_info};
pub use crate::miner::{CpuMiner, MinerStats};
pub use crate::pool::PoolClient;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[cfg(feature = "tui")]
pub mod tui;

// Changelog:
// - v1.0.2 (2025-06-15): Added help module support.
//   - Added help module export for comprehensive command-line assistance.
//   - Re-exported help display functions for easy access from main.rs.
//   - Maintained all existing functionality for mining and benchmarking.
// - v1.0.1 (2025-06-14): Added benchmark module support.
//   - Added benchmark module export for performance testing infrastructure.
//   - Re-exported BenchmarkRunner for easy access from main.rs.
//   - Maintained all existing functionality for mining operations.
// - v1.0.0 (2025-06-14): Initial modular breakout from monolithic main.rs.
//   - Purpose: Establishes the library root, organizing the project into core,
//     miner, pool, utils, and optional tui modules.
//   - Features: Exports key types (e.g., CpuMiner, PoolJob) for easy access,
//     defines a common Result type, and supports optional TUI via feature flag.
//   - Note: This file acts as the public interface, simplifying integration
//     with main.rs and enabling extensibility for future features.
