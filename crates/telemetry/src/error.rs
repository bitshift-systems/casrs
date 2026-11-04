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
