use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

use alloy::primitives::{Address, hex};
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
    #[serde(with = "humantime_serde", default = "default_rpc_call_timeout")]
    #[validate(custom(function = "validate_timeout"))]
    pub call_timeout: Duration,
    #[serde(with = "humantime_serde", default = "default_rpc_connect_timeout")]
    #[validate(custom(function = "validate_timeout"))]
    pub connect_timeout: Duration,
    #[validate(url)]
    pub rpc_url: String,
    #[validate(custom(function = "validate_nox_compute_contract_address"))]
    pub nox_compute_contract_address: Address,
}

#[derive(Deserialize, Validate)]
pub struct HandleGatewayConfig {
    #[validate(url)]
    pub url: String,
    #[serde(with = "humantime_serde")]
    #[validate(custom(function = "validate_timeout"))]
    pub connect_timeout: Duration,
    #[serde(with = "humantime_serde")]
    #[validate(custom(function = "validate_timeout"))]
    pub timeout: Duration,
}

#[derive(Deserialize, Validate)]
pub struct TlsConfig {
    pub enabled: bool,
    #[serde(default)]
    pub ca: String,
    #[serde(default)]
    pub cert: String,
    #[serde(default)]
    pub key: String,
}

#[derive(Deserialize, Validate)]
pub struct NatsConfig {
    #[validate(custom(function = "validate_nats_urls"))]
    pub urls: Vec<String>,
    #[validate(nested)]
    pub tls: TlsConfig,
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
    #[validate(nested)]
    pub handle_gateway: HandleGatewayConfig,
    #[validate(custom(function = "validate_wallet_key"))]
    pub wallet_key: String,
    pub otel: OtelConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtelConfig {
    pub enabled: bool,
    pub url: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config = ConfigBuilder::builder()
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", "8080")?
            .set_default("handle_gateway.url", "http://localhost:3000")?
            .set_default("handle_gateway.connect_timeout", "3s")?
            .set_default("handle_gateway.timeout", "15s")?
            .set_default("nats.tls.enabled", true)?
            .set_default("nats.tls.ca", "")?
            .set_default("nats.tls.cert", "")?
            .set_default("nats.tls.key", "")?
            .set_default("nats.stream_name", "nox_ingestor")?
            .set_default("nats.consumer_name", "nox_ingestor_consumer")?
            .set_default("nats.consumer_max_deliver", 10)?
            .set_default("nats.max_ack_pending", 10)?
            .set_default("nats.max_batch", 10)?
            .add_source(
                Environment::with_prefix("NOX_RUNNER")
                    .prefix_separator("_")
                    .separator("__")
                    .list_separator(",")
                    .with_list_parse_key("nats.urls")
                    .try_parsing(true),
            )
            .build()?;
        config.try_deserialize()
    }

