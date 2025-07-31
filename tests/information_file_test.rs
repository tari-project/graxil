#[cfg(test)]
mod tests {
    use sha3x_miner::miner::gpu::GpuInformationFileError;
    use sha3x_miner::miner::gpu::gpu_information_file::{
        GpuInformationFile, GpuInformationFileDevice, GpuInformationFileManager, KernelType,
    };
    use sha3x_miner::miner::{gpu::opencl::device::GpuDeviceType, stats::gpu_info::GpuVendor};
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};
    use tokio::fs;

    // Helper function to create a temporary directory for testing
    fn create_temp_dir() -> TempDir {
        tempdir().expect("Failed to create temporary directory")
    }

    // Helper function to create a sample GPU information file device
    fn create_sample_device_nvidia() -> GpuInformationFileDevice {
        GpuInformationFileDevice {
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

    fn create_sample_device_amd() -> GpuInformationFileDevice {
        GpuInformationFileDevice {
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

    // Helper function to create a sample GPU information file
    fn create_sample_information_file() -> GpuInformationFile {
        GpuInformationFile {
            devices: vec![create_sample_device_nvidia(), create_sample_device_amd()],
        }
    }

    #[test]
    fn test_kernel_type_as_str() {
        assert_eq!(KernelType::OpenCL.as_str(), "opencl");
    }

    #[test]
    fn test_gpu_information_file_serialization() {
        let gpu_information_file = create_sample_information_file();

        let serialized = serde_json::to_string(&gpu_information_file)
            .expect("Failed to serialize GPU information file");

        assert!(serialized.contains("NVIDIA GeForce RTX 4090"));
        assert!(serialized.contains("AMD Radeon RX 7900 XT"));
        assert!(serialized.contains("device_id"));
        assert!(serialized.contains("vendor"));

        let deserialized: GpuInformationFile =
            serde_json::from_str(&serialized).expect("Failed to deserialize GPU information file");

        assert_eq!(deserialized.devices.len(), 2);
        assert_eq!(deserialized.devices[0].name, "NVIDIA GeForce RTX 4090");
        assert_eq!(deserialized.devices[1].name, "AMD Radeon RX 7900 XT");
    }

    #[tokio::test]
    async fn test_gpu_information_file_manager_new_success() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuInformationFileManager::new(dir_path.clone(), KernelType::OpenCL)
            .await
            .expect("Failed to create GpuInformationFileManager");

        let expected_file_path = dir_path.join("gpu_information_opencl.json");
        assert_eq!(manager.file_path(), &expected_file_path);
        assert_eq!(manager.directory_path(), &dir_path);
    }

    #[tokio::test]
    async fn test_gpu_information_file_manager_new_creates_missing_directory() {
        let non_existent_path = PathBuf::from("/tmp/non_existent_test_directory");

        // Ensure the directory doesn't exist before the test
        if non_existent_path.exists() {
            fs::remove_dir_all(&non_existent_path)
                .await
                .expect("Failed to clean up test directory");
        }

        let result =
            GpuInformationFileManager::new(non_existent_path.clone(), KernelType::OpenCL).await;

        // Should succeed now since the directory will be created automatically
        assert!(result.is_ok());

        let manager = result.unwrap();
        assert_eq!(manager.directory_path(), &non_existent_path);
        assert!(non_existent_path.exists());
        assert!(non_existent_path.is_dir());

        // Clean up the created directory
        fs::remove_dir_all(&non_existent_path)
            .await
            .expect("Failed to clean up test directory");
    }

    #[tokio::test]
    async fn test_gpu_information_file_manager_new_file_path() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("some_file.txt");

        fs::write(&file_path, "test content")
            .await
            .expect("Failed to create test file");

        let result = GpuInformationFileManager::new(file_path.clone(), KernelType::OpenCL).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GpuInformationFileError::NotADirectory { path } if path == file_path
        ));
    }

    #[tokio::test]
    async fn test_resolve_file_name() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuInformationFileManager::new(dir_path.clone(), KernelType::OpenCL)
            .await
            .expect("Failed to create GpuInformationFileManager");

        let expected_file_path = dir_path.join("gpu_information_opencl.json");
        assert_eq!(manager.file_path(), &expected_file_path);
    }

    #[tokio::test]
    async fn test_save_and_load_success() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuInformationFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuInformationFileManager");

        let original_gpu_information = create_sample_information_file();

        manager
            .save(&original_gpu_information)
            .await
            .expect("Failed to save GPU information file");

        assert!(manager.file_path().exists());

        let loaded_information = manager
            .load()
            .await
            .expect("Failed to load GPU information file");

        assert_eq!(
            loaded_information.devices.len(),
            original_gpu_information.devices.len()
        );
        assert_eq!(
            loaded_information.devices[0].name,
            original_gpu_information.devices[0].name
        );
        assert_eq!(
            loaded_information.devices[1].name,
            original_gpu_information.devices[1].name
        );
        assert_eq!(
            loaded_information.devices[0].vendor,
            original_gpu_information.devices[0].vendor
        );
        assert_eq!(
            loaded_information.devices[1].vendor,
            original_gpu_information.devices[1].vendor
        );
    }

    #[tokio::test]
    async fn test_save_empty_information_file() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuInformationFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuInformationFileManager");

        let empty_gpu_information = GpuInformationFile { devices: vec![] };

        let save_result = manager.save(&empty_gpu_information).await;

        assert!(save_result.is_err());
        assert!(matches!(
            save_result.unwrap_err(),
            GpuInformationFileError::EmptyDeviceList
        ));
    }

    #[tokio::test]
    async fn test_load_nonexistent_file() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuInformationFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuInformationFileManager");

        let result = manager.load().await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GpuInformationFileError::FileNotFound { path } if path == manager.file_path().clone()
        ));
    }

    #[tokio::test]
    async fn test_automatically_creates_nested_directory() {
        let temp_dir = create_temp_dir();
        let nested_dir = temp_dir.path().join("nested").join("directories");

        // Ensure the nested directory doesn't exist initially
        assert!(!nested_dir.exists());

        // Now with auto-directory creation, this should succeed
        let manager = GpuInformationFileManager::new(nested_dir.clone(), KernelType::OpenCL)
            .await
            .expect("Failed to create GpuInformationFileManager - directory should be created automatically");

        // Verify the directory was created
        assert!(nested_dir.exists());
        assert!(nested_dir.is_dir());

        let information = create_sample_information_file();

        manager
            .save(&information)
            .await
            .expect("Failed to save to nested directory");

        assert!(manager.file_path().exists());
    }

    #[tokio::test]
    async fn test_save_atomic_write() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuInformationFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuInformationFileManager");

        let gpu_information = create_sample_information_file();

        manager
            .save(&gpu_information)
            .await
            .expect("Failed to save GPU information file");

        let temp_path = manager.file_path().with_extension("tmp");
        assert!(!temp_path.exists());

        assert!(manager.file_path().exists());
    }

    #[tokio::test]
    async fn test_multiple_save_load_cycles() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        let manager = GpuInformationFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuInformationFileManager");

        // Perform multiple save/load cycles
        for i in 0..5 {
            let mut gpu_information = create_sample_information_file();

            // Modify device names to make them unique for each cycle
            for device in &mut gpu_information.devices {
                device.name = format!("{} - Cycle {}", device.name, i);
            }

            // Save
            manager
                .save(&gpu_information)
                .await
                .expect(&format!("Failed to save on cycle {}", i));

            // Load and verify
            let loaded_gpu_information = manager
                .load()
                .await
                .expect(&format!("Failed to load on cycle {}", i));

            assert_eq!(
                loaded_gpu_information.devices.len(),
                gpu_information.devices.len()
            );
            for (original, loaded) in gpu_information
                .devices
                .iter()
                .zip(loaded_gpu_information.devices.iter())
            {
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

        let manager = GpuInformationFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuInformationFileManager");

        let gpu_information = create_sample_information_file();

        manager
            .save(&gpu_information)
            .await
            .expect("Failed to save GPU information file");

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

        let manager = GpuInformationFileManager::new(dir_path, KernelType::OpenCL)
            .await
            .expect("Failed to create GpuInformationFileManager");

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
            GpuInformationFileError::DeserializationError { .. }
        ));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let temp_dir = create_temp_dir();
        let dir_path = temp_dir.path().to_path_buf();

        // Initialize the manager once and share it
        let manager = std::sync::Arc::new(
            GpuInformationFileManager::new(dir_path, KernelType::OpenCL)
                .await
                .expect("Failed to create GpuInformationFileManager"),
        );

        // First, save an initial state
        let initial_gpu_information = create_sample_information_file();
        manager
            .save(&initial_gpu_information)
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
                let loaded_gpu_information = manager_clone
                    .load()
                    .await
                    .expect(&format!("Failed to load in task {}", i));

                // Verify structure is correct
                assert!(!loaded_gpu_information.devices.is_empty());
                assert_eq!(loaded_gpu_information.devices.len(), 2);
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Task panicked");
        }

        // Test a single write after all reads to ensure the file is still accessible
        let mut final_gpu_information = create_sample_information_file();
        final_gpu_information.devices[0].name = "Final Test Device".to_string();

        manager
            .save(&final_gpu_information)
            .await
            .expect("Final save failed");

        let verification_status = manager.load().await.expect("Final load failed");
        assert_eq!(verification_status.devices[0].name, "Final Test Device");
    }
}
