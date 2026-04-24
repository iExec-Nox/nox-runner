use std::collections::HashMap;

use alloy_primitives::{Address, hex};
use config::{Config as ConfigBuilder, ConfigError, Environment};
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Validate)]
pub struct ChainConfig {
    #[validate(url)]
    pub rpc_url: String,
    #[validate(custom(function = "validate_nox_compute_contract_address"))]
    pub nox_compute_contract_address: Address,
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
    #[validate(nested)]
    pub chains: HashMap<u32, ChainConfig>,
    #[validate(nested)]
    pub nats: NatsConfig,
    #[validate(url)]
    pub handle_gateway_url: String,
    #[validate(custom(function = "validate_wallet_key"))]
    pub wallet_key: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config = ConfigBuilder::builder()
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", "8080")?
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
            .build()?;
        config.try_deserialize()
    }

    /// Returns the `host:port` string used to bind the HTTP listener.
    pub fn binding_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

fn validate_nox_compute_contract_address(
    nox_compute_contract_address: &Address,
) -> Result<(), ValidationError> {
    if *nox_compute_contract_address == Address::ZERO {
        return Err(ValidationError::new(
            "NoxCompute contract address should not be zero address",
        ));
    }
    Ok(())
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
    use std::str::FromStr;
    use validator::ValidationErrors;

    #[test]
    fn check_config() {
        temp_env::with_vars(
            [
                (
                    "NOX_RUNNER_CHAINS__31337__RPC_URL",
                    Some("http://localhost:8545"),
                ),
                (
                    "NOX_RUNNER_CHAINS__31337__NOX_COMPUTE_CONTRACT_ADDRESS",
                    Some("0x0A59a4e1F7f740CD6474312AfFC1446fA9B5ad9B"),
                ),
                (
                    "NOX_RUNNER_WALLET_KEY",
                    Some("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
                ),
            ],
            || {
                let config = Config::load().expect("should load");
                config.validate().expect("should validate");
                assert_eq!("http://localhost:8545", config.chains[&31337].rpc_url);
                assert_eq!(
                    Address::from_str("0x0A59a4e1F7f740CD6474312AfFC1446fA9B5ad9B").unwrap(),
                    config.chains[&31337].nox_compute_contract_address
                );
            },
        )
    }

    #[test]
    fn check_invalid_config() {
        temp_env::with_vars(
            [
                ("NOX_RUNNER_CHAINS__31337__RPC_URL", Some("")),
                (
                    "NOX_RUNNER_CHAINS__31337__NOX_COMPUTE_CONTRACT_ADDRESS",
                    Some("0x0000000000000000000000000000000000000000"),
                ),
                ("NOX_RUNNER_NATS__MAX_ACK_PENDING", Some("500")),
                ("NOX_RUNNER_NATS__MAX_BATCH", Some("500")),
                ("NOX_RUNNER_WALLET_KEY", Some("0x")),
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

    #[test]
    fn check_invalid_chain_config() {
        let chain_config = ChainConfig {
            rpc_url: "".to_string(),
            nox_compute_contract_address: Address::ZERO,
        };
        let result = chain_config.validate();
        assert!(ValidationErrors::has_error(&result, "rpc_url"));
        assert!(ValidationErrors::has_error(
            &result,
            "nox_compute_contract_address"
        ));
    }
}
