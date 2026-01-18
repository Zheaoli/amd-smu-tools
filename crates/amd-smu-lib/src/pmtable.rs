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

/// PM table offset definitions for different processor generations
mod offsets {
    /// Offset structure for PM table fields
    #[derive(Debug, Clone, Copy)]
    pub struct PmTableOffsets {
        pub ppt_limit: usize,
        pub ppt_value: usize,
        pub tdc_limit: usize,
        pub tdc_value: usize,
        pub thm_limit: usize,
        pub thm_value: usize,     // Tctl/junction temp
        pub edc_limit: usize,
        pub edc_value: usize,
        pub cpu_power: usize,     // Package/CPU power
        pub soc_power: usize,
        pub cpu_voltage: usize,
        pub soc_voltage: usize,
        pub fclk: usize,
        pub mclk: usize,
        pub soc_temp: usize,
        pub core_power_base: usize,
        pub core_temp_base: usize,
        pub core_freq_base: usize,
        pub core_freqeff_base: usize,
        pub core_c0_base: usize,
        pub max_cores: usize,
    }

    /// PM table offsets for version 0x240903 (Matisse/Vermeer - Zen 2/3)
    pub const OFFSETS_0X240903: PmTableOffsets = PmTableOffsets {
        ppt_limit: 0x000,
        ppt_value: 0x004,
        tdc_limit: 0x008,
        tdc_value: 0x00C,
        thm_limit: 0x010,
        thm_value: 0x014,
        edc_limit: 0x020,
        edc_value: 0x024,
        cpu_power: 0x060,
        soc_power: 0x064,
        cpu_voltage: 0x0A0,
        soc_voltage: 0x0B4,
        fclk: 0x0C0,
        mclk: 0x0CC,
        soc_temp: 0x1CC,
        core_power_base: 0x24C,
        core_temp_base: 0x28C,
        core_freq_base: 0x2EC,
        core_freqeff_base: 0x30C,
        core_c0_base: 0x32C,
        max_cores: 16,
    };

    /// PM table offsets for version 0x00620205 (Granite Ridge - Zen 5)
    /// Reverse-engineered from actual PM table data on 9950X3D
    /// Note: Per-core frequencies not available in PM table, use /proc/cpuinfo instead
    pub const OFFSETS_0X620205: PmTableOffsets = PmTableOffsets {
        ppt_limit: 0x020,         // 160W
        ppt_value: 0x024,         // Current package power
        tdc_limit: 0x028,         // 95A
        tdc_value: 0x02C,         // Current TDC
        thm_limit: 0x008,         // 200°C thermal limit
        thm_value: 0x00C,         // Tctl junction temp
        edc_limit: 0x0FC,         // 225A
        edc_value: 0x100,         // Current EDC
        cpu_power: 0x024,         // Same as ppt_value (package power)
        soc_power: 0x054,         // SoC power ~18W
        cpu_voltage: 0x048,       // ~1.36V
        soc_voltage: 0x04C,       // ~1.22V
        fclk: 0x11C,              // 2000 MHz
        mclk: 0x12C,              // 2800 MHz
        soc_temp: 0x0F8,          // ~47-49°C
        core_power_base: 0x4B4,   // Per-core power (~0.5-2W each, sum ≈ package power)
        core_temp_base: 0x534,    // Per-core temps
        core_freq_base: 0xFFFF,   // Not available in PM table - use 0xFFFF as marker
        core_freqeff_base: 0xFFFF, // Not available in PM table
        core_c0_base: 0xFFFF,     // Not available in PM table
        max_cores: 16,
    };

    /// Get the appropriate offsets for a given PM table version
    pub fn get_offsets(version: u32) -> Option<PmTableOffsets> {
        match version {
            0x240903 => Some(OFFSETS_0X240903),
            0x00620205 => Some(OFFSETS_0X620205),
            _ => None,
        }
    }
}

