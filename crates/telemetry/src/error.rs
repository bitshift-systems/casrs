/// The signal an OTLP exporter carries, used to attribute build failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Distributed traces (reconcile spans).
    Traces,
    /// Structured logs bridged from `tracing` events.
    Logs,
    /// Metrics pushed via the OTLP periodic reader.
    Metrics,
}

impl Signal {
    /// Human label used in messages.
    pub fn as_str(self) -> &'static str {
        match self {
            Signal::Traces => "traces",
            Signal::Logs => "logs",
            Signal::Metrics => "metrics",
        }
    }
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    /// `OTEL_EXPORTER_OTLP_ENDPOINT` is set but not a usable URL.
    #[error(
        "OTEL_EXPORTER_OTLP_ENDPOINT='{value}' is not a valid URL ({source}). \
         Fix: use a URL like http://otel-collector:4317, or unset it to disable OTLP."
    )]
    InvalidOtlpEndpoint {
        /// The offending value.
        value: String,
        /// The underlying parse error.
        #[source]
        source: url::ParseError,
    },

    /// `OTEL_EXPORTER_OTLP_PROTOCOL` requests a transport this build doesn't ship.
    #[error(
        "OTEL_EXPORTER_OTLP_PROTOCOL='{value}' is not supported; only 'grpc' is. \
         Fix: set OTEL_EXPORTER_OTLP_PROTOCOL=grpc (or unset it) and point \
         OTEL_EXPORTER_OTLP_ENDPOINT at the collector's gRPC port (4317)."
    )]
    UnsupportedProtocol {
        /// The requested protocol.
        value: String,
    },

    /// An OTLP exporter for `signal` failed to build (bad config / runtime).
    #[error(
        "could not build the OTLP {signal} exporter: {source}. \
         Fix: point OTEL_EXPORTER_OTLP_ENDPOINT at a reachable collector \
         (e.g. http://otel-collector:4317), or unset it to disable OTLP."
    )]
    ExporterBuild {
        /// Which signal's exporter failed.
        signal: Signal,
        /// The underlying exporter build error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error(
        "could not install the tracing subscriber: {source}. \
         Refusing to run without logs. This is a bug; please report it."
    )]
    SubscriberInit {
        /// The underlying `try_init` error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
