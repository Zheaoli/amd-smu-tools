# AMD SMU Sensors Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI tool and TUI dashboard for reading AMD Ryzen processor metrics via the ryzen_smu kernel module.

**Architecture:** Workspace with three crates - `amd-smu-lib` (core library for PM table parsing), `amd-smu-cli` (CLI with text/JSON output), `amd-smu-tui` (live dashboard). Library reads from `/sys/kernel/ryzen_smu_drv/pm_table` and parses 32-bit floats at known offsets.

**Tech Stack:** Rust, clap, serde, ratatui, crossterm, thiserror, byteorder

---

## Task 1: Initialize Workspace Structure

**Files:**
- Create: `Cargo.toml`
- Create: `crates/amd-smu-lib/Cargo.toml`
- Create: `crates/amd-smu-lib/src/lib.rs`
- Create: `crates/amd-smu-cli/Cargo.toml`
- Create: `crates/amd-smu-cli/src/main.rs`
- Create: `crates/amd-smu-tui/Cargo.toml`
- Create: `crates/amd-smu-tui/src/main.rs`

**Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/user/amd-smu-tools"

[workspace.dependencies]
amd-smu-lib = { path = "crates/amd-smu-lib" }
thiserror = "2"
byteorder = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
humantime = "2"
ratatui = "0.29"
crossterm = "0.28"
```

**Step 2: Create amd-smu-lib crate**

`crates/amd-smu-lib/Cargo.toml`:
```toml
[package]
name = "amd-smu-lib"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
thiserror = { workspace = true }
byteorder = { workspace = true }
```

`crates/amd-smu-lib/src/lib.rs`:
```rust
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

**Step 3: Create amd-smu-cli crate**

`crates/amd-smu-cli/Cargo.toml`:
```toml
[package]
name = "amd-smu-cli"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "amd-smu-sensors"
path = "src/main.rs"

[dependencies]
amd-smu-lib = { workspace = true }
clap = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
humantime = { workspace = true }
```

`crates/amd-smu-cli/src/main.rs`:
```rust
fn main() {
    println!("amd-smu-sensors v{}", amd_smu_lib::version());
}
```

**Step 4: Create amd-smu-tui crate**

`crates/amd-smu-tui/Cargo.toml`:
```toml
[package]
name = "amd-smu-tui"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "amd-smu-tui"
path = "src/main.rs"

[dependencies]
amd-smu-lib = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
```

`crates/amd-smu-tui/src/main.rs`:
```rust
fn main() {
    println!("amd-smu-tui v{}", amd_smu_lib::version());
}
```

**Step 5: Verify workspace builds**

Run: `cargo build`
Expected: Compiles all three crates successfully

**Step 6: Commit**

```bash
git add -A && git commit -m "feat: initialize workspace with three crates"
```

---

## Task 2: Implement Error Types

**Files:**
- Create: `crates/amd-smu-lib/src/error.rs`
- Modify: `crates/amd-smu-lib/src/lib.rs`

**Step 1: Create error module**

`crates/amd-smu-lib/src/error.rs`:
```rust
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
```

**Step 2: Export from lib.rs**

`crates/amd-smu-lib/src/lib.rs`:
```rust
mod error;

pub use error::{Result, SmuError};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

**Step 3: Verify it compiles**

Run: `cargo build -p amd-smu-lib`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add -A && git commit -m "feat(lib): add error types"
```

---

## Task 3: Implement Codename Detection

**Files:**
- Create: `crates/amd-smu-lib/src/codename.rs`
- Modify: `crates/amd-smu-lib/src/lib.rs`

**Step 1: Create codename module**

`crates/amd-smu-lib/src/codename.rs`:
```rust
use std::fmt;

/// AMD processor codenames supported by ryzen_smu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Codename {
    Unsupported = 0,
    Colfax = 1,
    Renoir = 2,
    Picasso = 3,
    Matisse = 4,
    Threadripper = 5,
    CastlePeak = 6,
    Raven = 7,
    Raven2 = 8,
    SummitRidge = 9,
    PinnacleRidge = 10,
    Rembrandt = 11,
    Vermeer = 12,
    Vangogh = 13,
    Cezanne = 14,
    Milan = 15,
    Dali = 16,
    Lucienne = 17,
    Naples = 18,
    Chagall = 19,
    Raphael = 20,
    Phoenix = 21,
    HawkPoint = 22,
    GraniteRidge = 23,
    StrixPoint = 24,
    StormPeak = 25,
}

impl Codename {
    /// Parse codename from the numeric value in sysfs
    pub fn from_id(id: u32) -> Self {
        match id {
            1 => Self::Colfax,
            2 => Self::Renoir,
            3 => Self::Picasso,
            4 => Self::Matisse,
            5 => Self::Threadripper,
            6 => Self::CastlePeak,
            7 => Self::Raven,
            8 => Self::Raven2,
            9 => Self::SummitRidge,
            10 => Self::PinnacleRidge,
            11 => Self::Rembrandt,
            12 => Self::Vermeer,
            13 => Self::Vangogh,
            14 => Self::Cezanne,
            15 => Self::Milan,
            16 => Self::Dali,
            17 => Self::Lucienne,
            18 => Self::Naples,
            19 => Self::Chagall,
            20 => Self::Raphael,
            21 => Self::Phoenix,
            22 => Self::HawkPoint,
            23 => Self::GraniteRidge,
            24 => Self::StrixPoint,
            25 => Self::StormPeak,
            _ => Self::Unsupported,
        }
    }

    /// Get the number of cores per CCD for this processor family
    pub fn cores_per_ccd(&self) -> usize {
        match self {
            Self::Matisse | Self::Vermeer | Self::Milan | Self::Raphael | Self::GraniteRidge => 8,
            Self::Cezanne | Self::Rembrandt | Self::Phoenix | Self::HawkPoint | Self::StrixPoint => 8,
            Self::Renoir | Self::Lucienne => 8,
            _ => 8, // Default assumption
        }
    }

    /// Get max CCDs for this processor family
    pub fn max_ccds(&self) -> usize {
        match self {
            Self::Milan | Self::Naples | Self::Chagall | Self::StormPeak => 8,
            Self::Threadripper | Self::CastlePeak => 4,
            Self::Vermeer | Self::Matisse | Self::Raphael | Self::GraniteRidge => 2,
            _ => 1,
        }
    }
}

impl fmt::Display for Codename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Unsupported => "Unsupported",
            Self::Colfax => "Colfax",
            Self::Renoir => "Renoir",
            Self::Picasso => "Picasso",
            Self::Matisse => "Matisse",
            Self::Threadripper => "Threadripper",
            Self::CastlePeak => "Castle Peak",
            Self::Raven => "Raven",
            Self::Raven2 => "Raven 2",
            Self::SummitRidge => "Summit Ridge",
            Self::PinnacleRidge => "Pinnacle Ridge",
            Self::Rembrandt => "Rembrandt",
            Self::Vermeer => "Vermeer",
            Self::Vangogh => "Van Gogh",
            Self::Cezanne => "Cezanne",
            Self::Milan => "Milan",
            Self::Dali => "Dali",
            Self::Lucienne => "Lucienne",
            Self::Naples => "Naples",
            Self::Chagall => "Chagall",
            Self::Raphael => "Raphael",
            Self::Phoenix => "Phoenix",
            Self::HawkPoint => "Hawk Point",
            Self::GraniteRidge => "Granite Ridge",
            Self::StrixPoint => "Strix Point",
            Self::StormPeak => "Storm Peak",
        };
        write!(f, "{}", name)
    }
}
```

