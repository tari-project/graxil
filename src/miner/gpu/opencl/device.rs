// SHA3x Miner - Free and Open Source Software Statement
//
// File: src/miner/gpu/opencl/device.rs
// Version: 1.0.1
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// OpenCL device management for GPU mining - detects and manages OpenCL devices

use anyhow::{Error, Result};
use log::{debug, error, info, warn};
use opencl3::{
    device::{CL_DEVICE_TYPE_GPU, Device},
    platform::get_platforms,
};
use serde::{Deserialize, Serialize};
const LOG_TARGET: &str = "tari::graxil::device";
/// GPU device type classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GpuDeviceType {
    Integrated,
    Dedicated,
    Unknown,
}

/// OpenCL device information
#[derive(Debug, Clone)]
pub struct OpenClDevice {
    pub name: String,
    pub device_id: u32,
    pub platform_name: String,
    pub max_work_group_size: usize,
    pub max_compute_units: u32,
    pub global_mem_size: u64,
    pub device_type: GpuDeviceType,
    pub device: Device,
}

impl OpenClDevice {
    /// Create a new OpenCL device
    pub fn new(device: Device, device_id: u32, platform_name: String) -> Result<Self> {
        let name = device
            .name()
            .unwrap_or_else(|_| "Unknown Device".to_string());
        let max_work_group_size = device.max_work_group_size().unwrap_or(256);
        let max_compute_units = device.max_compute_units().unwrap_or(1);
        let global_mem_size = device.global_mem_size().unwrap_or(0);

        debug!(target: LOG_TARGET,
            "Created OpenCL device: {} (CU: {}, WG: {})",
            name, max_compute_units, max_work_group_size
        );

        let device_type = Self::detect_device_type(&device, &name, global_mem_size);

        Ok(Self {
            name,
            device_id,
            platform_name,
            max_work_group_size,
            max_compute_units,
            global_mem_size,
            device_type,
            device,
        })
    }

    /// Get device name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get device ID
    pub fn device_id(&self) -> u32 {
        self.device_id
    }

    /// Get platform name
    pub fn platform_name(&self) -> &str {
        &self.platform_name
    }

    /// Get maximum work group size
    pub fn max_work_group_size(&self) -> usize {
        self.max_work_group_size
    }

    /// Get maximum compute units
    pub fn max_compute_units(&self) -> u32 {
        self.max_compute_units
    }

    /// Get global memory size in bytes
    pub fn global_mem_size(&self) -> u64 {
        self.global_mem_size
    }

    /// Get the underlying OpenCL device
    pub fn device(&self) -> &Device {
        &self.device
    }
    /// Get device type (integrated/dedicated/Unknown)
    pub fn device_type(&self) -> &GpuDeviceType {
        &self.device_type
    }

    /// Detect if GPU device is integrated or dedicated
    fn detect_device_type(device: &Device, name: &str, global_mem_size: u64) -> GpuDeviceType {
        // Method 1: Check host unified memory (most reliable)
        if let Ok(host_unified) = device.host_unified_memory() {
            if host_unified {
                debug!(target: LOG_TARGET,
                    "Device {} detected as integrated (host unified memory)",
                    name
                );
                return GpuDeviceType::Integrated;
            }
        }

        // Method 3: Memory size heuristics
        // Integrated GPUs typically have smaller memory allocations
        // or share system memory (usually < 2GB dedicated)
        let mem_gb = global_mem_size as f64 / (1024.0 * 1024.0 * 1024.0);
        if mem_gb < 2.0 && mem_gb > 0.0 {
            debug!(target: LOG_TARGET,
                "Device {} likely integrated (small memory size: {:.1} GB)",
                name, mem_gb
            );
            return GpuDeviceType::Integrated;
        }

        // Method 4: Device name pattern matching
        let name_lower = name.to_lowercase();

        // Common integrated GPU patterns
        let integrated_patterns = [
            "intel hd graphics",
            "intel uhd graphics",
            "intel iris",
            "intel arc", // Some Intel Arc are integrated
            "amd radeon vega",
            "amd radeon graphics", // APU graphics
            "apple m1",
            "apple m2",
            "apple m3",
            "mali",
            "adreno",
            "powervr",
            "intel(r) hd graphics",
            "intel(r) uhd graphics",
            "intel(r) iris",
        ];

        for pattern in &integrated_patterns {
            if name_lower.contains(pattern) {
                debug!(target: LOG_TARGET,
                    "Device {} detected as integrated (name pattern: {})",
                    name, pattern
                );
                return GpuDeviceType::Integrated;
            }
        }

        // Common dedicated GPU patterns
        let dedicated_patterns = [
            "nvidia geforce",
            "nvidia rtx",
            "nvidia gtx",
            "nvidia tesla",
            "nvidia quadro",
            "amd radeon rx",
            "amd radeon r9",
            "amd radeon r7",
            "amd radeon pro",
            "amd firepro",
        ];

        for pattern in &dedicated_patterns {
            if name_lower.contains(pattern) {
                debug!(target: LOG_TARGET,
                    "Device {} detected as dedicated (name pattern: {})",
                    name, pattern
                );
                return GpuDeviceType::Dedicated;
            }
        }

        // Method 5: Memory size for dedicated detection
        // Dedicated GPUs typically have >= 2GB of dedicated memory
        if mem_gb >= 2.0 {
            debug!(target: LOG_TARGET,
                "Device {} likely dedicated (large memory size: {:.1} GB)",
                name, mem_gb
            );
            return GpuDeviceType::Dedicated;
        }

        debug!(target: LOG_TARGET,
            "Device {} type could not be determined reliably",
            name
        );
        GpuDeviceType::Unknown
    }

