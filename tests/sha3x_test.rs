// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: tests/sha3x_test.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file contains unit tests for the SHA3x algorithm in the SHA3x miner,
// located in the tests directory. It verifies the correctness of the triple
// SHA3-256 hashing implementation.
//
// Tree Location:
// - tests/sha3x_test.rs (SHA3x algorithm tests)
// - Depends on: sha3x-miner, sha3

#[cfg(test)]
mod tests {
    use graxil::core::sha3x::sha3x_hash_with_nonce;
    use sha3::{Digest, Sha3_256};

    #[test]
    fn test_sha3x_hash() {
        let header = vec![0u8; 32];
        let nonce = 12345u64;
        let hash = sha3x_hash_with_nonce(&header, nonce);

        // Manually compute triple SHA3-256 for verification
        let mut input = Vec::with_capacity(header.len() + 9);
        input.extend_from_slice(&nonce.to_le_bytes());
        input.extend_from_slice(&header);
        input.push(1u8);

        let hash1 = Sha3_256::digest(&input);
        let hash2 = Sha3_256::digest(&hash1);
        let hash3 = Sha3_256::digest(&hash2);

        assert_eq!(hash, hash3.to_vec(), "Triple SHA3-256 hash should match");
        assert_eq!(hash.len(), 32, "Hash length should be 32 bytes");
    }

    #[test]
    fn test_sha3x_hash_different_nonce() {
        let header = vec![0u8; 32];
        let nonce1 = 12345u64;
        let nonce2 = 67890u64;
        let hash1 = sha3x_hash_with_nonce(&header, nonce1);
        let hash2 = sha3x_hash_with_nonce(&header, nonce2);

        assert_ne!(hash1, hash2, "Hashes with different nonces should differ");
    }
}

// Changelog:
// - v1.0.0 (2025-06-14): Initial SHA3x unit test implementation.
//   - Purpose: Verifies the correctness of the SHA3x triple-hash algorithm,
//     ensuring accurate hash computation for mining operations.
//   - Features: Includes tests for sha3x_hash_with_nonce, checking hash length,
//     correctness against manual SHA3-256 computation, and nonce variation.
//   - Note: This file focuses on the core SHA3x algorithm, providing a foundation
//     for ensuring the miner's hashing logic is reliable.
