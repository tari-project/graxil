// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/core/mod.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file is the module declaration for the core functionality of the SHA3x
// miner, located in the core subdirectory. It declares submodules and re-exports
// key types for use throughout the project.
//
// Tree Location:
// - src/core/mod.rs (core module entry point)
// - Submodules: difficulty, sha3x, types

pub mod sha3x;
pub mod difficulty;
pub mod types;

// Re-export the most commonly used items
pub use sha3x::{sha3x_hash_with_nonce};
pub use difficulty::{calculate_difficulty, parse_target_difficulty};
pub use types::{Args, PoolJob, MiningJob, Share, ShareResponse, ShareResult, ShareError, Target};

// Changelog:
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Defines the core module, organizing essential mining functionality
//     into submodules for SHA3x hashing, difficulty calculations, and data types.
//   - Features: Declares difficulty, sha3x, and types submodules, with re-exports
//     of key functions and types (e.g., sha3x_hash_with_nonce, PoolJob) for easy
//     access by other modules like miner and pool.
//   - Note: This file ensures a clean structure for core logic, centralizing
//     critical components used across the miner.