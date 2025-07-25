// Example showing OpenCL GPU device type detection (integrated vs dedicated)

use log::info;
use sha3x_miner::miner::gpu::opencl::device::{GpuDeviceType, OpenClDevice};

const LOG_TARGET: &str = "gpu_device_example";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!(target: LOG_TARGET, "🔍 OpenCL GPU Device Type Detection Example");
    info!(target: LOG_TARGET, "================================================");

    // Detect all available OpenCL GPU devices
    match OpenClDevice::detect_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                info!(target: LOG_TARGET, "❌ No OpenCL GPU devices found");
                return Ok(());
            }

            info!(target: LOG_TARGET, "Found {} OpenCL GPU device(s):", devices.len());
            info!(target: LOG_TARGET, "");

            for (i, device) in devices.iter().enumerate() {
                let device_type_emoji = match device.device_type() {
                    GpuDeviceType::Integrated => "🔗",
                    GpuDeviceType::Dedicated => "🎮",
                    GpuDeviceType::Unknown => "❓",
                };

                info!(target: LOG_TARGET,
                    "{} Device {}: {} {}",
                    device_type_emoji, i, device.name(),
                    match device.device_type() {
                        GpuDeviceType::Integrated => "(Integrated GPU)",
                        GpuDeviceType::Dedicated => "(Dedicated GPU)",
                        GpuDeviceType::Unknown => "(Unknown Type)",
                    }
                );

                info!(target: LOG_TARGET, "   Platform: {}", device.platform_name());
                info!(target: LOG_TARGET, "   Compute Units: {}", device.max_compute_units());
                info!(target: LOG_TARGET, "   Work Group Size: {}", device.max_work_group_size());
                info!(target: LOG_TARGET, "   Global Memory: {:.2} GB", 
                    device.global_mem_size() as f64 / (1024.0 * 1024.0 * 1024.0));
                info!(target: LOG_TARGET, "   Type: {:?}", device.device_type());

                // Show OpenCL properties used for detection
                let cl_device = device.device();

                if let Ok(host_unified) = cl_device.host_unified_memory() {
                    info!(target: LOG_TARGET, "   Host Unified Memory: {}", host_unified);
                }

                if let Ok(integrated_nv) = cl_device.integrated_memory_nv() {
                    info!(target: LOG_TARGET, "   NVIDIA Integrated Memory: {}", integrated_nv != 0);
                }

                if let Ok(vendor) = cl_device.vendor() {
                    info!(target: LOG_TARGET, "   Vendor: {}", vendor);
                }

                info!(target: LOG_TARGET, "   Full Info: {}", device.info_string());
                info!(target: LOG_TARGET, "");
            }

            // Show statistics
            let integrated_count = devices
                .iter()
                .filter(|d| matches!(d.device_type(), GpuDeviceType::Integrated))
                .count();
            let dedicated_count = devices
                .iter()
                .filter(|d| matches!(d.device_type(), GpuDeviceType::Dedicated))
                .count();
            let unknown_count = devices
                .iter()
                .filter(|d| matches!(d.device_type(), GpuDeviceType::Unknown))
                .count();

            info!(target: LOG_TARGET, "📊 Device Type Summary:");
            info!(target: LOG_TARGET, "   🔗 Integrated GPUs: {}", integrated_count);
            info!(target: LOG_TARGET, "   🎮 Dedicated GPUs: {}", dedicated_count);
            info!(target: LOG_TARGET, "   ❓ Unknown Type: {}", unknown_count);

            // Mining suitability recommendations
            info!(target: LOG_TARGET, "");
            info!(target: LOG_TARGET, "💡 Mining Recommendations:");
            for (i, device) in devices.iter().enumerate() {
                let recommendation = match device.device_type() {
                    GpuDeviceType::Dedicated => {
                        if device.global_mem_size() >= 4 * 1024 * 1024 * 1024 {
                            "✅ Excellent for mining (dedicated GPU with good memory)"
                        } else {
                            "⚠️  Good for mining but limited by memory"
                        }
                    }
                    GpuDeviceType::Integrated => {
                        "⚠️  Can mine but performance will be limited (integrated GPU)"
                    }
                    GpuDeviceType::Unknown => "❓ Test mining performance to determine suitability",
                };

                info!(target: LOG_TARGET, "   Device {}: {}", i, recommendation);
            }
        }
        Err(e) => {
            info!(target: LOG_TARGET, "❌ Failed to detect GPU devices: {}", e);
        }
    }

    Ok(())
}
