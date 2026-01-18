use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use crate::{Result, SmuError};
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
