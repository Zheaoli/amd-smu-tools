mod codename;
mod error;
mod pmtable;

pub use codename::Codename;
pub use error::{Result, SmuError};
pub use pmtable::{PmTable, MAX_CORES};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
