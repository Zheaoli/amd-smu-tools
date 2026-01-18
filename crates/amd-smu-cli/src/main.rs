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
