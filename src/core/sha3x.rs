// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/core/sha3x.rs
// Version: 1.0.1
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements the SHA3x triple-hash algorithm used by Tari, located
// in the core subdirectory of the SHA3x miner source tree. It provides the
// hashing function for mining operations.
//
// Tree Location:
// - src/core/sha3x.rs (SHA3x algorithm implementation)
// - Depends on: sha3 crate

use sha3::{Digest, Sha3_256};
use core::array;

/// Perform SHA3x hash (triple SHA3-256) with a nonce for mining
pub fn sha3x_hash_with_nonce(header_template: &[u8], nonce: u64) -> Vec<u8> {
    let mut input = Vec::with_capacity(header_template.len() + 9);
    input.extend_from_slice(&nonce.to_le_bytes());
    input.extend_from_slice(header_template);
    input.push(1u8);

    let hash1 = Sha3_256::digest(&input);
    let hash2 = Sha3_256::digest(&hash1);
    let hash3 = Sha3_256::digest(&hash2);

    hash3.to_vec()
}

/// Perform SHA3x hash for 4 nonces in batch, reusing input buffer
pub fn sha3x_hash_with_nonce_batch(header_template: &[u8], nonce: u64) -> [(Vec<u8>, u64); 4] {
    let mut input = Vec::with_capacity(header_template.len() + 9);
    input.extend_from_slice(&[0u8; 8]); // Placeholder for nonce
    input.extend_from_slice(header_template);
    input.push(1u8);

    let mut results: [(Vec<u8>, u64); 4] = array::from_fn(|_| (Vec::new(), 0));
    for i in 0..4 {
        let n = nonce + i as u64;
        input[0..8].copy_from_slice(&n.to_le_bytes());
        let hash1 = Sha3_256::digest(&input);
        let hash2 = Sha3_256::digest(&hash1);
        let hash3 = Sha3_256::digest(&hash2);
        results[i] = (hash3.to_vec(), n);
    }

    results
}

// Changelog:
// - v1.0.1 (2025-06-14T20:40:00Z EDT): Added batch hashing optimization.
//   - Introduced sha3x_hash_with_nonce_batch to compute 4 hashes per call, reusing input buffer.
//   - Fixed compilation error by using core::array::from_fn for results array.
//   - Reduced allocation and copy overhead for improved hashrate.
//   - Maintained scalar sha3x_hash_with_nonce for compatibility.
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Implements SHA3x triple-hash for Tari mining.
//   - Features: Triple SHA3-256 with nonce, header, and marker byte.