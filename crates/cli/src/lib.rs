pub mod cli;
pub mod consts;
pub mod error;

use std::process::ExitCode;

pub use cli::Cli;
use error::CliError;

fn init_tracing(verbose: u8) {
    let filter = match verbose {
        0 => std::env::var(consts::LOG_ENV).unwrap_or_else(|_| "warn".to_string()),
        1 => "debug".to_string(),
        _ => "trace".to_string(),
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_writer(std::io::stderr)
        .try_init();
}

pub async fn run(cli: Cli) -> Result<ExitCode, CliError> {
    init_tracing(cli.global.verbose);

    Ok(std::process::ExitCode::SUCCESS)
}
