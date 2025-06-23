// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/main.rs
// Version: 1.1.2-dns
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file is the main entry point for the SHA3x miner binary, responsible
// for parsing command-line arguments and initializing the CPU miner or benchmark
// runner for SHA3x (Tari) mining, or testing SV2 connections. It includes
// real-time web dashboard integration for mining operations.
//
// Tree Location:
// - src/main.rs (main binary entry point)
// - Depends on: lib.rs, core/types.rs, miner/cpu/miner.rs, benchmark/runner.rs, web_server.rs

use sha3x_miner::{core::types::{Args, Algorithm}, miner::CpuMiner, benchmark::runner::BenchmarkRunner, Result};
use clap::Parser;
use tracing::info;
use tracing_subscriber;

// Web server module for real-time mining dashboard
mod web_server;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Check for SV2 test mode first
    if args.test_sv2 {
        // Initialize tracing for SV2 test
        #[cfg(not(feature = "tui"))]
        tracing_subscriber::fmt::init();

        info!("🔧 SV2 Connection Test Mode");
        
        // Validate required arguments for SV2 test
        let pool_address = match &args.pool {
            Some(pool) => pool,
            None => {
                eprintln!("❌ Error: --pool is required for SV2 testing");
                eprintln!("Example: cargo run --release -- --test-sv2 --pool 127.0.0.1:34254");
                std::process::exit(1);
            }
        };

        info!("🎯 Target JDS: {}", pool_address);

        // Create a test miner instance - pass pool address as string
        let miner = CpuMiner::new(
            "test-wallet".to_string(), // Dummy wallet for SV2 test
            pool_address.clone(), // Pass as string, miner will resolve DNS
            "sv2-test-worker".to_string(),
            1, // Single thread for test
            Algorithm::Sha3x, // Algorithm doesn't matter for connection test
        );

        // Run SV2 connection test
        match miner.test_sv2_connection().await {
            Ok(()) => {
                info!("✅ TCP connection to JDS successful");
                info!("❌ Noise protocol not implemented yet");
                info!("🔧 Next: Implement noise_sv2 handshake with step_0/step_2");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("❌ SV2 connection test failed: {}", e);
                eprintln!("💡 Make sure JDS is running and accepting connections");
                eprintln!("💡 Check the JDS address and port");
                std::process::exit(1);
            }
        }
    }

    // Continue with existing functionality...
    
    // Validate arguments
    if let Err(err) = args.validate() {
        eprintln!("❌ Error: {}", err);
        std::process::exit(1);
    }

    // Initialize tracing only if TUI is disabled
    #[cfg(not(feature = "tui"))]
    tracing_subscriber::fmt::init();

    let algo = match args.algo.as_str() {
        "sha3x" => Algorithm::Sha3x,
        "sha256" => Algorithm::Sha256,
        _ => {
            eprintln!("❌ Invalid algorithm: {}", args.algo);
            std::process::exit(1);
        }
    };

    if args.benchmark {
        info!("🧪 Starting Benchmark Mode (Algo: {:?})", algo);
        info!("🧵 Threads: {}", if args.threads == 0 { "auto".to_string() } else { args.threads.to_string() });
        info!("⏱️ Duration: {}s", args.benchmark_duration);
        info!("🎯 Target difficulty: {:.10}", args.benchmark_difficulty);

        let benchmark_runner = BenchmarkRunner::new(
            args.threads,
            args.benchmark_duration,
            args.benchmark_difficulty,
            algo,
        );

        let result = benchmark_runner.run().await?;
        
        info!("📊 Benchmark Complete!");
        info!("🧪 Algorithm: {:?}", algo);
        info!("🎯 Difficulty tested: {:.10}", args.benchmark_difficulty);
        info!("⏱️ Duration: {:.2}s", result.duration.as_secs_f64());
        info!("⚡ Average hashrate: {}", result.format_hashrate());
        info!("🔥 Peak hashrate: {:.2} MH/s", result.peak_hashrate / 1_000_000.0);
        info!("📈 Total hashes: {}", result.total_hashes);
        info!("💎 Shares found: {}", result.shares_found);
        info!("📊 Shares/MH: {:.2}", result.shares_found as f64 / (result.total_hashes as f64 / 1_000_000.0));
        info!("🧵 Threads used: {}", result.thread_count);

    } else {
        // Only SHA3x mining supported now
        if algo != Algorithm::Sha3x {
            eprintln!("❌ Only SHA3x algorithm is supported in this version");
            eprintln!("💡 Use --algo sha3x for mining");
            std::process::exit(1);
        }

        info!("🚀 Starting SHA3x Miner");
        info!("📍 Pool: {}", args.pool.as_ref().unwrap());
        info!("💳 Wallet: {}", args.wallet.as_ref().unwrap());
        info!("👷 Worker: {}", args.worker);
        info!("🧵 Threads: {}", if args.threads == 0 { "auto".to_string() } else { args.threads.to_string() });

        // Pass pool address as string - miner will handle DNS resolution
        let miner = CpuMiner::new(
            args.wallet.unwrap(),
            args.pool.unwrap(), // Pass as string, no parsing needed
            args.worker,
            args.threads,
            algo,
        ).into_arc();

        // Start web server in background if --web flag is enabled
        if args.web {
            let miner_clone = miner.clone();
            tokio::spawn(async move {
                // Access stats through the miner's public interface
                let stats = miner_clone.get_stats();
                info!("🌐 Starting web dashboard server...");
                web_server::start_web_server(stats).await;
            });

            // Log dashboard availability
            info!("📊 Real-time dashboard will be available at: http://localhost:8080");
            info!("📈 Live charts accessible via the 'Live Charts' tab");
            info!("🔗 WebSocket endpoint: ws://localhost:8080/ws");
        } else {
            info!("💡 Add --web flag to enable real-time web dashboard");
        }

        // Start the miner
        miner.run().await?;
    }

    Ok(())
}

// Changelog:
// - v1.1.2-dns (2025-06-23): Added DNS resolution support.
//   - Removed SocketAddr parsing - miner now handles DNS resolution internally
//   - Pass pool addresses as strings to support both IP addresses and domain names
//   - Updated both SV2 test mode and mining mode to use string pool addresses
//   - Maintains backward compatibility with IP:port format
//   - Now supports domain names like pool.sha3x.supportxtm.com:6118
// - v1.1.1-dashboard (2025-06-22): Added optional real-time web dashboard integration.
//   - Added web_server module import for mining dashboard functionality.
//   - Integrated WebSocket-based real-time statistics broadcasting.
//   - Added --web flag to optionally enable web server during mining operations.
//   - Enhanced logging to inform users about dashboard availability and usage.
//   - Maintained all existing SV2 testing and benchmark functionality.
//   - Dashboard serves at http://localhost:8080 with live charts when --web is used.
// - v1.1.0-sv2 (2025-06-20): Added SV2 connection testing support.
//   - Added --test-sv2 flag handling for Noise protocol connection testing.
//   - Added SV2 test mode with proper argument validation.
//   - Restricted mining mode to SHA3x only (SHA-256 removed for SV2 transition).
//   - Added comprehensive error handling and user guidance for SV2 tests.
//   - Enhanced logging for SV2 connection test results.
// - v1.0.8 (2025-06-18): Enhanced benchmark summary with more details.
//   - Added difficulty tested in the summary output.
//   - Added actual benchmark duration in seconds.
//   - Added shares per million hashes metric for easy performance comparison.
//   - Reordered summary fields for better readability.
//   - Compatible with all existing modules.