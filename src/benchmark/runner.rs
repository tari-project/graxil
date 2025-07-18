// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/benchmark/runner.rs
// Version: 1.0.29
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements the benchmark execution engine for testing SHA3x and SHA-256 mining
// performance without pool connectivity. It coordinates benchmark threads and
// collects performance metrics for optimization analysis.

use crate::Result;
use crate::benchmark::jobs::{
    calculate_difficulty_from_nbits, get_job_by_difficulty_and_algo, get_max_target,
};
use crate::benchmark::profiler::ProfilerData;
use crate::core::difficulty::{U256, bits_to_target};
use crate::core::types::{Algorithm, BenchmarkResult, MiningJob};
use crate::miner::stats::{MinerStats, ThreadStats};
use hex;
use log::{debug, info};
use num_cpus;
use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

const LOG_TARGET: &str = "tari::graxil::runner";

/// Configuration for benchmark execution
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub thread_count: usize,
    pub duration: Duration,
    pub target_difficulty: f64,
    pub algorithm: Algorithm,
    pub enable_profiling: bool,
    pub report_interval: Duration,
}

/// Main benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    stats: Arc<MinerStats>,
    profiler: Arc<ProfilerData>,
}

impl BenchmarkRunner {
    pub fn new(threads: usize, duration_secs: u64, difficulty: f64, algorithm: Algorithm) -> Self {
        let actual_threads = if threads == 0 {
            num_cpus::get()
        } else {
            threads
        };
        let config = BenchmarkConfig {
            thread_count: actual_threads,
            duration: Duration::from_secs(duration_secs),
            target_difficulty: difficulty,
            algorithm,
            enable_profiling: true,
            report_interval: Duration::from_secs(5),
        };
        let mut stats = MinerStats::new(actual_threads);
        stats.set_algorithm(algorithm);
        Self {
            config,
            stats: Arc::new(stats),
            profiler: Arc::new(ProfilerData::new()),
        }
    }

    pub async fn run(&self) -> Result<BenchmarkResult> {
        info!(target: LOG_TARGET,
            "🧪 Starting benchmark with {} threads, algo: {:?}",
            self.config.thread_count, self.config.algorithm
        );
        let benchmark_job =
            get_job_by_difficulty_and_algo(self.config.target_difficulty, self.config.algorithm);
        info!(target: LOG_TARGET,"📋 Using benchmark job: {}", benchmark_job.description);

        let (job_tx, _) = broadcast::channel::<MiningJob>(16);
        let (share_tx, share_rx): (Sender<BenchmarkShare>, Receiver<BenchmarkShare>) =
            mpsc::channel();
        let should_stop = Arc::new(AtomicBool::new(false));
        let total_hashes = Arc::new(AtomicU64::new(0));
        let shares_found = Arc::new(AtomicU64::new(0));
        let seen_nonces = Arc::new(Mutex::new(HashSet::new()));
        let share_tx = Arc::new(Mutex::new(share_tx));

        let mut thread_handles = Vec::new();
        let thread_count = self.config.thread_count;
        for thread_id in 0..thread_count {
            let _job_rx = job_tx.subscribe();
            let share_tx = Arc::clone(&share_tx);
            let should_stop = Arc::clone(&should_stop);
            let total_hashes = Arc::clone(&total_hashes);
            let thread_stats = Arc::clone(&self.stats.thread_stats[thread_id]);
            let benchmark_job = benchmark_job.clone();
            let seen_nonces = Arc::clone(&seen_nonces);

            let handle = thread::spawn(move || {
                benchmark_thread(
                    thread_id,
                    thread_count,
                    benchmark_job.mining_job,
                    should_stop,
                    total_hashes,
                    share_tx,
                    thread_stats,
                    seen_nonces,
                );
                debug!(target: LOG_TARGET,"Thread {}: Terminated", thread_id);
            });
            thread_handles.push(handle);
        }

        let shares_found_collector = Arc::clone(&shares_found);
        let should_stop_collector = Arc::clone(&should_stop);
        let share_handle = thread::spawn(move || {
            loop {
                match share_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(share) => {
                        shares_found_collector.fetch_add(1, Ordering::Relaxed);
                        debug!(target: LOG_TARGET,
                            "💎 Benchmark share found: difficulty {}, thread {}",
                            share.difficulty, share.thread_id
                        );
                    }
                    Err(_) => {
                        if should_stop_collector.load(Ordering::Relaxed) {
                            break;
                        }
                        // Continue waiting if benchmark is still running
                    }
                }
            }
            debug!(target: LOG_TARGET,"Share collector thread stopping");
        });

