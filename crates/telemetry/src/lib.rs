pub mod env;
mod error;

pub use error::{Signal, TelemetryError};
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider, trace::SdkTracerProvider};

/// Resolved OTLP configuration. Absent when `OTEL_EXPORTER_OTLP_ENDPOINT` is
/// unset — the operator then runs fully offline (fmt logs + Prometheus pull).
#[derive(Debug, Clone)]
pub struct OtlpConfig {
    /// The collector endpoint (validated as a URL).
    pub endpoint: String,
}

impl OtlpConfig {
    /// Read + validate OTLP config from the standard `OTEL_*` env.
    ///
    /// - `Ok(None)` — `OTEL_EXPORTER_OTLP_ENDPOINT` unset: OTLP disabled.
    /// - `Ok(Some)` — endpoint present and valid.
    /// - `Err` — endpoint set but malformed, or an unsupported protocol
    ///   requested. Callers degrade (log + continue) unless strict.
    pub fn from_env() -> Result<Option<Self>, TelemetryError> {
        let endpoint = match std::env::var(env::OTEL_EXPORTER_OTLP_ENDPOINT) {
            Ok(v) if !v.trim().is_empty() => v,
            _ => return Ok(None),
        };

        // Only gRPC is compiled in; reject other protocols with a clear fix.
        if let Ok(proto) = std::env::var(env::OTEL_EXPORTER_OTLP_PROTOCOL) {
            let proto = proto.trim();
            if !proto.is_empty() && !proto.eq_ignore_ascii_case("grpc") {
                return Err(TelemetryError::UnsupportedProtocol {
                    value: proto.to_string(),
                });
            }
        }

        url::Url::parse(&endpoint).map_err(|source| TelemetryError::InvalidOtlpEndpoint {
            value: endpoint.clone(),
            source,
        })?;

        Ok(Some(OtlpConfig { endpoint }))
    }

    /// Resolve OTLP config, honoring strict mode. On a non-strict config error
    /// this logs the actionable message and returns `Ok(None)` (degrade);
    /// strict mode propagates the error so the binary can fail-fast.
    fn resolve() -> Result<Option<Self>, TelemetryError> {
        match Self::from_env() {
            Ok(opt) => Ok(opt),
            Err(e) => {
                tracing::error!(error = %e, "OTLP config is invalid; OTLP export is off, local logs and metrics still work");
                Ok(None)
            }
        }
    }
}

fn resource(service_name: &str) -> Resource {
    Resource::builder()
        .with_service_name(service_name.to_string())
        .build()
}

/// Console log format for the fmt layer. Selected by `KOPIUR_LOG_FORMAT`.
///
/// `Text` is the default and preserves the human-readable console output;
/// `Json` emits one structured JSON object per event for log aggregators
/// (Loki/ELK/Datadog). An unrecognized value degrades to `Text` rather than
/// failing a backup operator's startup (see `LogFormat::from_env`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable console output (the default).
    Text,
    /// One structured JSON object per event, for log aggregators (Loki/ELK/Datadog).
    Json,
}

impl LogFormat {
    /// Resolve the format from `CASRS_LOG_FORMAT`. Unset/empty → `Text`. An
    /// unknown value → `Text` plus `Err(unknown_value)` so the caller can defer
    /// a degrade warning until after the subscriber is installed.
    fn from_env() -> (Self, Option<String>) {
        match std::env::var(env::CASRS_LOG_FORMAT) {
            Ok(v) => Self::parse(&v),
            Err(_) => (Self::Text, None),
        }
    }

    /// Parse a single value. Returns the resolved format and, when the input was
    /// non-empty but unrecognized, the offending string for a degrade message.
    fn parse(value: &str) -> (Self, Option<String>) {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "text" | "plain" | "fmt" => (Self::Text, None),
            "json" => (Self::Json, None),
            _ => (Self::Text, Some(value.trim().to_string())),
        }
    }
}

#[must_use = "drop the guard at process end so traces/logs flush"]
pub struct TelemetryGuard {
    tracer_provider: Option<SdkTracerProvider>,
    logger_provider: Option<SdkLoggerProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(tp) = self.tracer_provider.take() {
            let _ = tp.shutdown();
        }
        if let Some(lp) = self.logger_provider.take() {
            let _ = lp.shutdown();
        }
    }
}

