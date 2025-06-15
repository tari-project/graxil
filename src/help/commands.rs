// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/help/commands.rs
// Version: 1.0.1
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file provides comprehensive command-line help and examples for the SHA3x
// miner, including argument descriptions, usage patterns, and practical examples
// for both mining and benchmarking operations.
//
// Tree Location:
// - src/help/commands.rs (command help and examples)
// - Depends on: none

/// Print extended help information with detailed descriptions
pub fn print_extended_help() {
    println!("COMMAND LINE OPTIONS:");
    println!("=====================");
    println!();
    
    println!("REQUIRED FOR MINING:");
    println!("  -u, --wallet <ADDRESS>     Tari wallet address for mining rewards");
    println!("  -o, --pool <HOST:PORT>     Mining pool address (e.g., pool.tari.com:4200)");
    println!();
    
    println!("OPTIONAL:");
    println!("  -p, --password <PASS>      Pool password [default: x]");
    println!("  --worker <n>            Worker identifier [default: worker1]");
    println!("  -t, --threads <NUM>        Number of CPU threads (0 = auto-detect) [default: 0]");
    println!("  -g, --gpu                  Enable GPU mining (future feature) [default: false]");
    println!();
    
    println!("BENCHMARKING:");
    println!("  --benchmark                Run performance benchmark (no pool required)");
    println!("  --benchmark-duration <SEC> Benchmark duration in seconds [default: 30]");
    println!("  --benchmark-difficulty <N> Target difficulty for share finding [default: 1000000]");
    println!();
    
    println!("DISPLAY:");
    println!("  -h, --help                 Show this help message");
    println!("  -V, --version              Show version information");
    if cfg!(feature = "tui") {
        println!("  --tui                      Enable TUI dashboard [default: false]");
    }
}

/// Get practical command examples
pub fn get_command_examples() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "Basic Mining",
            "sha3x-miner -u 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW -o tari-pool.com:4200 --threads 6"
        ),
        (
            "Mining with Custom Password",
            "sha3x-miner -u 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW -o pool.tari.com:4200 -p worker-01 --threads 8"
        ),
        (
            "Mining with Specific Thread Count",
            "sha3x-miner -u 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW -o pool.tari.com:4200 --threads 64"
        ),
        (
            "Performance Benchmark (30 seconds)",
            "sha3x-miner --benchmark --threads 72 --benchmark-duration 30"
        ),
        (
            "Extended Benchmark (5 minutes)",
            "sha3x-miner --benchmark --benchmark-duration 300 --benchmark-difficulty 100000"
        ),
        (
            "Quick Hardware Test",
            "sha3x-miner --benchmark --benchmark-duration 10 --benchmark-difficulty 1000"
        ),
    ]
}

/// Print command examples with descriptions
pub fn print_command_examples() {
    println!("USAGE EXAMPLES:");
    println!("===============");
    println!();
    
    for (description, command) in get_command_examples() {
        println!("{}:", description);
        println!("  {}", command);
        println!();
    }
}

/// Print thread configuration guidance
pub fn print_thread_guidance() {
    println!("THREAD CONFIGURATION GUIDE:");
    println!("============================");
    println!();
    println!("AUTO-DETECTION (Recommended):");
    println!("  --threads 0                Use all available CPU threads");
    println!();
    println!("MANUAL CONFIGURATION:");
    println!("  --threads 6                Use 6 threads (good for 6-core CPUs)");
    println!("  --threads 12               Use 12 threads (good for 6-core with HT)");
    println!("  --threads 32               Use 32 threads (high-end CPUs)");
    println!("  --threads 64               Use 64 threads (workstation CPUs)");
    println!("  --threads 72               Use 72 threads (dual Xeon setups)");
    println!();
    println!("PERFORMANCE TIPS:");
    println!("  • Start with auto-detection (--threads 0)");
    println!("  • Use benchmark mode to find optimal thread count");
    println!("  • Consider leaving 1-2 threads for system processes");
    println!("  • Monitor CPU temperature under full load");
    println!();
    println!("EXAMPLE THREAD COUNTS BY CPU:");
    println!("  • Intel i5-12600K:        12 threads (6P+6E cores)");
    println!("  • Intel i7-12700K:        20 threads (8P+4E cores)");
    println!("  • Intel i9-12900K:        24 threads (8P+8E cores)");
    println!("  • AMD Ryzen 9 5950X:      32 threads (16 cores)");
    println!("  • Dual Xeon 2699v3:       72 threads (36 cores total)");
}

/// Print pool configuration help
pub fn print_pool_help() {
    println!("POOL CONFIGURATION:");
    println!("===================");
    println!();
    println!("POOL ADDRESS FORMAT:");
    println!("  <hostname>:<port>          Standard format");
    println!("  <ip-address>:<port>        Direct IP connection");
    println!();
    println!("POPULAR TARI POOLS:");
    println!("  tari-pool.com:4200         Community pool");
    println!("  pool.tari.com:4200         Official pool");
    println!("  192.168.1.100:4200        Local pool example");
    println!();
    println!("WALLET ADDRESS:");
    println!("  • Must be a valid Tari wallet address");
    println!("  • Usually starts with '7' and is 64 characters long");
    println!("  • Generate from Tari Aurora wallet or console wallet");
    println!();
    println!("WORKER NAMES:");
    println!("  • Helps identify different mining rigs");
    println!("  • Use descriptive names: rig-01, office-pc, server-1");
    println!("  • Avoid spaces, use hyphens or underscores");
}

/// Print troubleshooting quick fixes
pub fn print_quick_troubleshooting() {
    println!("QUICK TROUBLESHOOTING:");
    println!("======================");
    println!();
    println!("CONNECTION ISSUES:");
    println!("  • Check pool address and port");
    println!("  • Verify internet connection");
    println!("  • Try different pool if current one is down");
    println!();
    println!("PERFORMANCE ISSUES:");
    println!("  • Run benchmark to test hardware: --benchmark");
    println!("  • Check CPU temperature and throttling");
    println!("  • Try different thread counts: --threads 32");
    println!();
    println!("WALLET ISSUES:");
    println!("  • Verify wallet address is correct");
    println!("  • Ensure wallet address is for Tari network");
    println!("  • Check with pool operator if unsure");
}

/// Print quick help summary
pub fn print_quick_help() {
    println!("MINING:");
    println!("  sha3x-miner -u WALLET -o POOL:PORT --threads 6");
    println!();
    println!("BENCHMARKING:");
    println!("  sha3x-miner --benchmark --threads 72 --benchmark-duration 60 --benchmark-difficulty 100000");
}

// Changelog:
// - v1.0.1 (2025-06-15): Updated to use standard mining conventions.
//   - Changed wallet argument from -w to -u (standard mining convention)
//   - Updated all examples to use -u for wallet address
//   - Maintained -o for pool and -p for password
//   - Updated thread examples to show practical counts like 6, 8, 64
// - v1.0.0 (2025-06-15): Initial command help implementation.
//   - Purpose: Provides comprehensive command-line help with detailed option
//     descriptions, practical examples, and configuration guidance.
//   - Features: Extended help printing, command examples, thread configuration
//     guidance, pool setup help, and quick troubleshooting tips.
//   - Note: This module serves as the primary reference for users learning
//     to use the miner effectively, with real-world examples and best practices.