pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = casrs_telemetry::init_tracing("casrs-server")?;
    Ok(())
}
