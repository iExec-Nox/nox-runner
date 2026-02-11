use config::{Config as ConfigBuilder, ConfigError, Environment};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub handle_gateway_url: String,
    pub kms_url: String,
    pub nats_url: String,
    pub nats_stream_name: String,
    pub nats_consumer_name: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config = ConfigBuilder::builder()
            .set_default("handle_gateway_url", "http://localhost:3000")?
            .set_default("kms_url", "http://localhost:9000")?
            .set_default("nats_url", "nats://localhost:4222")?
            .set_default("nats_stream_name", "nox_ingestor")?
            .set_default("nats_consumer_name", "nox_ingestor_consumer")?
            .add_source(
                Environment::with_prefix("NOX_RUNNER")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;
        config.try_deserialize()
    }
}
