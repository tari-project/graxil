// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: tests/integration_test.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file contains integration tests for the SHA3x miner, located in the tests
// directory. It verifies the end-to-end functionality of core components like
// hashing, difficulty calculations, job processing, share creation, and pool
// message parsing.
//
// Tree Location:
// - tests/integration_test.rs (integration tests)
// - Depends on: sha3x-miner, tokio, hex

#[cfg(test)]
mod tests {
    use graxil::core::sha3x::sha3x_hash_with_nonce;
    use graxil::core::difficulty::{calculate_difficulty, parse_target_difficulty};
    use graxil::core::types::{MiningJob, PoolJob, Share};
    use graxil::pool::messages::{parse_pool_message, PoolMessage};
    use hex;

    #[tokio::test]
    async fn test_sha3x_hash_and_difficulty() {
        let header = vec![0u8; 32];
        let nonce = 12345u64;
        let hash = sha3x_hash_with_nonce(&header, nonce);
        let difficulty = calculate_difficulty(&hash);
        assert_eq!(hash.len(), 32, "Hash length should be 32 bytes");
        assert!(difficulty > 0, "Difficulty should be positive");
    }

    #[tokio::test]
    async fn test_target_difficulty_parsing() {
        let target_hex = "000000ffff000000000000000000000000000000000000000000000000000000";
        let difficulty = parse_target_difficulty(target_hex);
        assert!(difficulty > 0, "Parsed difficulty should be positive");
    }

    #[tokio::test]
    async fn test_pool_job_to_mining_job() {
        let pool_job = PoolJob {
            blob: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            job_id: "test_job".to_string(),
            target: "000000ffff000000000000000000000000000000000000000000000000000000".to_string(),
            algo: "sha3x".to_string(),
            height: 123456,
            seed_hash: None,
            difficulty: Some(1000),
        };
        let header_template = hex::decode(&pool_job.blob).unwrap();
        let mining_job = MiningJob {
            job_id: pool_job.job_id.clone(),
            mining_hash: header_template,
            target_difficulty: pool_job.difficulty.unwrap_or_else(|| parse_target_difficulty(&pool_job.target)),
            height: pool_job.height,
        };
        assert_eq!(mining_job.job_id, "test_job", "Job ID should match");
        assert_eq!(mining_job.mining_hash.len(), 32, "Mining hash length should be 32 bytes");
        assert_eq!(mining_job.target_difficulty, 1000, "Target difficulty should match");
        assert_eq!(mining_job.height, 123456, "Height should match");
    }

    #[tokio::test]
    async fn test_share_creation() {
        let share = Share::new(
            "test_job".to_string(),
            12345u64,
            vec![0u8; 32],
            1000u64,
            0,
        );
        assert_eq!(share.job_id, "test_job", "Share job ID should match");
        assert_eq!(share.nonce, 12345, "Share nonce should match");
        assert_eq!(share.hash.len(), 32, "Share hash length should be 32 bytes");
        assert_eq!(share.difficulty, 1000, "Share difficulty should match");
        assert_eq!(share.thread_id, 0, "Share thread ID should match");
        assert!(share.age().as_secs() < 1, "Share age should be recent");
    }

    #[tokio::test]
    async fn test_pool_message_parsing_login_success() {
        let message = r#"{"id":1,"jsonrpc":"2.0","result":{"status":"OK"}}"#;
        let result = parse_pool_message(message).unwrap();
        assert!(matches!(result, Some(PoolMessage::LoginSuccess)), "Should parse login success");
    }

    #[tokio::test]
    async fn test_pool_message_parsing_share_response() {
        let message = r#"{"id":100,"jsonrpc":"2.0","result":{"status":"accepted"}}"#;
        let result = parse_pool_message(message).unwrap();
        assert!(matches!(result, Some(PoolMessage::ShareResponse { thread_id: 0, accepted: true })), 
            "Should parse share accepted response");
    }
}

// Changelog:
// - v1.0.0 (2025-06-14): Enhanced integration test implementation.
//   - Purpose: Verifies the end-to-end functionality of the SHA3x miner's core
//     components, ensuring correct hash computation, difficulty calculation, job
//     processing, share creation, and pool message parsing.
//   - Features: Includes tests for sha3x_hash_with_nonce, calculate_difficulty,
//     parse_target_difficulty, PoolJob to MiningJob conversion, Share creation,
//     and pool message parsing for login and share responses.
//   - Note: This file provides comprehensive integration tests, supporting
//     validation of the miner's core logic without TUI functionality, with room
//     for expansion to include thread and pool communication tests.