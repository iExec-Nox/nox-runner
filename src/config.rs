use alloy_primitives::{Address, hex};
use config::{Config as ConfigBuilder, Environment};
use serde::Deserialize;
use tracing::error;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Validate)]
pub struct NatsConfig {
    #[validate(url)]
    pub url: String,
    pub stream_name: String,
    pub consumer_name: String,
    #[validate(range(min = 10))]
    pub consumer_max_deliver: i64,
    #[validate(range(min = 10, max = 200))]
    pub max_ack_pending: i64,
    #[validate(range(min = 10, max = 200))]
    pub max_batch: i64,
}

#[derive(Deserialize, Validate)]
pub struct Config {
    #[validate(nested)]
    pub server: ServerConfig,
    pub chain_id: u64,
    pub rpc_url: String,
    pub nox_compute_contract_address: Address,
    #[validate(nested)]
    pub nats: NatsConfig,
    #[validate(url)]
    pub handle_gateway_url: String,
    #[validate(custom(function = "validate_wallet_key"))]
    pub wallet_key: String,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config: Self = ConfigBuilder::builder()
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", "8080")?
            .set_default("rpc_url", "http://localhost:8545")?
            .set_default(
                "nox_compute_contract_address",
                "0x0000000000000000000000000000000000000000",
            )?
            .set_default("handle_gateway_url", "http://localhost:3000")?
            .set_default("nats.url", "nats://localhost:4222")?
            .set_default("nats.stream_name", "nox_ingestor")?
            .set_default("nats.consumer_name", "nox_ingestor_consumer")?
            .set_default("nats.consumer_max_deliver", 10)?
            .set_default("nats.max_ack_pending", 10)?
            .set_default("nats.max_batch", 10)?
            .add_source(
                Environment::with_prefix("NOX_RUNNER")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?
            .try_deserialize()?;
        config
            .validate()
            .inspect_err(|e| error!("failed to validate configuration: {e}"))?;
        Ok(config)
    }

    /// Returns the `host:port` string used to bind the HTTP listener.
    pub fn binding_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

fn validate_wallet_key(wallet_key: &str) -> Result<(), ValidationError> {
    let wallet_key_bytes = hex::decode(wallet_key)
        .map_err(|_| ValidationError::new("wallet key is not a valid hex"))?;
    if wallet_key_bytes.len() != 32 {
        return Err(ValidationError::new(
            "wallet key should have a 32-byte length",
        ));
    }
    if wallet_key_bytes == [0u8; 32] {
        return Err(ValidationError::new("wallet key should not contain only 0"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::ValidationErrors;

    #[test]
    fn check_config() {
        temp_env::with_vars(
            [
                ("NOX_RUNNER_CHAIN_ID", Some("31337")),
                (
                    "NOX_RUNNER_WALLET_KEY",
                    Some("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
                ),
            ],
            || {
                let config = Config::load().expect("should load");
                config.validate().expect("should validate");
            },
        )
    }

    #[test]
    fn check_invalid_config() {
        temp_env::with_vars(
            [
                ("NOX_RUNNER_CHAIN_ID", Some("31337")),
                ("NOX_RUNNER_WALLET_KEY", Some("0x")),
                ("NOX_RUNNER_RPC_URL", Some("")),
                ("NOX_RUNNER_NATS__MAX_ACK_PENDING", Some("500")),
                ("NOX_RUNNER_NATS__MAX_BATCH", Some("500")),
            ],
            || {
                let config = Config::load().expect("should load");
                let result = config.validate();
                assert!(result.is_err());
                assert!(ValidationErrors::has_error(&result, "nats"));
                assert!(ValidationErrors::has_error(&result, "wallet_key"));
            },
        )
    }
}
