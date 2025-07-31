// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: tests/bench_hashing.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file contains core hash function performance tests for the SHA3x miner.
// It validates the SHA3x implementation performance, correctness, and
// consistency across different inputs and conditions.
//
// Tree Location:
// - tests/bench_hashing.rs (hash function performance tests)
// - Depends on: core/sha3x, core/difficulty, sha3 crate

use graxil::core::{calculate_difficulty, sha3x::*};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_sha3x_performance_baseline() {
    println!("🧪 Testing SHA3x Hash Performance Baseline");
    println!("==========================================");

    let test_cases = vec![
        ("Empty header", vec![0u8; 32]),
        ("Pattern header", {
            let mut pattern = Vec::new();
            for _ in 0..4 {
                pattern.extend_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]);
            }
            pattern
        }),
        (
            "Random header",
            vec![
                0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18, 0x29, 0x3a, 0x4b, 0x5c, 0x6d, 0x7e,
                0x8f, 0x90, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x0f, 0xed, 0xcb, 0xa9,
                0x87, 0x65, 0x43, 0x21,
            ],
        ),
    ];

    let iterations = 10000;
    let start_nonce = 12345u64;

    println!("📊 Testing {} iterations per case...", iterations);
    println!();

    for (name, header) in test_cases {
        println!("Testing {}: ", name);

        let start_time = Instant::now();
        let mut hash_count = 0;

        for i in 0..iterations {
            let nonce = start_nonce + i as u64;
            let _hash = sha3x_hash_with_nonce(&header, nonce);
            hash_count += 1;
        }

        let duration = start_time.elapsed();
        let hashrate = hash_count as f64 / duration.as_secs_f64();

        println!(
            "  {} hashes in {:.3}s = {:.2} H/s",
            hash_count,
            duration.as_secs_f64(),
            hashrate
        );
    }

    println!();
    println!("💡 Baseline Performance Notes:");
    println!("  - SHA3x = triple SHA3-256 (3 hash operations per call)");
    println!("  - Input: 8-byte nonce + 32-byte header + 1-byte marker");
    println!("  - Output: 32-byte hash for difficulty calculation");
    println!("  - Performance scales with CPU single-thread speed");
}

#[test]
fn test_sha3x_correctness() {
    println!("🧪 Testing SHA3x Hash Correctness");
    println!("==================================");

    let test_header = vec![
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        0x00, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
        0x32, 0x10,
    ];

    // Test that SHA3x produces consistent results
    println!("📊 Testing hash consistency...");

    let test_nonce = 98765u64;
    let hash1 = sha3x_hash_with_nonce(&test_header, test_nonce);
    let hash2 = sha3x_hash_with_nonce(&test_header, test_nonce);

    assert_eq!(hash1, hash2, "Same input should produce same hash");
    println!("  ✅ Hash consistency verified");

    // Test that different nonces produce different hashes
    println!("📊 Testing nonce sensitivity...");

    let hash_a = sha3x_hash_with_nonce(&test_header, 1000);
    let hash_b = sha3x_hash_with_nonce(&test_header, 1001);

    assert_ne!(
        hash_a, hash_b,
        "Different nonces should produce different hashes"
    );
    println!("  ✅ Nonce sensitivity verified");

    // Test that different headers produce different hashes
    println!("📊 Testing header sensitivity...");

    let mut alt_header = test_header.clone();
    alt_header[0] = alt_header[0].wrapping_add(1);

    let hash_orig = sha3x_hash_with_nonce(&test_header, test_nonce);
    let hash_alt = sha3x_hash_with_nonce(&alt_header, test_nonce);

    assert_ne!(
        hash_orig, hash_alt,
        "Different headers should produce different hashes"
    );
    println!("  ✅ Header sensitivity verified");

    // Verify the SHA3x algorithm implementation manually
    println!("📊 Testing SHA3x algorithm implementation...");

    let manual_result = {
        let mut input = Vec::with_capacity(test_header.len() + 9);
        input.extend_from_slice(&test_nonce.to_le_bytes());
        input.extend_from_slice(&test_header);
        input.push(1u8);

        let hash1 = Sha3_256::digest(&input);
        let hash2 = Sha3_256::digest(&hash1);
        let hash3 = Sha3_256::digest(&hash2);
        hash3.to_vec()
    };

    let function_result = sha3x_hash_with_nonce(&test_header, test_nonce);

    assert_eq!(
        manual_result, function_result,
        "Function should match manual implementation"
    );
    println!("  ✅ SHA3x algorithm implementation verified");

    println!();
    println!("💡 Correctness Summary:");
    println!("  - Hash output is deterministic and consistent");
    println!("  - Small input changes produce completely different outputs");
    println!("  - Implementation matches manual SHA3x calculation");
    println!("  - Ready for mining operations");
}

