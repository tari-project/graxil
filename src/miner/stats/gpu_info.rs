// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/miner/stats/gpu_info.rs
// Version: 1.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This module provides GPU detection and monitoring capabilities for the SHA3x miner.
// It supports NVIDIA GPUs through nvidia-smi command-line interface with graceful
// fallback when no GPU is detected or nvidia-smi is unavailable.
//
// Features:
// - NVIDIA GPU detection via nvidia-smi
// - Real-time GPU metrics (utilization, temperature, power, memory)
// - Graceful error handling for missing drivers or hardware
// - Format utilities for dashboard display
// - Extensible architecture for future AMD/Intel GPU support

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const LOG_TARGET: &str = "tari::graxil::gpu_info";

/// GPU information structure for dashboard and monitoring
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct GpuInfo {
    /// Whether any GPU was detected
    pub detected: bool,
    /// GPU model name (e.g., "NVIDIA GeForce RTX 4090")
    pub name: String,
    /// Driver version if available
    pub driver_version: Option<String>,
    /// Current GPU temperature in Celsius
    pub temperature: Option<f32>,
    /// Current power consumption in Watts
    pub power_usage: Option<f32>,
    /// Used GPU memory in MB
    pub memory_used: Option<u64>,
    /// Total GPU memory in MB
    pub memory_total: Option<u64>,
    /// GPU utilization percentage (0-100)
    pub utilization: Option<f32>,
    /// Number of GPUs detected
    pub count: usize,
    /// GPU vendor (NVIDIA, AMD, Intel, Unknown)
    pub vendor: GpuVendor,
    /// Error message if detection failed
    pub error_message: Option<String>,
}

/// GPU vendor enumeration
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GpuVendor {
    NVIDIA,
    AMD,
    Intel,
    Unknown,
}

impl GpuVendor {
    pub fn as_str(&self) -> &'static str {
        match self {
            GpuVendor::NVIDIA => "NVIDIA",
            GpuVendor::AMD => "AMD",
            GpuVendor::Intel => "Intel",
            GpuVendor::Unknown => "Unknown",
        }
    }

    pub fn from_str(string_name: &str) -> Self {
        let sanitized_name = string_name.to_lowercase();
        let sanitized_name = sanitized_name.trim().to_string();
        if sanitized_name.contains("nvidia") {
            GpuVendor::NVIDIA
        } else if sanitized_name.contains("amd") {
            GpuVendor::AMD
        } else if sanitized_name.contains("intel") {
            GpuVendor::Intel
        } else {
            GpuVendor::Unknown
        }
    }
}

/// GPU detection result for better error handling
#[derive(Debug)]
pub enum GpuDetectionResult {
    Success(GpuInfo),
    NoGpu,
    DriverMissing(String),
    CommandFailed(String),
    ParseError(String),
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            detected: false,
            name: "Not detected".to_string(),
            driver_version: None,
            temperature: None,
            power_usage: None,
            memory_used: None,
            memory_total: None,
            utilization: None,
            count: 0,
            vendor: GpuVendor::Unknown,
            error_message: None,
        }
    }
}

impl GpuInfo {
    /// Create a new GPU info instance with immediate detection
    pub fn new() -> Self {
        Self::detect()
    }

