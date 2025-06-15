// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/miner/stats/miner_stats.rs
// Version: 1.0.3
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements miner-wide statistics tracking for the SHA3x miner,
// located in the stats subdirectory of the miner module. It manages shares,
// hashrate, and activity logs for the entire miner.
//
// Tree Location:
// - src/miner/stats/miner_stats.rs (miner-wide statistics logic)
// - Depends on: std, thread_stats

use super::thread_stats::ThreadStats;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, info};

#[allow(dead_code)] // Fields unused in non-TUI version but kept for future use
#[derive(Debug)]
pub struct ShareInfo {
    time: Instant,
    thread_id: usize,
    difficulty: u64,
    target: u64,
    accepted: bool,
}

pub struct MinerStats {
    pub shares_submitted: AtomicU64,
    pub shares_accepted: AtomicU64,
    pub shares_rejected: AtomicU64,
    pub hashes_computed: AtomicU64,
    start_time: Instant,
    pub thread_stats: Vec<Arc<ThreadStats>>,
    recent_shares: Arc<Mutex<VecDeque<ShareInfo>>>,
    recent_activity: Arc<Mutex<VecDeque<(Instant, String)>>>,
    hashrate_history: Arc<Mutex<VecDeque<(Instant, u64)>>>,
}

impl MinerStats {
    pub fn new(num_threads: usize) -> Self {
        let mut thread_stats = Vec::new();
        for i in 0..num_threads {
            thread_stats.push(Arc::new(ThreadStats::new(i)));
        }

        Self {
            shares_submitted: AtomicU64::new(0),
            shares_accepted: AtomicU64::new(0),
            shares_rejected: AtomicU64::new(0),
            hashes_computed: AtomicU64::new(0),
            start_time: Instant::now(),
            thread_stats,
            recent_shares: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            recent_activity: Arc::new(Mutex::new(VecDeque::with_capacity(50))),
            hashrate_history: Arc::new(Mutex::new(VecDeque::with_capacity(300))),
        }
    }

    pub fn add_activity(&self, message: String) {
        let mut activity = self.recent_activity.lock().unwrap();
        activity.push_back((Instant::now(), message));
        if activity.len() > 50 {
            activity.pop_front();
        }
    }

    pub fn record_share_found(&self, thread_id: usize, difficulty: u64, target: u64, accepted: bool) {
        if thread_id < self.thread_stats.len() {
            self.thread_stats[thread_id].record_share(difficulty, accepted);
        }

        let mut shares = self.recent_shares.lock().unwrap();
        shares.push_back(ShareInfo {
            time: Instant::now(),
            thread_id,
            difficulty,
            target,
            accepted,
        });
        if shares.len() > 100 {
            shares.pop_front();
        }
    }

