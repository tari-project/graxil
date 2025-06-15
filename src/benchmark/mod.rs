// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/benchmark/mod.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file declares the benchmark module for performance testing and profiling
// of the SHA3x miner. It provides infrastructure for testing hash performance,
// batch optimizations, and thread scaling without requiring pool connections.
//
// Tree Location:
// - src/benchmark/mod.rs (benchmark module entry point)
// - Submodules: jobs, runner, profiler

pub mod jobs;
pub mod runner;
pub mod profiler;

// Re-export key benchmark types and functions
pub use jobs::{BenchmarkJob, create_test_jobs};
pub use runner::{BenchmarkRunner, BenchmarkConfig};
pub use profiler::{ProfilerData, PerformanceMetrics};

// Changelog:
// - v1.0.0 (2025-06-14): Initial benchmark module creation.
//   - Purpose: Provides performance testing infrastructure for SHA3x mining
//     operations, including static job creation, benchmark execution, and
//     performance profiling capabilities.
//   - Features: Declares jobs, runner, and profiler submodules with re-exports
//     for easy access to benchmark functionality.
//   - Note: This module enables testing of optimizations like batching and
//     thread scaling without requiring pool connectivity.