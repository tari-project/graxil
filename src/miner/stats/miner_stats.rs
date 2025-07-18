// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/miner/stats/miner_stats.rs
// Version: 1.3.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements miner-wide statistics tracking for the SHA3x miner,
// located in the stats subdirectory of the miner module. It manages shares,
// hashrate, activity logs, job tracking, and GPU monitoring for the entire miner.
//
// Tree Location:
// - src/miner/stats/miner_stats.rs (miner-wide statistics logic)
// - Depends on: std, thread_stats, gpu_info, serde, sysinfo

use super::gpu_info::GpuInfo;
use super::thread_stats::ThreadStats;
use crate::core::types::Algorithm;
use crate::pool::client::PoolClient;
use log::{debug, info};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::{Components, System};

const LOG_TARGET: &str = "tari::graxil::miner_stats";

#[allow(dead_code)] // Fields unused in non-TUI version but kept for future use
#[derive(Debug)]
pub struct ShareInfo {
    time: Instant,
    thread_id: usize,
    difficulty: u64,
    target: u64,
    accepted: bool,
}

#[derive(Serialize, Debug, Clone)] // Added Clone
pub struct JobInfo {
    pub job_id: String,
    pub block_height: u64,
    pub difficulty: u64,
    pub timestamp: u64, // seconds since miner start
}

#[derive(Serialize)]
pub struct PoolInfo {
    pub pool_address: String,
    pub is_connected: bool,
    pub latency_ms: Option<u64>,
    pub connection_attempts: u32,
    pub uptime_seconds: Option<u64>,
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub cpu_usage: f32,
    pub cpu_cores: usize,
    pub cpu_name: String,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_usage: f64,
    pub os_name: Option<String>,
    pub kernel_version: Option<String>,
    pub hostname: Option<String>,
    pub cpu_temperature: Option<f32>,
    pub max_temperature: Option<f32>,
}

#[derive(Serialize)]
pub struct WebSocketData {
    pub current_hashrate: u64,
    pub session_average: u64,
    pub accepted_shares: u64,
    pub submitted_shares: u64,
    pub rejected_shares: u64,
    pub work_efficiency: f64,
    pub average_luck: f64,
    pub uptime: u64,
    pub thread_hashrates: Vec<u64>,
    pub algorithm: String,
    pub active_threads: usize,
    pub share_rate: f64,
    pub total_work: u64,
    pub current_difficulty: u64,
    pub current_job: JobInfo,
    pub recent_jobs: Vec<JobInfo>,
    pub session_time: u64,
    pub time_since_last_share: u64,
    pub avg_share_time: f64,
    pub acceptance_rate: f64,
    pub recent_shares: Vec<WebSocketShare>,
    pub top_shares: Vec<u64>,
    pub system_info: SystemInfo,
    pub pool_info: PoolInfo,
    pub gpu_info: GpuInfo, // Added GPU info
}

#[derive(Serialize)]
pub struct WebSocketShare {
    pub thread_id: usize,
    pub difficulty: u64,
    pub target: u64,
    pub timestamp: u64, // seconds since start
    pub luck_factor: f64,
}

pub struct MinerStats {
    pub shares_submitted: AtomicU64,
    pub shares_accepted: AtomicU64,
    pub shares_rejected: AtomicU64,
    pub hashes_computed: AtomicU64,
    pub total_work_submitted: AtomicU64,
    start_time: Instant,
    pub thread_stats: Vec<Arc<ThreadStats>>,
    recent_shares: Arc<Mutex<VecDeque<ShareInfo>>>,
    recent_activity: Arc<Mutex<VecDeque<(Instant, String)>>>,
    hashrate_history: Arc<Mutex<VecDeque<(Instant, u64)>>>,
    algo: Algorithm,
    current_job: Arc<Mutex<JobInfo>>,
    recent_jobs: Arc<Mutex<VecDeque<JobInfo>>>,
    system: Arc<Mutex<System>>,
    pool_client: Option<Arc<PoolClient>>,
    gpu_info: Arc<Mutex<GpuInfo>>,         // Added GPU monitoring
    gpu_last_refresh: Arc<Mutex<Instant>>, // Track GPU refresh timing
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
            total_work_submitted: AtomicU64::new(0),
            start_time: Instant::now(),
            thread_stats,
            recent_shares: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            recent_activity: Arc::new(Mutex::new(VecDeque::with_capacity(50))),
            hashrate_history: Arc::new(Mutex::new(VecDeque::with_capacity(300))),
            algo: Algorithm::Sha3x,
            current_job: Arc::new(Mutex::new(JobInfo {
                job_id: String::from("none"),
                block_height: 0,
                difficulty: 0,
                timestamp: 0,
            })),
            recent_jobs: Arc::new(Mutex::new(VecDeque::with_capacity(5))),
            system: Arc::new(Mutex::new(System::new_all())),
            pool_client: None,
            gpu_info: Arc::new(Mutex::new(GpuInfo::detect())), // Initialize GPU detection
            gpu_last_refresh: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn set_algorithm(&mut self, algo: Algorithm) {
        debug!(target: LOG_TARGET,"Setting algorithm to {:?}", algo);
        self.algo = algo;
    }

