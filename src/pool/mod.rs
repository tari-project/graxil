// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/pool/mod.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file is the module declaration for the pool communication functionality
// of the SHA3x miner, located in the pool subdirectory. It declares submodules
// and re-exports key types for use throughout the project.
//
// Tree Location:
// - src/pool/mod.rs (pool module entry point)
// - Submodules: client, messages, protocol

pub mod client;
pub mod messages;
pub mod protocol;

// Re-export key types for convenience
pub use client::PoolClient;

// Changelog:
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Defines the pool module, organizing communication logic with
//     mining pools into client, messages, and protocol submodules.
//   - Features: Declares client, messages, and protocol submodules, with
//     re-export of the PoolClient type for easy access by the miner module.
//   - Note: This file provides a structured entry point for pool-related
//     operations, centralizing the logic for connecting to and interacting
//     with mining pools.