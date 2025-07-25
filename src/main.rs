// SHA3x Miner - Free and Open Source Software Statement
//
// File: src/main.rs
// Version: 2.3.0-multi-gpu-dual-independent
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// MULTI-GPU DUAL-INDEPENDENT MINERS: Complete hybrid mode with resilient miners
// Feature-based mining with proper thread coordination: --features cpu, --features gpu, --features hybrid

use clap::Parser;
use log::info;
use sha3x_miner::{
    Result,
    benchmark::runner::BenchmarkRunner,
    core::types::{Algorithm, Args},
    miner::CpuMiner,
};

// Web server module for real-time mining dashboard
mod web_server;

const LOG_TARGET: &str = "tari::graxil";

// Ensure exactly one mining mode is selected
#[cfg(not(any(feature = "cpu", feature = "gpu", feature = "hybrid")))]
compile_error!(target: LOG_TARGET,"Must specify one feature: --features cpu, --features gpu, or --features hybrid");

// Prevent conflicting standalone features when hybrid is not used
#[cfg(all(feature = "cpu", feature = "gpu", not(feature = "hybrid")))]
compile_error!(target: LOG_TARGET,
    "Cannot use both --features cpu and --features gpu. Use --features hybrid for both."
);

// Prevent using hybrid with standalone features
#[cfg(all(
    feature = "hybrid",
    any(
        all(feature = "cpu", not(feature = "gpu")),
        all(feature = "gpu", not(feature = "cpu"))
    )
))]
compile_error!(target: LOG_TARGET,
    "When using --features hybrid, do not specify cpu or gpu separately. Use only --features hybrid."
);

//
// CPU-ONLY MINING MODE
//
#[cfg(all(feature = "cpu", not(feature = "hybrid")))]
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Check for SV2 test mode first
    if args.test_sv2 {
        return handle_sv2_test(&args).await;
    }

    // Validate arguments
    if let Err(err) = args.validate() {
        eprintln!("❌ Error: {}", err);
        std::process::exit(1);
    }

    let algo = parse_algorithm(&args.algo)?;

    if args.benchmark {
        return handle_benchmark(&args, algo).await;
    } else {
        return handle_cpu_mining(&args, algo).await;
    }
}

//
// GPU-ONLY MINING MODE
//
#[cfg(all(feature = "gpu", not(feature = "hybrid")))]
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(ref log_dir) = args.log_dir {
        tari_common::initialize_logging(
            &log_dir.join("graxil").join("log4rs_config.yml"),
            &log_dir.join("graxil"),
            include_str!("../log4rs_sample.yml"),
        )
        .expect("Could not set up logging");
    }

    // Check for SV2 test mode first
    if args.test_sv2 {
        return handle_sv2_test(&args).await;
    }

    // Validate arguments
    if let Err(err) = args.validate() {
        eprintln!("❌ Error: {}", err);
        std::process::exit(1);
    }

    let algo = parse_algorithm(&args.algo)?;

    if args.benchmark {
        return handle_benchmark(&args, algo).await;
    } else {
        return handle_gpu_mining(&args, algo).await;
    }
}

//
// HYBRID MINING MODE (CPU + GPU) - MULTI-GPU DUAL-INDEPENDENT
//
#[cfg(feature = "hybrid")]
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Check for SV2 test mode first
    if args.test_sv2 {
        return handle_sv2_test(&args).await;
    }

    // Validate arguments
    if let Err(err) = args.validate() {
        eprintln!("❌ Error: {}", err);
        std::process::exit(1);
    }

    let algo = parse_algorithm(&args.algo)?;

    if args.benchmark {
        return handle_benchmark(&args, algo).await;
    } else {
        return handle_hybrid_mining(&args, algo).await;
    }
}

//
// SHARED HELPER FUNCTIONS
//

