use crate::miner::{gpu::opencl::device::GpuDeviceType, stats::gpu_info::GpuVendor};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};

static LOG_TARGET: &str = "tari::graxil::gpu_status_file";

#[derive(Error, Debug)]
pub enum GpuStatusFileError {
    #[error("Path {path:?} is not a directory")]
    NotADirectory { path: PathBuf },

    #[error("Directory {path:?} does not exist")]
    DirectoryNotFound { path: PathBuf },

    #[error("Directory {path:?} is read-only")]
    ReadOnlyDirectory { path: PathBuf },

    #[error("Cannot save empty GPU status file")]
    EmptyDeviceList,

    #[error("GPU status file does not exist at {path:?}")]
    FileNotFound { path: PathBuf },

    #[error("Failed to serialize GPU status file")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("Failed to deserialize GPU status file: {message}")]
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
pub struct GpuStatusFileDevice {
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
pub struct GpuStatusFile {
    pub devices: Vec<GpuStatusFileDevice>,
}

#[derive(Debug, Clone)]
pub struct GpuStatusFileManager {
    directory_path: PathBuf,
    file_path: PathBuf,
}

impl GpuStatusFileManager {
    pub async fn new(
        directory_path: PathBuf,
        kernel_type: KernelType,
    ) -> Result<Self, GpuStatusFileError> {
        if !directory_path.is_dir() {
            return Err(GpuStatusFileError::NotADirectory {
                path: directory_path,
            });
        }

        let metadata =
            fs::metadata(&directory_path)
                .await
                .map_err(|e| GpuStatusFileError::IoError {
                    path: directory_path.clone(),
                    source: e,
                })?;
        if metadata.permissions().readonly() {
            return Err(GpuStatusFileError::ReadOnlyDirectory {
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

    const fn _resolve_file_name(kernel_type: KernelType) -> &'static str {
        match kernel_type {
            KernelType::OpenCL => "gpu_status_opencl.json",
        }
    }

    async fn _ensure_directory_exists(&self) -> Result<(), GpuStatusFileError> {
        if !self.directory_path.exists() {
            fs::create_dir_all(&self.directory_path)
                .await
                .map_err(|e| GpuStatusFileError::IoError {
                    path: self.directory_path.clone(),
                    source: e,
                })?;
        }
        Ok(())
    }

    async fn _write_file(&self, status: &GpuStatusFile) -> Result<(), GpuStatusFileError> {
        self._ensure_directory_exists().await?;

        debug!(target: LOG_TARGET, "Writing GPU status file to {:?}", self.file_path);

        let contents = serde_json::to_vec_pretty(status)?; // Auto-converts due to #[from]

        let temp_path = self.file_path.with_extension("tmp");

        {
            let mut temp_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&temp_path)
                .await
                .map_err(|e| GpuStatusFileError::AtomicWriteError { source: e })?;

            temp_file
                .write_all(&contents)
                .await
                .map_err(|e| GpuStatusFileError::AtomicWriteError { source: e })?;

            temp_file
                .flush()
                .await
                .map_err(|e| GpuStatusFileError::AtomicWriteError { source: e })?;
        }

        fs::rename(&temp_path, &self.file_path)
            .await
            .map_err(|e| GpuStatusFileError::AtomicWriteError { source: e })?;

        info!(target: LOG_TARGET, "Successfully saved GPU status file to {:?}", self.file_path);
        Ok(())
    }

    pub async fn load(&self) -> Result<GpuStatusFile, GpuStatusFileError> {
        debug!(target: LOG_TARGET, "Loading GPU status file from {:?}", self.file_path);

        if !self.file_path.exists() {
            return Err(GpuStatusFileError::FileNotFound {
                path: self.file_path.clone(),
            });
        }

        let contents =
            fs::read_to_string(&self.file_path)
                .await
                .map_err(|e| GpuStatusFileError::IoError {
                    path: self.file_path.clone(),
                    source: e,
                })?;

        let status: GpuStatusFile = serde_json::from_str(&contents).map_err(|e| {
            GpuStatusFileError::DeserializationError {
                message: e.to_string(),
            }
        })?;

        info!(target: LOG_TARGET, "Successfully loaded GPU status file with {} devices", status.devices.len());
        Ok(status)
    }

    pub async fn save(&self, status: &GpuStatusFile) -> Result<(), GpuStatusFileError> {
        debug!(target: LOG_TARGET, "Saving GPU status file to {:?}", self.file_path);
        if status.devices.is_empty() {
            return Err(GpuStatusFileError::EmptyDeviceList);
        }

        self._write_file(status).await
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn directory_path(&self) -> &PathBuf {
        &self.directory_path
    }
}
