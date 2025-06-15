// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/benchmark/jobs.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file provides static benchmark jobs for testing SHA3x mining performance
// without requiring pool connectivity. It creates deterministic test cases for
// profiling hash performance and optimization validation.
//
// Tree Location:
// - src/benchmark/jobs.rs (benchmark job creation)
// - Depends on: core/types

use crate::core::types::MiningJob;

/// Benchmark-specific job configuration
#[derive(Debug, Clone)]
pub struct BenchmarkJob {
    /// Static mining job for testing
    pub mining_job: MiningJob,
    
    /// Expected shares per million hashes (for validation)
    pub expected_shares_per_mh: f64,
    
    /// Description of this benchmark job
    pub description: String,
}

/// Create a collection of test jobs with different characteristics
pub fn create_test_jobs() -> Vec<BenchmarkJob> {
    vec![
        create_easy_job(),
        create_medium_job(),
        create_hard_job(),
        create_realistic_job(),
    ]
}

/// Create an easy job that finds shares frequently (for quick testing)
pub fn create_easy_job() -> BenchmarkJob {
    BenchmarkJob {
        mining_job: MiningJob {
            job_id: "bench_easy_001".to_string(),
            mining_hash: vec![
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
                0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
                0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
            ],
            target_difficulty: 1000,      // Very easy target
            height: 100000,
        },
        expected_shares_per_mh: 1000.0,
        description: "Easy job - high share frequency for quick validation".to_string(),
    }
}

/// Create a medium difficulty job (good for general testing)
pub fn create_medium_job() -> BenchmarkJob {
    BenchmarkJob {
        mining_job: MiningJob {
            job_id: "bench_medium_001".to_string(),
            mining_hash: vec![
                0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18,
                0x29, 0x3a, 0x4b, 0x5c, 0x6d, 0x7e, 0x8f, 0x90,
                0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
                0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65, 0x43, 0x21,
            ],
            target_difficulty: 100000,    // Medium difficulty
            height: 200000,
        },
        expected_shares_per_mh: 10.0,
        description: "Medium job - balanced difficulty for performance testing".to_string(),
    }
}

/// Create a hard job (tests performance under realistic conditions)
pub fn create_hard_job() -> BenchmarkJob {
    BenchmarkJob {
        mining_job: MiningJob {
            job_id: "bench_hard_001".to_string(),
            mining_hash: vec![
                0x5a, 0x6b, 0x7c, 0x8d, 0x9e, 0xaf, 0xb0, 0xc1,
                0xd2, 0xe3, 0xf4, 0x05, 0x16, 0x27, 0x38, 0x49,
                0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11,
                0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
            ],
            target_difficulty: 10000000,  // Hard difficulty
            height: 300000,
        },
        expected_shares_per_mh: 0.1,
        description: "Hard job - realistic mining difficulty for stress testing".to_string(),
    }
}

/// Create a realistic job matching typical Tari network conditions
pub fn create_realistic_job() -> BenchmarkJob {
    BenchmarkJob {
        mining_job: MiningJob {
            job_id: "bench_realistic_001".to_string(),
            mining_hash: vec![
                0x1f, 0x2e, 0x3d, 0x4c, 0x5b, 0x6a, 0x79, 0x88,
                0x97, 0xa6, 0xb5, 0xc4, 0xd3, 0xe2, 0xf1, 0x00,
                0x89, 0x67, 0x45, 0x23, 0x01, 0xef, 0xcd, 0xab,
                0xba, 0xdc, 0xfe, 0x10, 0x32, 0x54, 0x76, 0x98,
            ],
            target_difficulty: 1000000,   // Realistic Tari difficulty
            height: 400000,
        },
        expected_shares_per_mh: 1.0,
        description: "Realistic job - typical Tari network mining conditions".to_string(),
    }
}

/// Create a custom job with specified difficulty
pub fn create_custom_job(difficulty: u64, job_suffix: &str) -> BenchmarkJob {
    BenchmarkJob {
        mining_job: MiningJob {
            job_id: format!("bench_custom_{}", job_suffix),
            mining_hash: vec![
                0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe,
                0xfe, 0xed, 0xfa, 0xce, 0xd0, 0x0d, 0x00, 0x00,
                0x13, 0x37, 0x42, 0x69, 0x96, 0x24, 0x73, 0x31,
                0xc0, 0xff, 0xee, 0x15, 0x60, 0x0d, 0xf0, 0x0d,
            ],
            target_difficulty: difficulty,
            height: 500000,
        },
        expected_shares_per_mh: 1000000.0 / difficulty as f64,
        description: format!("Custom job - difficulty {} for specific testing", difficulty),
    }
}

/// Get a job by difficulty level for targeted testing
pub fn get_job_by_difficulty(difficulty: u64) -> BenchmarkJob {
    if difficulty <= 10000 {
        create_easy_job()
    } else if difficulty <= 1000000 {
        create_medium_job()
    } else if difficulty <= 10000000 {
        create_hard_job()
    } else {
        create_custom_job(difficulty, "extreme")
    }
}

/// Validate a benchmark job's expected metrics
pub fn validate_job_metrics(job: &BenchmarkJob, actual_shares: u64, total_hashes: u64) -> bool {
    if total_hashes == 0 {
        return false;
    }
    
    let actual_shares_per_mh = (actual_shares as f64 * 1_000_000.0) / total_hashes as f64;
    let tolerance = 0.5; // 50% tolerance for randomness
    let expected = job.expected_shares_per_mh;
    
    actual_shares_per_mh >= expected * (1.0 - tolerance) && 
    actual_shares_per_mh <= expected * (1.0 + tolerance)
}

// Changelog:
// - v1.0.0 (2025-06-14): Initial benchmark jobs implementation.
//   - Purpose: Provides static mining jobs for performance testing without
//     requiring pool connectivity, enabling isolated hash performance analysis.
//   - Features: Creates jobs with different difficulty levels (easy, medium,
//     hard, realistic), custom job creation, and validation metrics for
//     testing mining optimizations like batching and thread scaling.
//   - Note: Jobs use deterministic header templates to ensure reproducible
//     benchmark results across different test runs.