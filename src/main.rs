// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/main.rs
// Version: 1.0.1
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file is the main entry point for the SHA3x miner binary, responsible
// for parsing command-line arguments and initializing the CPU miner. It is
// located at the root of the source tree.
//
// Tree Location:
// - src/main.rs (main binary entry point)
// - Depends on: lib.rs, core/types.rs, miner/cpu/miner.rs

use sha3x_miner::{core::types::Args, miner::CpuMiner, benchmark::runner::BenchmarkRunner, Result};
use clap::Parser;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Validate arguments
    if let Err(err) = args.validate() {
        eprintln!("❌ Error: {}", err);
        std::process::exit(1);
    }

    // Initialize tracing only if TUI is disabled
    #[cfg(not(feature = "tui"))]
    tracing_subscriber::fmt::init();

    if args.benchmark {
        info!("🧪 Starting SHA3x Benchmark Mode");
        info!("🧵 Threads: {}", if args.threads == 0 { "auto".to_string() } else { args.threads.to_string() });
        info!("⏱️  Duration: {}s", args.benchmark_duration);
        info!("🎯 Target difficulty: {}", args.benchmark_difficulty);

        let benchmark_runner = BenchmarkRunner::new(
            args.threads,
            args.benchmark_duration,
            args.benchmark_difficulty,
        );

        let result = benchmark_runner.run().await?;
        
        info!("📊 Benchmark Complete!");
        info!("⚡ Average hashrate: {}", result.format_hashrate());
        info!("🔥 Peak hashrate: {:.2} MH/s", result.peak_hashrate / 1_000_000.0);
        info!("📈 Total hashes: {}", result.total_hashes);
        info!("💎 Shares found: {}", result.shares_found);
        info!("🧵 Threads used: {}", result.thread_count);

    } else {
        info!("🚀 Starting SHA3x Tari Miner");
        info!("📍 Pool: {}", args.pool.as_ref().unwrap());
        info!("💳 Wallet: {}", args.wallet.as_ref().unwrap());
        info!("👷 Worker: {}", args.worker);
        info!("🧵 Threads: {}", if args.threads == 0 { "auto".to_string() } else { args.threads.to_string() });

        let pool_address: SocketAddr = args.pool.as_ref().unwrap().parse()?;

        let miner = CpuMiner::new(
            args.wallet.unwrap(),
            pool_address,
            args.worker,
            args.threads,
        );

        miner.run().await?;
    }

    Ok(())
}

// Changelog:
// - v1.0.1 (2025-06-14): Added benchmark mode support.
//   - Added argument validation using Args::validate().
//   - Added benchmark mode execution path using BenchmarkRunner.
//   - Added benchmark result reporting with formatted output.
//   - Modified mining mode to handle optional wallet/pool arguments.
//   - Added appropriate logging for both benchmark and mining modes.
// - v1.0.0 (2025-06-14): Extracted from monolithic main.rs.
//   - Purpose: Serves as the entry point for the sha3x-miner binary, handling
//     command-line argument parsing and starting the CPU miner.
//   - Features: Uses clap for CLI parsing (wallet, pool, worker, threads),
//     initializes tracing for logging (disabled with TUI feature), and creates
//     a CpuMiner instance to connect to the pool and start mining.
//   - Note: This file is minimal, delegating core logic to the miner module,
//     ensuring a clean separation of concerns.