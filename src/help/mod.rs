// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/help/mod.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file declares the help module for providing comprehensive command-line
// help, examples, and documentation for the SHA3x miner. It organizes help
// content into logical sections for better user experience.
//
// Tree Location:
// - src/help/mod.rs (help module entry point)
// - Submodules: commands, benchmarks, mining

pub mod commands;
pub mod benchmarks;
pub mod mining;

// Re-export key help functions
pub use commands::{print_extended_help, get_command_examples};
pub use benchmarks::{print_benchmark_help, get_benchmark_examples};
pub use mining::{print_mining_help, get_mining_examples};

/// Display comprehensive help information
pub fn display_full_help() {
    println!("🚀 SHA3x Miner - High-Performance Tari Mining Software");
    println!("========================================================");
    println!();
    
    commands::print_extended_help();
    println!();
    
    benchmarks::print_benchmark_help();
    println!();
    
    mining::print_mining_help();
    
    println!();
    println!("📚 For detailed documentation, see:");
    println!("   • docs/USAGE.md - Complete usage guide");
    println!("   • docs/BENCHMARKING.md - Benchmark optimization guide");
    println!("   • docs/TROUBLESHOOTING.md - Common issues and solutions");
    println!();
    println!("🐛 Report issues: https://github.com/oieieio/sha3x-miner/issues");
    println!("💬 Community: https://discord.gg/tari");
}

/// Display quick help summary
pub fn display_quick_help() {
    println!("🚀 SHA3x Miner - Quick Help");
    println!("============================");
    println!();
    println!("MINING:");
    println!("  sha3x-miner -w WALLET -p POOL:PORT");
    println!();
    println!("BENCHMARKING:");
    println!("  sha3x-miner --benchmark --threads 72 --benchmark-duration 60");
    println!();
    println!("Use --help for detailed options or visit docs/ for guides.");
}

/// Display version and build information
pub fn display_version_info() {
    println!("SHA3x Miner v1.0.0");
    println!("High-Performance Tari (SHA3x) CPU Miner");
    println!();
    println!("Build Information:");
    println!("  • Rust version: {}", option_env!("RUSTC_VERSION").unwrap_or("unknown"));
    println!("  • Target: {}", std::env::consts::ARCH);
    println!("  • Profile: {}", if cfg!(debug_assertions) { "debug" } else { "release" });
    println!("  • Features: CPU mining, Benchmarking{}", if cfg!(feature = "tui") { ", TUI" } else { "" });
    println!();
    println!("License: MIT License");
    println!("Repository: https://github.com/oieieio/sha3x-miner");
}

// Changelog:
// - v1.0.0 (2025-06-15): Initial help module creation.
//   - Purpose: Provides comprehensive command-line help system with organized
//     sections for commands, benchmarking, and mining operations.
//   - Features: Declares help submodules, provides full help display function,
//     quick help summary, and version information with build details.
//   - Note: This module enhances user experience by providing clear guidance
//     for both mining and benchmarking operations with practical examples.