**Step 2: Export from lib.rs**

Add to `crates/amd-smu-lib/src/lib.rs`:
```rust
mod codename;

pub use codename::Codename;
```

**Step 3: Verify it compiles**

Run: `cargo build -p amd-smu-lib`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add -A && git commit -m "feat(lib): add processor codename detection"
```

---

## Task 4: Implement PM Table Data Structure

**Files:**
- Create: `crates/amd-smu-lib/src/pmtable.rs`
- Modify: `crates/amd-smu-lib/src/lib.rs`

**Step 1: Create PM table data structure**

`crates/amd-smu-lib/src/pmtable.rs`:
```rust
use crate::Codename;
use serde::Serialize;

/// Maximum number of cores supported
pub const MAX_CORES: usize = 16;

/// PM Table data parsed from the kernel module
#[derive(Debug, Clone, Serialize)]
pub struct PmTable {
    /// PM table format version
    pub version: u32,
    /// Processor codename
    #[serde(skip)]
    pub codename: Codename,
    /// Codename as string for JSON
    #[serde(rename = "codename")]
    pub codename_str: String,

    // Limits
    /// Package Power Tracking limit (W)
    pub ppt_limit: f32,
    /// Thermal Design Current limit (A)
    pub tdc_limit: f32,
    /// Electrical Design Current limit (A)
    pub edc_limit: f32,
    /// Thermal limit (°C)
    pub thm_limit: f32,

    // Current values
    /// Current PPT value (W)
    pub ppt_value: f32,
    /// Current TDC value (A)
    pub tdc_value: f32,
    /// Current EDC value (A)
    pub edc_value: f32,

    // Temperatures
    /// Tctl/Tdie junction temperature (°C)
    pub tctl: f32,
    /// SoC temperature (°C)
    pub soc_temp: f32,
    /// Per-core temperatures (°C)
    pub core_temps: Vec<f32>,

    // Frequencies (MHz)
    /// Per-core frequencies
    pub core_freqs: Vec<f32>,
    /// Per-core effective frequencies
    pub core_freqs_eff: Vec<f32>,
    /// Fabric clock
    pub fclk: f32,
    /// Memory clock
    pub mclk: f32,

    // Power (W)
    /// Per-core power
    pub core_power: Vec<f32>,
    /// Total package power
    pub package_power: f32,
    /// SoC power
    pub soc_power: f32,

    // Voltages (V) and residency (%)
    /// Core voltage
    pub core_voltage: f32,
    /// SoC voltage
    pub soc_voltage: f32,
    /// Per-core C0 residency (%)
    pub core_c0: Vec<f32>,
}

impl Default for PmTable {
    fn default() -> Self {
        Self {
            version: 0,
            codename: Codename::Unsupported,
            codename_str: String::new(),
            ppt_limit: 0.0,
            tdc_limit: 0.0,
            edc_limit: 0.0,
            thm_limit: 0.0,
            ppt_value: 0.0,
            tdc_value: 0.0,
            edc_value: 0.0,
            tctl: 0.0,
            soc_temp: 0.0,
            core_temps: Vec::new(),
            core_freqs: Vec::new(),
            core_freqs_eff: Vec::new(),
            fclk: 0.0,
            mclk: 0.0,
            core_power: Vec::new(),
            package_power: 0.0,
            soc_power: 0.0,
            core_voltage: 0.0,
            soc_voltage: 0.0,
            core_c0: Vec::new(),
        }
    }
}
```

**Step 2: Export from lib.rs**

Add to `crates/amd-smu-lib/src/lib.rs`:
```rust
mod pmtable;

pub use pmtable::{PmTable, MAX_CORES};
```

Also add serde to lib dependencies in `crates/amd-smu-lib/Cargo.toml`:
```toml
[dependencies]
thiserror = { workspace = true }
byteorder = { workspace = true }
serde = { workspace = true }
```

**Step 3: Verify it compiles**

Run: `cargo build -p amd-smu-lib`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add -A && git commit -m "feat(lib): add PM table data structure"
```

---

## Task 5: Implement PM Table Parsing

**Files:**
- Modify: `crates/amd-smu-lib/src/pmtable.rs`

**Step 1: Add parsing implementation**

