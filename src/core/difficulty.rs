// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/core/difficulty.rs
// Version: 1.2.10
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file contains functions for calculating difficulty and parsing target
// difficulty from pool-provided hex strings, located in the core subdirectory
// of the SHA3x miner source tree. It supports SHA3x (Tari) with u64 and
// SHA-256 (Bitcoin) with 256-bit precision.

use crate::core::types::Algorithm;
use hex;
use log::{debug, warn};
use uint::construct_uint;

const LOG_TARGET: &str = "tari::graxil::difficulty";

construct_uint! {
    pub struct U256(4);
}

const MAX_TARGET: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub fn parse_target_difficulty(target_hex: &str, algo: Algorithm) -> u64 {
    match algo {
        Algorithm::Sha3x => match hex::decode(target_hex) {
            Ok(target_bytes) => {
                if target_bytes.len() >= 8 {
                    let target_u64 = u64::from_le_bytes([
                        target_bytes[0],
                        target_bytes[1],
                        target_bytes[2],
                        target_bytes[3],
                        target_bytes[4],
                        target_bytes[5],
                        target_bytes[6],
                        target_bytes[7],
                    ]);
                    if target_u64 > 0 {
                        0xFFFFFFFFFFFFFFFFu64 / target_u64
                    } else {
                        warn!(target: LOG_TARGET,"Invalid SHA3x target: zero value");
                        1
                    }
                } else {
                    warn!(target: LOG_TARGET,
                        "Invalid SHA3x target: too short ({} bytes)",
                        target_bytes.len()
                    );
                    1
                }
            }
            Err(e) => {
                warn!(target: LOG_TARGET,"Failed to decode SHA3x target hex: {}", e);
                1
            }
        },
        Algorithm::Sha256 => {
            if target_hex.is_empty() {
                warn!(target: LOG_TARGET,"SHA-256 target hex is empty, using default difficulty");
                return 1;
            }
            match hex::decode(target_hex) {
                Ok(target_bytes) => {
                    if target_bytes.len() != 32 {
                        warn!(target: LOG_TARGET,
                            "Invalid SHA-256 target: wrong length ({} bytes)",
                            target_bytes.len()
                        );
                        return 1;
                    }
                    let target = U256::from_big_endian(&target_bytes); // Stratum target is big-endian
                    if target.is_zero() {
                        warn!(target: LOG_TARGET,"Invalid SHA-256 target: zero value, using default difficulty");
                        return 1;
                    }
                    let max_target = U256::from_big_endian(&MAX_TARGET);
                    let quotient = max_target / target;
                    debug!(target: LOG_TARGET,
                        "Parsed target: {:064x}, difficulty: {}",
                        target,
                        quotient.low_u64()
                    );
                    if quotient > U256::from(u64::MAX) {
                        u64::MAX
                    } else {
                        quotient.low_u64()
                    }
                }
                Err(e) => {
                    warn!(target: LOG_TARGET,"Failed to decode SHA-256 target hex: {}", e);
                    1
                }
            }
        }
    }
}

pub fn calculate_difficulty(hash: &[u8], algo: Algorithm) -> u64 {
    match algo {
        Algorithm::Sha3x => {
            if hash.len() < 8 {
                warn!(target: LOG_TARGET,"Invalid SHA3x hash: too short ({} bytes)", hash.len());
                return 0;
            }
            let hash_u64 = u64::from_be_bytes([
                hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
            ]);
            if hash_u64 == 0 {
                warn!(target: LOG_TARGET,"Invalid SHA3x hash: zero value");
                u64::MAX
            } else {
                0xFFFFFFFFFFFFFFFFu64 / hash_u64
            }
        }
        Algorithm::Sha256 => {
            if hash.len() != 32 {
                warn!(target: LOG_TARGET,"Invalid SHA-256 hash: wrong length ({} bytes)", hash.len());
                return 0;
            }
            let hash_value = U256::from_big_endian(hash); // Bitcoin hashes are big-endian in logs
            if hash_value.is_zero() {
                warn!(target: LOG_TARGET,"Invalid SHA-256 hash: all zeros");
                return 0;
            }
            let max_target = U256::from_big_endian(&MAX_TARGET);
            let quotient = max_target / hash_value;
            debug!(target: LOG_TARGET,
                "Calculated difficulty: hash={:064x}, difficulty={}",
                hash_value,
                quotient.low_u64()
            );
            if quotient > U256::from(u64::MAX) {
                u64::MAX
            } else {
                quotient.low_u64()
            }
        }
    }
}

pub fn difficulty_to_target(difficulty: f64) -> U256 {
    if difficulty <= 0.0 {
        warn!(target: LOG_TARGET,"Invalid Bitcoin difficulty: {}", difficulty);
        return U256::from(u64::MAX);
    }
    let max_target = U256::from_big_endian(&MAX_TARGET);
    let difficulty_u256 = U256::from((difficulty * 1_000_000.0) as u64);
    if difficulty_u256.is_zero() {
        warn!(target: LOG_TARGET,"Difficulty converts to zero U256");
        return U256::from(u64::MAX);
    }
    let target = max_target / difficulty_u256;
    debug!(target: LOG_TARGET,"Difficulty {} -> target: {:064x}", difficulty, target);
    target
}

pub fn hash_meets_target(hash: &[u8], target: U256) -> bool {
    if hash.len() != 32 {
        warn!(target: LOG_TARGET,
            "Invalid hash for SHA-256 target check: wrong length ({} bytes)",
            hash.len()
        );
        return false;
    }
    let hash_value = U256::from_big_endian(hash);
    debug!(target: LOG_TARGET,
        "Hash check: hash={:064x}, target={:064x}",
        hash_value, target
    );
    hash_value <= target
}

pub fn bits_to_target(bits: u32) -> U256 {
    let exponent = ((bits >> 24) & 0xFF) as i32;
    let mantissa = bits & 0x00FFFFFF;
    if exponent <= 0 || mantissa == 0 {
        warn!(target: LOG_TARGET,"Invalid nbits: {:08x}, returning zero target", bits);
        return U256::zero();
    }
    let shift = exponent - 3;
    let target = U256::from(mantissa);
    let result = if shift >= 0 {
        target << (shift * 8)
    } else {
        target >> ((-shift) * 8)
    };
    debug!(target: LOG_TARGET,"nbits={:08x} -> target={:064x}", bits, result);
    result
}

// Changelog:
// - v1.2.10 (2025-06-19): Fixed SHA-256 target calculation for share validation.
//   - Changed parse_target_difficulty to use from_big_endian for SHA-256 targets.
//   - Updated calculate_difficulty to use from_big_endian for SHA-256 hashes.
//   - Updated hash_meets_target to use from_big_endian for consistency.
//   - Added debug logging for targets, hashes, and difficulties.
//   - Preserved SHA3x logic unchanged.
//   - Compatible with miner.rs v1.2.30, thread.rs v1.1.4, sha256.rs v1.0.4, protocol.rs v1.0.1.
