// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/utils/mod.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file is the module declaration for utility functions in the SHA3x miner,
// located in the utils subdirectory. It declares submodules for shared utility
// logic used across the project.
//
// Tree Location:
// - src/utils/mod.rs (utils module entry point)
// - Submodules: format

pub mod format;

// Changelog:
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Defines the utils module, organizing shared utility functions
//     into the format submodule for use throughout the miner.
//   - Features: Declares the format submodule, which contains functions for
//     formatting hashrate, duration, and numbers for consistent output.
//   - Note: This file provides a structured entry point for utility logic,
//     promoting code reuse across modules like miner and stats.