    /// Returns the `host:port` string used to bind the HTTP listener.
    pub fn binding_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

fn default_rpc_call_timeout() -> Duration {
    Duration::from_secs(8)
}

fn default_rpc_connect_timeout() -> Duration {
    Duration::from_secs(5)
}

fn validate_nats_urls(urls: &Vec<String>) -> Result<(), ValidationError> {
    if urls.is_empty() {
        return Err(ValidationError::new(
            "nats.urls must contain at least one URL",
        ));
    }
    for u in urls {
        if !u.starts_with("nats://") && !u.starts_with("tls://") {
            return Err(ValidationError::new(
                "each nats url must start with nats:// or tls://",
            ));
        }
    }
    Ok(())
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

fn validate_timeout(value: &Duration) -> Result<(), ValidationError> {
    if *value == Duration::ZERO {
        let err =
            ValidationError::new("timeout_zero").with_message(Cow::from("must be greater than 0s"));
        return Err(err);
    }
    if *value > Duration::from_secs(60) {
        let err = ValidationError::new("timeout_too_large")
            .with_message(Cow::from("must not exceed 60s"));
        return Err(err);
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
                ("NOX_RUNNER_CHAINS__31337__CALL_TIMEOUT", Some("10s")),
                ("NOX_RUNNER_CHAINS__31337__CONNECT_TIMEOUT", Some("5s")),
                (
                    "NOX_RUNNER_CHAINS__31337__NOX_COMPUTE_CONTRACT_ADDRESS",
                    Some("0x0A59a4e1F7f740CD6474312AfFC1446fA9B5ad9B"),
                ),
                (
                    "NOX_RUNNER_CHAINS__31337__RPC_URL",
                    Some("http://localhost:8545"),
                ),
                ("NOX_RUNNER_NATS__TLS__ENABLED", Some("true")),
                ("NOX_RUNNER_NATS__TLS__CA", Some("ca-pem")),
                ("NOX_RUNNER_NATS__TLS__CERT", Some("cert-pem")),
                ("NOX_RUNNER_NATS__TLS__KEY", Some("key-pem")),
                (
                    "NOX_RUNNER_NATS__URLS",
                    Some("nats://localhost:4221,nats://localhost:4222,nats://localhost:4223"),
                ),
                (
                    "NOX_RUNNER_WALLET_KEY",
                    Some("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
                ),
                ("NOX_RUNNER_OTEL__ENABLED", Some("false")),
                ("NOX_RUNNER_OTEL__URL", Some("false")),
            ],
            || {
                let config = Config::load().expect("should load");
                config.validate().expect("should validate");
                assert_eq!(Duration::from_secs(10), config.chains[&31337].call_timeout);
                assert_eq!(
                    Duration::from_secs(5),
                    config.chains[&31337].connect_timeout
                );
                assert_eq!(
                    Address::from_str("0x0A59a4e1F7f740CD6474312AfFC1446fA9B5ad9B").unwrap(),
                    config.chains[&31337].nox_compute_contract_address
                );
                assert_eq!("http://localhost:8545", config.chains[&31337].rpc_url);
                assert!(config.nats.tls.enabled);
                assert_eq!("ca-pem", config.nats.tls.ca);
                assert_eq!("cert-pem", config.nats.tls.cert);
                assert_eq!("key-pem", config.nats.tls.key);
                assert_eq!(3, config.nats.urls.len());
            },
        )
    }

    #[test]
    fn check_config_tls_disabled_by_default_material() {
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
                (
                    "NOX_RUNNER_NATS__URLS",
                    Some("tls://nats-1.internal,nats://nats-2.internal"),
                ),
                ("NOX_RUNNER_NATS__TLS__ENABLED", Some("false")),
            ],
            || {
                let config = Config::load().expect("should load");
                config.validate().expect("should validate");
                assert_eq!(2, config.nats.urls.len());
                assert!(!config.nats.tls.enabled);
                assert_eq!("", config.nats.tls.ca);
                assert_eq!(false, config.otel.enabled);
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
                (
                    "NOX_RUNNER_NATS__URLS",
                    Some("nats://localhost:4221,nats://localhost:4222"),
                ),
                ("NOX_RUNNER_NATS__TLS__ENABLED", Some("false")),
                ("NOX_RUNNER_NATS__MAX_ACK_PENDING", Some("500")),
                ("NOX_RUNNER_NATS__MAX_BATCH", Some("500")),
                ("NOX_RUNNER_WALLET_KEY", Some("0x")),
                ("NOX_RUNNER_OTEL__ENABLED", Some("true")),
                ("NOX_RUNNER_OTEL__URL", Some("false")),
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
    fn check_invalid_nats_urls_empty() {
        let nats_config = NatsConfig {
            urls: vec![],
            tls: TlsConfig {
                enabled: false,
                ca: String::new(),
                cert: String::new(),
                key: String::new(),
            },
            stream_name: "nox_ingestor".to_string(),
            consumer_name: "nox_ingestor_consumer".to_string(),
            consumer_max_deliver: 10,
            max_ack_pending: 10,
            max_batch: 10,
        };
        let result = nats_config.validate();
        assert!(result.is_err());
        assert!(ValidationErrors::has_error(&result, "urls"));
    }

    #[test]
    fn check_invalid_chain_config() {
        let chain_config = ChainConfig {
            call_timeout: Duration::from_secs(120),
            connect_timeout: Duration::from_secs(90),
            nox_compute_contract_address: Address::ZERO,
            rpc_url: "".to_string(),
        };
        let result = chain_config.validate();
        assert!(ValidationErrors::has_error(&result, "call_timeout"));
        assert!(ValidationErrors::has_error(&result, "connect_timeout"));
        assert!(ValidationErrors::has_error(
            &result,
            "nox_compute_contract_address"
        ));
        assert!(ValidationErrors::has_error(&result, "rpc_url"));
    }
}
