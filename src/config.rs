use config::{Config as ConfigBuilder, ConfigError, Environment};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub kms_url: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config = ConfigBuilder::builder()
            .set_default("kms_url", "http://localhost:9000")?
            .add_source(
                Environment::with_prefix("NOX_RUNNER")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;
        config.try_deserialize()
    }
}
