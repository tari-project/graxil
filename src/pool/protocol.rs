// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/pool/protocol.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements the Stratum protocol for communication with the mining
// pool in the SHA3x miner, located in the pool subdirectory. It constructs
// messages for login and share submission.
//
// Tree Location:
// - src/pool/protocol.rs (Stratum protocol logic)
// - Depends on: serde_json

use serde_json::{json, Value};

/// Constructs messages for the Stratum protocol
pub struct StratumProtocol;

impl StratumProtocol {
    /// Create a login request message
    pub fn create_login_request(wallet_address: &str, worker_name: &str) -> Value {
        json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "login",
            "params": {
                "login": wallet_address,
                "pass": worker_name,
                "agent": "sha3x-miner/3.0",
                "algo": "sha3x"
            }
        })
    }

    /// Create a share submission request message
    pub fn create_submit_request(
        wallet_address: &str,
        job_id: &str,
        nonce: &str,
        result: &str,
        submit_id: u64,
    ) -> Value {
        json!({
            "id": submit_id,
            "jsonrpc": "2.0",
            "method": "submit",
            "params": {
                "id": wallet_address,
                "job_id": job_id,
                "nonce": nonce,
                "result": result
            }
        })
    }

    /// Convert a JSON message to a string with newline
    pub fn to_message(json: Value) -> String {
        format!("{}\n", json)
    }
}

// Changelog:
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Implements the Stratum protocol for formatting messages sent to
//     the mining pool, ensuring compliance with pool communication standards.
//   - Features: Provides functions to create login requests and share submission
//     messages, with proper JSON formatting and newline termination for Stratum.
//   - Note: This file is crucial for constructing valid messages that the pool
//     can process, used by the miner module to initiate sessions and submit shares.