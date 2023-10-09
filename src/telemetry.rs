//! src/telemetry.rs

use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Compose multiple layers into a `tracing`'s subscriber
pub fn get_subscriber(name: &str, log_level: &str) -> impl Subscriber + Send + Sync {
    let formatting_layer = BunyanFormattingLayer::new(String::from(name), std::io::stdout);
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(String::from(log_level)));
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    subscriber
}

/// Register a subscriber as global default to process span data
///
/// It should be called only once for the entire lifetime of the application!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to initialize logger.");
    set_global_default(subscriber).expect("Failed to set subscriber.");
}
