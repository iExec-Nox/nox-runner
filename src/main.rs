mod application;
mod compute;
mod config;
mod crypto;
mod events;
mod handle_gateway;
mod queue;
mod utils;

use tracing::{debug, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::application::Application;
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::load().map_err(|e| {
        error!("Failed to load configuration: {e}");
        e
    })?;

    debug!("Configuration loaded");

    Application::new(config).await?.run().await?;
    Ok(())
}
