// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/utils/format.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file provides utility functions for formatting statistics in the SHA3x
// miner, located in the utils subdirectory. It formats hashrate, duration, and
// numbers for consistent output in logs and displays.
//
// Tree Location:
// - src/utils/format.rs (formatting utilities)
// - Depends on: std

use std::time::Duration;

/// Utility functions for formatting miner statistics
pub struct FormatUtils;

impl FormatUtils {
    /// Format hashrate in appropriate units (H/s, KH/s, MH/s, GH/s)
    pub fn format_hashrate(hashrate: f64) -> String {
        if hashrate >= 1_000_000_000.0 {
            format!("{:.2} GH/s", hashrate / 1_000_000_000.0)
        } else if hashrate >= 1_000_000.0 {
            format!("{:.2} MH/s", hashrate / 1_000_000.0)
        } else if hashrate >= 1_000.0 {
            format!("{:.2} KH/s", hashrate / 1_000.0)
        } else {
            format!("{:.2} H/s", hashrate)
        }
    }

    /// Format duration for human-readable output (seconds, minutes, hours)
    pub fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else {
            format!("{}h ago", secs / 3600)
        }
    }

    /// Format large numbers with suffixes (K, M, B)
    pub fn format_number(num: u64) -> String {
        if num >= 1_000_000_000 {
            format!("{:.1}B", num as f64 / 1_000_000_000.0)
        } else if num >= 1_000_000 {
            format!("{:.1}M", num as f64 / 1_000_000.0)
        } else if num >= 1_000 {
            format!("{:.1}K", num as f64 / 1_000.0)
        } else {
            num.to_string()
        }
    }
}

// Changelog:
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Provides utility functions for formatting miner statistics, ensuring
//     consistent and human-readable output for hashrate, duration, and numbers.
//   - Features: Includes functions to format hashrate in H/s to GH/s units, duration
//     in seconds to hours, and numbers with K/M/B suffixes for large values.
//   - Note: This file is used by the stats module to format metrics for logging,
//     promoting consistency across the miner's output.
