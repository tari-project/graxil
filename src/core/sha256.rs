// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/core/sha256.rs
// Version: 1.0.4
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements the SHA256d (double SHA-256) algorithm used by Bitcoin,
// following the same pattern as the SHA3X implementation. It provides functions
// for single and batch hashing with nonce iteration.

use sha2::{Digest, Sha256};
use core::array;
use tracing::debug;

pub fn sha256d_hash(header: &[u8]) -> [u8; 32] {
    if header.len() != 80 {
        debug!("Invalid SHA-256 header length: {} bytes (expected 80)", header.len());
        return [0xFF; 32];
    }

    let mut hasher = Sha256::new();
    hasher.update(header);
    let first = hasher.finalize();
    
    let mut hasher = Sha256::new();
    hasher.update(first);
    let result = hasher.finalize().into();
    
    debug!("SHA256d hash: {}", hex::encode(&result));
    result
}

pub fn sha256d_hash_with_nonce_batch(header_base: &[u8], start_nonce: u32) -> [([u8; 32], u32); 4] {
    if header_base.len() != 80 {
        debug!("Invalid SHA-256 header length: {} bytes (expected 80)", header_base.len());
        return [
            ([0xFF; 32], start_nonce),
            ([0xFF; 32], start_nonce.wrapping_add(1)),
            ([0xFF; 32], start_nonce.wrapping_add(2)),
            ([0xFF; 32], start_nonce.wrapping_add(3)),
        ];
    }
    
    let mut results: [([u8; 32], u32); 4] = array::from_fn(|_| ([0; 32], 0));
    let mut header = [0u8; 80];
    header.copy_from_slice(header_base);
    
    for i in 0..4 {
        let nonce = start_nonce.wrapping_add(i as u32);
        header[76..80].copy_from_slice(&nonce.to_le_bytes());
        let hash = sha256d_hash(&header);
        results[i] = (hash, nonce);
        debug!("Batch hash {}: nonce={:08x}, hash={}", i, nonce, hex::encode(&hash));
    }
    
    results
}

// Changelog:
// - v1.0.4 (2025-06-18): Fixed type mismatches and compilation errors.
//   - Reverted sha256d_hash to return [u8; 32] for compatibility with thread.rs and runner.rs.
//   - Reverted sha256d_hash_with_nonce_batch to return [([u8; 32], u32); 4].
//   - Removed uint::construct_uint import and U256 usage, as byte arrays suffice.
//   - Simplified error handling to return [0xFF; 32] for invalid headers.
//   - Compatible with miner.rs v1.2.24, thread.rs v1.1.2, difficulty.rs v1.2.8, protocol.rs v1.0.1.