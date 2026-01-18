# AMD SMU Sensors - Rust CLI Tool Design

## Overview

A Rust-based CLI tool and library for reading AMD Ryzen processor metrics via the `ryzen_smu` kernel module. Provides functionality similar to `lm_sensors` with per-CCD and per-core temperature monitoring, plus frequencies, power, and voltages.

## Project Structure

```
amd-smu-tools/
├── Cargo.toml                 # Workspace definition
├── crates/
│   ├── amd-smu-lib/          # Core library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs        # Public API
│   │       ├── smu.rs        # SMU interface (read pm_table, smn, etc.)
│   │       ├── pmtable.rs    # PM table parsing & field definitions
│   │       ├── codename.rs   # Processor codename detection
│   │       └── error.rs      # Error types
│   │
│   ├── amd-smu-cli/          # Command-line tool
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs       # Entry point & arg parsing
│   │       ├── output.rs     # Text/JSON formatters
│   │       └── watch.rs      # Watch mode loop
│   │
│   └── amd-smu-tui/          # TUI dashboard
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs       # TUI entry point
│           ├── app.rs        # Application state
│           └── ui.rs         # Layout & widgets
```

## Core Library (`amd-smu-lib`)

### Data Structures

```rust
pub struct SmuReader {
    sysfs_path: PathBuf,  // /sys/kernel/ryzen_smu_drv
}

pub struct PmTable {
    pub version: u32,
    pub codename: Codename,

    // Limits
    pub ppt_limit: f32,    // Package Power (W)
    pub tdc_limit: f32,    // Thermal Design Current (A)
    pub edc_limit: f32,    // Electrical Design Current (A)
    pub thm_limit: f32,    // Thermal limit (°C)

    // Current values
    pub ppt_value: f32,
    pub tdc_value: f32,
    pub edc_value: f32,

    // Temperatures
    pub tctl: f32,         // Junction temp
    pub soc_temp: f32,
    pub core_temps: Vec<f32>,

    // Frequencies
    pub core_freqs: Vec<f32>,
    pub core_freqs_eff: Vec<f32>,
    pub fclk: f32,
    pub mclk: f32,

    // Power
    pub core_power: Vec<f32>,
    pub package_power: f32,
    pub soc_power: f32,

    // Voltages & residency
    pub core_voltage: f32,
    pub soc_voltage: f32,
    pub core_c0: Vec<f32>,
}
```

### API

```rust
impl SmuReader {
    pub fn new() -> Result<Self>;           // Auto-detect sysfs path
    pub fn read_pm_table(&self) -> Result<PmTable>;
    pub fn version(&self) -> Result<String>;
    pub fn codename(&self) -> Result<Codename>;
}
```

### PM Table Offsets (Version 0x240903)

| Offset | Field | Description |
|--------|-------|-------------|
| 0x000 | PPT_LIMIT | Package Power limit (W) |
| 0x004 | PPT_VALUE | Current PPT (W) |
| 0x008 | TDC_LIMIT | Thermal Design Current limit (A) |
| 0x00C | TDC_VALUE | Current TDC (A) |
| 0x010 | THM_LIMIT | Thermal limit (°C) |
| 0x014 | THM_VALUE | Junction Temperature (°C) |
| 0x020 | EDC_LIMIT | EDC limit (A) |
| 0x024 | EDC_VALUE | Current EDC (A) |
| 0x060 | VDDCR_CPU_POWER | Core power (W) |
| 0x064 | VDDCR_SOC_POWER | SoC power (W) |
| 0x0A0 | CPU_TELEMETRY_VOLTAGE | Core voltage (V) |
| 0x0A8 | SOC_TEMP | SoC Temperature (°C) |
| 0x0B4 | VDDCR_SOC_VOLTAGE | SoC voltage (V) |
| 0x0C0 | FCLK_FREQ | Fabric clock (MHz) |
| 0x24C + (i×4) | CORE_POWER[i] | Per-core power (W) |
| 0x2C0 + (i×4) | CORE_TEMP[i] | Per-core temperature (°C) |
| 0x2EC + (i×4) | CORE_FREQ[i] | Per-core frequency (MHz) |
| 0x30C + (i×4) | CORE_FREQEFF[i] | Effective frequency (MHz) |
| 0x32C + (i×4) | CORE_C0[i] | Per-core C0 residency (%) |

## CLI Tool (`amd-smu-cli`)

### Command Interface

```bash
# Single shot (default) - text output
amd-smu-sensors

# JSON output
amd-smu-sensors --json

# Watch mode - refresh every second
amd-smu-sensors --watch
amd-smu-sensors --watch --interval 500ms

# Filter specific metrics
amd-smu-sensors --temps          # Temperatures only
amd-smu-sensors --power          # Power only
amd-smu-sensors --freq           # Frequencies only

# Launch TUI dashboard
amd-smu-sensors --tui
# Or as separate binary:
amd-smu-tui
```

### Text Output Format