Add to `crates/amd-smu-lib/src/pmtable.rs`:
```rust
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use crate::{Result, SmuError};

/// PM table offsets for version 0x240903 (Matisse/Vermeer and later)
mod offsets {
    pub const PPT_LIMIT: usize = 0x000;
    pub const PPT_VALUE: usize = 0x004;
    pub const TDC_LIMIT: usize = 0x008;
    pub const TDC_VALUE: usize = 0x00C;
    pub const THM_LIMIT: usize = 0x010;
    pub const THM_VALUE: usize = 0x014;
    pub const EDC_LIMIT: usize = 0x020;
    pub const EDC_VALUE: usize = 0x024;
    pub const VDDCR_CPU_POWER: usize = 0x060;
    pub const VDDCR_SOC_POWER: usize = 0x064;
    pub const CPU_VOLTAGE: usize = 0x0A0;
    pub const SOC_TEMP: usize = 0x0A8;
    pub const SOC_VOLTAGE: usize = 0x0B4;
    pub const FCLK: usize = 0x0C0;
    pub const MCLK: usize = 0x0C8;
    pub const CORE_POWER_BASE: usize = 0x24C;
    pub const CORE_TEMP_BASE: usize = 0x2C0;
    pub const CORE_FREQ_BASE: usize = 0x2EC;
    pub const CORE_FREQEFF_BASE: usize = 0x30C;
    pub const CORE_C0_BASE: usize = 0x32C;
}

impl PmTable {
    /// Parse PM table from raw bytes
    pub fn parse(data: &[u8], version: u32, codename: Codename, core_count: usize) -> Result<Self> {
        // Minimum size check
        let min_size = offsets::CORE_C0_BASE + (core_count * 4);
        if data.len() < min_size {
            return Err(SmuError::InvalidPmTableSize {
                expected: min_size,
                actual: data.len(),
            });
        }

        let mut table = PmTable {
            version,
            codename,
            codename_str: codename.to_string(),
            ..Default::default()
        };

        // Parse limits
        table.ppt_limit = read_f32(data, offsets::PPT_LIMIT)?;
        table.ppt_value = read_f32(data, offsets::PPT_VALUE)?;
        table.tdc_limit = read_f32(data, offsets::TDC_LIMIT)?;
        table.tdc_value = read_f32(data, offsets::TDC_VALUE)?;
        table.thm_limit = read_f32(data, offsets::THM_LIMIT)?;
        table.tctl = read_f32(data, offsets::THM_VALUE)?;
        table.edc_limit = read_f32(data, offsets::EDC_LIMIT)?;
        table.edc_value = read_f32(data, offsets::EDC_VALUE)?;

        // Parse power
        table.package_power = read_f32(data, offsets::VDDCR_CPU_POWER)?;
        table.soc_power = read_f32(data, offsets::VDDCR_SOC_POWER)?;

        // Parse voltages and temps
        table.core_voltage = read_f32(data, offsets::CPU_VOLTAGE)?;
        table.soc_temp = read_f32(data, offsets::SOC_TEMP)?;
        table.soc_voltage = read_f32(data, offsets::SOC_VOLTAGE)?;

        // Parse clocks
        table.fclk = read_f32(data, offsets::FCLK)?;
        table.mclk = read_f32(data, offsets::MCLK)?;

        // Parse per-core data
        for i in 0..core_count {
            table.core_power.push(read_f32(data, offsets::CORE_POWER_BASE + i * 4)?);
            table.core_temps.push(read_f32(data, offsets::CORE_TEMP_BASE + i * 4)?);
            table.core_freqs.push(read_f32(data, offsets::CORE_FREQ_BASE + i * 4)?);
            table.core_freqs_eff.push(read_f32(data, offsets::CORE_FREQEFF_BASE + i * 4)?);
            table.core_c0.push(read_f32(data, offsets::CORE_C0_BASE + i * 4)?);
        }

        Ok(table)
    }
}

/// Read a little-endian f32 from buffer at offset
fn read_f32(data: &[u8], offset: usize) -> Result<f32> {
    let mut cursor = Cursor::new(&data[offset..offset + 4]);
    Ok(cursor.read_f32::<LittleEndian>()?)
}
```

**Step 2: Verify it compiles**

Run: `cargo build -p amd-smu-lib`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add -A && git commit -m "feat(lib): implement PM table parsing"
```

---

## Task 6: Implement SMU Reader

**Files:**
- Create: `crates/amd-smu-lib/src/smu.rs`
- Modify: `crates/amd-smu-lib/src/lib.rs`

**Step 1: Create SMU reader**

`crates/amd-smu-lib/src/smu.rs`:
```rust
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
```

**Step 2: Export from lib.rs**

Update `crates/amd-smu-lib/src/lib.rs`:
```rust
mod codename;
mod error;
mod pmtable;
mod smu;

pub use codename::Codename;
pub use error::{Result, SmuError};
pub use pmtable::{PmTable, MAX_CORES};
pub use smu::SmuReader;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

**Step 3: Verify it compiles**

Run: `cargo build -p amd-smu-lib`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add -A && git commit -m "feat(lib): implement SMU reader"
```

---

## Task 7: Implement CLI Argument Parsing

**Files:**
- Modify: `crates/amd-smu-cli/src/main.rs`

**Step 1: Implement CLI arguments with clap**

`crates/amd-smu-cli/src/main.rs`:
```rust
use clap::Parser;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "amd-smu-sensors")]
#[command(about = "Read AMD Ryzen CPU sensors via ryzen_smu kernel module")]
#[command(version)]
pub struct Args {
    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Watch mode: continuously update readings
    #[arg(short, long)]
    pub watch: bool,

    /// Update interval for watch mode (e.g., "500ms", "1s")
    #[arg(short, long, default_value = "1s", value_parser = parse_duration)]
    pub interval: Duration,

    /// Show only temperature readings
    #[arg(long)]
    pub temps: bool,

    /// Show only power readings
    #[arg(long)]
    pub power: bool,

    /// Show only frequency readings
    #[arg(long)]
    pub freq: bool,

