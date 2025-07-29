// Example showing GPU device detection and information file generation
// This demonstrates the --detect functionality from src/main.rs

use log::{error, info};
use sha3x_miner::miner::gpu::GpuManager;
use std::env;
const LOG_TARGET: &str = "tari::graxil::device_detection_example";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logs_directory = env::current_dir()
        .expect("Could not get current directory")
        .join("logs");

    // Log4s configuration
    tari_common::initialize_logging(
        &logs_directory.join("graxil").join("log4rs_config.yml"),
        &logs_directory.join("graxil"),
        include_str!("../log4rs_sample.yml"),
    )
    .expect("Could not set up logging");

    info!(target: LOG_TARGET, "🔍 GPU Device Detection Example");
    info!(target: LOG_TARGET, "=====================================");

    let information_file_directory = env::current_dir()
        .expect("Could not get current directory")
        .join("temp_information_files");

    // Ensure the directory exists
    if !information_file_directory.exists() {
        std::fs::create_dir_all(&information_file_directory)?;
        info!(target: LOG_TARGET, "📁 Created information file directory: {:?}", information_file_directory);
    }

    info!(target: LOG_TARGET, "🎯 Information file directory: {:?}", information_file_directory);
    info!(target: LOG_TARGET, "");

    info!(target: LOG_TARGET, "🔍 Detecting OpenCL devices...");
    match GpuManager::generate_information_files(information_file_directory.clone()).await {
        Ok(_) => {
            info!(target: LOG_TARGET, "✅ Device detection complete!");

            info!(target: LOG_TARGET, "");
            info!(target: LOG_TARGET, "📄 Generated information files:");

            if let Ok(entries) = std::fs::read_dir(&information_file_directory) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".json") {
                            info!(target: LOG_TARGET, "   - {}", filename);

                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                info!(target: LOG_TARGET, "     Content preview:");

                                if let Ok(json_value) =
                                    serde_json::from_str::<serde_json::Value>(&content)
                                {
                                    let pretty_json = serde_json::to_string_pretty(&json_value)?;

                                    // Show the full JSON content
                                    for line in pretty_json.lines() {
                                        info!(target: LOG_TARGET, "     {}", line);
                                    }
                                } else {
                                    info!(target: LOG_TARGET, "     {} (raw)", content);
                                }
                            }
                            info!(target: LOG_TARGET, "");
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!(target: LOG_TARGET, "❌ Failed to detect devices: {}", e);

            // Provide helpful troubleshooting information
            info!(target: LOG_TARGET, "");
            info!(target: LOG_TARGET, "💡 Troubleshooting Tips:");
            info!(target: LOG_TARGET, "   1. Make sure OpenCL drivers are installed");
            info!(target: LOG_TARGET, "   2. For NVIDIA: Install CUDA toolkit or OpenCL drivers");
            info!(target: LOG_TARGET, "   3. For AMD: Install ROCm or AMD OpenCL drivers");
            info!(target: LOG_TARGET, "   4. For Intel: Install Intel OpenCL runtime");
            info!(target: LOG_TARGET, "   5. Check if your GPU supports OpenCL");
            info!(target: LOG_TARGET, "");
            info!(target: LOG_TARGET, "🔧 You can test OpenCL availability with:");
            info!(target: LOG_TARGET, "   - clinfo (shows all OpenCL platforms and devices)");
            info!(target: LOG_TARGET, "   - nvidia-smi (for NVIDIA GPUs)");
            info!(target: LOG_TARGET, "   - rocm-smi (for AMD GPUs)");

            return Err(e.into());
        }
    }

    info!(target: LOG_TARGET, "🔍 Device detection complete! Check information files in the specified directory.");
    info!(target: LOG_TARGET, "🧹 Cleanup: Removing temporary information files...");

    // Clean up the temporary directory
    if information_file_directory.exists() {
        if let Err(e) = std::fs::remove_dir_all(&information_file_directory) {
            error!(target: LOG_TARGET, "⚠️ Failed to clean up temporary directory: {}", e);
        } else {
            info!(target: LOG_TARGET, "✅ Temporary files cleaned up");
        }
    }

    Ok(())
}
