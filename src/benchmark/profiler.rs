// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/benchmark/profiler.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file provides performance monitoring and profiling utilities for the
// SHA3x miner benchmark system. It tracks memory usage, allocation patterns,
// and system-level performance metrics during benchmark execution.
//
// Tree Location:
// - src/benchmark/profiler.rs (performance monitoring utilities)
// - Depends on: std, sysinfo (optional)

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Performance metrics collected during benchmarking
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total memory allocations tracked
    pub allocations: u64,

    /// Peak memory usage in bytes
    pub peak_memory_usage: u64,

    /// Average memory usage in bytes
    pub avg_memory_usage: u64,

    /// CPU usage percentage (if available)
    pub cpu_usage: f64,

    /// Cache miss rate (if trackable)
    pub cache_miss_rate: f64,

    /// Context switches per second
    pub context_switches_per_sec: f64,

    /// System load average
    pub load_average: f64,
}

/// Real-time profiler data collection
pub struct ProfilerData {
    /// Allocation counter
    allocation_count: AtomicU64,

    /// Memory usage samples
    memory_samples: Arc<std::sync::Mutex<VecDeque<u64>>>,

    /// Start time for profiling session
    start_time: Instant,

    /// Peak memory usage observed
    peak_memory: AtomicU64,

    /// Maximum samples to keep in memory
    max_samples: usize,
}

impl ProfilerData {
    /// Create a new profiler data collector
    pub fn new() -> Self {
        Self {
            allocation_count: AtomicU64::new(0),
            memory_samples: Arc::new(std::sync::Mutex::new(VecDeque::new())),
            start_time: Instant::now(),
            peak_memory: AtomicU64::new(0),
            max_samples: 10000,
        }
    }

    /// Record a memory allocation
    pub fn record_allocation(&self, size: u64) {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        self.update_memory_usage(size);
    }

    /// Update current memory usage
    pub fn update_memory_usage(&self, current_usage: u64) {
        // Update peak memory
        let mut peak = self.peak_memory.load(Ordering::Relaxed);
        while current_usage > peak {
            match self.peak_memory.compare_exchange_weak(
                peak,
                current_usage,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }

        // Add sample to history
        if let Ok(mut samples) = self.memory_samples.lock() {
            samples.push_back(current_usage);

            // Keep only recent samples
            while samples.len() > self.max_samples {
                samples.pop_front();
            }
        }
    }

    /// Get current allocation count
    pub fn get_allocation_count(&self) -> Option<u64> {
        Some(self.allocation_count.load(Ordering::Relaxed))
    }

    /// Get peak memory usage
    pub fn get_peak_memory(&self) -> u64 {
        self.peak_memory.load(Ordering::Relaxed)
    }

    /// Calculate average memory usage
    pub fn get_average_memory(&self) -> u64 {
        if let Ok(samples) = self.memory_samples.lock() {
            if samples.is_empty() {
                return 0;
            }

            let sum: u64 = samples.iter().sum();
            sum / samples.len() as u64
        } else {
            0
        }
    }

    /// Get profiling duration
    pub fn get_duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Collect comprehensive performance metrics
    pub fn collect_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            allocations: self.allocation_count.load(Ordering::Relaxed),
            peak_memory_usage: self.get_peak_memory(),
            avg_memory_usage: self.get_average_memory(),
            cpu_usage: self.get_cpu_usage(),
            cache_miss_rate: 0.0,          // Would need hardware counters
            context_switches_per_sec: 0.0, // Would need system monitoring
            load_average: self.get_load_average(),
        }
    }

    /// Get current CPU usage (basic implementation)
    fn get_cpu_usage(&self) -> f64 {
        // Basic CPU usage - would need proper system monitoring for accuracy
        // For now, return a placeholder
        0.0
    }

    /// Get system load average (basic implementation)
    fn get_load_average(&self) -> f64 {
        // On Linux, we could read /proc/loadavg
        // For cross-platform compatibility, return placeholder
        0.0
    }
}

/// Memory allocation tracker for benchmarking
pub struct AllocationTracker {
    profiler: Arc<ProfilerData>,
    enabled: bool,
}

impl AllocationTracker {
    /// Create a new allocation tracker
    pub fn new(profiler: Arc<ProfilerData>) -> Self {
        Self {
            profiler,
            enabled: true,
        }
    }