    /// Launch TUI dashboard
    #[arg(long)]
    pub tui: bool,
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    humantime::parse_duration(s).map_err(|e| e.to_string())
}

fn main() {
    let args = Args::parse();

    if args.tui {
        eprintln!("TUI mode not yet implemented. Use amd-smu-tui binary.");
        std::process::exit(1);
    }

    println!("amd-smu-sensors v{}", amd_smu_lib::version());
    println!("Args: {:?}", args);
}
```

**Step 2: Verify it compiles and runs**

Run: `cargo run -p amd-smu-cli -- --help`
Expected: Shows help output with all options

**Step 3: Commit**

```bash
git add -A && git commit -m "feat(cli): implement argument parsing"
```

---

## Task 8: Implement Text Output Formatter

**Files:**
- Create: `crates/amd-smu-cli/src/output.rs`
- Modify: `crates/amd-smu-cli/src/main.rs`

**Step 1: Create output formatter**

`crates/amd-smu-cli/src/output.rs`:
```rust
use amd_smu_lib::PmTable;

pub struct OutputOptions {
    pub temps_only: bool,
    pub power_only: bool,
    pub freq_only: bool,
}

impl OutputOptions {
    pub fn show_all(&self) -> bool {
        !self.temps_only && !self.power_only && !self.freq_only
    }
}

pub fn format_text(table: &PmTable, smu_version: &str, opts: &OutputOptions) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!("AMD Ryzen ({})\n", table.codename_str));
    out.push_str(&format!("{} | PM Table v{:#x}\n", smu_version, table.version));
    out.push('\n');

    // Temperatures
    if opts.show_all() || opts.temps_only {
        out.push_str("Temperatures:\n");
        out.push_str(&format!("  Tctl:           {:+.1}°C  (limit: {:.1}°C)\n",
            table.tctl, table.thm_limit));
        out.push_str(&format!("  SoC:            {:+.1}°C\n", table.soc_temp));

        for (i, temp) in table.core_temps.iter().enumerate() {
            if *temp > 0.0 {
                out.push_str(&format!("  Core {:2}:        {:+.1}°C\n", i, temp));
            }
        }
        out.push('\n');
    }

    // Power
    if opts.show_all() || opts.power_only {
        out.push_str("Power:\n");
        out.push_str(&format!("  Package:        {:.1}W / {:.1}W (PPT)\n",
            table.ppt_value, table.ppt_limit));
        out.push_str(&format!("  TDC:            {:.1}A / {:.1}A\n",
            table.tdc_value, table.tdc_limit));
        out.push_str(&format!("  EDC:            {:.1}A / {:.1}A\n",
            table.edc_value, table.edc_limit));
        out.push_str(&format!("  SoC:            {:.1}W\n", table.soc_power));

        for (i, power) in table.core_power.iter().enumerate() {
            if *power > 0.0 {
                out.push_str(&format!("  Core {:2}:        {:.2}W\n", i, power));
            }
        }
        out.push('\n');
    }

    // Frequencies
    if opts.show_all() || opts.freq_only {
        out.push_str("Frequencies:\n");
        out.push_str(&format!("  FCLK:           {:.0} MHz\n", table.fclk));
        out.push_str(&format!("  MCLK:           {:.0} MHz\n", table.mclk));

        for (i, (freq, eff)) in table.core_freqs.iter()
            .zip(table.core_freqs_eff.iter())
            .enumerate()
        {
            if *freq > 0.0 {
                let c0 = table.core_c0.get(i).unwrap_or(&0.0);
                out.push_str(&format!("  Core {:2}:        {:.0} MHz (eff: {:.0})  C0: {:.1}%\n",
                    i, freq, eff, c0));
            }
        }
        out.push('\n');
    }

    // Voltages
    if opts.show_all() {
        out.push_str("Voltages:\n");
        out.push_str(&format!("  VCore:          {:.3}V\n", table.core_voltage));
        out.push_str(&format!("  VSoC:           {:.3}V\n", table.soc_voltage));
    }

    out
}

pub fn format_json(table: &PmTable) -> String {
    serde_json::to_string_pretty(table).unwrap_or_else(|_| "{}".to_string())
}
```

**Step 2: Update main.rs to use the formatter**

`crates/amd-smu-cli/src/main.rs`:
```rust
mod output;

use amd_smu_lib::SmuReader;
use clap::Parser;
use output::{format_json, format_text, OutputOptions};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "amd-smu-sensors")]
#[command(about = "Read AMD Ryzen CPU sensors via ryzen_smu kernel module")]
#[command(version)]
pub struct Args {
    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Watch mode: continuously update readings
    #[arg(short, long)]
    pub watch: bool,

    /// Update interval for watch mode (e.g., "500ms", "1s")
    #[arg(short, long, default_value = "1s", value_parser = parse_duration)]
    pub interval: Duration,

    /// Show only temperature readings
    #[arg(long)]
    pub temps: bool,

    /// Show only power readings
    #[arg(long)]
    pub power: bool,

    /// Show only frequency readings
    #[arg(long)]
    pub freq: bool,

    /// Launch TUI dashboard
    #[arg(long)]
    pub tui: bool,
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    humantime::parse_duration(s).map_err(|e| e.to_string())
}

fn main() {
    let args = Args::parse();

    if args.tui {
        eprintln!("TUI mode not yet implemented. Use amd-smu-tui binary.");
        std::process::exit(1);
    }

    let reader = match SmuReader::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let smu_version = reader.smu_version().unwrap_or_else(|_| "Unknown".to_string());
    let opts = OutputOptions {
        temps_only: args.temps,
        power_only: args.power,
        freq_only: args.freq,
    };

    if args.watch {
        run_watch_mode(&reader, &smu_version, &opts, args.json, args.interval);
    } else {
        run_single_shot(&reader, &smu_version, &opts, args.json);
    }
}

