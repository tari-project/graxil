// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/miner/cpu/mod.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file is the module declaration for the CPU mining functionality of the
// SHA3x miner, located in the cpu subdirectory of the miner module. It declares
// submodules and re-exports key types for use throughout the project.
//
// Tree Location:
// - src/miner/cpu/mod.rs (CPU miner module entry point)
// - Submodules: miner, thread

pub mod miner;
pub mod thread;

// Re-export key types for convenience
pub use miner::CpuMiner;

// Changelog:
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Defines the cpu submodule, organizing CPU-specific mining logic
//     into miner and thread submodules.
//   - Features: Declares miner and thread submodules, with re-export of the
//     CpuMiner type for easy access by main.rs and other modules.
//   - Note: This file provides a structured entry point for CPU mining
//     operations, separating the main miner logic from thread-specific tasks.
