// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/miner/stats/thread_stats.rs
// Version: 1.0.1
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements per-thread statistics tracking for the SHA3x miner,
// located in the stats subdirectory of the miner module. It monitors individual
// thread performance, including shares, hashrate, and difficulty.
//
// Tree Location:
// - src/miner/stats/thread_stats.rs (per-thread statistics logic)
// - Depends on: std

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct ThreadStats {
    #[allow(dead_code)] // Field unused in non-TUI version but kept for future use
    thread_id: usize,
    pub hashes_computed: AtomicU64,
    pub shares_found: AtomicU64,
    pub shares_rejected: AtomicU64,
    last_share_time: Arc<Mutex<Option<Instant>>>,
    start_time: Instant,
    current_hashrate: Arc<Mutex<f64>>,
    pub peak_hashrate: AtomicU64,
    pub best_difficulty: AtomicU64,
    pub current_difficulty_target: AtomicU64,
}

impl ThreadStats {
    /// Create a new ThreadStats instance for a specific thread
    pub fn new(thread_id: usize) -> Self {
        Self {
            thread_id,
            hashes_computed: AtomicU64::new(0),
            shares_found: AtomicU64::new(0),
            shares_rejected: AtomicU64::new(0),
            last_share_time: Arc::new(Mutex::new(None)),
            start_time: Instant::now(),
            current_hashrate: Arc::new(Mutex::new(0.0)),
            peak_hashrate: AtomicU64::new(0),
            best_difficulty: AtomicU64::new(0),
            current_difficulty_target: AtomicU64::new(0),
        }
    }

    /// Record a share (accepted or rejected)
    pub fn record_share(&self, difficulty: u64, accepted: bool) {
        if accepted {
            self.shares_found.fetch_add(1, Ordering::Relaxed);
        } else {
            self.shares_rejected.fetch_add(1, Ordering::Relaxed);
        }

        *self.last_share_time.lock().unwrap() = Some(Instant::now());

        let current_best = self.best_difficulty.load(Ordering::Relaxed);
        if difficulty > current_best {
            self.best_difficulty.store(difficulty, Ordering::Relaxed);
        }
    }

    /// Update hashrate based on computed hashes
    pub fn update_hashrate(&self, hashes: u64) {
        self.hashes_computed.fetch_add(hashes, Ordering::Relaxed);
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            let total_hashes = self.hashes_computed.load(Ordering::Relaxed);
            let current_rate = total_hashes as f64 / elapsed;
            
            // Update current hashrate
            *self.current_hashrate.lock().unwrap() = current_rate;
            
            // Update peak hashrate if this is higher
            let current_rate_u64 = current_rate as u64;
            let current_peak = self.peak_hashrate.load(Ordering::Relaxed);
            if current_rate_u64 > current_peak {
                self.peak_hashrate.store(current_rate_u64, Ordering::Relaxed);
            }
        }
    }

    /// Get the current hashrate
    pub fn get_hashrate(&self) -> f64 {
        *self.current_hashrate.lock().unwrap()
    }

    /// Get the peak hashrate achieved
    pub fn get_peak_hashrate(&self) -> f64 {
        self.peak_hashrate.load(Ordering::Relaxed) as f64
    }

    /// Reset peak hashrate (useful for benchmarking)
    pub fn reset_peak_hashrate(&self) {
        self.peak_hashrate.store(0, Ordering::Relaxed);
    }

    /// Get a string of share indicators (dots)
    pub fn get_share_dots(&self) -> String {
        let accepted = self.shares_found.load(Ordering::Relaxed);
        let rejected = self.shares_rejected.load(Ordering::Relaxed);

        let mut dots = String::new();
        for _ in 0..accepted.min(5) {
            dots.push('●');
        }
        for _ in 0..rejected.min(5) {
            dots.push('○');
        }
        dots
    }
}

// Changelog:
// - v1.0.1 (2025-06-14): Added peak hashrate tracking for benchmarking.
//   - Added peak_hashrate field to track maximum hashrate achieved.
//   - Updated update_hashrate to track peak performance automatically.
//   - Added get_peak_hashrate and reset_peak_hashrate methods.
//   - Maintained all existing functionality for mining operations.
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Tracks statistics for individual mining threads, monitoring
//     performance metrics like shares, hashrate, and difficulty.
//   - Features: Records accepted/rejected shares, updates hashrate periodically,
//     tracks best difficulty, and provides visual share indicators (dots) for
//     display. Uses thread-safe atomic counters and mutexes for concurrency.
//   - Note: This file is crucial for diagnosing thread-specific performance,
//     complementing MinerStats for a complete view of mining operations.