    /// Enable/disable tracking
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Track an allocation
    pub fn track_allocation(&self, size: usize) {
        if self.enabled {
            self.profiler.record_allocation(size as u64);
        }
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.profiler.collect_metrics()
    }
}

/// System resource monitor
pub struct ResourceMonitor {
    profiler: Arc<ProfilerData>,
    monitoring: Arc<std::sync::atomic::AtomicBool>,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(profiler: Arc<ProfilerData>) -> Self {
        Self {
            profiler,
            monitoring: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start monitoring system resources
    pub fn start_monitoring(&self) {
        self.monitoring.store(true, Ordering::Relaxed);

        let profiler = Arc::clone(&self.profiler);
        let monitoring = Arc::clone(&self.monitoring);

        std::thread::spawn(move || {
            while monitoring.load(Ordering::Relaxed) {
                // Get current memory usage
                let memory_usage = Self::get_memory_usage();
                profiler.update_memory_usage(memory_usage);

                // Sleep for sampling interval
                std::thread::sleep(Duration::from_millis(100));
            }
        });
    }

    /// Stop monitoring
    pub fn stop_monitoring(&self) {
        self.monitoring.store(false, Ordering::Relaxed);
    }

    /// Get current memory usage of the process
    fn get_memory_usage() -> u64 {
        // Basic memory usage tracking
        // On Linux, could read /proc/self/statm
        // For now, return estimated usage based on allocations
        0
    }
}

/// Performance analysis utilities
pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    /// Compare two performance metrics
    pub fn compare_metrics(
        baseline: &PerformanceMetrics,
        current: &PerformanceMetrics,
    ) -> MetricsComparison {
        MetricsComparison {
            allocation_change: Self::calculate_percentage_change(
                baseline.allocations as f64,
                current.allocations as f64,
            ),
            memory_change: Self::calculate_percentage_change(
                baseline.peak_memory_usage as f64,
                current.peak_memory_usage as f64,
            ),
            cpu_change: current.cpu_usage - baseline.cpu_usage,
        }
    }

    /// Calculate percentage change between two values
    fn calculate_percentage_change(baseline: f64, current: f64) -> f64 {
        if baseline == 0.0 {
            return 0.0;
        }
        ((current - baseline) / baseline) * 100.0
    }

    /// Analyze allocation patterns
    pub fn analyze_allocations(metrics: &PerformanceMetrics) -> AllocationAnalysis {
        AllocationAnalysis {
            total_allocations: metrics.allocations,
            allocation_efficiency: if metrics.peak_memory_usage > 0 {
                metrics.allocations as f64 / metrics.peak_memory_usage as f64
            } else {
                0.0
            },
            memory_pressure: if metrics.peak_memory_usage > 1_000_000_000 {
                // 1GB
                "High".to_string()
            } else if metrics.peak_memory_usage > 100_000_000 {
                // 100MB
                "Medium".to_string()
            } else {
                "Low".to_string()
            },
        }
    }
}

/// Comparison results between two performance metrics
#[derive(Debug, Clone)]
pub struct MetricsComparison {
    /// Percentage change in allocations
    pub allocation_change: f64,

    /// Percentage change in memory usage
    pub memory_change: f64,

    /// Change in CPU usage
    pub cpu_change: f64,
}

/// Analysis of allocation patterns
#[derive(Debug, Clone)]
pub struct AllocationAnalysis {
    /// Total number of allocations
    pub total_allocations: u64,

    /// Allocations per byte of peak memory
    pub allocation_efficiency: f64,

    /// Memory pressure level
    pub memory_pressure: String,
}

impl Default for ProfilerData {
    fn default() -> Self {
        Self::new()
    }
}

// Changelog:
// - v1.0.0 (2025-06-14): Initial profiler implementation.
//   - Purpose: Provides performance monitoring and profiling utilities for
//     benchmark analysis, tracking memory usage, allocations, and system metrics.
//   - Features: Real-time data collection, allocation tracking, resource monitoring,
//     and performance analysis tools for optimization validation.
//   - Note: Includes basic system monitoring with placeholders for platform-specific
//     implementations, enabling cross-platform benchmark profiling.