type BoxedLayer = Box<dyn tracing_subscriber::Layer<tracing_subscriber::Registry> + Send + Sync>;

fn build_layers<W>(
    log_format: LogFormat,
    make_writer: W,
    ansi: bool,
    mut otel_layers: Vec<BoxedLayer>,
) -> Vec<BoxedLayer>
where
    W: for<'a> tracing_subscriber::fmt::MakeWriter<'a> + Send + Sync + 'static,
{
    use tracing_subscriber::Layer as _;
    let fmt_layer: BoxedLayer = match log_format {
        LogFormat::Text => tracing_subscriber::fmt::layer()
            .with_ansi(ansi)
            .with_writer(make_writer)
            .boxed(),
        LogFormat::Json => tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(false)
            .with_writer(make_writer)
            .boxed(),
    };
    let mut layers = Vec::with_capacity(otel_layers.len() + 1);
    layers.push(fmt_layer);
    layers.append(&mut otel_layers);
    layers
}

pub fn init_tracing(service_name: &str) -> Result<TelemetryGuard, TelemetryError> {
    use std::io::IsTerminal;
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::prelude::*;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let mut deferred: Vec<String> = Vec::new();

    let (log_format, bad_format) = LogFormat::from_env();
    if let Some(v) = bad_format {
        deferred.push(format!(
            "{}={v:?} not recognized (use `text` or `json`); defaulting to `text`",
            env::CASRS_LOG_FORMAT,
        ));
    }
    let ansi = std::io::stdout().is_terminal();

    let otlp = match OtlpConfig::from_env() {
        Ok(o) => o,
        Err(e) => {
            deferred.push(format!("OTLP config is invalid; OTLP export is off: {e}"));
            None
        }
    };

    let mut tracer_provider = None;
    let mut logger_provider = None;
    let mut otel_layers: Vec<BoxedLayer> = Vec::new();

    if let Some(cfg) = otlp.as_ref() {
        match build_trace_provider(service_name, cfg) {
            Ok(tp) => {
                let tracer = tp.tracer("kopiur");
                otel_layers.push(tracing_opentelemetry::layer().with_tracer(tracer).boxed());
                tracer_provider = Some(tp);
            }
            Err(e) => deferred.push(format!("OTLP trace export disabled: {e}")),
        }
        match build_log_provider(service_name, cfg) {
            Ok(lp) => {
                otel_layers.push(
                    opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&lp)
                        .boxed(),
                );
                logger_provider = Some(lp);
            }
            Err(e) => deferred.push(format!("OTLP log export disabled: {e}")),
        }
    }

    let layers = build_layers(log_format, std::io::stdout, ansi, otel_layers);

    if INITIALIZED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Ok(TelemetryGuard {
            tracer_provider,
            logger_provider,
        });
    }

    if let Err(e) = tracing_subscriber::registry()
        .with(layers)
        .with(filter)
        .try_init()
    {
        eprintln!(
            "casrs-telemetry: could not install the tracing subscriber for {service_name}: {e}"
        );
        return Err(TelemetryError::SubscriberInit {
            source: Box::new(e),
        });
    }

    for msg in deferred {
        tracing::warn!("{msg}");
    }

    Ok(TelemetryGuard {
        tracer_provider,
        logger_provider,
    })
}

/// Set the first time [`init_tracing`] attempts to install the global subscriber,
/// so subsequent calls in the same process are no-ops (test binaries init twice).
static INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn build_trace_provider(
    service_name: &str,
    cfg: &OtlpConfig,
) -> Result<SdkTracerProvider, TelemetryError> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(cfg.endpoint.clone())
        .build()
        .map_err(|e| TelemetryError::ExporterBuild {
            signal: Signal::Traces,
            source: Box::new(e),
        })?;
    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource(service_name))
        .build())
}

fn build_log_provider(
    service_name: &str,
    cfg: &OtlpConfig,
) -> Result<SdkLoggerProvider, TelemetryError> {
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(cfg.endpoint.clone())
        .build()
        .map_err(|e| TelemetryError::ExporterBuild {
            signal: Signal::Logs,
            source: Box::new(e),
        })?;
    Ok(SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource(service_name))
        .build())
}
