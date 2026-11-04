pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry = casrs_telemetry::init_tracing("casrs-server")?;

    tracing::info!("starting casrs-server");

    tokio::select! {
        _ = shutdown_signal() => tracing::info!("shutdown signal received, exiting"),
    }
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
