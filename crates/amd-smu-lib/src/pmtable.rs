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

/// PM table offsets for version 0x240903 (Matisse/Vermeer)
/// Calculated from pm_table_0x240903 struct in monitor_cpu.c
mod offsets {
    // Limits and values (fields 1-12)
    pub const PPT_LIMIT: usize = 0x000;      // field 1
    pub const PPT_VALUE: usize = 0x004;      // field 2
    pub const TDC_LIMIT: usize = 0x008;      // field 3
    pub const TDC_VALUE: usize = 0x00C;      // field 4
    pub const THM_LIMIT: usize = 0x010;      // field 5
    pub const THM_VALUE: usize = 0x014;      // field 6 - Tctl/junction temp
    pub const EDC_LIMIT: usize = 0x020;      // field 9
    pub const EDC_VALUE: usize = 0x024;      // field 10

    // Power (fields 25-26)
    pub const VDDCR_CPU_POWER: usize = 0x060; // field 25
    pub const VDDCR_SOC_POWER: usize = 0x064; // field 26

    // Telemetry (fields 41-48)
    pub const CPU_VOLTAGE: usize = 0x0A0;     // field 41 - CPU_TELEMETRY_VOLTAGE
    pub const SOC_VOLTAGE: usize = 0x0B4;     // field 46 - SOC_TELEMETRY_VOLTAGE

    // Clocks (fields 49-52)
    pub const FCLK: usize = 0x0C0;            // field 49 - FCLK_FREQ
    pub const MCLK: usize = 0x0CC;            // field 52 - MEMCLK_FREQ

    // SOC temperature (field 116)
    pub const SOC_TEMP: usize = 0x1CC;        // field 116 - SOC_TEMP

    // Per-core arrays (8 elements each)
    pub const CORE_POWER_BASE: usize = 0x24C; // field 148 - CORE_POWER[0]
    pub const CORE_TEMP_BASE: usize = 0x28C;  // field 164 - CORE_TEMP[0]
    pub const CORE_FREQ_BASE: usize = 0x2EC;  // field 188 - CORE_FREQ[0]
    pub const CORE_FREQEFF_BASE: usize = 0x30C; // field 196 - CORE_FREQEFF[0]
    pub const CORE_C0_BASE: usize = 0x32C;    // field 204 - CORE_C0[0]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pm_table(core_count: usize) -> Vec<u8> {
        // PM table 0x240903 is 0x518 bytes (1304 bytes) = 326 floats
        // But we need at least up to CORE_C0 array end
        let size = offsets::CORE_C0_BASE + (core_count * 4) + 4;
        let mut data = vec![0u8; size];

        // Helper to write f32 at offset
        let write_f32 = |data: &mut [u8], offset: usize, value: f32| {
            let bytes = value.to_le_bytes();
            data[offset..offset + 4].copy_from_slice(&bytes);
        };

        // Write test values at correct offsets
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
        write_f32(&mut data, offsets::SOC_VOLTAGE, 1.10);
        write_f32(&mut data, offsets::FCLK, 1800.0);
        write_f32(&mut data, offsets::MCLK, 1800.0);
        write_f32(&mut data, offsets::SOC_TEMP, 42.1);  // Now at correct offset 0x1CC

        // Write per-core data at correct offsets
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
