// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/core/difficulty.rs
// Version: 1.0.1
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file contains functions for calculating difficulty and parsing target
// difficulty from pool-provided hex strings, located in the core subdirectory
// of the SHA3x miner source tree.
//
// Tree Location:
// - src/core/difficulty.rs (difficulty calculation logic)
// - Depends on: hex

use hex;

/// Parse target difficulty from a hex-encoded string provided by the pool
pub fn parse_target_difficulty(target_hex: &str) -> u64 {
    match hex::decode(target_hex) {
        Ok(target_bytes) => {
            if target_bytes.len() >= 8 {
                let target_u64 = u64::from_le_bytes([
                    target_bytes[0], target_bytes[1], target_bytes[2], target_bytes[3],
                    target_bytes[4], target_bytes[5], target_bytes[6], target_bytes[7],
                ]);
                if target_u64 > 0 {
                    0xFFFFFFFFFFFFFFFFu64 / target_u64
                } else {
                    1
                }
            } else {
                1
            }
        }
        Err(_) => 1,
    }
}

/// Calculate the difficulty of a given hash to determine if it meets the target
pub fn calculate_difficulty(hash: &[u8]) -> u64 {
    if hash.len() < 8 {
        return 0;
    }
    
    let hash_u64 = u64::from_be_bytes([
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7],
    ]);
    
    if hash_u64 == 0 {
        return u64::MAX;
    }
    
    0xFFFFFFFFFFFFFFFFu64 / hash_u64
}

// Changelog:
// - v1.0.1 (2025-06-14T01:16:00Z): Restored original difficulty logic.
//   - Reverted calculate_difficulty and parse_target_difficulty to match main.rs logic, removing debug logging to stop spam and fix low difficulty values (~1 or ~2), aiming to restore MH/s and VarDiff updates.
// - v1.0.0 (2025-06-14T00:00:00Z): Extracted from monolithic main.rs.
//   - Purpose: Computes difficulty for shares and parses pool target hex strings.
//   - Features: Includes calculate_difficulty and parse_target_difficulty for share validation.