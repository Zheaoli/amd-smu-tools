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
