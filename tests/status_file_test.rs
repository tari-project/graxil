// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: tests/status_file_test.rs
// Version: 1.0.0
// Developer: Test Implementation
//
// This file contains comprehensive tests for the GPU status file management functionality.
// Tests cover file creation, reading, writing, serialization, error handling, and edge cases.

#[cfg(test)]
mod tests {
    use sha3x_miner::miner::gpu::GpuStatusFileError;
    use sha3x_miner::miner::gpu::status_file::{
        GpuStatusFile, GpuStatusFileDevice, GpuStatusFileManager, KernelType,
    };
    use sha3x_miner::miner::{gpu::opencl::device::GpuDeviceType, stats::gpu_info::GpuVendor};
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};
    use tokio::fs;

    // Helper function to create a temporary directory for testing
    fn create_temp_dir() -> TempDir {
        tempdir().expect("Failed to create temporary directory")
    }

    // Helper function to create a sample GPU status file device
    fn create_sample_device_nvidia() -> GpuStatusFileDevice {
        GpuStatusFileDevice {
            name: "NVIDIA GeForce RTX 4090".to_string(),
            device_id: 0,
            platform_name: "NVIDIA CUDA".to_string(),
            vendor: GpuVendor::NVIDIA,
            max_work_group_size: 1024,
            max_compute_units: 128,
            global_mem_size: 24_576_000_000, // 24GB
            device_type: GpuDeviceType::Dedicated,
        }
    }

    fn create_sample_device_amd() -> GpuStatusFileDevice {
        GpuStatusFileDevice {
            name: "AMD Radeon RX 7900 XT".to_string(),
            device_id: 1,
            platform_name: "AMD Accelerated Parallel Processing".to_string(),
            vendor: GpuVendor::AMD,
            max_work_group_size: 256,
            max_compute_units: 84,
            global_mem_size: 20_401_094_656, // 20GB
            device_type: GpuDeviceType::Dedicated,
        }
    }

    // Helper function to create a sample GPU status file
    fn create_sample_status_file() -> GpuStatusFile {
        GpuStatusFile {
            devices: vec![create_sample_device_nvidia(), create_sample_device_amd()],
        }
    }

    #[test]
    fn test_kernel_type_as_str() {
        assert_eq!(KernelType::OpenCL.as_str(), "opencl");
    }

    #[test]
    fn test_gpu_status_file_serialization() {
        let status_file = create_sample_status_file();

        // Test serialization
        let serialized =
            serde_json::to_string(&status_file).expect("Failed to serialize GPU status file");

        assert!(serialized.contains("NVIDIA GeForce RTX 4090"));
        assert!(serialized.contains("AMD Radeon RX 7900 XT"));
        assert!(serialized.contains("device_id"));
        assert!(serialized.contains("vendor"));

        // Test deserialization
        let deserialized: GpuStatusFile =
            serde_json::from_str(&serialized).expect("Failed to deserialize GPU status file");

        assert_eq!(deserialized.devices.len(), 2);
        assert_eq!(deserialized.devices[0].name, "NVIDIA GeForce RTX 4090");
        assert_eq!(deserialized.devices[1].name, "AMD Radeon RX 7900 XT");
    }

    #[tokio::test]
    async fn test_gpu_status_file_manager_new_success() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuStatusFileManager::new(dir_path.clone(), KernelType::OpenCL)
            .await
            .expect("Failed to create GpuStatusFileManager");

        // Verify the file path is constructed correctly
        let expected_file_path = dir_path.join("gpu_status_opencl.json");
        assert_eq!(manager.file_path(), &expected_file_path);
        assert_eq!(manager.directory_path(), &dir_path);
    }

    #[tokio::test]
    async fn test_gpu_status_file_manager_new_invalid_directory() {
        let non_existent_path = PathBuf::from("/non/existent/directory");

        let result = GpuStatusFileManager::new(non_existent_path.clone(), KernelType::OpenCL).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GpuStatusFileError::NotADirectory { path } if path == non_existent_path
        ));
    }

    #[tokio::test]
    async fn test_gpu_status_file_manager_new_file_path() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("some_file.txt");

        // Create a file instead of a directory
        fs::write(&file_path, "test content")
            .await
            .expect("Failed to create test file");

        let result = GpuStatusFileManager::new(file_path.clone(), KernelType::OpenCL).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GpuStatusFileError::NotADirectory { path } if path == file_path
        ));
    }

    #[tokio::test]
    async fn test_resolve_file_name() {
        // This tests the internal _resolve_file_name method indirectly
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuStatusFileManager::new(dir_path.clone(), KernelType::OpenCL)
            .await
            .expect("Failed to create GpuStatusFileManager");

        let expected_file_path = dir_path.join("gpu_status_opencl.json");
        assert_eq!(manager.file_path(), &expected_file_path);
    }

    #[tokio::test]
    async fn test_save_and_load_success() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuStatusFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuStatusFileManager");

        let original_status = create_sample_status_file();

        // Test saving
        manager
            .save(&original_status)
            .await
            .expect("Failed to save GPU status file");

        // Verify file exists
        assert!(manager.file_path().exists());

        // Test loading
        let loaded_status = manager
            .load()
            .await
            .expect("Failed to load GPU status file");

        // Verify content matches
        assert_eq!(loaded_status.devices.len(), original_status.devices.len());
        assert_eq!(
            loaded_status.devices[0].name,
            original_status.devices[0].name
        );
        assert_eq!(
            loaded_status.devices[1].name,
            original_status.devices[1].name
        );
        assert_eq!(
            loaded_status.devices[0].vendor,
            original_status.devices[0].vendor
        );
        assert_eq!(
            loaded_status.devices[1].vendor,
            original_status.devices[1].vendor
        );
    }

    #[tokio::test]
    async fn test_save_empty_status_file() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuStatusFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuStatusFileManager");

        let empty_status = GpuStatusFile { devices: vec![] };

        // Test saving empty file
        let save_result = manager.save(&empty_status).await;

        assert!(save_result.is_err());
        assert!(matches!(
            save_result.unwrap_err(),
            GpuStatusFileError::EmptyDeviceList
        ));
    }

    #[tokio::test]
    async fn test_load_nonexistent_file() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuStatusFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuStatusFileManager");

        let result = manager.load().await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GpuStatusFileError::FileNotFound { path } if path == manager.file_path().clone()
        ));
    }

    #[tokio::test]
    async fn test_save_creates_directory() {
        let temp_dir = create_temp_dir();
        let nested_dir = temp_dir.path().join("nested").join("directories");

        // Directory doesn't exist yet
        assert!(!nested_dir.exists());

        let manager = GpuStatusFileManager::new(nested_dir.clone(), KernelType::OpenCL).await;

        // Should fail because directory doesn't exist
        assert!(manager.is_err());

        // Create the directory first
        fs::create_dir_all(&nested_dir)
            .await
            .expect("Failed to create nested directory");

        let manager = GpuStatusFileManager::new(nested_dir, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuStatusFileManager");

        let status = create_sample_status_file();

        // This should succeed and create the file
        manager
            .save(&status)
            .await
            .expect("Failed to save to nested directory");

        assert!(manager.file_path().exists());
    }

    #[tokio::test]
    async fn test_save_atomic_write() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuStatusFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuStatusFileManager");

        let status = create_sample_status_file();

        // Save the file
        manager
            .save(&status)
            .await
            .expect("Failed to save GPU status file");

        // Verify no temporary file exists after successful save
        let temp_path = manager.file_path().with_extension("tmp");
        assert!(!temp_path.exists());

        // Verify the actual file exists
        assert!(manager.file_path().exists());
    }

    #[tokio::test]
    async fn test_multiple_save_load_cycles() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuStatusFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuStatusFileManager");

        // Perform multiple save/load cycles
        for i in 0..5 {
            let mut status = create_sample_status_file();

            // Modify device names to make them unique for each cycle
            for device in &mut status.devices {
                device.name = format!("{} - Cycle {}", device.name, i);
            }

            // Save
            manager
                .save(&status)
                .await
                .expect(&format!("Failed to save on cycle {}", i));

            // Load and verify
            let loaded_status = manager
                .load()
                .await
                .expect(&format!("Failed to load on cycle {}", i));

            assert_eq!(loaded_status.devices.len(), status.devices.len());
            for (original, loaded) in status.devices.iter().zip(loaded_status.devices.iter()) {
                assert_eq!(original.name, loaded.name);
                assert_eq!(original.device_id, loaded.device_id);
                assert_eq!(original.vendor, loaded.vendor);
            }
        }
    }

    #[tokio::test]
    async fn test_json_format_validation() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuStatusFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuStatusFileManager");

        let status = create_sample_status_file();

        // Save the file
        manager
            .save(&status)
            .await
            .expect("Failed to save GPU status file");

        // Read the raw file content and verify it's valid JSON
        let file_content = fs::read_to_string(manager.file_path())
            .await
            .expect("Failed to read saved file");

        // Verify it can be parsed as JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&file_content).expect("Saved file is not valid JSON");

        // Verify it has the expected structure
        assert!(parsed.get("devices").is_some());
        assert!(parsed["devices"].is_array());

        let devices = parsed["devices"].as_array().unwrap();
        assert_eq!(devices.len(), 2);

        // Verify first device structure
        let first_device = &devices[0];
        assert!(first_device.get("name").is_some());
        assert!(first_device.get("device_id").is_some());
        assert!(first_device.get("vendor").is_some());
        assert!(first_device.get("device_type").is_some());
    }

    #[tokio::test]
    async fn test_malformed_json_handling() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuStatusFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuStatusFileManager");

        // Write malformed JSON to the file
        let malformed_json = r#"{"devices": [{"name": "Test", "incomplete": true"#;
        fs::write(manager.file_path(), malformed_json)
            .await
            .expect("Failed to write malformed JSON");

        // Attempt to load should fail
        let result = manager.load().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GpuStatusFileError::DeserializationError { .. }
        ));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        // Initialize the manager once and share it
        let manager = std::sync::Arc::new(
            GpuStatusFileManager::new(dir_path, KernelType::OpenCL)
                .await
                .expect("Failed to create GpuStatusFileManager"),
        );

        // First, save an initial state
        let initial_status = create_sample_status_file();
        manager
            .save(&initial_status)
            .await
            .expect("Failed to save initial state");

        // Test concurrent reads (which should be safe)
        let mut handles = vec![];

        for i in 0..10 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                // Small delay to spread out the reads
                tokio::time::sleep(tokio::time::Duration::from_millis(i * 2)).await;

                // Load and verify we can read something valid
                let loaded_status = manager_clone
                    .load()
                    .await
                    .expect(&format!("Failed to load in task {}", i));

                // Verify structure is correct
                assert!(!loaded_status.devices.is_empty());
                assert_eq!(loaded_status.devices.len(), 2);
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Task panicked");
        }

        // Test a single write after all reads to ensure the file is still accessible
        let mut final_status = create_sample_status_file();
        final_status.devices[0].name = "Final Test Device".to_string();

        manager
            .save(&final_status)
            .await
            .expect("Final save failed");

        let verification_status = manager.load().await.expect("Final load failed");
        assert_eq!(verification_status.devices[0].name, "Final Test Device");
    }
}
