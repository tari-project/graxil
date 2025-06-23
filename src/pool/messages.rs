// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/core/difficulty.rs
// Version: 1.2.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file contains functions for calculating difficulty and parsing target
// difficulty from pool-provided hex strings, located in the core subdirectory
// of the SHA3x miner source tree. It supports both SHA3x (Tari) with u64 and
// SHA-256 (Bitcoin) with 256-bit precision.

use crate::core::types::Algorithm;
use hex;
use uint::construct_uint;
use tracing::{warn};

construct_uint! {
    pub struct U256(4);
}

/// Bitcoin's maximum target (difficulty 1)
const MAX_TARGET: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Parse target difficulty from a hex-encoded string provided by the pool
pub fn parse_target_difficulty(target_hex: &str, algo: Algorithm) -> u64 {
    match algo {
        Algorithm::Sha3x => {
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
                            warn!("Invalid SHA3x target: zero value");
                            1
                        }
                    } else {
                        warn!("Invalid SHA3x target: too short ({} bytes)", target_bytes.len());
                        1
                    }
                }
                Err(e) => {
                    warn!("Failed to decode SHA3x target hex: {}", e);
                    1
                }
            }
        }
        Algorithm::Sha256 => {
            match hex::decode(target_hex) {
                Ok(target_bytes) => {
                    if target_bytes.len() == 32 {
                        let target = U256::from_big_endian(&target_bytes);
                        if target.is_zero() {
                            warn!("Invalid SHA-256 target: zero value");
                            1
                        } else {
                            let max_target = U256::from_big_endian(&MAX_TARGET);
                            (max_target / target).low_u64()
                        }
                    } else {
                        warn!("Invalid SHA-256 target: wrong length ({} bytes)", target_bytes.len());
                        1
                    }
                }
                Err(e) => {
                    warn!("Failed to decode SHA-256 target hex: {}", e);
                    1
                }
            }
        }
    }
}

/// Calculate the difficulty of a given hash to determine if it meets the target
pub fn calculate_difficulty(hash: &[u8], algo: Algorithm) -> u64 {
    match algo {
        Algorithm::Sha3x => {
            if hash.len() < 8 {
                warn!("Invalid SHA3x hash: too short ({} bytes)", hash.len());
                return 0;
            }
            let hash_u64 = u64::from_be_bytes([
                hash[0], hash[1], hash[2], hash[3],
                hash[4], hash[5], hash[6], hash[7],
            ]);
            if hash_u64 == 0 {
                warn!("Invalid SHA3x hash: zero value");
                u64::MAX
            } else {
                0xFFFFFFFFFFFFFFFFu64 / hash_u64
            }
        }
        Algorithm::Sha256 => {
            if hash.len() != 32 {
                warn!("Invalid SHA-256 hash: wrong length ({} bytes)", hash.len());
                return 0;
            }
            let hash_value = U256::from_big_endian(hash);
            if hash_value.is_zero() {
                warn!("Invalid SHA-256 hash: zero value");
                u64::MAX
            } else {
                let max_target = U256::from_big_endian(&MAX_TARGET);
                (max_target / hash_value).low_u64()
            }
        }
    }
}

/// Convert Bitcoin network difficulty to target value
pub fn difficulty_to_target(difficulty: f64) -> u64 {
    if difficulty <= 0.0 {
        warn!("Invalid Bitcoin difficulty: {}", difficulty);
        return u64::MAX;
    }
    let max_target = U256::from_big_endian(&MAX_TARGET);
    let target_u256 = max_target / U256::from(difficulty as u64);
    let target_bytes = target_u256.to_big_endian();
    u64::from_be_bytes([
        target_bytes[0], target_bytes[1], target_bytes[2], target_bytes[3],
        target_bytes[4], target_bytes[5], target_bytes[6], target_bytes[7]
    ])
}

/// Check if a hash meets the target difficulty for Bitcoin
pub fn hash_meets_target(hash: &[u8], target: u64) -> bool {
    if hash.len() < 8 {
        warn!("Invalid hash for target check: too short ({} bytes)", hash.len());
        return false;
    }
    let hash_value = u64::from_be_bytes([
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7]
    ]);
    hash_value <= target
}

/// Convert Bitcoin difficulty bits (nBits) to target
pub fn bits_to_target(bits: u32) -> U256 {
    let exponent = ((bits >> 24) & 0xFF) as usize;
    let mantissa = bits & 0x00FFFFFF;
    if exponent <= 3 {
        U256::from(mantissa >> (8 * (3 - exponent)))
    } else {
        U256::from(mantissa) << (8 * (exponent - 3))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha3x_difficulty_calculation() {
        let hash = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let difficulty = calculate_difficulty(&hash, Algorithm::Sha3x);
        assert_eq!(difficulty, 0xFFFFFFFFFFFFFFFFu64 / 1);
    }

    #[test]
    fn test_sha256_difficulty_calculation() {
        let hash = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let difficulty = calculate_difficulty(&hash, Algorithm::Sha256);
        assert!(difficulty > 0);
    }

    #[test]
    fn test_sha3x_parse_target() {
        let target_hex = "0100000000000000";
        let difficulty = parse_target_difficulty(target_hex, Algorithm::Sha3x);
        assert_eq!(difficulty, 0xFFFFFFFFFFFFFFFFu64 / 1);
    }

    #[test]
    fn test_sha256_hash_meets_target() {
        let hash = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let target = 0x0000000000000002u64;
        assert!(hash_meets_target(&hash, target));
        let hard_target = 0x0000000000000000u64;
        assert!(!hash_meets_target(&hash, hard_target));
    }

    #[test]
    fn test_difficulty_to_target() {
        let target = difficulty_to_target(1.0);
        assert!(target > 0);
        let hard_target = difficulty_to_target(1000.0);
        assert!(hard_target < target);
    }
}

// Changelog:
// - v1.2.0 (2025-06-16): Restored u64 for SHA3x, kept U256 for SHA-256.
//   - Restored original u64-based logic for SHA3x in parse_target_difficulty and calculate_difficulty to fix inaccurate difficulty calculations.
//   - Retained U256 for SHA-256 to support Bitcoin's 256-bit targets, ensuring compatibility with Stratum V1.
//   - Added Algorithm parameter to parse_target_difficulty and calculate_difficulty for algorithm-specific logic.
//   - Improved error handling with tracing logs for invalid inputs.
//   - Updated tests to cover both SHA3x and SHA-256 cases.
// - v1.1.0 (2025-06-16): Enhanced Bitcoin difficulty support.
//   - Added difficulty_to_target function for Bitcoin network difficulty conversion.
//   - Added hash_meets_target function for efficient target comparison.
//   - Added bits_to_target function for nBits conversion.
//   - Updated MAX_TARGET to proper Bitcoin difficulty 1 target.
//   - Added comprehensive test suite for difficulty calculations.
// - v1.0.1 (2025-06-14): Restored original difficulty logic.
//   - Reverted calculate_difficulty and parse_target_difficulty to match main.rs logic, removing debug logging to stop spam and fix low difficulty values (~1 or ~2), aiming to restore MH/s and VarDiff updates.
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Computes difficulty for shares and parses pool target hex strings.
//   - Features: Includes calculate_difficulty and parse_target_difficulty for share validation.