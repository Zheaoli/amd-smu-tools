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
