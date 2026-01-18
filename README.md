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