        let total_hashes_reporter = Arc::clone(&total_hashes);
        let shares_found_reporter = Arc::clone(&shares_found);
        let should_stop_reporter = Arc::clone(&should_stop);
        let report_interval = self.config.report_interval;

        let progress_handle = thread::spawn(move || {
            let mut last_hashes = 0u64;
            let mut last_time = Instant::now();
            while !should_stop_reporter.load(Ordering::Relaxed) {
                thread::sleep(report_interval);
                if should_stop_reporter.load(Ordering::Relaxed) {
                    break;
                }
                let current_hashes = total_hashes_reporter.load(Ordering::Relaxed);
                let current_shares = shares_found_reporter.load(Ordering::Relaxed);
                let now = Instant::now();
                let hashes_delta = current_hashes - last_hashes;
                let time_delta = now.duration_since(last_time).as_secs_f64();
                if time_delta > 0.0 {
                    let hashrate = hashes_delta as f64 / time_delta;
                    info!(target: LOG_TARGET,
                        "📊 Progress: {:.2} MH/s | Total: {} MH | Shares: {}",
                        hashrate / 1_000_000.0,
                        current_hashes / 1_000_000,
                        current_shares
                    );
                }
                last_hashes = current_hashes;
                last_time = now;
            }
            debug!(target: LOG_TARGET,"Progress reporter thread stopping");
        });

        let _ = job_tx.send(benchmark_job.mining_job);
        let start_time = Instant::now();
        while start_time.elapsed() < self.config.duration {
            thread::sleep(Duration::from_millis(100));
        }

        should_stop.store(true, Ordering::Relaxed);
        debug!(target: LOG_TARGET,"Signaled threads to stop");
        thread::sleep(Duration::from_millis(500));

        info!(target: LOG_TARGET,"🛑 Stopping benchmark threads...");
        for (i, handle) in thread_handles.into_iter().enumerate() {
            if let Err(e) = handle.join() {
                debug!(target: LOG_TARGET,"Thread {} failed to join: {:?}", i, e);
            } else {
                debug!(target: LOG_TARGET,"Thread {} joined successfully", i);
            }
        }

        let share_tx = share_tx.lock().unwrap();
        drop(share_tx);
        if let Err(e) = share_handle.join() {
            debug!(target: LOG_TARGET,"Share collector thread failed to join: {:?}", e);
        }
        if let Err(e) = progress_handle.join() {
            debug!(target: LOG_TARGET,"Progress reporter thread failed to join: {:?}", e);
        }

        info!(target: LOG_TARGET,"✅ All threads stopped");

        let end_time = Instant::now();
        let actual_duration = end_time.duration_since(start_time);
        let final_hashes = total_hashes.load(Ordering::Relaxed);
        let final_shares = shares_found.load(Ordering::Relaxed);
        let average_hashrate = final_hashes as f64 / actual_duration.as_secs_f64();
        let peak_hashrate = self.calculate_peak_hashrate();

        Ok(BenchmarkResult {
            total_hashes: final_hashes,
            duration: actual_duration,
            hashrate: average_hashrate,
            peak_hashrate,
            shares_found: final_shares,
            thread_count: self.config.thread_count,
            allocations: self.profiler.get_allocation_count(),
        })
    }

    fn calculate_peak_hashrate(&self) -> f64 {
        let mut peak = 0.0;
        for thread_stat in &self.stats.thread_stats {
            let thread_peak = thread_stat.peak_hashrate.load(Ordering::Relaxed) as f64;
            peak += thread_peak;
        }
        peak
    }
}

#[derive(Debug, Clone)]
struct BenchmarkShare {
    difficulty: f64,
    thread_id: usize,
}

