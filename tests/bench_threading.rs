// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: tests/bench_threading.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file contains thread scaling performance tests for the SHA3x miner.
// It tests different thread counts to find optimal scaling for various
// hardware configurations and identifies performance bottlenecks.
//
// Tree Location:
// - tests/bench_threading.rs (thread scaling performance tests)
// - Depends on: core/sha3x, num_cpus

use sha3x_miner::core::sha3x::sha3x_hash_with_nonce_batch;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use num_cpus;

#[test]
fn test_thread_scaling() {
    println!("🧪 Testing Thread Scaling Performance");
    println!("=====================================");
    
    let cpu_count = num_cpus::get();
    println!("📊 Detected {} CPU threads", cpu_count);
    
    let test_header = vec![
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
    ];
    
    // Test different thread counts
    let thread_counts = vec![1, 2, 4, 8, 16, 32, 64, cpu_count];
    let test_duration = Duration::from_secs(3); // Short test duration
    
    println!();
    println!("🔄 Testing thread counts: {:?}", thread_counts);
    println!();
    
    let mut results = Vec::new();
    
    for &thread_count in &thread_counts {
        if thread_count > cpu_count * 2 {
            continue; // Skip excessive thread counts
        }
        
        println!("📊 Testing {} threads...", thread_count);
        
        let total_hashes = Arc::new(AtomicU64::new(0));
        let should_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        
        let start_time = Instant::now();
        
        // Spawn worker threads
        let mut handles = Vec::new();
        for thread_id in 0..thread_count {
            let header = test_header.clone();
            let hashes = Arc::clone(&total_hashes);
            let stop_flag = Arc::clone(&should_stop);
            
            let handle = thread::spawn(move || {
                let mut local_hashes = 0u64;
                let mut nonce = (thread_id as u64) * 1000000; // Spread nonces
                
                while !stop_flag.load(Ordering::Relaxed) {
                    // Process in batches for efficiency
                    let _batch_results = sha3x_hash_with_nonce_batch(&header, nonce);
                    local_hashes += 4;
                    nonce = nonce.wrapping_add(thread_count as u64 * 4);
                    
                    // Check stop condition periodically
                    if local_hashes % 1000 == 0 && stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                }
                
                hashes.fetch_add(local_hashes, Ordering::Relaxed);
            });
            
            handles.push(handle);
        }
        
        // Let threads run for test duration
        thread::sleep(test_duration);
        
        // Signal threads to stop
        should_stop.store(true, Ordering::Relaxed);
        
        // Wait for all threads to finish
        for handle in handles {
            let _ = handle.join();
        }
        
        let actual_duration = start_time.elapsed();
        let final_hashes = total_hashes.load(Ordering::Relaxed);
        let hashrate = final_hashes as f64 / actual_duration.as_secs_f64();
        let per_thread_rate = hashrate / thread_count as f64;
        
        results.push((thread_count, hashrate, per_thread_rate));
        
        println!("  {} threads: {:.1} H/s ({:.1} H/s per thread)", 
            thread_count, hashrate, per_thread_rate);
    }
    
    // Analyze scaling efficiency
    println!();
    println!("📈 THREAD SCALING ANALYSIS:");
    println!("  Threads | Total H/s  | Per-Thread | Efficiency");
    println!("  --------|------------|------------|----------");
    
    let single_thread_rate = results[0].2; // Per-thread rate for 1 thread
    
    for (threads, total_rate, per_thread_rate) in &results {
        let efficiency = (per_thread_rate / single_thread_rate) * 100.0;
        println!("  {:7} | {:8.1}  | {:8.1}   | {:6.1}%", 
            threads, total_rate, per_thread_rate, efficiency);
    }
    
    // Find optimal thread count (best total hashrate)
    let optimal = results.iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap();
    
    println!();
    println!("🏆 Optimal configuration: {} threads ({:.1} H/s total)", 
        optimal.0, optimal.1);
    
    // Check for reasonable scaling
    let last_result = results.last().unwrap();
    assert!(last_result.1 > results[0].1, 
        "Multi-threading should provide some performance benefit");
    
    println!();
    println!("💡 Scaling Insights:");
    if optimal.0 == cpu_count {
        println!("  ✅ Optimal thread count matches CPU count");
    } else if optimal.0 < cpu_count {
        println!("  ⚠️  Optimal thread count below CPU count (possible bottleneck)");
    } else {
        println!("  📈 Optimal thread count exceeds CPU count (good hyperthreading)");
    }
    
    println!("  - Test your specific hardware with: --benchmark --threads {}", optimal.0);
    println!("  - Consider system load when choosing thread count");
    println!("  - Monitor CPU temperature under full load");
}