    /// Create a thread-safe GPU monitor for periodic updates
    pub fn new_monitor() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::detect()))
    }

    /// Refresh GPU information (call this periodically)
    pub fn refresh(&mut self) {
        *self = Self::detect();
    }

    /// Detect and gather GPU information from available sources
    pub fn detect() -> Self {
        debug!(target: LOG_TARGET,"Starting GPU detection...");

        // Try NVIDIA first (most common for mining)
        match Self::detect_nvidia() {
            GpuDetectionResult::Success(gpu_info) => {
                info!(target: LOG_TARGET,"NVIDIA GPU detected: {}", gpu_info.name);
                return gpu_info;
            }
            GpuDetectionResult::NoGpu => {
                debug!(target: LOG_TARGET,"No NVIDIA GPU detected");
            }
            GpuDetectionResult::DriverMissing(msg) => {
                debug!(target: LOG_TARGET,"NVIDIA driver issue: {}", msg);
            }
            GpuDetectionResult::CommandFailed(msg) => {
                debug!(target: LOG_TARGET,"nvidia-smi command failed: {}", msg);
            }
            GpuDetectionResult::ParseError(msg) => {
                warn!(target: LOG_TARGET,"Failed to parse nvidia-smi output: {}", msg);
            }
        }

        // Try AMD (future implementation)
        match Self::detect_amd() {
            GpuDetectionResult::Success(gpu_info) => {
                info!(target: LOG_TARGET,"AMD GPU detected: {}", gpu_info.name);
                return gpu_info;
            }
            _ => debug!(target: LOG_TARGET,"No AMD GPU detected"),
        }

        // Try Intel (future implementation)
        match Self::detect_intel() {
            GpuDetectionResult::Success(gpu_info) => {
                info!(target: LOG_TARGET,"Intel GPU detected: {}", gpu_info.name);
                return gpu_info;
            }
            _ => debug!(target: LOG_TARGET,"No Intel GPU detected"),
        }

        // No GPU found
        debug!(target: LOG_TARGET,"No compatible GPU detected");
        Self::default()
    }

    /// Detect NVIDIA GPU using nvidia-smi command
    fn detect_nvidia() -> GpuDetectionResult {
        // Check if nvidia-smi is available
        let output = match Command::new("nvidia-smi")
            .arg("--query-gpu=name,driver_version,temperature.gpu,power.draw,memory.used,memory.total,utilization.gpu")
            .arg("--format=csv,noheader,nounits")
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                debug!(target: LOG_TARGET,"nvidia-smi command not found: {}", e);
                return GpuDetectionResult::DriverMissing(format!("nvidia-smi not available: {}", e));
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return GpuDetectionResult::CommandFailed(format!("nvidia-smi failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout
            .trim()
            .split('\n')
            .filter(|line| !line.trim().is_empty())
            .collect();

        if lines.is_empty() {
            return GpuDetectionResult::NoGpu;
        }

        // Parse first GPU (primary GPU for mining)
        match Self::parse_nvidia_smi_line(lines[0]) {
            Ok(gpu_data) => GpuDetectionResult::Success(GpuInfo {
                detected: true,
                name: gpu_data.0,
                driver_version: Some(gpu_data.1),
                temperature: gpu_data.2,
                power_usage: gpu_data.3,
                memory_used: gpu_data.4,
                memory_total: gpu_data.5,
                utilization: gpu_data.6,
                count: lines.len(),
                vendor: GpuVendor::NVIDIA,
                error_message: None,
            }),
            Err(e) => GpuDetectionResult::ParseError(e),
        }
    }

    /// Detect AMD GPU (placeholder for future implementation)
    fn detect_amd() -> GpuDetectionResult {
        // Future: Use rocm-smi or similar for AMD GPU detection
        // For now, return NoGpu to allow fallback
        GpuDetectionResult::NoGpu
    }

    /// Detect Intel GPU (placeholder for future implementation)
    fn detect_intel() -> GpuDetectionResult {
        // Future: Use intel-gpu-top or similar for Intel GPU detection
        // For now, return NoGpu to allow fallback
        GpuDetectionResult::NoGpu
    }

    /// Parse a single line of nvidia-smi CSV output
    /// Expected format: "NVIDIA GeForce RTX 4090, 535.104.05, 65, 350.2, 8192, 24576, 85"
    fn parse_nvidia_smi_line(
        line: &str,
    ) -> Result<
        (
            String,
            String,
            Option<f32>,
            Option<f32>,
            Option<u64>,
            Option<u64>,
            Option<f32>,
        ),
        String,
    > {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        if parts.len() < 7 {
            return Err(format!(
                "Invalid nvidia-smi output format, expected 7 fields but got {}: {}",
                parts.len(),
                line
            ));
        }

        let name = parts[0].to_string();
        let driver_version = parts[1].to_string();

        // Parse numeric values with comprehensive error handling
        let temperature = Self::parse_optional_float(parts[2], "temperature")?;
        let power_usage = Self::parse_optional_float(parts[3], "power")?;
        let memory_used = Self::parse_optional_u64(parts[4], "memory_used")?;
        let memory_total = Self::parse_optional_u64(parts[5], "memory_total")?;
        let utilization = Self::parse_optional_float(parts[6], "utilization")?;

        Ok((
            name,
            driver_version,
            temperature,
            power_usage,
            memory_used,
            memory_total,
            utilization,
        ))
    }

    /// Parse optional float value with field name for better error messages
    fn parse_optional_float(value: &str, field_name: &str) -> Result<Option<f32>, String> {
        if value == "N/A"
            || value == "[Not Supported]"
            || value.is_empty()
            || value == "[Unknown Error]"
        {
            Ok(None)
        } else {
            value
                .parse::<f32>()
                .map(Some)
                .map_err(|e| format!("Failed to parse {} '{}': {}", field_name, value, e))
        }
    }

    /// Parse optional u64 value with field name for better error messages
    fn parse_optional_u64(value: &str, field_name: &str) -> Result<Option<u64>, String> {
        if value == "N/A"
            || value == "[Not Supported]"
            || value.is_empty()
            || value == "[Unknown Error]"
        {
            Ok(None)
        } else {
            value
                .parse::<u64>()
                .map(Some)
                .map_err(|e| format!("Failed to parse {} '{}': {}", field_name, value, e))
        }
    }

    /// Format memory usage as "used / total GB"
    pub fn format_memory(&self) -> String {
        match (self.memory_used, self.memory_total) {
            (Some(used), Some(total)) => {
                format!(
                    "{:.1} / {:.1} GB",
                    used as f32 / 1024.0,
                    total as f32 / 1024.0
                )
            }
            _ => "-- / -- GB".to_string(),
        }
    }

    /// Format memory usage percentage
    pub fn format_memory_usage(&self) -> String {
        match (self.memory_used, self.memory_total) {
            (Some(used), Some(total)) if total > 0 => {
                format!("{:.1}%", (used as f32 / total as f32) * 100.0)
            }
            _ => "-- %".to_string(),
        }
    }

    /// Format temperature with unit
    pub fn format_temperature(&self) -> String {
        match self.temperature {
            Some(temp) => format!("{:.0}°C", temp),
            None => "--°C".to_string(),
        }
    }

    /// Format power usage with unit
    pub fn format_power(&self) -> String {
        match self.power_usage {
            Some(power) => format!("{:.0} W", power),
            None => "-- W".to_string(),
        }
    }

    /// Format utilization percentage
    pub fn format_utilization(&self) -> String {
        match self.utilization {
            Some(util) => format!("{:.0}%", util),
            None => "--".to_string(),
        }
    }

    /// Check if this is a valid GPU detection with usable data
    pub fn is_available(&self) -> bool {
        self.detected && self.name != "Not detected" && self.vendor != GpuVendor::Unknown
    }

    /// Check if GPU is currently under load (for mining detection)
    pub fn is_under_load(&self) -> bool {
        match self.utilization {
            Some(util) => util > 80.0, // Consider >80% as under load
            None => false,
        }
    }

    /// Get a human-readable status string for console display
    pub fn get_status_string(&self) -> String {
        if !self.is_available() {
            return "No GPU detected".to_string();
        }

        let mut status_parts = vec![self.name.clone()];

        if let Some(util) = self.utilization {
            status_parts.push(format!("{}% load", util as u8));
        }

        if let Some(temp) = self.temperature {
            status_parts.push(format!("{}°C", temp as u8));
        }

        if let Some(power) = self.power_usage {
            status_parts.push(format!("{}W", power as u16));
        }

        status_parts.join(" | ")
    }

    /// Get GPU memory pressure indicator
    pub fn get_memory_pressure(&self) -> String {
        match (self.memory_used, self.memory_total) {
            (Some(used), Some(total)) if total > 0 => {
                let usage_percent = (used as f32 / total as f32) * 100.0;
                match usage_percent {
                    p if p >= 90.0 => "🔴 High".to_string(),
                    p if p >= 70.0 => "🟡 Medium".to_string(),
                    _ => "🟢 Low".to_string(),
                }
            }
            _ => "❓ Unknown".to_string(),
        }
    }

    /// Check if GPU temperature is in safe range
    pub fn is_temperature_safe(&self) -> bool {
        match self.temperature {
            Some(temp) => temp < 85.0, // Consider 85°C as safe threshold
            None => true,              // Unknown temperature assumed safe
        }
    }

    /// Get thermal status indicator
    pub fn get_thermal_status(&self) -> String {
        match self.temperature {
            Some(temp) => match temp {
                t if t >= 90.0 => "🔥 Hot".to_string(),
                t if t >= 80.0 => "🟡 Warm".to_string(),
                _ => "❄️ Cool".to_string(),
            },
            None => "❓ Unknown".to_string(),
        }
    }
}

/// GPU Monitor for periodic updates
pub struct GpuMonitor {
    gpu_info: Arc<Mutex<GpuInfo>>,
    last_update: Arc<Mutex<Instant>>,
    update_interval: Duration,
}

impl GpuMonitor {
    /// Create a new GPU monitor with specified update interval
    pub fn new(update_interval: Duration) -> Self {
        Self {
            gpu_info: Arc::new(Mutex::new(GpuInfo::detect())),
            last_update: Arc::new(Mutex::new(Instant::now())),
            update_interval,
        }
    }

    /// Create a GPU monitor with default 5-second update interval
    pub fn new_default() -> Self {
        Self::new(Duration::from_secs(5))
    }

    /// Get current GPU info (updates if needed)
    pub fn get_info(&self) -> GpuInfo {
        self.update_if_needed();
        self.gpu_info.lock().unwrap().clone()
    }

    /// Force update GPU information
    pub fn force_update(&self) {
        let mut gpu_info = self.gpu_info.lock().unwrap();
        gpu_info.refresh();
        *self.last_update.lock().unwrap() = Instant::now();

        if gpu_info.is_available() {
            debug!(target: LOG_TARGET,"GPU monitor updated: {}", gpu_info.get_status_string());
        }
    }

    /// Update GPU info if enough time has passed
    fn update_if_needed(&self) {
        let last_update = *self.last_update.lock().unwrap();
        if last_update.elapsed() >= self.update_interval {
            self.force_update();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nvidia_smi_line_valid() {
        let test_line = "NVIDIA GeForce RTX 4090, 535.104.05, 65, 350.2, 8192, 24576, 85";
        let result = GpuInfo::parse_nvidia_smi_line(test_line);

        assert!(result.is_ok());
        let (name, driver, temp, power, mem_used, mem_total, util) = result.unwrap();

        assert_eq!(name, "NVIDIA GeForce RTX 4090");
        assert_eq!(driver, "535.104.05");
        assert_eq!(temp, Some(65.0));
        assert_eq!(power, Some(350.2));
        assert_eq!(mem_used, Some(8192));
        assert_eq!(mem_total, Some(24576));
        assert_eq!(util, Some(85.0));
    }

    #[test]
    fn test_parse_nvidia_smi_line_with_na_values() {
        let test_line = "NVIDIA GeForce GTX 1060, 470.161.03, N/A, [Not Supported], 2048, 6144, 45";
        let result = GpuInfo::parse_nvidia_smi_line(test_line);

        assert!(result.is_ok());
        let (name, driver, temp, power, mem_used, mem_total, util) = result.unwrap();

        assert_eq!(name, "NVIDIA GeForce GTX 1060");
        assert_eq!(driver, "470.161.03");
        assert_eq!(temp, None);
        assert_eq!(power, None);
        assert_eq!(mem_used, Some(2048));
        assert_eq!(mem_total, Some(6144));
        assert_eq!(util, Some(45.0));
    }

    #[test]
    fn test_parse_nvidia_smi_line_invalid_format() {
        let test_line = "NVIDIA GeForce RTX 4090, 535.104.05"; // Too few fields
        let result = GpuInfo::parse_nvidia_smi_line(test_line);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected 7 fields"));
    }

    #[test]
    fn test_default_gpu_info() {
        let gpu = GpuInfo::default();
        assert!(!gpu.detected);
        assert_eq!(gpu.name, "Not detected");
        assert_eq!(gpu.count, 0);
        assert_eq!(gpu.vendor, GpuVendor::Unknown);
        assert!(!gpu.is_available());
    }

    #[test]
    fn test_format_methods() {
        let gpu = GpuInfo {
            detected: true,
            name: "Test GPU".to_string(),
            driver_version: Some("123.45".to_string()),
            temperature: Some(65.7),
            power_usage: Some(250.8),
            memory_used: Some(4096),
            memory_total: Some(8192),
            utilization: Some(87.3),
            count: 1,
            vendor: GpuVendor::NVIDIA,
            error_message: None,
        };

        assert_eq!(gpu.format_memory(), "4.0 / 8.0 GB");
        assert_eq!(gpu.format_memory_usage(), "50.0%");
        assert_eq!(gpu.format_temperature(), "66°C");
        assert_eq!(gpu.format_power(), "251 W");
        assert_eq!(gpu.format_utilization(), "87%");
        assert!(gpu.is_available());
        assert!(gpu.is_under_load());
        assert!(gpu.is_temperature_safe());
    }

    #[test]
    fn test_status_indicators() {
        let gpu = GpuInfo {
            detected: true,
            name: "RTX 4090".to_string(),
            driver_version: Some("535.104.05".to_string()),
            temperature: Some(75.0),
            power_usage: Some(300.0),
            memory_used: Some(16384),
            memory_total: Some(24576),
            utilization: Some(95.0),
            count: 1,
            vendor: GpuVendor::NVIDIA,
            error_message: None,
        };

        assert_eq!(gpu.get_memory_pressure(), "🟡 Medium");
        assert_eq!(gpu.get_thermal_status(), "❄️ Cool");
        assert!(gpu.is_under_load());
        assert!(gpu.is_temperature_safe());
    }

    #[test]
    fn test_gpu_monitor() {
        let monitor = GpuMonitor::new_default();
        let info = monitor.get_info();

        // Should have some default values
        assert!(info.vendor == GpuVendor::Unknown || info.vendor == GpuVendor::NVIDIA);
    }
}

// Changelog:
// - v1.0.0 (2025-06-24): Initial implementation
//   - Comprehensive NVIDIA GPU detection via nvidia-smi
//   - Robust error handling and parsing
//   - Multiple format utilities for dashboard display
//   - Extensible architecture for AMD/Intel GPU support
//   - GPU monitor for periodic updates
//   - Status indicators and safety checks
//   - Comprehensive test coverage