    /// Detect all available OpenCL GPU devices
    pub fn detect_devices() -> Result<Vec<OpenClDevice>> {
        debug!(target: LOG_TARGET,"Starting OpenCL device detection");

        let platforms = get_platforms().map_err(|e| {
            error!(target: LOG_TARGET,"Failed to get OpenCL platforms: {}", e);
            Error::msg(format!("OpenCL platform detection failed: {}", e))
        })?;

        if platforms.is_empty() {
            warn!(target: LOG_TARGET,"No OpenCL platforms found");
            return Ok(Vec::new());
        }

        info!(target: LOG_TARGET,"Found {} OpenCL platform(s)", platforms.len());

        let mut all_devices = Vec::new();
        let mut device_counter = 0;

        for platform in platforms {
            let platform_name = platform
                .name()
                .unwrap_or_else(|_| "Unknown Platform".to_string());
            debug!(target: LOG_TARGET,"Checking platform: {}", platform_name);

            match platform.get_devices(CL_DEVICE_TYPE_GPU) {
                Ok(devices) => {
                    debug!(target: LOG_TARGET,
                        "Found {} GPU device(s) on platform: {}",
                        devices.len(),
                        platform_name
                    );

                    for device_cl_id in devices {
                        let device = Device::new(device_cl_id);

                        match OpenClDevice::new(device, device_counter, platform_name.clone()) {
                            Ok(opencl_device) => {
                                info!(target: LOG_TARGET,
                                    "Detected OpenCL device {}: {} (Platform: {})",
                                    device_counter,
                                    opencl_device.name(),
                                    platform_name
                                );
                                all_devices.push(opencl_device);
                                device_counter += 1;
                            }
                            Err(e) => {
                                warn!(target: LOG_TARGET,"Failed to create OpenCL device {}: {}", device_counter, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!(target: LOG_TARGET,"No GPU devices found on platform {}: {}", platform_name, e);
                }
            }
        }

        if all_devices.is_empty() {
            warn!(target: LOG_TARGET,"No OpenCL GPU devices detected");
        } else {
            info!(target: LOG_TARGET,
                "Successfully detected {} OpenCL GPU device(s)",
                all_devices.len()
            );
        }

        Ok(all_devices)
    }

    /// Get device info string for display
    pub fn info_string(&self) -> String {
        format!(
            "{} (CU: {}, WG: {}, MEM: {:.1} GB, Type: {:?})",
            self.name,
            self.max_compute_units,
            self.max_work_group_size,
            self.global_mem_size as f64 / (1024.0 * 1024.0 * 1024.0),
            self.device_type
        )
    }

    /// Check if device is suitable for mining
    pub fn is_suitable_for_mining(&self) -> bool {
        // Basic requirements for SHA3x mining
        self.max_compute_units >= 1
            && self.max_work_group_size >= 64
            && self.global_mem_size >= 512 * 1024 * 1024 // At least 512MB
    }
}
