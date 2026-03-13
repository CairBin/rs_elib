use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger(level: Option<&str>) {
    let filter = level
        .map(|s| s.to_string())
        .or_else(|| std::env::var("RUST_LOG").ok())
        .unwrap_or_else(|| "info".to_string());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .compact(),
        )
        .init();
}