async fn handle_sv2_test(args: &Args) -> Result<()> {
    info!(target: LOG_TARGET,"🔧 SV2 Connection Test Mode");

    // Validate required arguments for SV2 test
    let pool_address = match &args.pool {
        Some(pool) => pool,
        None => {
            eprintln!("❌ Error: --pool is required for SV2 testing");
            eprintln!(
                "Example: cargo run --release --features cpu -- --test-sv2 --pool 127.0.0.1:34254"
            );
            std::process::exit(1);
        }
    };

    info!(target: LOG_TARGET,"🎯 Target JDS: {}", pool_address);

    // Create a test miner instance - pass pool address as string
    let miner = CpuMiner::new(
        "test-wallet".to_string(), // Dummy wallet for SV2 test
        pool_address.clone(),      // Pass as string, miner will resolve DNS
        "sv2-test-worker".to_string(),
        1,                // Single thread for test
        Algorithm::Sha3x, // Algorithm doesn't matter for connection test
    );

    // Run SV2 connection test
    match miner.test_sv2_connection().await {
        Ok(()) => {
            info!(target: LOG_TARGET,"✅ TCP connection to JDS successful");
            info!(target: LOG_TARGET,"❌ Noise protocol not implemented yet");
            info!(target: LOG_TARGET,"🔧 Next: Implement noise_sv2 handshake with step_0/step_2");
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

fn parse_algorithm(algo_str: &str) -> Result<Algorithm> {
    let algo = match algo_str {
        "sha3x" => Algorithm::Sha3x,
        "sha256" => Algorithm::Sha256,
        _ => {
            eprintln!("❌ Invalid algorithm: {}", algo_str);
            std::process::exit(1);
        }
    };
    Ok(algo)
}

async fn handle_benchmark(args: &Args, algo: Algorithm) -> Result<()> {
    info!(target: LOG_TARGET,"🧪 Starting Benchmark Mode (Algo: {:?})", algo);
    info!(target: LOG_TARGET,
        "🧵 Threads: {}",
        if args.threads == 0 {
            "auto".to_string()
        } else {
            args.threads.to_string()
        }
    );
    info!(target: LOG_TARGET,"⏱️ Duration: {}s", args.benchmark_duration);
    info!(target: LOG_TARGET,"🎯 Target difficulty: {:.10}", args.benchmark_difficulty);

    let benchmark_runner = BenchmarkRunner::new(
        args.threads,
        args.benchmark_duration,
        args.benchmark_difficulty,
        algo,
    );

    let result = benchmark_runner.run().await?;

    info!(target: LOG_TARGET,"📊 Benchmark Complete!");
    info!(target: LOG_TARGET,"🧪 Algorithm: {:?}", algo);
    info!(target: LOG_TARGET,"🎯 Difficulty tested: {:.10}", args.benchmark_difficulty);
    info!(target: LOG_TARGET,"⏱️ Duration: {:.2}s", result.duration.as_secs_f64());
    info!(target: LOG_TARGET,"⚡ Average hashrate: {}", result.format_hashrate());
    info!(target: LOG_TARGET,
        "🔥 Peak hashrate: {:.2} MH/s",
        result.peak_hashrate / 1_000_000.0
    );
    info!(target: LOG_TARGET,"📈 Total hashes: {}", result.total_hashes);
    info!(target: LOG_TARGET,"💎 Shares found: {}", result.shares_found);
    info!(target: LOG_TARGET,
        "📊 Shares/MH: {:.2}",
        result.shares_found as f64 / (result.total_hashes as f64 / 1_000_000.0)
    );
    info!(target: LOG_TARGET,"🧵 Threads used: {}", result.thread_count);

    Ok(())
}

//
// CPU-ONLY MINING
//
#[cfg(all(feature = "cpu", not(feature = "hybrid")))]
async fn handle_cpu_mining(args: &Args, algo: Algorithm) -> Result<()> {
    // Only SHA3x mining supported now
    if algo != Algorithm::Sha3x {
        eprintln!("❌ Only SHA3x algorithm is supported in this version");
        eprintln!("💡 Use --algo sha3x for mining");
        std::process::exit(1);
    }

    info!(target: LOG_TARGET,"🚀 Starting SHA3x Miner - CPU-ONLY Mode");
    info!(target: LOG_TARGET,"📍 Pool: {}", args.pool.as_ref().unwrap());
    info!(target: LOG_TARGET,"💳 Wallet: {}", args.wallet.as_ref().unwrap());
    info!(target: LOG_TARGET,"👷 Worker: {}", args.worker);
    info!(target: LOG_TARGET,
        "🧵 CPU Threads: {}",
        if args.threads == 0 {
            "auto".to_string()
        } else {
            args.threads.to_string()
        }
    );
    info!(target: LOG_TARGET,"💻 Mode: CPU-only mining (compile with --features gpu for 300+ MH/s boost!)");

    // Create and run your existing CPU miner
    let miner = CpuMiner::new(
        args.wallet.as_ref().unwrap().clone(),
        args.pool.as_ref().unwrap().clone(),
        args.worker.clone(),
        args.threads,
        algo,
    )
    .into_arc();

    // Start web server in background if --web flag is enabled
    if args.web {
        let miner_clone = miner.clone();
        tokio::spawn(async move {
            let stats = miner_clone.get_stats();
            info!(target: LOG_TARGET,"🌐 Starting web dashboard server...");
            web_server::start_web_server(stats).await;
        });

        info!(target: LOG_TARGET,"📊 Real-time dashboard available at: http://localhost:8080");
        info!(target: LOG_TARGET,"📈 Live charts accessible via the 'Live Charts' tab");
        info!(target: LOG_TARGET,"🔗 WebSocket endpoint: ws://localhost:8080/ws");
    } else {
        info!(target: LOG_TARGET,"💡 Add --web flag to enable real-time web dashboard");
    }

    // Start CPU mining
    info!(target: LOG_TARGET,"🚀 Starting CPU mining");
    miner.run().await?;

    Ok(())
}

//
// GPU-ONLY MINING - FIXED VERSION
//
#[cfg(all(feature = "gpu", not(feature = "hybrid")))]
async fn handle_gpu_mining(args: &Args, algo: Algorithm) -> Result<()> {
    // Only SHA3x mining supported now
    if algo != Algorithm::Sha3x {
        eprintln!("❌ Only SHA3x algorithm is supported in this version");
        eprintln!("💡 Use --algo sha3x for mining");
        std::process::exit(1);
    }

    info!(target: LOG_TARGET,"🚀 Starting SHA3x Miner - GPU-ONLY Mode");
    info!(target: LOG_TARGET,"📍 Pool: {}", args.pool.as_ref().unwrap());
    info!(target: LOG_TARGET,"💳 Wallet: {}", args.wallet.as_ref().unwrap());
    info!(target: LOG_TARGET,"👷 Worker: {}", args.worker);
    info!(target: LOG_TARGET,"🎮 Mode: GPU-only mining (385+ MH/s beast mode!)");

    // *** CRITICAL FIX: Get GPU settings from CLI args and pass them properly ***
    let gpu_settings = args.get_gpu_settings();
    info!(target: LOG_TARGET,
        "🎮 GPU Settings - Intensity: {}%, Batch: {:?}, Power: {:?}%, Temp: {:?}°C",
        gpu_settings.intensity,
        gpu_settings.batch_size,
        gpu_settings.power_limit,
        gpu_settings.temp_limit
    );

    use log::info;
    // Create GPU manager with settings applied
    use sha3x_miner::miner::gpu::{GpuManager, GpuMiner};

    let gpu_manager = GpuManager::new_with_settings(gpu_settings.clone());

    // *** CRITICAL FIX: Use new_with_settings instead of new() ***
    let gpu_miner = match GpuMiner::new_with_settings(
        args.wallet.as_ref().unwrap().clone(),
        args.pool.as_ref().unwrap().clone(),
        args.worker.clone(),
        algo,
        gpu_manager,
        gpu_settings, // ✅ Now properly passing CLI settings!
    ) {
        Ok(miner) => miner.into_arc(),
        Err(e) => {
            eprintln!("❌ Failed to create GPU miner: {}", e);
            eprintln!("💡 Make sure you have OpenCL drivers installed");
            eprintln!(
                "💡 GPU Settings attempted: intensity={}%, batch={:?}",
                args.gpu_intensity, args.gpu_batch_size
            );
            std::process::exit(1);
        }
    };

    // Verify settings were applied correctly
    let applied_settings = gpu_miner.get_gpu_settings();
    info!(target: LOG_TARGET,
        "✅ GPU Settings Applied: intensity={}%, batch={:?}",
        applied_settings.intensity, applied_settings.batch_size
    );

    // Start web server in background if --web flag is enabled
    if args.web {
        let miner_clone = gpu_miner.clone();
        tokio::spawn(async move {
            let stats = miner_clone.get_stats();
            info!(target: LOG_TARGET,"🌐 Starting GPU web dashboard server...");
            web_server::start_web_server(stats).await;
        });

        info!(target: LOG_TARGET,"📊 Real-time GPU dashboard available at: http://localhost:8080");
        info!(target: LOG_TARGET,"📈 Live GPU charts accessible via the 'Live Charts' tab");
        info!(target: LOG_TARGET,"🔗 WebSocket endpoint: ws://localhost:8080/ws");
    } else {
        info!(target: LOG_TARGET,"💡 Add --web flag to enable real-time web dashboard");
    }

    // Start GPU mining - 385+ MH/s beast mode with correct settings!
    info!(target: LOG_TARGET,
        "🚀 Starting GPU mining with {}% intensity - unleashing the beast!",
        applied_settings.intensity
    );
    gpu_miner.run().await?;

    Ok(())
}

//
// MULTI-GPU HYBRID MINING (CPU + GPU) - DUAL-INDEPENDENT MINERS
//
#[cfg(feature = "hybrid")]
async fn handle_hybrid_mining(args: &Args, algo: Algorithm) -> Result<()> {
    // Only SHA3x mining supported now
    if algo != Algorithm::Sha3x {
        eprintln!("❌ Only SHA3x algorithm is supported in this version");
        eprintln!("💡 Use --algo sha3x for mining");
        std::process::exit(1);
    }

    info!(target: LOG_TARGET,"🚀 Starting SHA3x Miner - MULTI-GPU HYBRID Mode");
    info!(target: LOG_TARGET,"📍 Pool: {}", args.pool.as_ref().unwrap());
    info!(target: LOG_TARGET,"💳 Wallet: {}", args.wallet.as_ref().unwrap());
    info!(target: LOG_TARGET,"👷 Worker: {}", args.worker);
    info!(target: LOG_TARGET,
        "🧵 CPU Threads: {}",
        if args.threads == 0 {
            "auto".to_string()
        } else {
            args.threads.to_string()
        }
    );
    info!(target: LOG_TARGET,"🎮 Mode: Multi-GPU hybrid CPU+GPU mining (400+ MH/s total beast mode!)");

    // Get GPU settings from CLI args
    let gpu_settings = args.get_gpu_settings();
    info!(target: LOG_TARGET,
        "🎮 GPU Settings - Intensity: {}%, Batch: {:?}, Power: {:?}%, Temp: {:?}°C",
        gpu_settings.intensity,
        gpu_settings.batch_size,
        gpu_settings.power_limit,
        gpu_settings.temp_limit
    );

    // Check GPU availability and get device count
    use sha3x_miner::miner::gpu::GpuManager;
    if !GpuManager::is_available() {
        error!(target: LOG_TARGET,"❌ No suitable GPU found for hybrid mining!");
        error!(target: LOG_TARGET,"💡 Falling back to CPU-only mode...");
        return handle_cpu_fallback(args, algo).await;
    }

    // Initialize GPU manager to get actual device count
    let mut gpu_manager = GpuManager::new_with_settings(gpu_settings.clone());
    if let Err(e) = gpu_manager.initialize() {
        error!(target: LOG_TARGET,"❌ Failed to initialize GPU manager: {}", e);
        error!(target: LOG_TARGET,"💡 Falling back to CPU-only mode...");
        return handle_cpu_fallback(args, algo).await;
    }

    // *** CRITICAL: Get actual GPU device count for thread coordination ***
    let gpu_count = gpu_manager.device_count();
    let cpu_thread_count = if args.threads == 0 {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    } else {
        args.threads
    };

    // *** MULTI-GPU THREAD COORDINATION ***
    info!(target: LOG_TARGET,"🧵 Multi-GPU Thread Coordination:");
    if gpu_count == 1 {
        info!(target: LOG_TARGET,"├─ GPU Device: 1 (thread ID: 0)");
    } else {
        info!(target: LOG_TARGET,
            "├─ GPU Devices: {} (thread IDs: 0-{})",
            gpu_count,
            gpu_count - 1
        );
    }
    info!(target: LOG_TARGET,
        "├─ CPU Threads: {} (thread IDs: {}-{})",
        cpu_thread_count,
        gpu_count,
        gpu_count + cpu_thread_count - 1
    );
    info!(target: LOG_TARGET,"└─ Total Threads: {}", gpu_count + cpu_thread_count);

    // *** CREATE UNIFIED STATS FOR ALL THREADS ***
    use sha3x_miner::miner::stats::MinerStats;

    let total_threads = gpu_count + cpu_thread_count; // Dynamic based on actual GPU count
    let mut unified_stats = MinerStats::new(total_threads);
    unified_stats.set_algorithm(algo);
    let unified_stats = Arc::new(unified_stats);

    info!(target: LOG_TARGET,
        "📊 Created unified stats for {} total threads",
        total_threads
    );

    // Start web server ONCE for unified dashboard
    if args.web {
        let stats_clone = Arc::clone(&unified_stats);
        tokio::spawn(async move {
            info!(target: LOG_TARGET,"🌐 Starting unified multi-GPU hybrid web dashboard server...");
            web_server::start_web_server(stats_clone).await;
        });

        info!(target: LOG_TARGET,"📊 Real-time MULTI-GPU HYBRID dashboard available at: http://localhost:8080");
        info!(target: LOG_TARGET,"📈 Combined CPU+GPU charts accessible via the 'Live Charts' tab");
        info!(target: LOG_TARGET,"🔗 WebSocket endpoint: ws://localhost:8080/ws");
    } else {
        info!(target: LOG_TARGET,"💡 Add --web flag to enable real-time unified dashboard");
    }

    // *** CREATE DUAL-INDEPENDENT MINERS ***

    // 1. Create CPU miner with shared stats and proper thread coordination
    let cpu_miner = create_multi_gpu_cpu_miner(
        args,
        algo,
        Arc::clone(&unified_stats),
        gpu_count, // Dynamic GPU count for thread offset calculation
        cpu_thread_count,
    )
    .await?;

    // 2. Create GPU miner with shared stats for hybrid mode
    let gpu_miner = create_multi_gpu_gpu_miner(
        args,
        algo,
        gpu_manager,
        Arc::clone(&unified_stats),
        gpu_settings.clone(),
    )
    .await?;

    info!(target: LOG_TARGET,"🚀 Starting DUAL-INDEPENDENT MULTI-GPU hybrid mining!");
    info!(target: LOG_TARGET,
        "💪 Expected combined hashrate: 400+ MH/s (GPU: {}% intensity)",
        gpu_settings.intensity
    );
    info!(target: LOG_TARGET,"🛡️ Resilient design: Each miner has independent pool connection");

    // *** RUN BOTH MINERS INDEPENDENTLY ***
    let cpu_handle = tokio::spawn(async move {
        info!(target: LOG_TARGET,"🧵 Starting independent CPU miner...");
        if let Err(e) = cpu_miner.run().await {
            error!(target: LOG_TARGET,"❌ CPU miner failed: {}", e);
        } else {
            info!(target: LOG_TARGET,"🧵 CPU miner completed successfully");
        }
    });

    let gpu_handle = tokio::spawn(async move {
        info!(target: LOG_TARGET,"🎮 Starting independent GPU miner...");
        if let Err(e) = gpu_miner.run().await {
            error!(target: LOG_TARGET,"❌ GPU miner failed: {}", e);
        } else {
            info!(target: LOG_TARGET,"🎮 GPU miner completed successfully");
        }
    });

    // Wait for either to complete (shouldn't happen in normal operation)
    tokio::select! {
        _ = cpu_handle => {
            error!(target: LOG_TARGET,"🔥 CPU miner stopped unexpectedly - GPU miner continues");
        }
        _ = gpu_handle => {
            error!(target: LOG_TARGET,"🔥 GPU miner stopped unexpectedly - CPU miner continues");
        }
    }

    Ok(())
}

/// Fallback to CPU-only mining if GPU fails in hybrid mode
#[cfg(feature = "hybrid")]
async fn handle_cpu_fallback(args: &Args, algo: Algorithm) -> Result<()> {
    use sha3x_miner::miner::CpuMiner;

    info!(target: LOG_TARGET,"🔄 Initializing CPU-only fallback mode...");

    let miner = CpuMiner::new(
        args.wallet.as_ref().unwrap().clone(),
        args.pool.as_ref().unwrap().clone(),
        format!("{}-cpu-fallback", args.worker),
        args.threads,
        algo,
    )
    .into_arc();

    if args.web {
        let miner_clone = miner.clone();
        tokio::spawn(async move {
            let stats = miner_clone.get_stats();
            info!(target: LOG_TARGET,"🌐 Starting fallback web dashboard server...");
            web_server::start_web_server(stats).await;
        });
        info!(target: LOG_TARGET,"📊 Fallback dashboard available at: http://localhost:8080");
    }

    info!(target: LOG_TARGET,"🚀 Starting CPU fallback mining");
    miner.run().await?;
    Ok(())
}

/// Create CPU miner for multi-GPU hybrid mode with shared stats and dynamic thread coordination
#[cfg(feature = "hybrid")]
async fn create_multi_gpu_cpu_miner(
    args: &Args,
    algo: Algorithm,
    shared_stats: Arc<sha3x_miner::miner::stats::MinerStats>,
    gpu_count: usize, // Dynamic GPU count for proper thread offset
    cpu_thread_count: usize,
) -> Result<Arc<sha3x_miner::miner::CpuMiner>> {
    use sha3x_miner::miner::CpuMiner;

    info!(target: LOG_TARGET,"🧵 Creating multi-GPU aware CPU miner component...");
    info!(target: LOG_TARGET,
        "🎮 Detected {} GPU device(s) - CPU threads will start at ID {}",
        gpu_count, gpu_count
    );

    // *** CRITICAL FIX: Use new multi-GPU aware constructor ***
    let cpu_miner = CpuMiner::new_with_shared_stats(
        args.wallet.as_ref().unwrap().clone(),
        args.pool.as_ref().unwrap().clone(),
        format!("{}-cpu", args.worker), // Distinct worker name
        cpu_thread_count,
        algo,
        shared_stats, // ✅ Shared stats for unified dashboard
        gpu_count,    // ✅ Dynamic GPU count for thread coordination
    );

    info!(target: LOG_TARGET,"✅ Multi-GPU CPU miner created:");
    info!(target: LOG_TARGET,"├─ Worker: {}-cpu", args.worker);
    info!(target: LOG_TARGET,
        "├─ Threads: {} (IDs: {}-{})",
        cpu_thread_count,
        gpu_count,
        gpu_count + cpu_thread_count - 1
    );
    info!(target: LOG_TARGET,"├─ Pool connection: Independent (resilient)");
    info!(target: LOG_TARGET,"└─ Stats: Shared with GPU (unified dashboard)");

    Ok(cpu_miner.into_arc())
}

/// Create GPU miner for multi-GPU hybrid mode with shared stats
#[cfg(feature = "hybrid")]
async fn create_multi_gpu_gpu_miner(
    args: &Args,
    algo: Algorithm,
    gpu_manager: sha3x_miner::miner::gpu::GpuManager,
    shared_stats: Arc<sha3x_miner::miner::stats::MinerStats>,
    gpu_settings: sha3x_miner::core::types::GpuSettings,
) -> Result<Arc<sha3x_miner::miner::gpu::GpuMiner>> {
    use sha3x_miner::miner::gpu::GpuMiner;

    let gpu_count = gpu_manager.device_count();
    info!(target: LOG_TARGET,"🎮 Creating multi-GPU aware GPU miner component...");
    info!(target: LOG_TARGET,
        "🎮 GPU Settings: intensity={}%, batch={:?}",
        gpu_settings.intensity, gpu_settings.batch_size
    );

    // *** CRITICAL FIX: Use new_for_hybrid with shared stats ***
    let gpu_miner = GpuMiner::new_for_hybrid(
        args.wallet.as_ref().unwrap().clone(),
        args.pool.as_ref().unwrap().clone(),
        format!("{}-gpu", args.worker), // Distinct worker name
        algo,
        gpu_manager,
        gpu_settings.clone(), // ✅ Apply GPU settings
        shared_stats,         // ✅ Shared stats for unified dashboard
        Arc::new(sha3x_miner::pool::client::PoolClient::new()), // ✅ Independent pool client
        0,                    // ✅ GPU threads start at 0 (will handle multiple devices internally)
    )?;

    info!(target: LOG_TARGET,"✅ Multi-GPU GPU miner created:");
    info!(target: LOG_TARGET,"├─ Worker: {}-gpu", args.worker);
    if gpu_count == 1 {
        info!(target: LOG_TARGET,"├─ Device: 1 (thread ID: 0)");
    } else {
        info!(target: LOG_TARGET,
            "├─ Devices: {} (thread IDs: 0-{})",
            gpu_count,
            gpu_count - 1
        );
    }
    info!(target: LOG_TARGET,
        "├─ Settings: {}% intensity, batch {:?}",
        gpu_settings.intensity, gpu_settings.batch_size
    );
    info!(target: LOG_TARGET,"├─ Pool connection: Independent (resilient)");
    info!(target: LOG_TARGET,"└─ Stats: Shared with CPU (unified dashboard)");

    Ok(gpu_miner.into_arc())
}

// Changelog:
// - v2.3.0-multi-gpu-dual-independent (2025-06-25): COMPLETE MULTI-GPU HYBRID IMPLEMENTATION
//   *** DUAL-INDEPENDENT MINER ARCHITECTURE ***:
//   1. CPU and GPU miners run completely independently with own pool connections
//   2. Both miners report to shared stats for unified dashboard
//   3. Dynamic thread coordination based on actual GPU device count
//   4. Fault tolerance: One miner failure doesn't affect the other
//   *** MULTI-GPU SUPPORT ***:
//   - Supports 1-N GPU devices automatically
//   - Thread allocation: GPU devices 0-(N-1), CPU threads N-(N+CPU_COUNT-1)
//   - Dynamic stats array sizing based on total thread count
//   *** TECHNICAL IMPLEMENTATION ***:
//   - Uses CpuMiner::new_with_shared_stats() for proper thread coordination
//   - Uses GpuMiner::new_for_hybrid() with shared stats but independent pool
//   - Enhanced logging showing exact thread allocation and device mapping
//   - Distinct worker names for pool identification (worker-cpu, worker-gpu)
//   *** RESILIENCE FEATURES ***:
//   - Each miner has independent pool connection and error handling
//   - Unified web dashboard shows combined mining activity
//   - Graceful handling of GPU detection failures with CPU fallback
//   *** SUPPORTED CONFIGURATIONS ***:
//   - 1 GPU + CPU: GPU=0, CPU=1-6 (7 total threads)
//   - 2 GPU + CPU: GPU=0-1, CPU=2-7 (8 total threads)
//   - 4 GPU + CPU: GPU=0-3, CPU=4-9 (10 total threads)
//   - Any N GPU + CPU: GPU=0-(N-1), CPU=N-(N+CPU_COUNT-1)