fn benchmark_thread(
    thread_id: usize,
    num_threads: usize,
    job: MiningJob,
    should_stop: Arc<AtomicBool>,
    total_hashes: Arc<AtomicU64>,
    share_tx: Arc<Mutex<Sender<BenchmarkShare>>>,
    thread_stats: Arc<ThreadStats>,
    seen_nonces: Arc<Mutex<HashSet<u32>>>,
) {
    use crate::core::{sha3x::sha3x_hash_with_nonce_batch, sha256::sha256d_hash_with_nonce_batch};
    use rand::{Rng, rngs::ThreadRng};
    use sha2::{Digest, Sha256};

    let mut rng: ThreadRng = rand::thread_rng();
    let mut local_hash_count = 0u64;
    let mut last_report = Instant::now();

    match job.algo {
        Algorithm::Sha3x => {
            let mut nonce: u64 = rng.r#gen();
            nonce = nonce.wrapping_add(thread_id as u64);
            let target_difficulty = job.target_difficulty as f64;

            // Get the correct max target for SHA3x
            let max_target = get_max_target(Algorithm::Sha3x);

            // Log the max target once per thread
            if thread_id == 0 {
                debug!(target: LOG_TARGET,"Thread 0: SHA3x max_target: {:064x}", max_target);
                debug!(target: LOG_TARGET,"Thread 0: Target difficulty: {}", target_difficulty);
            }

            while !should_stop.load(Ordering::Relaxed) {
                for _ in (0..10000).step_by(4) {
                    if should_stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let batch_results = sha3x_hash_with_nonce_batch(&job.mining_hash, nonce);
                    for (hash, _batch_nonce) in batch_results.iter() {
                        let hash_u256 = U256::from_big_endian(hash);
                        let difficulty = if !hash_u256.is_zero() {
                            (max_target / hash_u256).low_u64() as f64
                        } else {
                            0.0
                        };
                        local_hash_count += 1;

                        // Log first few hash difficulties for debugging
                        if thread_id == 0 && local_hash_count <= 10 {
                            debug!(target: LOG_TARGET,
                                "Thread 0: Hash {}: difficulty = {}, target = {}",
                                local_hash_count, difficulty, target_difficulty
                            );
                        }

                        if difficulty >= target_difficulty {
                            let share = BenchmarkShare {
                                difficulty,
                                thread_id,
                            };
                            thread_stats.record_share(difficulty as u64, true);
                            if let Ok(tx) = share_tx.lock() {
                                let _ = tx.send(share);
                            }
                            info!(target: LOG_TARGET,
                                "🎯 Thread {}: Found SHA3x share! Difficulty: {}",
                                thread_id, difficulty
                            );
                        }
                    }
                    nonce = nonce.wrapping_add((4 * num_threads) as u64);
                }
                if last_report.elapsed() > Duration::from_secs(1) {
                    thread_stats.update_hashrate(local_hash_count);
                    total_hashes.fetch_add(local_hash_count, Ordering::Relaxed);
                    local_hash_count = 0;
                    last_report = Instant::now();
                }
            }
        }
        Algorithm::Sha256 => {
            let nonce: u32 = rng.r#gen();
            let nonce_start = nonce.wrapping_add(thread_id as u32);
            let mut header = [0u8; 80];
            if job.mining_hash.len() >= 80 {
                header.copy_from_slice(&job.mining_hash[..80]);
            } else {
                debug!(target: LOG_TARGET,
                    "Thread {}: Invalid mining_hash length for SHA-256: {}, stopping thread",
                    thread_id,
                    job.mining_hash.len()
                );
                return;
            }
            static HEADER_LOGGED: [std::sync::atomic::AtomicBool; 36] = [
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
                std::sync::atomic::AtomicBool::new(false),
            ];
            if !HEADER_LOGGED[thread_id].swap(true, Ordering::Relaxed) {
                debug!(target: LOG_TARGET,
                    "Thread {}: Full header: {}",
                    thread_id,
                    hex::encode(&header)
                );
            }
            let (target_value, target_difficulty) = if let Some(nbits) = job.nbits {
                let target = bits_to_target(nbits);
                let difficulty = calculate_difficulty_from_nbits(nbits);
                debug!(target: LOG_TARGET,
                    "Thread {}: SHA-256 nbits: {:08x}, target: {:064x}, difficulty: {:.10}",
                    thread_id, nbits, target, difficulty
                );
                if thread_id == 0 {
                    let target_bytes = target.to_big_endian();
                    debug!(target: LOG_TARGET,"Thread 0: Full target: {}", hex::encode(&target_bytes));
                }
                (target, difficulty)
            } else {
                debug!(target: LOG_TARGET,
                    "Thread {}: No nbits provided for SHA-256 job, stopping thread",
                    thread_id
                );
                return;
            };
            let mut shares_found_by_thread = 0u64;
            let mut logged_hashes = 0;
            let mut nonce = nonce_start;
            while !should_stop.load(Ordering::Relaxed) {
                for _ in (0..10000).step_by(4) {
                    if should_stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let batch_results = sha256d_hash_with_nonce_batch(&header, nonce);
                    for (hash, batch_nonce) in batch_results.iter() {
                        local_hash_count += 1;
                        let hash_u256 = U256::from_big_endian(hash);
                        if thread_id == 0 && logged_hashes < 10 {
                            debug!(target: LOG_TARGET,"Thread 0: Hash bytes: {}", hex::encode(hash));
                            debug!(target: LOG_TARGET,
                                "Thread 0: U256 hash: {:064x}, Nonce: {:08x}, Target: {:064x}",
                                hash_u256, batch_nonce, target_value
                            );
                            let diff = if target_value > hash_u256 {
                                target_value - hash_u256
                            } else {
                                hash_u256 - target_value
                            };
                            debug!(target: LOG_TARGET,"Thread 0: Target - Hash difference: {:064x}", diff);
                            if logged_hashes == 0 {
                                let mut hasher = Sha256::new();
                                hasher.update(&header);
                                let first_hash = hasher.finalize();
                                debug!(target: LOG_TARGET,"Thread 0: First SHA-256: {}", hex::encode(&first_hash));
                            }
                            logged_hashes += 1;
                        }
                        if hash_u256 <= target_value {
                            let mut nonces = seen_nonces.lock().unwrap();
                            if !nonces.contains(batch_nonce) {
                                nonces.insert(*batch_nonce);
                                let difficulty = target_difficulty;
                                let share = BenchmarkShare {
                                    difficulty,
                                    thread_id,
                                };
                                thread_stats.record_share(difficulty as u64, true);
                                if let Ok(tx) = share_tx.lock() {
                                    let _ = tx.send(share);
                                }
                                shares_found_by_thread += 1;
                                debug!(target: LOG_TARGET,
                                    "🎯 Thread {}: Share attempt #{}! Difficulty: {:.10}, Hash: {}, Nonce: {}, Target: {:064x}",
                                    thread_id,
                                    shares_found_by_thread,
                                    difficulty,
                                    hex::encode(&hash[..8]),
                                    batch_nonce,
                                    target_value
                                );
                            }
                        }
                    }
                    nonce = nonce.wrapping_add((4 * num_threads) as u32);
                }
                if last_report.elapsed() > Duration::from_secs(1) {
                    thread_stats.update_hashrate(local_hash_count);
                    total_hashes.fetch_add(local_hash_count, Ordering::Relaxed);
                    local_hash_count = 0;
                    last_report = Instant::now();
                }
            }
        }
    }
    thread_stats.update_hashrate(local_hash_count);
    total_hashes.fetch_add(local_hash_count, Ordering::Relaxed);
}

// Changelog:
// - v1.0.29 (2025-06-23): Fixed benchmark duration logic.
//   - Removed confusing duration multiplier that was extending high-difficulty benchmarks by 10x
//   - Now benchmark duration matches exactly what user specifies (30 seconds = 30 seconds)
//   - Simplified BenchmarkConfig construction for clearer behavior
//   - No functional changes to benchmark logic, just fixed timing
// - v1.0.28 (2025-06-17): Fixed SHA3x share validation and share collector thread.
//   - Added get_max_target import from jobs.rs to use algorithm-specific max targets.
//   - Fixed SHA3x to use full 256-bit max target (all 0xFF) instead of Bitcoin's max target.
//   - Fixed share collector thread to continue running until benchmark completes.
//   - Added debug logging for max target and difficulty calculations.
//   - Compatible with main.rs v1.0.7, jobs.rs v1.0.14, types.rs v1.0.5, sha256.rs v1.0.1, difficulty.rs v1.2.4.
