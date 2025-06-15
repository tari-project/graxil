// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/miner/cpu/thread.rs
// Version: 1.0.4
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file contains the implementation of individual mining threads for the
// SHA3x miner, located in the cpu subdirectory of the miner module. It handles
// nonce iteration, hash computation, and share detection for CPU mining.
//
// Tree Location:
// - src/miner/cpu/thread.rs (mining thread logic)
// - Depends on: core, stats, tokio, rand, hex

use crate::core::{calculate_difficulty, MiningJob, sha3x::sha3x_hash_with_nonce_batch};
use crate::miner::stats::{MinerStats, ThreadStats};
use tokio::sync::mpsc::UnboundedSender as MpscSender;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;
use hex;
use rand::{rngs::ThreadRng, Rng};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Instant, Duration};
use tracing::info;

pub fn start_mining_thread(
    thread_id: usize,
    num_threads: usize,
    job_rx: BroadcastReceiver<MiningJob>,
    share_tx: MpscSender<(String, String, String, usize, u64)>,
    thread_stats: Arc<ThreadStats>,
    stats: Arc<MinerStats>,
) {
    let should_stop = Arc::new(AtomicBool::new(false));
    let should_stop_clone = Arc::clone(&should_stop);

    std::thread::spawn(move || {
        mining_thread(
            thread_id,
            num_threads,
            job_rx,
            share_tx,
            thread_stats,
            stats,
            should_stop_clone,
        );
    });
}

fn mining_thread(
    thread_id: usize,
    num_threads: usize,
    mut job_rx: BroadcastReceiver<MiningJob>,
    share_tx: MpscSender<(String, String, String, usize, u64)>,
    thread_stats: Arc<ThreadStats>,
    stats: Arc<MinerStats>,
    should_stop: Arc<AtomicBool>,
) {
    let mut rng: ThreadRng = rand::thread_rng();
    let mut current_job: Option<MiningJob> = None;
    let mut hash_count = 0u64;
    let mut last_report = Instant::now();

    loop {
        if should_stop.load(Ordering::Relaxed) {
            break;
        }

        match job_rx.try_recv() {
            Ok(job) => {
                thread_stats.current_difficulty_target.store(job.target_difficulty, Ordering::Relaxed);
                current_job = Some(job);
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {},
            Err(_) => break,
        }

        if let Some(ref job) = current_job {
            let mut nonce = rng.r#gen::<u64>();
            nonce = nonce.wrapping_add(thread_id as u64);

            // Process in batches of 4, maintaining 10,000 total iterations
            for _ in (0..10000).step_by(4) {
                let batch_results = sha3x_hash_with_nonce_batch(&job.mining_hash, nonce);
                
                for (hash, batch_nonce) in batch_results.iter() {
                    let difficulty = calculate_difficulty(&hash);
                    hash_count += 1;

                    if difficulty >= job.target_difficulty {
                        let nonce_hex_le = hex::encode(batch_nonce.to_le_bytes());
                        let nonce_hex_be = hex::encode(batch_nonce.to_be_bytes());
                        let result_hex = hex::encode(&hash);

                        thread_stats.record_share(difficulty, true);
                        stats.record_share_found(thread_id, difficulty, job.target_difficulty, true);

                        info!("💎 Thread {} found share! Difficulty: {}, Target: {}", 
                            thread_id, 
                            MinerStats::format_number(difficulty), 
                            MinerStats::format_number(job.target_difficulty));
                        
                        info!("🔍 Share details - Nonce LE: {}, Nonce BE: {}", nonce_hex_le, nonce_hex_be);
                        info!("🔍 Hash result: {}", result_hex);

                        stats.add_activity(format!(
                            "💎 Thread {} found share! Difficulty: {}",
                            thread_id,
                            MinerStats::format_number(difficulty)
                        ));

                        let _ = share_tx.send((
                            job.job_id.clone(),
                            nonce_hex_le, // Send LE nonce
                            result_hex,
                            thread_id,
                            difficulty,
                        ));

                        stats.shares_submitted.fetch_add(1, Ordering::Relaxed);
                    }
                }

                nonce = nonce.wrapping_add((4 * num_threads) as u64);
            }

            if last_report.elapsed() > Duration::from_secs(1) {
                thread_stats.update_hashrate(hash_count);
                stats.hashes_computed.fetch_add(hash_count, Ordering::Relaxed);
                stats.update_hashrate_history(stats.hashes_computed.load(Ordering::Relaxed));
                hash_count = 0;
                last_report = Instant::now();
            }
        } else {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

// Changelog:
// - v1.0.4 (2025-06-14T20:45:00Z): Added batch processing optimization.
//   - Replaced single hash calls with sha3x_hash_with_nonce_batch for 4x efficiency.
//   - Modified nonce stepping to account for batch size (4 * num_threads).
//   - Maintained 10,000 total hash iterations per loop via step_by(4).
//   - Optimized for dual Xeon 2699v3 systems (72 threads) with reduced allocations.
// - v1.0.3 (2025-06-14T02:38:00Z): Tested LE nonce for share submission.
//   - Changed share submission to use LE nonce (hex::encode(nonce.to_le_bytes())) to test compatibility with mate's pool.
//   - Fixed nonce logging to correctly display LE and BE nonces.
//   - Maintained original 10,000-nonce loop, BE nonce generation, and 1-second stats updates.
// - v1.0.2 (2025-06-14T01:20:00Z): Fixed nonce generation.
//   - Corrected rng.r#gen::<u64>() to rng.gen::<u64>().
// - v1.0.1 (2025-06-13T23:51:00Z): Restored tokio::sync::mpsc channels.
//   - Reverted share channels to tokio::sync::mpsc::UnboundedSender.
// - v1.0.0 (2025-06-14T00:00:00Z): Extracted from monolithic main.rs.