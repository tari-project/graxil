// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/pool/protocol.rs
// Version: 1.0.2
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements the Stratum protocol for communication with the mining
// pool in the SHA3x miner, located in the pool subdirectory. It constructs
// messages for login and share submission for both SHA3x and SHA-256 algorithms.
//
// Tree Location:
// - src/pool/protocol.rs (Stratum protocol logic)
// - Depends on: serde_json, crate::core::types

use crate::core::types::Algorithm;
use log::{debug, error};
use serde_json::{Value, json};

const LOG_TARGET: &str = "tari::graxil::pool::protocol";

/// Constructs messages for the Stratum protocol
pub struct StratumProtocol;

impl StratumProtocol {
    /// Create a login request message
    pub fn create_login_request(wallet_address: &str, worker_name: &str, algo: Algorithm) -> Value {
        if wallet_address.is_empty() {
            error!(target: LOG_TARGET,"Invalid wallet address: empty");
            return json!({});
        }
        match algo {
            Algorithm::Sha3x => json!({
                "id": 1,
                "jsonrpc": "2.0",
                "method": "login",
                "params": {
                    "login": wallet_address,
                    "pass": worker_name,
                    "agent": "sha3x-miner/3.0",
                    "algo": ["sha3x"]
                }
            }),
            Algorithm::Sha256 => json!({
                "id": 1,
                "method": "mining.subscribe",
                "params": ["sha3x-miner/3.0"]
            }),
        }
    }

    /// Create an authorization request for SHA-256 (Stratum V1)
    pub fn create_authorize_request(wallet_address: &str) -> Value {
        if wallet_address.is_empty() {
            error!(target: LOG_TARGET,"Invalid wallet address for authorize: empty");
            return json!({});
        }
        json!({
            "id": 2,
            "method": "mining.authorize",
            "params": [wallet_address, ""]
        })
    }

    /// Create a share submission request message
    pub fn create_submit_request(
        wallet_address: &str,
        job_id: &str,
        nonce: &str,
        result: &str,
        submit_id: u64,
        algo: Algorithm,
        extranonce2: Option<&str>,
        ntime: Option<u32>,
    ) -> Value {
        if job_id.is_empty() || nonce.is_empty() || result.is_empty() {
            error!(target: LOG_TARGET,
                "Invalid share submission: job_id={}, nonce={}, result={}",
                job_id, nonce, result
            );
            return json!({});
        }
        match algo {
            Algorithm::Sha3x => json!({
                "id": submit_id,
                "jsonrpc": "2.0",
                "method": "submit",
                "params": {
                    "id": wallet_address,
                    "job_id": job_id,
                    "nonce": nonce,
                    "result": result
                }
            }),
            Algorithm::Sha256 => {
                let extranonce2 = extranonce2.unwrap_or("");
                let ntime_hex = ntime.map(|n| format!("{:08x}", n)).unwrap_or_default();
                if extranonce2.is_empty() || ntime_hex.is_empty() {
                    error!(target: LOG_TARGET,
                        "Invalid SHA-256 share: extranonce2={}, ntime={}",
                        extranonce2, ntime_hex
                    );
                    return json!({});
                }
                json!({
                    "id": submit_id,
                    "method": "mining.submit",
                    "params": [
                        "",
                        job_id,
                        extranonce2,
                        ntime_hex,
                        nonce
                    ]
                })
            }
        }
    }

    /// Convert a JSON message to a string with newline
    pub fn to_message(json: Value) -> String {
        if json.is_null() {
            error!(target: LOG_TARGET,"Attempted to serialize empty JSON message");
            return String::new();
        }
        debug!(target: LOG_TARGET,"Serialized Stratum message: {}", json);
        format!("{}\n", json)
    }
}

// Changelog:
// - v1.0.2 (2025-06-23): Fixed algo field format for pool compatibility.
//   - Changed "algo": "sha3x" to "algo": ["sha3x"] (array format) in login request.
//   - This fixes compatibility with pools that expect algo as a list of strings.
//   - Maintains backward compatibility with pools that accept both formats.
//   - Resolves connection drops from strict pool implementations.
// - v1.0.1 (2025-06-19): Updated for SHA-256 share submission compatibility.
//   - Added create_authorize_request for SHA-256 mining.authorize.
//   - Added algo parameter to create_login_request and create_submit_request.
//   - Added validation for empty wallet_address, job_id, nonce, result, extranonce2, and ntime.
//   - Ensured SHA-256 mining.submit matches Stratum V1 format: ["", job_id, extranonce2, ntime, nonce].
//   - Preserved SHA3x logic unchanged.
//   - Compatible with miner.rs v1.2.24, thread.rs v1.1.1, difficulty.rs v1.2.7, sha256.rs v1.0.2.