fn run_single_shot(reader: &SmuReader, smu_version: &str, opts: &OutputOptions, json: bool) {
    match reader.read_pm_table() {
        Ok(table) => {
            if json {
                println!("{}", format_json(&table));
            } else {
                print!("{}", format_text(&table, smu_version, opts));
            }
        }
        Err(e) => {
            eprintln!("Error reading PM table: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_watch_mode(
    reader: &SmuReader,
    smu_version: &str,
    opts: &OutputOptions,
    json: bool,
    interval: Duration,
) {
    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        match reader.read_pm_table() {
            Ok(table) => {
                if json {
                    println!("{}", format_json(&table));
                } else {
                    print!("{}", format_text(&table, smu_version, opts));
                }
            }
            Err(e) => {
                eprintln!("Error reading PM table: {}", e);
            }
        }

        std::thread::sleep(interval);
    }
}
```

**Step 3: Verify it compiles**

Run: `cargo build -p amd-smu-cli`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add -A && git commit -m "feat(cli): implement text and JSON output formatters"
```

---

## Task 9: Implement TUI Application State

**Files:**
- Create: `crates/amd-smu-tui/src/app.rs`
- Modify: `crates/amd-smu-tui/src/main.rs`

**Step 1: Create application state**

`crates/amd-smu-tui/src/app.rs`:
```rust
use amd_smu_lib::{PmTable, SmuReader};
use std::time::Duration;

pub struct App {
    pub reader: SmuReader,
    pub smu_version: String,
    pub pm_table: Option<PmTable>,
    pub error: Option<String>,
    pub interval: Duration,
    pub running: bool,
    pub show_temps: bool,
    pub show_power: bool,
    pub show_freq: bool,
}

impl App {
    pub fn new(interval: Duration) -> Result<Self, String> {
        let reader = SmuReader::new().map_err(|e| e.to_string())?;
        let smu_version = reader.smu_version().unwrap_or_else(|_| "Unknown".to_string());

        Ok(Self {
            reader,
            smu_version,
            pm_table: None,
            error: None,
            interval,
            running: true,
            show_temps: true,
            show_power: true,
            show_freq: true,
        })
    }

    pub fn tick(&mut self) {
        match self.reader.read_pm_table() {
            Ok(table) => {
                self.pm_table = Some(table);
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn toggle_temps(&mut self) {
        self.show_temps = !self.show_temps;
    }

    pub fn toggle_power(&mut self) {
        self.show_power = !self.show_power;
    }

    pub fn toggle_freq(&mut self) {
        self.show_freq = !self.show_freq;
    }

    pub fn increase_interval(&mut self) {
        self.interval = self.interval.saturating_add(Duration::from_millis(100));
    }

    pub fn decrease_interval(&mut self) {
        let new_interval = self.interval.saturating_sub(Duration::from_millis(100));
        if new_interval >= Duration::from_millis(100) {
            self.interval = new_interval;
        }
    }
}
```

**Step 2: Update main.rs skeleton**

`crates/amd-smu-tui/src/main.rs`:
```rust
mod app;

use app::App;
use std::time::Duration;

fn main() {
    let app = match App::new(Duration::from_millis(500)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("TUI app initialized: {}", app.smu_version);
    println!("Full TUI implementation coming next...");
}
```

**Step 3: Verify it compiles**

Run: `cargo build -p amd-smu-tui`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add -A && git commit -m "feat(tui): implement application state"
```

---

## Task 10: Implement TUI Rendering

**Files:**
- Create: `crates/amd-smu-tui/src/ui.rs`
- Modify: `crates/amd-smu-tui/src/main.rs`

**Step 1: Create UI rendering module**

`crates/amd-smu-tui/src/ui.rs`:
```rust
use crate::app::App;
use amd_smu_lib::PmTable;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(1),  // Footer
        ])
        .split(frame.area());

    draw_header(frame, app, chunks[0]);
    draw_main(frame, app, chunks[1]);
    draw_footer(frame, chunks[2]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let codename = app.pm_table.as_ref()
        .map(|t| t.codename_str.as_str())
        .unwrap_or("Unknown");

    let version = app.pm_table.as_ref()
        .map(|t| format!("{:#x}", t.version))
        .unwrap_or_else(|| "?".to_string());

    let title = format!(
        " AMD Ryzen ({}) | {} | PM Table v{} | Refresh: {}ms ",
        codename,
        app.smu_version,
        version,
        app.interval.as_millis()
    );

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn draw_main(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref error) = app.error {
        let error_msg = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        frame.render_widget(error_msg, area);
        return;
    }

    let Some(ref table) = app.pm_table else {
        let loading = Paragraph::new("Loading...")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, area);
        return;
    };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),   // Limits (PPT/TDC/EDC)
            Constraint::Length(6),   // Temperatures
            Constraint::Min(4),      // Cores
        ])
        .split(area);

    if app.show_power {
        draw_limits(frame, table, main_chunks[0]);
    }
    if app.show_temps {
        draw_temps(frame, table, main_chunks[1]);
    }
    if app.show_freq {
        draw_cores(frame, table, main_chunks[2]);
    }
}

fn draw_limits(frame: &mut Frame, table: &PmTable, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    // PPT gauge
    let ppt_pct = (table.ppt_value / table.ppt_limit * 100.0).min(100.0) as u16;
    let ppt_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("PPT (Power)"))
        .gauge_style(Style::default().fg(temp_color(ppt_pct as f32, 70.0, 90.0)))
        .percent(ppt_pct)
        .label(format!("{:.1}W / {:.1}W", table.ppt_value, table.ppt_limit));
    frame.render_widget(ppt_gauge, chunks[0]);

    // TDC gauge
    let tdc_pct = (table.tdc_value / table.tdc_limit * 100.0).min(100.0) as u16;
    let tdc_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("TDC (Current)"))
        .gauge_style(Style::default().fg(temp_color(tdc_pct as f32, 70.0, 90.0)))
        .percent(tdc_pct)
        .label(format!("{:.1}A / {:.1}A", table.tdc_value, table.tdc_limit));
    frame.render_widget(tdc_gauge, chunks[1]);

    // EDC gauge
    let edc_pct = (table.edc_value / table.edc_limit * 100.0).min(100.0) as u16;
    let edc_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("EDC (Peak)"))
        .gauge_style(Style::default().fg(temp_color(edc_pct as f32, 70.0, 90.0)))
        .percent(edc_pct)
        .label(format!("{:.1}A / {:.1}A", table.edc_value, table.edc_limit));
    frame.render_widget(edc_gauge, chunks[2]);
}

