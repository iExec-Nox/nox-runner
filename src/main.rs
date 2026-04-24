use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use validator::Validate;

use crate::application::Application;
use crate::config::Config;

mod application;
mod compute;
mod config;
mod events;
mod handlers;
mod handles;
mod queue;
mod rpc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::load().inspect_err(|e| error!("Failed to load configuration: {e}"))?;
    config
        .validate()
        .inspect_err(|e| error!("Invalid configuration: {e}"))?;

    info!("Configuration loaded");

    Application::new(config).await?.run().await?;
    Ok(())
}
