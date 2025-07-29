use crate::miner::{gpu::opencl::device::GpuDeviceType, stats::gpu_info::GpuVendor};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::{
    fs::{self, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};

static LOG_TARGET: &str = "tari::graxil::gpu_information_file";

#[derive(Error, Debug)]
pub enum GpuInformationFileError {
    #[error("Path {path:?} is not a directory")]
    NotADirectory { path: PathBuf },

    #[error("Directory {path:?} does not exist")]
    DirectoryNotFound { path: PathBuf },

    #[error("Directory {path:?} is read-only")]
    ReadOnlyDirectory { path: PathBuf },

    #[error("Cannot save empty GPU information file")]
    EmptyDeviceList,

    #[error("GPU information file does not exist at {path:?}")]
    FileNotFound { path: PathBuf },

    #[error("Failed to serialize GPU information file")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("Failed to deserialize GPU information file: {message}")]
    DeserializationError { message: String },

    #[error("IO operation failed on {path:?}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to create temporary file for atomic write")]
    AtomicWriteError {
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid kernel type: {kernel_type}")]
    InvalidKernelType { kernel_type: String },
}
pub enum KernelType {
    OpenCL,
}

impl KernelType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            KernelType::OpenCL => "opencl",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInformationFileDevice {
    pub name: String,
    pub device_id: u32,
    pub platform_name: String,
    pub vendor: GpuVendor,
    pub max_work_group_size: usize,
    pub max_compute_units: u32,
    pub global_mem_size: u64,
    pub device_type: GpuDeviceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInformationFile {
    pub devices: Vec<GpuInformationFileDevice>,
}

#[derive(Debug, Clone)]
pub struct GpuInformationFileManager {
    directory_path: PathBuf,
    file_path: PathBuf,
}

impl GpuInformationFileManager {
    /// Create a new GpuInformationFileManager instance.
    /// Kernel type currently supports only OpenCL.
    /// ### Arguments
    /// * `directory_path` - The directory where the GPU information file will be stored. It must be a valid directory path and writable.
    /// * `kernel_type` - The type of kernel to use (currently only OpenCL is supported).
    /// ### Returns
    /// * `Result<Self, GpuInformationFileError>` - Returns an instance of GpuInformationFileManager or an error if the directory is not valid.
    pub async fn new(
        directory_path: PathBuf,
        kernel_type: KernelType,
    ) -> Result<Self, GpuInformationFileError> {
        if !directory_path.is_dir() {
            return Err(GpuInformationFileError::NotADirectory {
                path: directory_path,
            });
        }

        let metadata =
            fs::metadata(&directory_path)
                .await
                .map_err(|e| GpuInformationFileError::IoError {
                    path: directory_path.clone(),
                    source: e,
                })?;
        if metadata.permissions().readonly() {
            return Err(GpuInformationFileError::ReadOnlyDirectory {
                path: directory_path,
            });
        }

        let file_name = Self::_resolve_file_name(kernel_type);
        let file_path = directory_path.join(&file_name);

        Ok(Self {
            directory_path,
            file_path,
        })
    }

    /// Information file name could be different in future if we implement other kernel types.
    const fn _resolve_file_name(kernel_type: KernelType) -> &'static str {
        match kernel_type {
            KernelType::OpenCL => "gpu_information_opencl.json",
        }
    }

    /// Ensure the directory exists, creating it if necessary.
    async fn _ensure_directory_exists(&self) -> Result<(), GpuInformationFileError> {
        if !self.directory_path.exists() {
            fs::create_dir_all(&self.directory_path)
                .await
                .map_err(|e| GpuInformationFileError::IoError {
                    path: self.directory_path.clone(),
                    source: e,
                })?;
        }
        Ok(())
    }

    /// Write the GPU information file atomically.
    /// This method creates a temporary file, writes the content, and then renames it to the final file name.
    /// This ensures that if the write fails, the original file remains unchanged
    async fn _write_file(
        &self,
        information_file_content: &GpuInformationFile,
    ) -> Result<(), GpuInformationFileError> {
        self._ensure_directory_exists().await?;

        debug!(target: LOG_TARGET, "Writing GPU information file to {:?}", self.file_path);

        let contents = serde_json::to_vec_pretty(information_file_content)?; // Auto-converts due to #[from]

        let temp_path = self.file_path.with_extension("tmp");

        {
            let mut temp_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&temp_path)
                .await
                .map_err(|e| GpuInformationFileError::AtomicWriteError { source: e })?;

            temp_file
                .write_all(&contents)
                .await
                .map_err(|e| GpuInformationFileError::AtomicWriteError { source: e })?;

            temp_file
                .flush()
                .await
                .map_err(|e| GpuInformationFileError::AtomicWriteError { source: e })?;
        }

        fs::rename(&temp_path, &self.file_path)
            .await
            .map_err(|e| GpuInformationFileError::AtomicWriteError { source: e })?;

        info!(target: LOG_TARGET, "Successfully saved GPU information file to {:?}", self.file_path);
        Ok(())
    }

    /// Load the GPU information file from the specified path.
    pub async fn load(&self) -> Result<GpuInformationFile, GpuInformationFileError> {
        debug!(target: LOG_TARGET, "Loading GPU information file from {:?}", self.file_path);

        if !self.file_path.exists() {
            return Err(GpuInformationFileError::FileNotFound {
                path: self.file_path.clone(),
            });
        }

        let file = OpenOptions::new()
            .read(true)
            .open(&self.file_path)
            .await
            .map_err(|e| GpuInformationFileError::IoError {
                path: self.file_path.clone(),
                source: e,
            })?;

        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).await.map_err(|e| {
            GpuInformationFileError::IoError {
                path: self.file_path.clone(),
                source: e,
            }
        })?;

        let information_file_content: GpuInformationFile = serde_json::from_str(&contents)
            .map_err(|e| GpuInformationFileError::DeserializationError {
                message: e.to_string(),
            })?;

        info!(target: LOG_TARGET, "Successfully loaded GPU information file with {} devices", information_file_content.devices.len());
        Ok(information_file_content)
    }

    /// Save the GPU information file to the specified path.
    /// This method will overwrite the existing file or create a new one if it doesn't exist.
    pub async fn save(
        &self,
        information_file_content: &GpuInformationFile,
    ) -> Result<(), GpuInformationFileError> {
        debug!(target: LOG_TARGET, "Saving GPU information file to {:?}", self.file_path);
        if information_file_content.devices.is_empty() {
            return Err(GpuInformationFileError::EmptyDeviceList);
        }

        self._write_file(information_file_content).await
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn directory_path(&self) -> &PathBuf {
        &self.directory_path
    }
}