fn draw_temps(frame: &mut Frame, table: &PmTable, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Tctl gauge
    let tctl_pct = (table.tctl / table.thm_limit * 100.0).min(100.0) as u16;
    let tctl_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Tctl (Junction)"))
        .gauge_style(Style::default().fg(temp_color(table.tctl, 70.0, 85.0)))
        .percent(tctl_pct)
        .label(format!("{:.1}°C / {:.1}°C", table.tctl, table.thm_limit));
    frame.render_widget(tctl_gauge, chunks[0]);

    // SoC temp
    let soc_pct = (table.soc_temp / 80.0 * 100.0).min(100.0) as u16;
    let soc_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("SoC Temperature"))
        .gauge_style(Style::default().fg(temp_color(table.soc_temp, 50.0, 70.0)))
        .percent(soc_pct)
        .label(format!("{:.1}°C", table.soc_temp));
    frame.render_widget(soc_gauge, chunks[1]);
}

fn draw_cores(frame: &mut Frame, table: &PmTable, area: Rect) {
    let mut lines = Vec::new();

    // Core temps line
    let mut temp_spans = vec![Span::raw("Temps:  ")];
    for (i, temp) in table.core_temps.iter().enumerate() {
        if *temp > 0.0 {
            let color = temp_color(*temp, 70.0, 85.0);
            temp_spans.push(Span::styled(
                format!("C{}: {:5.1}°C  ", i, temp),
                Style::default().fg(color),
            ));
        }
    }
    lines.push(Line::from(temp_spans));

    // Core freqs line
    let mut freq_spans = vec![Span::raw("Freqs:  ")];
    for (i, freq) in table.core_freqs.iter().enumerate() {
        if *freq > 0.0 {
            freq_spans.push(Span::styled(
                format!("C{}: {:4.0}MHz  ", i, freq),
                Style::default().fg(Color::White),
            ));
        }
    }
    lines.push(Line::from(freq_spans));

    // Core power line
    let mut power_spans = vec![Span::raw("Power:  ")];
    for (i, power) in table.core_power.iter().enumerate() {
        if *power > 0.0 {
            power_spans.push(Span::styled(
                format!("C{}: {:5.2}W  ", i, power),
                Style::default().fg(Color::Yellow),
            ));
        }
    }
    lines.push(Line::from(power_spans));

    // C0 residency line
    let mut c0_spans = vec![Span::raw("C0:     ")];
    for (i, c0) in table.core_c0.iter().enumerate() {
        c0_spans.push(Span::styled(
            format!("C{}: {:5.1}%  ", i, c0),
            Style::default().fg(Color::Cyan),
        ));
    }
    lines.push(Line::from(c0_spans));

    let cores = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Per-Core Metrics"));
    frame.render_widget(cores, area);
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(" [q] Quit  [t] Temps  [p] Power  [f] Freq  [+/-] Interval ")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, area);
}

