// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/miner/stats/mod.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file is the module declaration for the statistics tracking functionality
// of the SHA3x miner, located in the stats subdirectory of the miner module. It
// declares submodules and re-exports key types for use throughout the project.
//
// Tree Location:
// - src/miner/stats/mod.rs (stats module entry point)
// - Submodules: miner_stats, thread_stats

pub mod miner_stats;
pub mod thread_stats;
pub mod gpu_info;

// Re-export key types for convenience
pub use miner_stats::MinerStats;
pub use thread_stats::ThreadStats;
pub use gpu_info::GpuInfo;

// Changelog:
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Defines the stats module, organizing statistics tracking for the
//     overall miner and individual threads into miner_stats and thread_stats
//     submodules.
//   - Features: Declares miner_stats and thread_stats submodules, with re-exports
//     of MinerStats and ThreadStats types for easy access by miner and main.rs.
//   - Note: This file provides a structured entry point for statistics-related
//     logic, enabling efficient tracking of mining performance metrics.