    /// Set the pool client for connection tracking
    pub fn set_pool_client(&mut self, pool_client: Arc<PoolClient>) {
        self.pool_client = Some(pool_client);
    }

    /// Refresh GPU information if enough time has passed (every 5 seconds)
    fn refresh_gpu_info_if_needed(&self) {
        let mut last_refresh = self.gpu_last_refresh.lock().unwrap();
        if last_refresh.elapsed() >= Duration::from_secs(5) {
            let mut gpu_info = self.gpu_info.lock().unwrap();
            gpu_info.refresh();
            *last_refresh = Instant::now();

            if gpu_info.is_available() {
                debug!(target: LOG_TARGET,
                    "GPU info refreshed: {} at {}°C, {}% usage",
                    gpu_info.name,
                    gpu_info.temperature.unwrap_or(0.0),
                    gpu_info.utilization.unwrap_or(0.0)
                );
            }
        }
    }

    /// Update the current job and add it to recent jobs
    pub fn update_job(&self, job_id: String, block_height: u64, difficulty: u64) {
        let timestamp = self.start_time.elapsed().as_secs();
        let new_job = JobInfo {
            job_id,
            block_height,
            difficulty,
            timestamp,
        };

        // Update current job
        let mut current_job = self.current_job.lock().unwrap();
        *current_job = new_job.clone(); // Clone JobInfo

        // Add to recent jobs
        let mut recent_jobs = self.recent_jobs.lock().unwrap();
        recent_jobs.push_back(new_job);
        if recent_jobs.len() > 5 {
            recent_jobs.pop_front();
        }

        debug!(target: LOG_TARGET,"Updated job: {:?}", current_job);
    }

    pub fn add_activity(&self, message: String) {
        let mut activity = self.recent_activity.lock().unwrap();
        activity.push_back((Instant::now(), message));
        if activity.len() > 50 {
            activity.pop_front();
        }
    }

