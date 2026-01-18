mod codename;
mod error;
mod pmtable;
mod smu;

pub use codename::Codename;
pub use error::{Result, SmuError};
pub use pmtable::{PmTable, MAX_CORES};
pub use smu::SmuReader;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