impl PmTable {
    /// Parse PM table from raw bytes
    pub fn parse(data: &[u8], version: u32, codename: Codename, core_count: usize) -> Result<Self> {
        // Get offsets for this PM table version
        let off = offsets::get_offsets(version).ok_or_else(|| {
            SmuError::UnsupportedPmTableVersion(version)
        })?;

        // Minimum size check based on the largest per-core offset (excluding 0xFFFF markers)
        let max_per_core_base = [
            off.core_c0_base,
            off.core_power_base,
            off.core_temp_base,
            off.core_freq_base,
            off.core_freqeff_base,
        ].into_iter()
            .filter(|&x| x < 0xFFFF)  // Exclude marker values
            .max()
            .unwrap_or(0);
        let min_size = max_per_core_base + (core_count * 4);
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
        table.ppt_limit = read_f32(data, off.ppt_limit)?;
        table.ppt_value = read_f32(data, off.ppt_value)?;
        table.tdc_limit = read_f32(data, off.tdc_limit)?;
        table.tdc_value = read_f32(data, off.tdc_value)?;
        table.thm_limit = read_f32(data, off.thm_limit)?;
        table.tctl = read_f32(data, off.thm_value)?;
        table.edc_limit = read_f32(data, off.edc_limit)?;
        table.edc_value = read_f32(data, off.edc_value)?;

        // Parse power
        table.package_power = read_f32(data, off.cpu_power)?;
        table.soc_power = read_f32(data, off.soc_power)?;

        // Parse voltages and temps
        table.core_voltage = read_f32(data, off.cpu_voltage)?;
        table.soc_temp = read_f32(data, off.soc_temp)?;
        table.soc_voltage = read_f32(data, off.soc_voltage)?;

        // Parse clocks
        table.fclk = read_f32(data, off.fclk)?;
        table.mclk = read_f32(data, off.mclk)?;

        // Parse per-core data (limit to actual core count and available data)
        let actual_cores = core_count.min(off.max_cores);
        for i in 0..actual_cores {
            // Safely read per-core data, using 0.0 if offset is 0xFFFF (not available) or out of bounds
            let power_off = off.core_power_base + i * 4;
            let temp_off = off.core_temp_base + i * 4;

            table.core_power.push(read_f32_safe_with_marker(data, power_off));
            table.core_temps.push(read_f32_safe_with_marker(data, temp_off));

            // For frequency and C0, check if offset is marked as unavailable (0xFFFF)
            if off.core_freq_base != 0xFFFF {
                let freq_off = off.core_freq_base + i * 4;
                let freqeff_off = off.core_freqeff_base + i * 4;
                table.core_freqs.push(read_f32_safe_with_marker(data, freq_off));
                table.core_freqs_eff.push(read_f32_safe_with_marker(data, freqeff_off));
            }

            if off.core_c0_base != 0xFFFF {
                let c0_off = off.core_c0_base + i * 4;
                table.core_c0.push(read_f32_safe_with_marker(data, c0_off));
            }
        }

        // If frequencies are not in PM table, try to read from /proc/cpuinfo
        if off.core_freq_base == 0xFFFF {
            if let Ok(freqs) = read_cpuinfo_frequencies(actual_cores) {
                table.core_freqs = freqs.clone();
                table.core_freqs_eff = freqs;
            }
        }

        Ok(table)
    }
}

/// Read a little-endian f32 from buffer at offset
fn read_f32(data: &[u8], offset: usize) -> Result<f32> {
    if offset + 4 > data.len() {
        return Err(SmuError::InvalidPmTableSize {
            expected: offset + 4,
            actual: data.len(),
        });
    }
    let mut cursor = Cursor::new(&data[offset..offset + 4]);
    Ok(cursor.read_f32::<LittleEndian>()?)
}

/// Read a little-endian f32, returning 0.0 if offset is marker (0xFFFF) or out of bounds
fn read_f32_safe_with_marker(data: &[u8], offset: usize) -> f32 {
    if offset >= 0xFFFF || offset + 4 > data.len() {
        return 0.0;
    }
    let mut cursor = Cursor::new(&data[offset..offset + 4]);
    cursor.read_f32::<LittleEndian>().unwrap_or(0.0)
}

