use alloy_primitives::Address;
use config::{Config as ConfigBuilder, ConfigError, Environment};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub chain_id: u64,
    pub rpc_url: String,
    pub nox_compute_contract_address: Address,
    pub handle_gateway_url: String,
    pub nats_url: String,
    pub nats_stream_name: String,
    pub nats_consumer_name: String,
    pub nats_consumer_max_deliver: i64,
    pub wallet_key: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config = ConfigBuilder::builder()
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", "8080")?
            .set_default("rpc_url", "http://localhost:8545")?
            .set_default(
                "nox_compute_contract_address",
                "0x0000000000000000000000000000000000000000",
            )?
            .set_default("handle_gateway_url", "http://localhost:3000")?
            .set_default("nats_url", "nats://localhost:4222")?
            .set_default("nats_stream_name", "nox_ingestor")?
            .set_default("nats_consumer_name", "nox_ingestor_consumer")?
            .set_default("nats_consumer_max_deliver", 3)?
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
