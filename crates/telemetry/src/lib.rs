pub mod env;
mod error;

pub use error::TelemetryError;
use opentelemetry_sdk::{logs::SdkLoggerProvider, trace::SdkTracerProvider};

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

    let mut otel_layers: Vec<BoxedLayer> = Vec::new();

    let layers = build_layers(log_format, std::io::stdout, ansi, otel_layers);

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
        tracer_provider: None,
        logger_provider: None,
    })
}
