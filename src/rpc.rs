//! This modules provides a service to interact with NoxCompute methods.

use alloy_primitives::{Address, Bytes};
use alloy_provider::RootProvider;
use alloy_sol_types::sol;
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
    /// Creates a NoxClient instance while checking connection on a blockchain node.
    pub async fn new(rpc_url: &str, contract_address: Address) -> Result<Self, String> {
        let trimmed_rpc_url = rpc_url.trim_end_matches('/');
        let provider = RootProvider::connect(trimmed_rpc_url)
            .await
            .map_err(|e| format!("Connection to blockchain node failed: {e}"))
            .inspect_err(|e| error!("{e}"))?;
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
            .inspect_err(|e| error!(e))?;
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
    pub async fn get_kms_public_key(&self) -> Result<Vec<u8>, String> {
        let protocol_key_bytes: Bytes = self
            .contract
            .kmsPublicKey()
            .call()
            .await
            .map_err(|e| format!("Call to kmsPublicKey() failed: {e}"))
            .inspect_err(|e| error!("{e}"))?;
        Ok(protocol_key_bytes.to_vec())
    }
}
