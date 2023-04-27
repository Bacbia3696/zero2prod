use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn telemetry() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer().json();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