fn temp_color(value: f32, warn: f32, crit: f32) -> Color {
    if value >= crit {
        Color::Red
    } else if value >= warn {
        Color::Yellow
    } else {
        Color::Green
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build -p amd-smu-tui`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add -A && git commit -m "feat(tui): implement UI rendering with gauges and core metrics"
```

---

## Task 11: Implement TUI Event Loop

**Files:**
- Modify: `crates/amd-smu-tui/src/main.rs`

**Step 1: Implement full TUI with event handling**

`crates/amd-smu-tui/src/main.rs`:
```rust
mod app;
mod ui;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = match App::new(Duration::from_millis(500)) {
        Ok(a) => a,
        Err(e) => {
            // Restore terminal before printing error
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Initial data fetch
    app.tick();

    // Run event loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app))?;

        // Poll for events with timeout matching refresh interval
        if event::poll(app.interval)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                        KeyCode::Char('t') => app.toggle_temps(),
                        KeyCode::Char('p') => app.toggle_power(),
                        KeyCode::Char('f') => app.toggle_freq(),
                        KeyCode::Char('+') | KeyCode::Char('=') => app.decrease_interval(),
                        KeyCode::Char('-') => app.increase_interval(),
                        _ => {}
                    }
                }
            }
        }

        // Refresh data
        app.tick();
    }

    Ok(())
}
```

**Step 2: Verify it compiles**

Run: `cargo build -p amd-smu-tui`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add -A && git commit -m "feat(tui): implement event loop with keyboard handling"
```

---

## Task 12: Add Unit Tests for PM Table Parsing

**Files:**
- Create: `crates/amd-smu-lib/src/pmtable/tests.rs`
- Modify: `crates/amd-smu-lib/src/pmtable.rs`

**Step 1: Create test module**

Add at the end of `crates/amd-smu-lib/src/pmtable.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pm_table(core_count: usize) -> Vec<u8> {
        // Create a buffer large enough for all fields
        let size = offsets::CORE_C0_BASE + (core_count * 4) + 4;
        let mut data = vec![0u8; size];

        // Helper to write f32 at offset
        let write_f32 = |data: &mut [u8], offset: usize, value: f32| {
            let bytes = value.to_le_bytes();
            data[offset..offset + 4].copy_from_slice(&bytes);
        };

        // Write test values
        write_f32(&mut data, offsets::PPT_LIMIT, 142.0);
        write_f32(&mut data, offsets::PPT_VALUE, 89.5);
        write_f32(&mut data, offsets::TDC_LIMIT, 95.0);
        write_f32(&mut data, offsets::TDC_VALUE, 62.3);
        write_f32(&mut data, offsets::THM_LIMIT, 90.0);
        write_f32(&mut data, offsets::THM_VALUE, 65.2);
        write_f32(&mut data, offsets::EDC_LIMIT, 140.0);
        write_f32(&mut data, offsets::EDC_VALUE, 98.7);
        write_f32(&mut data, offsets::VDDCR_CPU_POWER, 88.5);
        write_f32(&mut data, offsets::VDDCR_SOC_POWER, 12.4);
        write_f32(&mut data, offsets::CPU_VOLTAGE, 1.35);
        write_f32(&mut data, offsets::SOC_TEMP, 42.1);
        write_f32(&mut data, offsets::SOC_VOLTAGE, 1.10);
        write_f32(&mut data, offsets::FCLK, 1800.0);
        write_f32(&mut data, offsets::MCLK, 1800.0);

        // Write per-core data
        for i in 0..core_count {
            write_f32(&mut data, offsets::CORE_POWER_BASE + i * 4, 8.0 + i as f32 * 0.5);
            write_f32(&mut data, offsets::CORE_TEMP_BASE + i * 4, 60.0 + i as f32 * 0.5);
            write_f32(&mut data, offsets::CORE_FREQ_BASE + i * 4, 4500.0 + i as f32 * 50.0);
            write_f32(&mut data, offsets::CORE_FREQEFF_BASE + i * 4, 4400.0 + i as f32 * 50.0);
            write_f32(&mut data, offsets::CORE_C0_BASE + i * 4, 90.0 + i as f32);
        }

        data
    }

    #[test]
    fn test_parse_limits() {
        let data = create_test_pm_table(8);
        let table = PmTable::parse(&data, 0x240903, Codename::Vermeer, 8).unwrap();

        assert!((table.ppt_limit - 142.0).abs() < 0.01);
        assert!((table.ppt_value - 89.5).abs() < 0.01);
        assert!((table.tdc_limit - 95.0).abs() < 0.01);
        assert!((table.edc_limit - 140.0).abs() < 0.01);
        assert!((table.thm_limit - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_temperatures() {
        let data = create_test_pm_table(8);
        let table = PmTable::parse(&data, 0x240903, Codename::Vermeer, 8).unwrap();

        assert!((table.tctl - 65.2).abs() < 0.01);
        assert!((table.soc_temp - 42.1).abs() < 0.01);
        assert_eq!(table.core_temps.len(), 8);
        assert!((table.core_temps[0] - 60.0).abs() < 0.01);
        assert!((table.core_temps[7] - 63.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_frequencies() {
        let data = create_test_pm_table(8);
        let table = PmTable::parse(&data, 0x240903, Codename::Vermeer, 8).unwrap();

        assert!((table.fclk - 1800.0).abs() < 0.01);
        assert!((table.mclk - 1800.0).abs() < 0.01);
        assert_eq!(table.core_freqs.len(), 8);
        assert!((table.core_freqs[0] - 4500.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_power_and_voltage() {
        let data = create_test_pm_table(8);
        let table = PmTable::parse(&data, 0x240903, Codename::Vermeer, 8).unwrap();

        assert!((table.package_power - 88.5).abs() < 0.01);
        assert!((table.soc_power - 12.4).abs() < 0.01);
        assert!((table.core_voltage - 1.35).abs() < 0.01);
        assert!((table.soc_voltage - 1.10).abs() < 0.01);
    }

    #[test]
    fn test_invalid_size() {
        let data = vec![0u8; 100]; // Too small
        let result = PmTable::parse(&data, 0x240903, Codename::Vermeer, 8);
        assert!(matches!(result, Err(SmuError::InvalidPmTableSize { .. })));
    }

    #[test]
    fn test_different_core_counts() {
        for cores in [4, 8, 12, 16] {
            let data = create_test_pm_table(cores);
            let table = PmTable::parse(&data, 0x240903, Codename::Vermeer, cores).unwrap();
            assert_eq!(table.core_temps.len(), cores);
            assert_eq!(table.core_freqs.len(), cores);
            assert_eq!(table.core_power.len(), cores);
        }
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p amd-smu-lib`
Expected: All tests pass

**Step 3: Commit**

```bash
git add -A && git commit -m "test(lib): add unit tests for PM table parsing"
```

---

## Task 13: Add Integration Test with Mock Sysfs

**Files:**
- Create: `crates/amd-smu-lib/tests/integration.rs`

**Step 1: Create integration test**

`crates/amd-smu-lib/tests/integration.rs`:
```rust
use amd_smu_lib::{Codename, SmuReader};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

fn create_mock_sysfs() -> TempDir {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    // Create mock sysfs files
    fs::write(path.join("version"), "SMU v46.54.0\n").unwrap();
    fs::write(path.join("drv_version"), "0.1.7\n").unwrap();
    fs::write(path.join("codename"), "12\n").unwrap(); // Vermeer
    fs::write(path.join("pm_table_version"), "0x240903\n").unwrap();
    fs::write(path.join("pm_table_size"), "6832\n").unwrap();

    // Create mock PM table
    let pm_table = create_mock_pm_table();
    let mut file = fs::File::create(path.join("pm_table")).unwrap();
    file.write_all(&pm_table).unwrap();

    dir
}

fn create_mock_pm_table() -> Vec<u8> {
    let mut data = vec![0u8; 6832];

    let write_f32 = |data: &mut [u8], offset: usize, value: f32| {
        let bytes = value.to_le_bytes();
        data[offset..offset + 4].copy_from_slice(&bytes);
    };

    // Limits and values
    write_f32(&mut data, 0x000, 142.0);  // PPT_LIMIT
    write_f32(&mut data, 0x004, 89.5);   // PPT_VALUE
    write_f32(&mut data, 0x008, 95.0);   // TDC_LIMIT
    write_f32(&mut data, 0x00C, 62.3);   // TDC_VALUE
    write_f32(&mut data, 0x010, 90.0);   // THM_LIMIT
    write_f32(&mut data, 0x014, 65.2);   // THM_VALUE (Tctl)
    write_f32(&mut data, 0x020, 140.0);  // EDC_LIMIT
    write_f32(&mut data, 0x024, 98.7);   // EDC_VALUE
    write_f32(&mut data, 0x060, 88.5);   // CPU_POWER
    write_f32(&mut data, 0x064, 12.4);   // SOC_POWER
    write_f32(&mut data, 0x0A0, 1.35);   // CPU_VOLTAGE
    write_f32(&mut data, 0x0A8, 42.1);   // SOC_TEMP
    write_f32(&mut data, 0x0B4, 1.10);   // SOC_VOLTAGE
    write_f32(&mut data, 0x0C0, 1800.0); // FCLK
    write_f32(&mut data, 0x0C8, 1800.0); // MCLK

    // Per-core data (8 cores)
    for i in 0..8 {
        write_f32(&mut data, 0x24C + i * 4, 8.0 + i as f32 * 0.5);   // CORE_POWER
        write_f32(&mut data, 0x2C0 + i * 4, 60.0 + i as f32 * 0.5);  // CORE_TEMP
        write_f32(&mut data, 0x2EC + i * 4, 4500.0 + i as f32 * 50.0); // CORE_FREQ
        write_f32(&mut data, 0x30C + i * 4, 4400.0 + i as f32 * 50.0); // CORE_FREQEFF
        write_f32(&mut data, 0x32C + i * 4, 90.0 + i as f32);        // CORE_C0
    }

    data
}

#[test]
fn test_smu_reader_with_mock_sysfs() {
    let mock_dir = create_mock_sysfs();
    let reader = SmuReader::with_path(mock_dir.path()).unwrap();

    assert_eq!(reader.smu_version().unwrap().trim(), "SMU v46.54.0");
    assert_eq!(reader.driver_version().unwrap().trim(), "0.1.7");
    assert_eq!(reader.codename().unwrap(), Codename::Vermeer);
    assert_eq!(reader.pm_table_version().unwrap(), 0x240903);
}

#[test]
fn test_read_pm_table_with_mock() {
    let mock_dir = create_mock_sysfs();
    let reader = SmuReader::with_path(mock_dir.path()).unwrap();
    let table = reader.read_pm_table().unwrap();

    assert!((table.tctl - 65.2).abs() < 0.01);
    assert!((table.soc_temp - 42.1).abs() < 0.01);
    assert!((table.ppt_limit - 142.0).abs() < 0.01);
    assert!((table.fclk - 1800.0).abs() < 0.01);
}

#[test]
fn test_module_not_loaded() {
    let result = SmuReader::with_path("/nonexistent/path");
    assert!(result.is_err());
}
```

**Step 2: Add tempfile dev dependency**

Add to `crates/amd-smu-lib/Cargo.toml`:
```toml
[dev-dependencies]
tempfile = "3"
```

**Step 3: Run tests**

Run: `cargo test -p amd-smu-lib`
Expected: All tests pass

**Step 4: Commit**

```bash
git add -A && git commit -m "test(lib): add integration tests with mock sysfs"
```

---

## Task 14: Final Polish and Documentation

**Files:**
- Update: `README.md` (create if not exists beyond license)

**Step 1: Create README**

Create `README.md`:
```markdown
# AMD SMU Tools

Rust CLI tools for reading AMD Ryzen processor metrics via the `ryzen_smu` kernel module.

## Features

- Read CPU temperatures (Tctl, SoC, per-core)
- Monitor power consumption (PPT, TDC, EDC, per-core)
- Track frequencies (per-core, FCLK, MCLK)
- View voltages and C0 residency
- Text and JSON output formats
- Watch mode with configurable interval
- Live TUI dashboard

## Requirements

- AMD Ryzen processor (Matisse/Vermeer/Raphael/etc.)
- [ryzen_smu](https://github.com/leogx9r/ryzen_smu) kernel module loaded
- Root access (or configured udev rules)

## Installation

```bash
cargo install --path crates/amd-smu-cli
cargo install --path crates/amd-smu-tui
```

## Usage

### CLI Tool

```bash
# Single reading (text output)
sudo amd-smu-sensors

# JSON output
sudo amd-smu-sensors --json

# Watch mode (updates every second)
sudo amd-smu-sensors --watch

# Custom interval
sudo amd-smu-sensors --watch --interval 500ms

# Filter output
sudo amd-smu-sensors --temps   # Temperatures only
sudo amd-smu-sensors --power   # Power only
sudo amd-smu-sensors --freq    # Frequencies only
```

### TUI Dashboard

```bash
sudo amd-smu-tui
```

**Keyboard shortcuts:**
- `q` / `Esc` - Quit
- `t` - Toggle temperatures
- `p` - Toggle power
- `f` - Toggle frequencies
- `+` / `-` - Adjust refresh interval

## Library Usage

```rust
use amd_smu_lib::SmuReader;

fn main() -> amd_smu_lib::Result<()> {
    let reader = SmuReader::new()?;
    let table = reader.read_pm_table()?;

    println!("Tctl: {:.1}°C", table.tctl);
    println!("Package Power: {:.1}W", table.ppt_value);

    for (i, temp) in table.core_temps.iter().enumerate() {
        println!("Core {}: {:.1}°C", i, temp);
    }

    Ok(())
}
```

## License

MIT
```

**Step 2: Verify full workspace builds and tests pass**

Run: `cargo build --workspace && cargo test --workspace`
Expected: Everything builds and all tests pass

**Step 3: Commit**

```bash
git add -A && git commit -m "docs: add README with usage documentation"
```

---

## Summary

This plan creates a complete Rust workspace with:

1. **amd-smu-lib** - Core library for reading PM table from ryzen_smu kernel module
2. **amd-smu-cli** - CLI tool with text/JSON output, watch mode, filtering
3. **amd-smu-tui** - Live TUI dashboard with gauges and keyboard controls

Each task includes a commit, building up the functionality incrementally with tests.