```
AMD Ryzen 9 5950X (Vermeer)
SMU v46.54.0 | PM Table v0x240903

Temperatures:
  Tctl:           +65.2°C  (limit: 90.0°C)
  SoC:            +42.1°C
  CCD0 Core 0:    +62.5°C
  CCD0 Core 1:    +61.8°C
  ...

Power:
  Package:        89.2W / 142.0W (PPT)
  SoC:            12.4W
  Core 0:          8.2W
  ...

Frequencies:
  Core 0:         4850 MHz (eff: 4720 MHz)  C0: 98.2%
  ...
  FCLK:           1800 MHz
```

## TUI Dashboard (`amd-smu-tui`)

### Layout

```
┌─ AMD Ryzen 9 5950X (Vermeer) ─────────────────────────────────────┐
│ SMU v46.54.0 │ PM Table v0x240903 │ Refresh: 500ms               │
├─ Temperatures ────────────────────┬─ Power ──────────────────────┤
│ Tctl    [██████████░░░░] 65.2°C   │ PPT  [████████░░░░] 89W/142W │
│ SoC     [████░░░░░░░░░░] 42.1°C   │ TDC  [██████░░░░░░] 62A/95A  │
│                                   │ EDC  [████████░░░░] 98A/140A │
├─ Core Temperatures ───────────────┴──────────────────────────────┤
│  C0: 62.5°C  C1: 61.8°C  C2: 58.2°C  C3: 59.1°C                  │
│  C4: 60.2°C  C5: 57.9°C  C6: 58.8°C  C7: 59.5°C                  │
│  C8: 55.1°C  C9: 54.2°C  C10: 53.8°C ...                         │
├─ Core Frequencies (MHz) ─────────────────────────────────────────┤
│  C0: 4850 eff:4720  C1: 4825 eff:4690  C2: 4200 eff:3980  ...    │
├─ Core Power & C0 Residency ──────────────────────────────────────┤
│  C0: 8.2W 98%  C1: 7.9W 95%  C2: 4.1W 62%  C3: 4.5W 71%  ...     │
├──────────────────────────────────────────────────────────────────┤
│ FCLK: 1800MHz │ MCLK: 1800MHz │ VCore: 1.35V │ VSoC: 1.10V       │
└─ [q]uit  [+/-] interval  [t]emps  [p]ower  [f]req ───────────────┘
```

### Features

- Colored progress bars (green → yellow → red based on thresholds)
- Keyboard shortcuts to toggle sections
- Adjustable refresh interval
- Graceful terminal restore on exit/panic

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum SmuError {
    #[error("Kernel module not loaded: /sys/kernel/ryzen_smu_drv not found")]
    ModuleNotLoaded,

    #[error("Permission denied: run as root or add user to appropriate group")]
    PermissionDenied,

    #[error("Unsupported PM table version: {0:#x}")]
    UnsupportedPmTableVersion(u32),

    #[error("Unsupported processor: {0}")]
    UnsupportedProcessor(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Edge Cases

1. **Module not loaded** - Clear error message with instructions
2. **Permission denied** - Suggest `sudo` or udev rules
3. **Unknown PM table version** - Graceful degradation
4. **Variable core counts** - Detect from PM table or `/proc/cpuinfo`
5. **CCD detection** - Derive from codename
6. **Offline cores** - Handle 0 MHz / NaN gracefully

### Permissions (udev rule for non-root access)

```
KERNEL=="ryzen_smu_drv", SUBSYSTEM=="module", MODE="0644"
```

## Dependencies

```toml
# amd-smu-lib
[dependencies]
thiserror = "2"
byteorder = "1"

# amd-smu-cli
[dependencies]
amd-smu-lib = { path = "../amd-smu-lib" }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
humantime = "2"

# amd-smu-tui
[dependencies]
amd-smu-lib = { path = "../amd-smu-lib" }
ratatui = "0.29"
crossterm = "0.28"
```

## Testing Strategy

1. **Unit tests** - Mock PM table bytes, verify parsing logic
2. **Integration tests** - Test against fixture files (captured real PM tables)
3. **Manual testing** - Run on actual hardware with module loaded

```rust
#[cfg(test)]
mod tests {
    const VERMEER_PM_TABLE: &[u8] = include_bytes!("fixtures/vermeer.bin");

    #[test]
    fn parse_vermeer_temps() {
        let pm = PmTable::parse(VERMEER_PM_TABLE, 0x240903).unwrap();
        assert!(pm.tctl > 0.0 && pm.tctl < 100.0);
    }
}
```

## Kernel Interface Reference

Sysfs path: `/sys/kernel/ryzen_smu_drv/`

| File | Access | Description |
|------|--------|-------------|
| `pm_table` | RO | Binary PM table data |
| `pm_table_version` | RO | PM table format version |
| `pm_table_size` | RO | PM table size in bytes |
| `version` | RO | SMU firmware version |
| `codename` | RO | Processor codename index |
| `drv_version` | RO | Driver version |
