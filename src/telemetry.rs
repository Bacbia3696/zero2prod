use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn telemetry(default_env: &str) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(default_env));
    let fmt_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
