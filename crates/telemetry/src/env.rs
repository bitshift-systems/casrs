//! The single place that names every environment variable this crate reads.

/// Collector endpoint. When unset, OTLP export is disabled entirely.
pub const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";

/// OTLP transport. Only `grpc` is compiled in; other values are rejected.
pub const OTEL_EXPORTER_OTLP_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";

/// Extra OTLP headers (honored by `opentelemetry-otlp` directly).
pub const OTEL_EXPORTER_OTLP_HEADERS: &str = "OTEL_EXPORTER_OTLP_HEADERS";

/// Standard `tracing` filter directive (e.g. `info`, `info,axum=debug`). Read by
/// `EnvFilter::try_from_default_env`; named here so the API can pass it
/// through to child processes.
pub const RUST_LOG: &str = "RUST_LOG";

/// Console log format for the fmt layer: `text` (default) or `json`. Unknown
/// values degrade to `text` with a warning.
pub const CASRS_LOG_FORMAT: &str = "CASRS_LOG_FORMAT";

/// The OTLP env vars a parent process should pass through to children so their
/// telemetry reaches the same collector. Ordered with the endpoint first.
///
/// ```
/// use casrs_telemetry::env;
/// // The endpoint is forwarded first, and the protocol var is included.
/// assert_eq!(env::OTLP_PASSTHROUGH[0], env::OTEL_EXPORTER_OTLP_ENDPOINT);
/// assert!(env::OTLP_PASSTHROUGH.contains(&env::OTEL_EXPORTER_OTLP_PROTOCOL));
/// ```
pub const OTLP_PASSTHROUGH: &[&str] = &[
    OTEL_EXPORTER_OTLP_ENDPOINT,
    OTEL_EXPORTER_OTLP_PROTOCOL,
    OTEL_EXPORTER_OTLP_HEADERS,
];

/// The logging env vars a parent process should pass through to children so a
/// mover `Job` inherits the controller's log level and format. Unlike
/// [`OTLP_PASSTHROUGH`] these apply even with no collector configured.
pub const LOG_PASSTHROUGH: &[&str] = &[RUST_LOG, CASRS_LOG_FORMAT];
