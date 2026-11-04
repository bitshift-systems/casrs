use tokio::net::TcpListener;

mod router;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry = casrs_telemetry::init_tracing("casrs-server")?;

    tracing::info!("starting casrs-server");

    let router = router::router();

    let listener = TcpListener::bind("0.0.0.0:8000").await.inspect_err(|e| {
        tracing::error!(?e, "failed to bind to address");
    })?;
    tracing::info!(addr = ?listener.local_addr()?, "server listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

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
