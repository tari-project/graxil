// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/benchmark/runner.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements the benchmark execution engine for testing SHA3x mining
// performance without pool connectivity. It coordinates benchmark threads and
// collects performance metrics for optimization analysis.
//
// Tree Location:
// - src/benchmark/runner.rs (benchmark execution engine)
// - Depends on: core, miner/stats, benchmark/jobs

use crate::core::types::{BenchmarkResult, MiningJob};
use crate::benchmark::jobs::{get_job_by_difficulty};
use crate::benchmark::profiler::ProfilerData;
use crate::miner::stats::{MinerStats, ThreadStats};
use crate::Result;
use num_cpus;
use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};
use std::thread;
use tokio::sync::broadcast;
use tracing::{info, debug};

/// Configuration for benchmark execution
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of threads to use (0 = auto-detect)
    pub thread_count: usize,
    
    /// Duration to run benchmark
    pub duration: Duration,
    
    /// Target difficulty for share finding
    pub target_difficulty: u64,
    
    /// Enable detailed profiling
    pub enable_profiling: bool,
    
    /// Report interval for progress updates
    pub report_interval: Duration,
}

/// Main benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    stats: Arc<MinerStats>,
    profiler: Arc<ProfilerData>,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new(threads: usize, duration_secs: u64, difficulty: u64) -> Self {
        let actual_threads = if threads == 0 {
            num_cpus::get()
        } else {
            threads
        };

        let config = BenchmarkConfig {
            thread_count: actual_threads,
            duration: Duration::from_secs(duration_secs),
            target_difficulty: difficulty,
            enable_profiling: true,
            report_interval: Duration::from_secs(5),
        };

        Self {
            config,
            stats: Arc::new(MinerStats::new(actual_threads)),
            profiler: Arc::new(ProfilerData::new()),
        }
    }

    /// Run the benchmark and return results
    pub async fn run(&self) -> Result<BenchmarkResult> {
        info!("🧪 Starting benchmark with {} threads", self.config.thread_count);
        
        // Create benchmark job
        let benchmark_job = get_job_by_difficulty(self.config.target_difficulty);
        info!("📋 Using benchmark job: {}", benchmark_job.description);

        // Setup communication channels
        let (job_tx, _) = broadcast::channel::<MiningJob>(16);
        let (share_tx, share_rx): (Sender<BenchmarkShare>, Receiver<BenchmarkShare>) = mpsc::channel();

        // Shared state
        let should_stop = Arc::new(AtomicBool::new(false));
        let total_hashes = Arc::new(AtomicU64::new(0));
        let shares_found = Arc::new(AtomicU64::new(0));

        // Start benchmark threads
        let mut thread_handles = Vec::new();
        let thread_count = self.config.thread_count;
        for thread_id in 0..thread_count {
            let _job_rx = job_tx.subscribe();
            let share_tx = share_tx.clone();
            let should_stop = Arc::clone(&should_stop);
            let total_hashes = Arc::clone(&total_hashes);
            let thread_stats = Arc::clone(&self.stats.thread_stats[thread_id]);
            let benchmark_job = benchmark_job.clone();

            let handle = thread::spawn(move || {
                benchmark_thread(
                    thread_id,
                    thread_count,
                    benchmark_job.mining_job,
                    should_stop,
                    total_hashes,
                    share_tx,
                    thread_stats,
                );
            });
            thread_handles.push(handle);
        }

        // Start share collector
        let shares_found_collector = Arc::clone(&shares_found);
        let share_handle = thread::spawn(move || {
            while let Ok(share) = share_rx.recv() {
                shares_found_collector.fetch_add(1, Ordering::Relaxed);
                debug!("💎 Benchmark share found: difficulty {}, thread {}", 
                    share.difficulty, share.thread_id);
            }
        });

        // Start progress reporter
        let _stats_clone = Arc::clone(&self.stats);
        let total_hashes_reporter = Arc::clone(&total_hashes);
        let shares_found_reporter = Arc::clone(&shares_found);
        let should_stop_reporter = Arc::clone(&should_stop);
        let report_interval = self.config.report_interval;

        let progress_handle = thread::spawn(move || {
            let mut last_hashes = 0u64;
            let mut last_time = Instant::now();

            while !should_stop_reporter.load(Ordering::Relaxed) {
                thread::sleep(report_interval);
                
                // Check again after sleep to exit quickly
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
                    info!("📊 Progress: {:.2} MH/s | Total: {} MH | Shares: {}", 
                        hashrate / 1_000_000.0,
                        current_hashes / 1_000_000,
                        current_shares
                    );
                }
                
                last_hashes = current_hashes;
                last_time = now;
            }
            debug!("Progress reporter thread stopping");
        });

        // Send job to all threads
        let _ = job_tx.send(benchmark_job.mining_job);

        // Run for specified duration with proper thread coordination
        let start_time = Instant::now();
        
        // Wait for duration while allowing progress reports
        while start_time.elapsed() < self.config.duration {
            std::thread::sleep(Duration::from_millis(100));
        }

        // Stop all threads
        should_stop.store(true, Ordering::Relaxed);
        
        // Give threads a moment to finish
        std::thread::sleep(Duration::from_millis(500));
        
        // Wait for threads to finish with timeout
        info!("🛑 Stopping benchmark threads...");
        for handle in thread_handles {
            let _ = handle.join();
        }
        
        // Stop and wait for share collector
        drop(share_tx); // Close the channel to stop share collector
        let _ = share_handle.join();
        
        // Wait for progress reporter to finish
        let _ = progress_handle.join();
        
        info!("✅ All threads stopped");

        // Calculate final results
        let end_time = Instant::now();
        let actual_duration = end_time - start_time;
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

    /// Calculate peak hashrate from thread statistics
    fn calculate_peak_hashrate(&self) -> f64 {
        let mut peak = 0.0;
        for thread_stat in &self.stats.thread_stats {
            let thread_peak = thread_stat.peak_hashrate.load(Ordering::Relaxed) as f64;
            peak += thread_peak;
        }
        peak
    }
}