    pub fn record_share_found(
        &self,
        thread_id: usize,
        difficulty: u64,
        target: u64,
        accepted: bool,
    ) {
        if thread_id < self.thread_stats.len() {
            self.thread_stats[thread_id].record_share(difficulty, accepted);
        }

        self.total_work_submitted
            .fetch_add(difficulty, Ordering::Relaxed);

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

    /// Convert current miner statistics to WebSocket data format
    pub fn to_websocket_data(&self) -> WebSocketData {
        // Refresh GPU info if needed
        self.refresh_gpu_info_if_needed();

        let shares_submitted = self.shares_submitted.load(Ordering::Relaxed);
        let shares_accepted = self.shares_accepted.load(Ordering::Relaxed);
        let shares_rejected = self.shares_rejected.load(Ordering::Relaxed);
        let total_hashes = self.hashes_computed.load(Ordering::Relaxed);
        let total_work = self.total_work_submitted.load(Ordering::Relaxed);

        let current_difficulty = self
            .thread_stats
            .iter()
            .map(|t| t.current_difficulty_target.load(Ordering::Relaxed))
            .max()
            .unwrap_or(0);
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

        let shares = self.recent_shares.lock().unwrap();
        let avg_luck = if !shares.is_empty() {
            let total_luck: f64 = shares
                .iter()
                .map(|s| s.difficulty as f64 / s.target as f64)
                .sum();
            total_luck / shares.len() as f64
        } else {
            0.0
        };

        let recent_shares: Vec<WebSocketShare> = shares
            .iter()
            .rev()
            .take(20)
            .map(|s| WebSocketShare {
                thread_id: s.thread_id,
                difficulty: s.difficulty,
                target: s.target,
                timestamp: s.time.elapsed().as_secs(),
                luck_factor: s.difficulty as f64 / s.target as f64,
            })
            .collect();

        let mut top_shares: Vec<u64> = shares.iter().map(|s| s.difficulty).collect();
        top_shares.sort_by(|a, b| b.cmp(a));
        let top_shares: Vec<u64> = top_shares.into_iter().take(5).collect();

        let session_duration = self.start_time.elapsed();
        let time_since_last_share = shares
            .back()
            .map(|s| Instant::now().duration_since(s.time).as_secs())
            .unwrap_or(0);

        let avg_share_time = if shares_submitted > 0 {
            session_duration.as_secs_f64() / shares_submitted as f64
        } else {
            0.0
        };

        let acceptance_rate = if shares_submitted > 0 {
            (shares_accepted as f64 / shares_submitted as f64) * 100.0
        } else {
            0.0
        };

        let thread_hashrates: Vec<u64> = self
            .thread_stats
            .iter()
            .map(|t| t.get_hashrate() as u64)
            .collect();

        // Get job data
        let current_job = self.current_job.lock().unwrap(); // Lock the mutex
        let current_job = JobInfo {
            job_id: current_job.job_id.clone(), // Clone individual fields
            block_height: current_job.block_height,
            difficulty: current_job.difficulty,
            timestamp: current_job.timestamp,
        };
        let recent_jobs: Vec<JobInfo> = self
            .recent_jobs
            .lock()
            .unwrap()
            .iter()
            .map(|job| JobInfo {
                job_id: job.job_id.clone(),
                block_height: job.block_height,
                difficulty: job.difficulty,
                timestamp: job.timestamp,
            })
            .collect();

        // Get system info
        let mut system = self.system.lock().unwrap();
        system.refresh_all();

        // Calculate average CPU usage across all cores (proper system-wide usage)
        let cpu_usage = if !system.cpus().is_empty() {
            let total_usage: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
            total_usage / system.cpus().len() as f32
        } else {
            0.0
        };

        // Get temperature information
        let components = Components::new_with_refreshed_list();
        let (cpu_temp, max_temp) = get_temperatures(&components);

        let system_info = SystemInfo {
            cpu_usage,
            cpu_cores: system.cpus().len(),
            cpu_name: system
                .cpus()
                .first()
                .map(|cpu| cpu.brand().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            memory_total: system.total_memory(),
            memory_used: system.used_memory(),
            memory_usage: (system.used_memory() as f64 / system.total_memory() as f64) * 100.0,
            os_name: System::name(),
            kernel_version: System::kernel_version(),
            hostname: System::host_name(),
            cpu_temperature: cpu_temp,
            max_temperature: max_temp,
        };

        // Get pool connection info
        let pool_info = if let Some(ref pool_client) = self.pool_client {
            let conn_info = pool_client.get_connection_info();
            PoolInfo {
                pool_address: conn_info.display_address(),
                is_connected: conn_info.is_connected,
                latency_ms: conn_info.latency_ms(),
                connection_attempts: conn_info.connection_attempts,
                uptime_seconds: conn_info.uptime().map(|d| d.as_secs()),
            }
        } else {
            PoolInfo {
                pool_address: "Not configured".to_string(),
                is_connected: false,
                latency_ms: None,
                connection_attempts: 0,
                uptime_seconds: None,
            }
        };

        // Get GPU info
        let gpu_info = self.gpu_info.lock().unwrap().clone();

        debug!(target: LOG_TARGET,
            "WebSocket data - Thread count: {}, Hashrates: {:?}, GPU: {}",
            self.thread_stats.len(),
            thread_hashrates,
            gpu_info.name
        );

        WebSocketData {
            current_hashrate: self.get_total_hashrate() as u64,
            session_average: self.get_total_hashrate() as u64,
            accepted_shares: shares_accepted,
            submitted_shares: shares_submitted,
            rejected_shares: shares_rejected,
            work_efficiency,
            average_luck: avg_luck,
            uptime: session_duration.as_secs(),
            thread_hashrates,
            algorithm: format!("{:?}", self.algo),
            active_threads: self.get_active_thread_count(),
            share_rate: self.get_share_rate_per_minute(),
            total_work,
            current_difficulty,
            current_job,
            recent_jobs,
            session_time: session_duration.as_secs(),
            time_since_last_share,
            avg_share_time,
            acceptance_rate,
            recent_shares,
            top_shares,
            system_info,
            pool_info,
            gpu_info, // Include GPU information
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
        let total_work = self.total_work_submitted.load(Ordering::Relaxed);
        let shares_submitted = self.shares_submitted.load(Ordering::Relaxed);
        let shares_accepted = self.shares_accepted.load(Ordering::Relaxed);
        let shares_rejected = self.shares_rejected.load(Ordering::Relaxed);
        let acceptance_rate = if shares_submitted > 0 {
            (shares_accepted as f64 / shares_submitted as f64) * 100.0
        } else {
            0.0
        };
        let session_time = self.start_time.elapsed();
        let current_difficulty = self
            .thread_stats
            .iter()
            .map(|t| t.current_difficulty_target.load(Ordering::Relaxed))
            .max()
            .unwrap_or(0);

        let shares = self.recent_shares.lock().unwrap();
        let mut top_shares: Vec<u64> = shares.iter().map(|s| s.difficulty).collect();
        debug!(target: LOG_TARGET,"All share difficulties: {:?}", top_shares);
        top_shares.sort_by(|a, b| b.cmp(a));
        let top_shares: Vec<u64> = top_shares.into_iter().take(5).collect();
        debug!(target: LOG_TARGET,"Top 5 share difficulties: {:?}", top_shares);
        let top_shares_str = if top_shares.is_empty() {
            "None".to_string()
        } else {
            top_shares
                .iter()
                .map(|d| Self::format_number(*d))
                .collect::<Vec<_>>()
                .join(" | ")
        };

        let avg_luck = if !shares.is_empty() {
            let total_luck: f64 = shares
                .iter()
                .map(|s| s.difficulty as f64 / s.target as f64)
                .sum();
            total_luck / shares.len() as f64
        } else {
            0.0
        };

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

        let time_since_last_share = shares
            .back()
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

        // Display GPU information if available
        let gpu_info = self.gpu_info.lock().unwrap();
        let gpu_status = if gpu_info.is_available() {
            format!("{} ({})", gpu_info.name, gpu_info.format_utilization())
        } else {
            "No GPU detected".to_string()
        };

        info!(target: LOG_TARGET,"📊 MINER DASHBOARD - {}", dashboard_id);
        info!(target: LOG_TARGET,"├─ Algorithm: {:?}", self.algo);
        info!(target: LOG_TARGET,"├─ Current Hashrate: {}", Self::format_hashrate(hashrate));
        info!(target: LOG_TARGET,"├─ Session Avg: {}", Self::format_hashrate(hashrate));
        info!(target: LOG_TARGET,"├─ Total Work: {}", Self::format_number(total_work));
        info!(target: LOG_TARGET,"├─ Top 5 Shares: {}", top_shares_str);
        info!(target: LOG_TARGET,
            "├─ Shares: {}/{} ({:.1}% accepted)",
            shares_accepted, shares_submitted, acceptance_rate
        );
        info!(target: LOG_TARGET,"├─ Rejected Shares: {}", shares_rejected);
        info!(target: LOG_TARGET,"├─ Work Efficiency: {:.1}%", work_efficiency);
        info!(target: LOG_TARGET,"├─ Average Luck: {:.2}x", avg_luck);
        info!(target: LOG_TARGET,"├─ Share Rate: {:.2} shares/min", share_rate);
        info!(target: LOG_TARGET,
            "├─ Time Since Last Share: {}",
            Self::format_duration(time_since_last_share)
        );
        info!(target: LOG_TARGET,
            "├─ Average Share Time: {}",
            Self::format_duration(avg_share_time)
        );
        info!(target: LOG_TARGET,"├─ Session Time: {}", Self::format_duration(session_time));
        info!(target: LOG_TARGET,
            "├─ Active Threads: {}/{}",
            active_threads,
            self.thread_stats.len()
        );
        info!(target: LOG_TARGET,
            "├─ Current Difficulty: {}",
            Self::format_number(current_difficulty)
        );
        info!(target: LOG_TARGET,"└─ GPU Status: {}", gpu_status);
    }
}

/// Extract temperature information from system components
fn get_temperatures(components: &Components) -> (Option<f32>, Option<f32>) {
    let mut cpu_temp: Option<f32> = None;
    let mut max_temp: Option<f32> = None;
    let mut highest_temp = 0.0f32;

    for component in components {
        if let Some(temp) = component.temperature() {
            let label = component.label().to_lowercase();

            // Look for CPU-related temperature sensors
            if cpu_temp.is_none()
                && (label.contains("cpu")
                    || label.contains("core")
                    || label.contains("package")
                    || label.contains("processor"))
            {
                cpu_temp = Some(temp);
            }

            // Track highest temperature across all components
            if temp > highest_temp {
                highest_temp = temp;
                max_temp = Some(temp);
            }
        }
    }

    (cpu_temp, max_temp)
}

// Changelog:
// - v1.3.0 (2025-06-24): Added GPU monitoring integration
//   - Added gpu_info module dependency for NVIDIA GPU detection
//   - Added gpu_info and gpu_last_refresh fields to MinerStats struct
//   - Added refresh_gpu_info_if_needed method for periodic GPU data updates (every 5 seconds)
//   - Added gpu_info field to WebSocketData struct for dashboard integration
//   - Updated to_websocket_data to include GPU information in dashboard data
//   - Updated display_dashboard to show GPU status in console output
//   - GPU detection uses nvidia-smi command with graceful fallback when unavailable
//   - Provides GPU name, utilization, temperature, power, and memory usage
//   - Maintains existing functionality while adding comprehensive GPU monitoring
// - v1.2.0 (2025-06-24): Added pool connection tracking integration
//   - Added PoolInfo struct for tracking pool connection status, latency, and statistics
//   - Added pool_client field to MinerStats for connection monitoring
//   - Added set_pool_client method to register pool client for tracking
//   - Updated WebSocketData to include pool_info field
//   - Enhanced to_websocket_data to collect pool connection information
//   - Provides real-time pool connectivity status and performance metrics for dashboard
// - v1.1.0 (2025-06-23): Added system information monitoring.
//   - Added sysinfo crate integration for real-time system monitoring.
//   - Added SystemInfo struct to track CPU usage, cores, name, and memory usage.
//   - Added system field to MinerStats struct for system monitoring.
//   - Updated WebSocketData to include system_info field.
//   - Updated to_websocket_data to collect and send system information.
//   - System info refreshed on every WebSocket update (1 second intervals).
// - v1.0.9 (2025-06-23): Added job tracking.
//   - Added JobInfo struct to store job_id, block_height, difficulty, and timestamp.
//   - Added current_job and recent_jobs fields to MinerStats to track current and last 5 jobs.
//   - Added update_job method to update job information.
//   - Updated WebSocketData to include current_job and recent_jobs.
//   - Updated to_websocket_data to send job data to dashboard.
//   - Fixed Clone trait implementation for JobInfo and corrected job data cloning in to_websocket_data.
// - v1.0.8 (2025-06-22): Fixed Total Work calculation.
//   - Added total_work_submitted AtomicU64 to track sum of all share difficulties.
//   - Updated record_share_found to add each share's difficulty to total_work_submitted.
//   - Changed WebSocketData to send total_work instead of total_hashes.
//   - Updated display_dashboard to show sum of difficulties as Total Work.
//   - Fixed recent_shares timestamp to use index (0 = most recent) for clearer display.
// - v1.0.7 (2025-06-22): Added WebSocket support for real-time dashboard.
//   - Added serde::Serialize import for JSON serialization support.
//   - Added WebSocketData struct for structured dashboard data transmission.
//   - Added to_websocket_data method for converting stats to WebSocket format.
//   - Maintained all existing functionality and calculation logic.
//   - Compatible with web_server.rs for real-time mining dashboard.
// - v1.0.6 (2025-06-17, 08:50 AM EDT): Fixed algorithm display in dashboard.
//   - Updated display_dashboard to use self.algo dynamically with {:?} formatting instead of hardcoding "SHA3X".
//   - Ensured compatibility with miner.rs v1.2.5 and types.rs v1.0.4.
//   - Maintained all existing stats and display functionality.
// - v1.0.5 (2025-06-16, 02:53 PM EDT): Quick fix for SHA3x benchmark compatibility.
//   - Added set_algorithm method to resolve compilation error in miner.rs (v1.2.1).
//   - Retained v1.0.3 logic, using current_difficulty_target directly as difficulty to fix display issue (~1–2 vs. ~1M).
//   - Added algo field and default to Algorithm::Sha3x for SHA3x-specific operation.
//   - Maintained all other stats and display functionality.
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