/// Read CPU frequencies from /proc/cpuinfo
fn read_cpuinfo_frequencies(core_count: usize) -> std::io::Result<Vec<f32>> {
    use std::fs;

    let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;
    let mut freqs = Vec::with_capacity(core_count);

    for line in cpuinfo.lines() {
        if line.starts_with("cpu MHz") {
            if let Some(value_str) = line.split(':').nth(1) {
                if let Ok(freq) = value_str.trim().parse::<f32>() {
                    freqs.push(freq);
                    if freqs.len() >= core_count {
                        break;
                    }
                }
            }
        }
    }

    // Pad with zeros if we didn't get enough values
    while freqs.len() < core_count {
        freqs.push(0.0);
    }

    Ok(freqs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pm_table(core_count: usize, version: u32) -> Vec<u8> {
        let off = offsets::get_offsets(version).unwrap();
        // Calculate size based on the maximum offset we'll use (find max of all per-core bases, excluding 0xFFFF markers)
        let max_base = [
            off.core_c0_base,
            off.core_power_base,
            off.core_temp_base,
            off.core_freq_base,
            off.core_freqeff_base,
        ].into_iter()
            .filter(|&x| x < 0xFFFF)
            .max()
            .unwrap_or(0);
        let size = max_base + (core_count * 4) + 4;
        let mut data = vec![0u8; size];

        // Helper to write f32 at offset
        let write_f32 = |data: &mut [u8], offset: usize, value: f32| {
            let bytes = value.to_le_bytes();
            data[offset..offset + 4].copy_from_slice(&bytes);
        };

        // Write test values at correct offsets for this version
        write_f32(&mut data, off.ppt_limit, 142.0);
        write_f32(&mut data, off.ppt_value, 89.5);
        write_f32(&mut data, off.tdc_limit, 95.0);
        write_f32(&mut data, off.tdc_value, 62.3);
        write_f32(&mut data, off.thm_limit, 90.0);
        write_f32(&mut data, off.thm_value, 65.2);
        write_f32(&mut data, off.edc_limit, 140.0);
        write_f32(&mut data, off.edc_value, 98.7);
        write_f32(&mut data, off.cpu_power, 88.5);
        write_f32(&mut data, off.soc_power, 12.4);
        write_f32(&mut data, off.cpu_voltage, 1.35);
        write_f32(&mut data, off.soc_voltage, 1.10);
        write_f32(&mut data, off.fclk, 1800.0);
        write_f32(&mut data, off.mclk, 1800.0);
        write_f32(&mut data, off.soc_temp, 42.1);

        // Write per-core data at correct offsets (skip 0xFFFF marker offsets)
        for i in 0..core_count {
            if off.core_power_base < 0xFFFF {
                write_f32(&mut data, off.core_power_base + i * 4, 8.0 + i as f32 * 0.5);
            }
            if off.core_temp_base < 0xFFFF {
                write_f32(&mut data, off.core_temp_base + i * 4, 60.0 + i as f32 * 0.5);
            }
            if off.core_freq_base < 0xFFFF {
                write_f32(&mut data, off.core_freq_base + i * 4, 4500.0 + i as f32 * 50.0);
            }
            if off.core_freqeff_base < 0xFFFF {
                write_f32(&mut data, off.core_freqeff_base + i * 4, 4400.0 + i as f32 * 50.0);
            }
            if off.core_c0_base < 0xFFFF {
                write_f32(&mut data, off.core_c0_base + i * 4, 90.0 + i as f32);
            }
        }

        data
    }

    #[test]
    fn test_parse_limits() {
        let data = create_test_pm_table(8, 0x240903);
        let table = PmTable::parse(&data, 0x240903, Codename::Vermeer, 8).unwrap();

        assert!((table.ppt_limit - 142.0).abs() < 0.01);
        assert!((table.ppt_value - 89.5).abs() < 0.01);
        assert!((table.tdc_limit - 95.0).abs() < 0.01);
        assert!((table.edc_limit - 140.0).abs() < 0.01);
        assert!((table.thm_limit - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_temperatures() {
        let data = create_test_pm_table(8, 0x240903);
        let table = PmTable::parse(&data, 0x240903, Codename::Vermeer, 8).unwrap();

        assert!((table.tctl - 65.2).abs() < 0.01);
        assert!((table.soc_temp - 42.1).abs() < 0.01);
        assert_eq!(table.core_temps.len(), 8);
        assert!((table.core_temps[0] - 60.0).abs() < 0.01);
        assert!((table.core_temps[7] - 63.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_frequencies() {
        let data = create_test_pm_table(8, 0x240903);
        let table = PmTable::parse(&data, 0x240903, Codename::Vermeer, 8).unwrap();

        assert!((table.fclk - 1800.0).abs() < 0.01);
        assert!((table.mclk - 1800.0).abs() < 0.01);
        assert_eq!(table.core_freqs.len(), 8);
        assert!((table.core_freqs[0] - 4500.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_power_and_voltage() {
        let data = create_test_pm_table(8, 0x240903);
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
    fn test_unsupported_version() {
        let data = vec![0u8; 1000];
        let result = PmTable::parse(&data, 0x999999, Codename::Vermeer, 8);
        assert!(matches!(result, Err(SmuError::UnsupportedPmTableVersion(_))));
    }

    #[test]
    fn test_different_core_counts() {
        for cores in [4, 8, 12, 16] {
            let data = create_test_pm_table(cores, 0x240903);
            let table = PmTable::parse(&data, 0x240903, Codename::Vermeer, cores).unwrap();
            assert_eq!(table.core_temps.len(), cores);
            assert_eq!(table.core_freqs.len(), cores);
            assert_eq!(table.core_power.len(), cores);
        }
    }

    #[test]
    fn test_granite_ridge_offsets() {
        let data = create_test_pm_table(16, 0x00620205);
        let table = PmTable::parse(&data, 0x00620205, Codename::GraniteRidge, 16).unwrap();

        assert!((table.ppt_limit - 142.0).abs() < 0.01);
        assert!((table.tctl - 65.2).abs() < 0.01);
        assert!((table.soc_temp - 42.1).abs() < 0.01);
        assert_eq!(table.core_temps.len(), 16);
    }
}
