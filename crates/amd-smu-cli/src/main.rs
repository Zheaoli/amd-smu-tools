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
