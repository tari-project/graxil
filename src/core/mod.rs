// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/core/mod.rs
// Version: 1.0.1
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file is the module declaration for the core functionality of the SHA3x
// miner, located in the core subdirectory. It declares submodules and re-exports
// key types for use throughout the project.

pub mod sha3x;
pub mod sha256;
pub mod difficulty;
pub mod types;

// Re-export the most commonly used items
pub use sha3x::{sha3x_hash_with_nonce_batch};
pub use sha256::{sha256d_hash, sha256d_hash_with_nonce_batch};
pub use difficulty::{calculate_difficulty, parse_target_difficulty};
pub use types::{Algorithm, Args, PoolJob, MiningJob, Share, ShareResponse, ShareResult, ShareError, Target};

// Changelog:
// - v1.0.1 (2025-06-16): Added simple SHA-256 support.
//   - Added sha256 module with basic double SHA-256 implementation.
//   - Added Algorithm enum export for sha3x/sha256 distinction.
//   - Kept original SHA3X exports unchanged for compatibility.