#[test]
fn test_thread_efficiency_limits() {
    println!("🧪 Testing Thread Efficiency Limits");
    println!("====================================");
    
    let cpu_count = num_cpus::get();
    let test_header = vec![0u8; 32];
    let test_duration = Duration::from_secs(2);
    
    // Test extreme thread counts to find efficiency drop-off
    let extreme_counts = vec![
        cpu_count / 2,     // Half threads
        cpu_count,         // Equal to CPUs
        cpu_count * 2,     // Double (hyperthreading)
        cpu_count * 4,     // Quad (oversubscription)
    ];
    
    println!("🔄 Testing thread efficiency at extreme counts...");
    println!();
    
    let mut efficiency_results = Vec::new();
    
    for &thread_count in &extreme_counts {
        if thread_count == 0 || thread_count > 256 {
            continue; // Skip invalid counts
        }
        
        println!("📊 Testing {} threads ({}x CPU count)...", 
            thread_count, thread_count as f32 / cpu_count as f32);
        
        let total_hashes = Arc::new(AtomicU64::new(0));
        let should_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        
        let start_time = Instant::now();
        
        // Spawn threads with contention
        let mut handles = Vec::new();
        for thread_id in 0..thread_count {
            let header = test_header.clone();
            let hashes = Arc::clone(&total_hashes);
            let stop_flag = Arc::clone(&should_stop);
            
            let handle = thread::spawn(move || {
                let mut local_hashes = 0u64;
                let mut nonce = thread_id as u64 * 10000;
                
                while !stop_flag.load(Ordering::Relaxed) {
                    let _batch_results = sha3x_hash_with_nonce_batch(&header, nonce);
                    local_hashes += 4;
                    nonce = nonce.wrapping_add(thread_count as u64 * 4);
                    
                    if local_hashes % 100 == 0 && stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                }
                
                hashes.fetch_add(local_hashes, Ordering::Relaxed);
            });
            
            handles.push(handle);
        }
        
        thread::sleep(test_duration);
        should_stop.store(true, Ordering::Relaxed);
        
        for handle in handles {
            let _ = handle.join();
        }
        
        let duration = start_time.elapsed();
        let final_hashes = total_hashes.load(Ordering::Relaxed);
        let total_rate = final_hashes as f64 / duration.as_secs_f64();
        let per_thread_rate = total_rate / thread_count as f64;
        
        efficiency_results.push((thread_count, total_rate, per_thread_rate));
        
        println!("  Result: {:.1} H/s total, {:.1} H/s per thread", 
            total_rate, per_thread_rate);
    }
    
    println!();
    println!("📊 EFFICIENCY ANALYSIS:");
    println!("  Thread Count | CPU Ratio | Total Rate | Per-Thread | Efficiency Loss");
    println!("  -------------|-----------|------------|------------|----------------");
    
    let baseline_per_thread = efficiency_results[0].2;
    
    for (threads, total_rate, per_thread_rate) in &efficiency_results {
        let cpu_ratio = *threads as f32 / cpu_count as f32;
        let efficiency_loss = ((baseline_per_thread - per_thread_rate) / baseline_per_thread) * 100.0;
        
        println!("  {:11} | {:8.1}x | {:8.1}   | {:8.1}   | {:11.1}%", 
            threads, cpu_ratio, total_rate, per_thread_rate, efficiency_loss);
    }
    
    println!();
    println!("💡 Efficiency Insights:");
    
    // Check for efficiency drop-offs
    let worst_efficiency = efficiency_results.iter()
        .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
        .unwrap();
    
    if worst_efficiency.0 > cpu_count * 2 {
        println!("  ⚠️  Significant efficiency loss at high thread counts");
        println!("  📊 Recommendation: Stay at or below {}x CPU count", 
            if cpu_count >= 32 { 1 } else { 2 });
    } else {
        println!("  ✅ Thread scaling remains efficient within tested range");
        println!("  📊 Your hardware handles high thread counts well");
    }
    
    // Basic sanity check
    assert!(efficiency_results.len() > 0, "Should have efficiency results");
}