#[test]
fn test_difficulty_calculation_performance() {
    println!("🧪 Testing Difficulty Calculation Performance");
    println!("==============================================");

    let test_header = vec![0x42u8; 32];
    let iterations = 20000;

    println!("📊 Generating {} test hashes...", iterations);

    // Generate test hashes first
    let mut test_hashes = Vec::new();
    for i in 0..iterations {
        let hash = sha3x_hash_with_nonce(&test_header, i as u64);
        test_hashes.push(hash);
    }

    println!("📊 Testing difficulty calculation performance...");

    let start_time = Instant::now();
    let mut difficulty_sum = 0u64;

    for hash in &test_hashes {
        let difficulty = calculate_difficulty(hash);
        difficulty_sum = difficulty_sum.wrapping_add(difficulty);
    }

    let duration = start_time.elapsed();
    let calc_rate = iterations as f64 / duration.as_secs_f64();

    println!(
        "  {} difficulty calculations in {:.3}s = {:.2} calc/s",
        iterations,
        duration.as_secs_f64(),
        calc_rate
    );
    println!(
        "  Average difficulty: {:.2}",
        difficulty_sum as f64 / iterations as f64
    );

    // Test difficulty distribution
    println!("📊 Analyzing difficulty distribution...");

    let mut difficulty_ranges = HashMap::new();
    let ranges = vec![
        (0, 1000, "Very Low"),
        (1000, 10000, "Low"),
        (10000, 100000, "Medium"),
        (100000, 1000000, "High"),
        (1000000, u64::MAX, "Very High"),
    ];

    for hash in &test_hashes {
        let difficulty = calculate_difficulty(hash);

        for &(min, max, label) in &ranges {
            if difficulty >= min && difficulty < max {
                *difficulty_ranges.entry(label).or_insert(0) += 1;
                break;
            }
        }
    }

    println!("  Difficulty distribution:");
    for &(_, _, label) in &ranges {
        let count = difficulty_ranges.get(label).unwrap_or(&0);
        let percentage = (*count as f64 / iterations as f64) * 100.0;
        println!("    {}: {} ({:.1}%)", label, count, percentage);
    }

    println!();
    println!("💡 Difficulty Calculation Notes:");
    println!("  - Difficulty calculation is very fast (pure arithmetic)");
    println!("  - Distribution should be roughly exponential");
    println!("  - Higher difficulties are exponentially rarer");
    println!("  - Mining finds hashes meeting target difficulty");

    // Sanity check - should be fast
    assert!(
        calc_rate > 100000.0,
        "Difficulty calculation should be very fast"
    );
}

#[test]
fn test_hash_distribution_quality() {
    println!("🧪 Testing Hash Distribution Quality");
    println!("====================================");

    let test_header = vec![
        0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0xfe, 0xed, 0xfa, 0xce, 0xd0, 0x0d, 0x00,
        0x00, 0x13, 0x37, 0x42, 0x69, 0x96, 0x24, 0x73, 0x31, 0xc0, 0xff, 0xee, 0x15, 0x60, 0x0d,
        0xf0, 0x0d,
    ];

    let sample_size = 10000;

    println!(
        "📊 Analyzing hash distribution with {} samples...",
        sample_size
    );

    // Collect hash samples
    let mut byte_counts = vec![vec![0usize; 256]; 32]; // 32 bytes, 256 possible values each
    let mut first_byte_zero_count = 0;

    for i in 0..sample_size {
        let hash = sha3x_hash_with_nonce(&test_header, i as u64);

        // Count byte value distribution
        for (byte_pos, &byte_val) in hash.iter().enumerate() {
            if byte_pos < 32 {
                byte_counts[byte_pos][byte_val as usize] += 1;
            }
        }

        // Count leading zero bytes (for difficulty analysis)
        if hash[0] == 0 {
            first_byte_zero_count += 1;
        }
    }

    // Analyze first byte distribution (most important for difficulty)
    println!("📊 First byte distribution analysis:");

    let expected_per_value = sample_size as f64 / 256.0;
    let mut chi_square = 0.0;
    let mut min_count = usize::MAX;
    let mut max_count = 0;

    for &count in &byte_counts[0] {
        let diff = count as f64 - expected_per_value;
        chi_square += (diff * diff) / expected_per_value;
        min_count = min_count.min(count);
        max_count = max_count.max(count);
    }

    println!("  Expected count per byte value: {:.1}", expected_per_value);
    println!("  Actual range: {} to {}", min_count, max_count);
    println!("  Chi-square statistic: {:.2}", chi_square);
    println!(
        "  First byte zero count: {} ({:.2}%)",
        first_byte_zero_count,
        (first_byte_zero_count as f64 / sample_size as f64) * 100.0
    );

    // Test for reasonable distribution
    let max_deviation = (max_count as f64 - expected_per_value).abs();
    let _min_deviation = (expected_per_value - min_count as f64).abs();
    let max_relative_deviation = (max_deviation / expected_per_value) * 100.0;

    println!(
        "  Maximum relative deviation: {:.1}%",
        max_relative_deviation
    );

    // Check avalanche effect (bit change propagation)
    println!("📊 Testing avalanche effect...");

    let base_nonce = 12345u64;
    let base_hash = sha3x_hash_with_nonce(&test_header, base_nonce);

    let mut bit_flip_differences = 0;
    let test_count = 100;

    for i in 0..test_count {
        let flipped_nonce = base_nonce ^ (1u64 << (i % 64)); // Flip one bit
        let flipped_hash = sha3x_hash_with_nonce(&test_header, flipped_nonce);

        // Count different bits
        for (b1, b2) in base_hash.iter().zip(flipped_hash.iter()) {
            bit_flip_differences += (b1 ^ b2).count_ones() as usize;
        }
    }

    let avg_bit_flips = bit_flip_differences as f64 / test_count as f64;
    let expected_bit_flips = 256.0 / 2.0; // ~50% of 256 bits should flip

    println!(
        "  Average bit flips per input bit change: {:.1}",
        avg_bit_flips
    );
    println!("  Expected (good avalanche): ~{:.1}", expected_bit_flips);

    println!();
    println!("💡 Hash Quality Summary:");

    if max_relative_deviation < 20.0 {
        println!("  ✅ Byte distribution is reasonably uniform");
    } else {
        println!("  ⚠️  Byte distribution shows some bias");
    }

    if avg_bit_flips > 100.0 && avg_bit_flips < 200.0 {
        println!("  ✅ Avalanche effect is good");
    } else {
        println!("  ⚠️  Avalanche effect may be suboptimal");
    }

    let zero_byte_percentage = (first_byte_zero_count as f64 / sample_size as f64) * 100.0;
    if zero_byte_percentage > 0.2 && zero_byte_percentage < 0.6 {
        println!("  ✅ Leading zero distribution looks normal");
    } else {
        println!("  ⚠️  Leading zero distribution is unusual");
    }

    println!("  - Hash function appears suitable for mining");
    println!("  - Distribution supports fair difficulty calculation");

    // Basic sanity checks
    assert!(max_relative_deviation < 50.0, "Distribution too biased");
    assert!(avg_bit_flips > 50.0, "Avalanche effect too weak");
}