/// Benchmark-specific share structure
#[derive(Debug, Clone)]
struct BenchmarkShare {
    difficulty: u64,
    thread_id: usize,
}

/// Benchmark mining thread function
fn benchmark_thread(
    thread_id: usize,
    num_threads: usize,
    job: MiningJob,
    should_stop: Arc<AtomicBool>,
    total_hashes: Arc<AtomicU64>,
    share_tx: Sender<BenchmarkShare>,
    thread_stats: Arc<ThreadStats>,
) {
    use crate::core::{calculate_difficulty, sha3x::sha3x_hash_with_nonce_batch};
    use rand::{rngs::ThreadRng, Rng};

    let mut rng: ThreadRng = rand::thread_rng();
    let mut local_hash_count = 0u64;
    let mut last_report = Instant::now();

    // Start with random nonce for this thread
    let mut nonce = rng.r#gen::<u64>();
    nonce = nonce.wrapping_add(thread_id as u64);

    while !should_stop.load(Ordering::Relaxed) {
        // Process in batches of 4 for efficiency
        for _ in (0..10000).step_by(4) {
            let batch_results = sha3x_hash_with_nonce_batch(&job.mining_hash, nonce);
            
            for (hash, _batch_nonce) in batch_results.iter() {
                let difficulty = calculate_difficulty(hash);
                local_hash_count += 1;

                // Check if we found a share
                if difficulty >= job.target_difficulty {
                    let share = BenchmarkShare {
                        difficulty,
                        thread_id,
                    };
                    
                    thread_stats.record_share(difficulty, true);
                    let _ = share_tx.send(share);
                }
            }

            nonce = nonce.wrapping_add((4 * num_threads) as u64);

            // Early exit check
            if should_stop.load(Ordering::Relaxed) {
                break;
            }
        }

        // Update statistics every second
        if last_report.elapsed() > Duration::from_secs(1) {
            thread_stats.update_hashrate(local_hash_count);
            total_hashes.fetch_add(local_hash_count, Ordering::Relaxed);
            local_hash_count = 0;
            last_report = Instant::now();
        }
    }

    // Final update
    thread_stats.update_hashrate(local_hash_count);
    total_hashes.fetch_add(local_hash_count, Ordering::Relaxed);
}

// Changelog:
// - v1.0.0 (2025-06-14): Initial benchmark runner implementation.
//   - Purpose: Provides isolated mining performance testing without pool
//     connectivity, enabling optimization validation and performance profiling.
//   - Features: Configurable thread count, duration, and difficulty targeting,
//     with real-time progress reporting and comprehensive result collection.
//   - Note: Uses batch processing by default and provides detailed metrics
//     for analyzing the effectiveness of mining optimizations.