    pub fn update_hashrate_history(&self, total_hashes: u64) {
        let mut history = self.hashrate_history.lock().unwrap();
        history.push_back((Instant::now(), total_hashes));

        let cutoff = Instant::now() - Duration::from_secs(300);
        while let Some((time, _)) = history.front() {
            if *time < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn get_total_hashrate(&self) -> f64 {
        let total_hashes = self.hashes_computed.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            total_hashes as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn get_total_hashrate_formatted(&self) -> String {
        Self::format_hashrate(self.get_total_hashrate())
    }

    pub fn get_active_thread_count(&self) -> usize {
        self.thread_stats
            .iter()
            .filter(|t| t.get_hashrate() > 0.0)
            .count()
    }

    pub fn get_avg_hashrate_per_thread(&self) -> f64 {
        let active = self.get_active_thread_count();
        if active > 0 {
            self.get_total_hashrate() / active as f64
        } else {
            0.0
        }
    }

    pub fn get_share_rate_per_minute(&self) -> f64 {
        let shares = self.shares_submitted.load(Ordering::Relaxed);
        let elapsed_mins = self.start_time.elapsed().as_secs_f64() / 60.0;
        if elapsed_mins > 0.0 {
            shares as f64 / elapsed_mins
        } else {
            0.0
        }
    }

    fn format_hashrate(hashrate: f64) -> String {
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

    pub fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m", secs / 60)
        } else {
            format!("{:.1}h", secs as f64 / 3600.0)
        }
    }

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

    /// Display a text-based dashboard with miner statistics
    pub fn display_dashboard(&self, dashboard_id: &str) {
        let hashrate = self.get_total_hashrate();
        let total_hashes = self.hashes_computed.load(Ordering::Relaxed);
        let shares_submitted = self.shares_submitted.load(Ordering::Relaxed);
        let shares_accepted = self.shares_accepted.load(Ordering::Relaxed);
        let shares_rejected = self.shares_rejected.load(Ordering::Relaxed);
        let acceptance_rate = if shares_submitted > 0 {
            (shares_accepted as f64 / shares_submitted as f64) * 100.0
        } else {
            0.0
        };
        let session_time = self.start_time.elapsed();
        let current_difficulty = self.thread_stats
            .iter()
            .map(|t| t.current_difficulty_target.load(Ordering::Relaxed))
            .max()
            .unwrap_or(0);

        let shares = self.recent_shares.lock().unwrap();
        let mut top_shares: Vec<u64> = shares.iter()
            .map(|s| s.difficulty)
            .collect();
        debug!("All share difficulties: {:?}", top_shares);
        top_shares.sort_by(|a, b| b.cmp(a)); // Sort descending
        let top_shares: Vec<u64> = top_shares.into_iter().take(5).collect();
        debug!("Top 5 share difficulties: {:?}", top_shares);
        let top_shares_str = if top_shares.is_empty() {
            "None".to_string()
        } else {
            top_shares.iter()
                .map(|d| Self::format_number(*d))
                .collect::<Vec<_>>()
                .join(" | ")
        };

        let avg_luck = if !shares.is_empty() {
            let total_luck: f64 = shares.iter()
                .map(|s| s.difficulty as f64 / s.target as f64)
                .sum();
            total_luck / shares.len() as f64
        } else {
            0.0
        };

        // Work efficiency: Ratio of accepted shares to expected shares based on hashrate
        let expected_shares = if current_difficulty > 0 {
            (total_hashes as f64 / current_difficulty as f64).max(1.0)
        } else {
            1.0
        };
        let work_efficiency = if expected_shares > 0.0 {
            (shares_accepted as f64 / expected_shares) * 100.0
        } else {
            0.0
        };

        let time_since_last_share = shares.back()
            .map(|s| Instant::now().duration_since(s.time))
            .unwrap_or(Duration::from_secs(0));

        let avg_share_time = if shares_submitted > 0 {
            let session_secs = session_time.as_secs_f64();
            Duration::from_secs_f64(session_secs / shares_submitted as f64)
        } else {
            Duration::from_secs(0)
        };

        let active_threads = self.get_active_thread_count();
        let share_rate = self.get_share_rate_per_minute();

        info!("📊 MINER DASHBOARD - {}", dashboard_id);
        info!("├─ Algorithm: SHA3X");
        info!("├─ Current Hashrate: {}", Self::format_hashrate(hashrate));
        info!("├─ Session Avg: {}", Self::format_hashrate(hashrate));
        info!("├─ Total Work: {} hashes", Self::format_number(total_hashes));
        info!("├─ Top 5 Shares: {}", top_shares_str);
        info!("├─ Shares: {}/{} ({:.1}% accepted)", shares_accepted, shares_submitted, acceptance_rate);
        info!("├─ Rejected Shares: {}", shares_rejected);
        info!("├─ Work Efficiency: {:.1}%", work_efficiency);
        info!("├─ Average Luck: {:.2}x", avg_luck);
        info!("├─ Share Rate: {:.2} shares/min", share_rate);
        info!("├─ Time Since Last Share: {}", Self::format_duration(time_since_last_share));
        info!("├─ Average Share Time: {}", Self::format_duration(avg_share_time));
        info!("├─ Session Time: {}", Self::format_duration(session_time));
        info!("├─ Active Threads: {}/{}", active_threads, self.thread_stats.len());
        info!("└─ Current Difficulty: {}", Self::format_number(current_difficulty));
    }
}

// Changelog:
// - v1.0.3 (2025-06-14T11:50:00Z EDT): Fixed Top 5 Shares sorting.
//   - Sorted Top 5 Shares by difficulty in descending order (largest first).
//   - Added debug logging to trace share difficulties for validation.
//   - Verified Total Work accuracy, no changes needed.
//   - Maintained dashboard metrics and work efficiency calculation.
// - v1.0.2 (2025-06-14T02:15:00Z): Enhanced dashboard display.
//   - Added Time Since Last Share, Average Share Time, Active Threads, Share Rate, Rejected Shares.
//   - Removed Best Share Difficulty.
// - v1.0.1 (2025-06-14T01:25:00Z): Added dashboard display.
// - v1.0.0 (2025-06-14T00:00:00Z): Extracted from monolithic main.rs.