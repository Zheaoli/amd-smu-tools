use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmuError {
    #[error("Kernel module not loaded: {0} not found")]
    ModuleNotLoaded(PathBuf),

    #[error("Permission denied reading {0}: run as root or configure udev rules")]
    PermissionDenied(PathBuf),

    #[error("Unsupported PM table version: {0:#x}")]
    UnsupportedPmTableVersion(u32),

    #[error("Unsupported processor codename: {0}")]
    UnsupportedProcessor(u32),

    #[error("Invalid PM table size: expected at least {expected} bytes, got {actual}")]
    InvalidPmTableSize { expected: usize, actual: usize },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SmuError>;