#[test]
fn test_performance_consistency() {
    println!("🧪 Testing Hash Performance Consistency");
    println!("========================================");

    let test_header = vec![0x5a; 32];
    let iterations_per_run = 5000;
    let num_runs = 10;

    println!(
        "📊 Running {} performance tests of {} hashes each...",
        num_runs, iterations_per_run
    );

    let mut hashrates = Vec::new();

    for run in 0..num_runs {
        let start_time = Instant::now();

        for i in 0..iterations_per_run {
            let nonce = (run * iterations_per_run + i) as u64;
            let _hash = sha3x_hash_with_nonce(&test_header, nonce);
        }

        let duration = start_time.elapsed();
        let hashrate = iterations_per_run as f64 / duration.as_secs_f64();
        hashrates.push(hashrate);

        println!("  Run {}: {:.2} H/s", run + 1, hashrate);
    }

    // Calculate statistics
    let sum: f64 = hashrates.iter().sum();
    let mean = sum / hashrates.len() as f64;

    let variance = hashrates
        .iter()
        .map(|rate| (rate - mean).powi(2))
        .sum::<f64>()
        / hashrates.len() as f64;
    let std_dev = variance.sqrt();

    let min_rate = hashrates.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_rate = hashrates.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let coefficient_of_variation = (std_dev / mean) * 100.0;

    println!();
    println!("📊 PERFORMANCE STATISTICS:");
    println!("  Mean hashrate: {:.2} H/s", mean);
    println!("  Standard deviation: {:.2} H/s", std_dev);
    println!("  Min hashrate: {:.2} H/s", min_rate);
    println!("  Max hashrate: {:.2} H/s", max_rate);
    println!(
        "  Range: {:.2} H/s ({:.1}%)",
        max_rate - min_rate,
        ((max_rate - min_rate) / mean) * 100.0
    );
    println!(
        "  Coefficient of variation: {:.1}%",
        coefficient_of_variation
    );

    println!();
    println!("💡 Consistency Analysis:");

    if coefficient_of_variation < 5.0 {
        println!("  ✅ Performance is very consistent (CV < 5%)");
    } else if coefficient_of_variation < 10.0 {
        println!("  ✅ Performance is reasonably consistent (CV < 10%)");
    } else {
        println!("  ⚠️  Performance shows significant variation (CV > 10%)");
        println!("     This may indicate system load or thermal throttling");
    }

    println!("  - Consistent performance is important for mining");
    println!("  - Monitor CPU temperature during extended mining");
    println!("  - Consider system load and background processes");

    // Reasonable consistency check
    assert!(
        coefficient_of_variation < 25.0,
        "Performance too inconsistent"
    );
}

// Changelog:
// - v1.0.0 (2025-06-15): Initial hash function performance tests implementation.
//   - Purpose: Validates SHA3x hash function performance, correctness, and
//     distribution quality for mining operations.
//   - Features: Performance baselines, correctness verification, difficulty
//     calculation testing, hash distribution analysis, and consistency testing.
//   - Note: These tests ensure the core hashing functionality meets performance
//     and quality requirements for effective mining operations.