#[test]
fn test_numa_awareness() {
    println!("🧪 Testing NUMA Awareness (Basic)");
    println!("==================================");
    
    let cpu_count = num_cpus::get();
    
    if cpu_count < 16 {
        println!("⚠️  NUMA testing most relevant for systems with 16+ threads");
        println!("  Your system: {} threads", cpu_count);
        return;
    }
    
    println!("🖥️  System has {} threads - NUMA effects may be relevant", cpu_count);
    
    let test_header = vec![
        0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x9a,
        0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55,
        0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
        0xee, 0xff, 0x00, 0x12, 0x34, 0x56, 0x78, 0x90,
    ];
    
    let test_duration = Duration::from_secs(3);
    
    // Test thread counts that might reveal NUMA effects
    let numa_test_counts = vec![
        cpu_count / 4,     // Quarter threads
        cpu_count / 2,     // Half threads (single socket?)
        cpu_count,         // All threads
    ];
    
    println!();
    println!("🔄 Testing for potential NUMA effects...");
    
    for &thread_count in &numa_test_counts {
        if thread_count == 0 {
            continue;
        }
        
        println!("📊 Testing {} threads...", thread_count);
        
        let total_hashes = Arc::new(AtomicU64::new(0));
        let should_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        
        let start_time = Instant::now();
        
        let mut handles = Vec::new();
        for thread_id in 0..thread_count {
            let header = test_header.clone();
            let hashes = Arc::clone(&total_hashes);
            let stop_flag = Arc::clone(&should_stop);
            
            let handle = thread::spawn(move || {
                let mut local_hashes = 0u64;
                let mut nonce = thread_id as u64 * 50000;
                
                while !stop_flag.load(Ordering::Relaxed) {
                    let _batch_results = sha3x_hash_with_nonce_batch(&header, nonce);
                    local_hashes += 4;
                    nonce = nonce.wrapping_add(thread_count as u64 * 4);
                    
                    if local_hashes % 500 == 0 && stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                }
                
                hashes.fetch_add(local_hashes, Ordering::Relaxed);
            });
            
            handles.push(handle);
        }
        
        thread::sleep(test_duration);
        should_stop.store(true, Ordering::Relaxed);
        
        for handle in handles {
            let _ = handle.join();
        }
        
        let duration = start_time.elapsed();
        let final_hashes = total_hashes.load(Ordering::Relaxed);
        let hashrate = final_hashes as f64 / duration.as_secs_f64();
        let per_thread_rate = hashrate / thread_count as f64;
        
        println!("  {} threads: {:.1} H/s ({:.1} per thread)", 
            thread_count, hashrate, per_thread_rate);
    }
    
    println!();
    println!("💡 NUMA Considerations for Dual Xeon Systems:");
    println!("  - Each CPU has its own memory controller");
    println!("  - Cross-socket memory access is slower");
    println!("  - Consider CPU affinity for optimal performance");
    println!("  - Monitor memory bandwidth with 'numastat'");
    println!("  - Your 72-thread system likely benefits from NUMA optimization");
    
    println!();
    println!("🔧 Optimization Tips:");
    println!("  - Test thread counts: 36, 54, 72 (socket-aware)");
    println!("  - Use 'numactl' for thread affinity if available");
    println!("  - Monitor both sockets with system tools");
    println!("  - Batch processing helps reduce memory pressure");
}

// Changelog:
// - v1.0.0 (2025-06-15): Initial thread scaling tests implementation.
//   - Purpose: Tests thread scaling performance to find optimal thread counts
//     for different hardware configurations and identify bottlenecks.
//   - Features: Thread scaling analysis, efficiency limits testing, and basic
//     NUMA awareness for multi-socket systems like dual Xeon setups.
//   - Note: These tests help users optimize thread counts for their specific
//     hardware and understand scaling characteristics of the mining software.