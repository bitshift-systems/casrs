//! CLI-wide constants

pub const LOG_ENV: &str = "CASRS_LOG";

/// The CLI version.
pub const VERSION: &str = match option_env!("CASRS_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};
