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
