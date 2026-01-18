use std::fs;
use std::path::{Path, PathBuf};
use crate::{Codename, PmTable, Result, SmuError};

const DEFAULT_SYSFS_PATH: &str = "/sys/kernel/ryzen_smu_drv";

/// Reader for AMD SMU data via the ryzen_smu kernel module
pub struct SmuReader {
    sysfs_path: PathBuf,
}

impl SmuReader {
    /// Create a new SMU reader with the default sysfs path
    pub fn new() -> Result<Self> {
        Self::with_path(DEFAULT_SYSFS_PATH)
    }

    /// Create a new SMU reader with a custom sysfs path
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let sysfs_path = path.as_ref().to_path_buf();

        if !sysfs_path.exists() {
            return Err(SmuError::ModuleNotLoaded(sysfs_path));
        }

        Ok(Self { sysfs_path })
    }

    /// Get the SMU firmware version string
    pub fn smu_version(&self) -> Result<String> {
        self.read_string("version")
    }

    /// Get the driver version string
    pub fn driver_version(&self) -> Result<String> {
        self.read_string("drv_version")
    }

    /// Get the processor codename
    pub fn codename(&self) -> Result<Codename> {
        let id_str = self.read_string("codename")?;
        let id: u32 = id_str.trim().parse().unwrap_or(0);
        Ok(Codename::from_id(id))
    }

    /// Get the PM table version
    pub fn pm_table_version(&self) -> Result<u32> {
        let ver_str = self.read_string("pm_table_version")?;
        // Handle both decimal and hex formats
        let trimmed = ver_str.trim();
        if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
            Ok(u32::from_str_radix(&trimmed[2..], 16).unwrap_or(0))
        } else {
            Ok(trimmed.parse().unwrap_or(0))
        }
    }

    /// Get the PM table size in bytes
    pub fn pm_table_size(&self) -> Result<usize> {
        let size_str = self.read_string("pm_table_size")?;
        Ok(size_str.trim().parse().unwrap_or(0))
    }

    /// Read and parse the PM table
    pub fn read_pm_table(&self) -> Result<PmTable> {
        let version = self.pm_table_version()?;
        let codename = self.codename()?;
        let data = self.read_binary("pm_table")?;

        // Detect core count from the data or use a reasonable default
        let core_count = self.detect_core_count(&data, codename);

        PmTable::parse(&data, version, codename, core_count)
    }

    /// Detect the number of active cores
    fn detect_core_count(&self, _data: &[u8], codename: Codename) -> usize {
        // Try to read from /proc/cpuinfo or use codename defaults
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            let count = cpuinfo.matches("processor\t:").count();
            if count > 0 {
                return count;
            }
        }
        // Fallback based on codename
        codename.cores_per_ccd() * codename.max_ccds()
    }

    fn read_string(&self, name: &str) -> Result<String> {
        let path = self.sysfs_path.join(name);
        self.check_readable(&path)?;
        Ok(fs::read_to_string(&path)?)
    }

    fn read_binary(&self, name: &str) -> Result<Vec<u8>> {
        let path = self.sysfs_path.join(name);
        self.check_readable(&path)?;
        Ok(fs::read(&path)?)
    }

    fn check_readable(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(SmuError::ModuleNotLoaded(path.to_path_buf()));
        }
        // Try to check permissions
        match fs::metadata(path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                Err(SmuError::PermissionDenied(path.to_path_buf()))
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl Default for SmuReader {
    fn default() -> Self {
        Self::new().expect("Failed to initialize SMU reader")
    }
}
