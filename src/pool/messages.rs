// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/pool/messages.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file handles parsing and processing of messages received from the mining
// pool in the SHA3x miner, located in the pool subdirectory. It processes job
// notifications and share responses.
//
// Tree Location:
// - src/pool/messages.rs (pool message handling logic)
// - Depends on: serde_json, core/types, tracing

use crate::core::PoolJob;
use crate::Result;
use serde_json::Value;
use tracing::{error, info};

pub fn parse_pool_message(message: &str) -> Result<Option<PoolMessage>> {
    let response: Value = serde_json::from_str(message)?;
    info!("📨 Received pool message: {}", message);

    if let Some(method) = response.get("method").and_then(|m| m.as_str()) {
        if method == "job" {
            if let Some(params) = response.get("params").and_then(|p| p.as_object()) {
                let job: PoolJob = serde_json::from_value(Value::Object(params.clone()))?;
                return Ok(Some(PoolMessage::Job(job)));
            }
        }
    } else if let Some(result) = response.get("result") {
        if let Some(id) = response.get("id").and_then(|id| id.as_u64()) {
            if id == 1 {
                info!("✅ Login successful");
                return Ok(Some(PoolMessage::LoginSuccess));
            } else if id >= 100 {
                let thread_id = (id - 100) as usize;
                let accepted = if let Some(status) = result.get("status").and_then(|s| s.as_str()) {
                    matches!(status.to_lowercase().as_str(), "ok" | "accepted")
                } else if result.is_null() {
                    info!("✅ Share accepted (null response)");
                    true
                } else if let Some(accepted) = result.as_bool() {
                    accepted
                } else {
                    error!("❌ Unknown share response format: {:?}", result);
                    false
                };
                return Ok(Some(PoolMessage::ShareResponse { thread_id, accepted }));
            }
        }
    } else if let Some(error) = response.get("error") {
        error!("❌ Pool error: {:?}", error);
        return Ok(Some(PoolMessage::Error(error.to_string())));
    }

    Ok(None)
}

#[derive(Debug)]
pub enum PoolMessage {
    Job(PoolJob),
    LoginSuccess,
    ShareResponse { thread_id: usize, accepted: bool },
    Error(String),
}

// Changelog:
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Parses and processes messages from the mining pool, converting
//     JSON responses into structured types for further handling by the miner.
//   - Features: Handles job notifications, login success, share responses, and
//     errors using a PoolMessage enum. Supports VarDiff job updates and various
//     share response formats (status, null, boolean).
//   - Note: This file is essential for interpreting pool communication, enabling
//     the miner to react to new jobs, track share acceptance, and handle errors
//     effectively.