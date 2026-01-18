mod error;

pub use error::{Result, SmuError};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
