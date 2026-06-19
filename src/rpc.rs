//! This modules provides a service to interact with NoxCompute methods.

use std::time::Duration;

use alloy::{primitives::Address, providers::RootProvider, rpc::client::RpcClient, sol};
use k256::PublicKey;
use reqwest::{Client, Url};
use tracing::error;

sol! {
    #[sol(rpc)]
    interface INoxCompute {
        function gateway() external view returns (address);
        function kmsPublicKey() external view returns (bytes memory);
    }
}

/// Connection to a NoxCompute Smart Contract deployment.
pub struct NoxClient {
    contract: INoxCompute::INoxComputeInstance<RootProvider>,
}

impl NoxClient {
    /// Creates a NoxClient configured with the given timeouts.
    pub fn new(
        rpc_url: &str,
        call_timeout: Duration,
        connect_timeout: Duration,
        contract_address: Address,
    ) -> Result<Self, String> {
        let rpc_url = Url::parse(rpc_url.trim_end_matches('/'))
            .map_err(|e| format!("failed to parse URL: {e}"))?;
        let client = Client::builder()
            .connect_timeout(connect_timeout)
            .timeout(call_timeout)
            .build()
            .map_err(|e| format!("Failed to build RPC HTTP client: {e}"))
            .inspect_err(|e| error!("{e}"))?;
        let rpc_client = RpcClient::new_http_with_client(client, rpc_url);
        let provider = RootProvider::new(rpc_client);
        let contract = INoxCompute::new(contract_address, provider);
        Ok(Self { contract })
    }

    /// Returns value of ETH call to gateway().
    ///
    /// # Errors
    ///
    /// Returns [`Err`] in case of transport error or zero value.
    pub async fn get_gateway_address(&self) -> Result<Address, String> {
        let gateway_address = self
            .contract
            .gateway()
            .call()
            .await
            .map_err(|e| format!("Call to gateway() failed: {e}"))
            .inspect_err(|e| error!("{e}"))?;
        if gateway_address == Address::ZERO {
            return Err(format!("Call to gateway() returned {}", Address::ZERO));
        }
        Ok(gateway_address)
    }

    /// Returns value of ETH call to kmsPublicKey().
    ///
    /// # Errors
    ///
    /// Returns [`Err`] in case of transport error.
    pub async fn get_kms_public_key(&self) -> Result<PublicKey, String> {
        let protocol_key_bytes = self
            .contract
            .kmsPublicKey()
            .call()
            .await
            .map_err(|e| format!("Call to kmsPublicKey() failed: {e}"))?;
        PublicKey::from_sec1_bytes(&protocol_key_bytes)
            .map_err(|e| format!("Failed to decode KMS public key {e}"))
    }
}
