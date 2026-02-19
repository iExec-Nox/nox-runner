//! This modules provides a service to interact with NoxCompute methods.

use alloy_primitives::{Address, Bytes};
use alloy_provider::RootProvider;
use alloy_sol_types::sol;

sol! {
    #[sol(rpc)]
    interface INoxCompute {
        function kmsPublicKey() external view returns (bytes memory);
    }
}

/// Connection to a NoxCompute Smart Contract deployment.
pub struct NoxClient {
    contract: INoxCompute::INoxComputeInstance<RootProvider>,
}

impl NoxClient {
    pub async fn new(
        rpc_url: &str,
        contract_address: Address,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let trimmed_rpc_url = rpc_url.trim_end_matches('/');
        let provider = RootProvider::connect(trimmed_rpc_url).await?;
        let contract = INoxCompute::new(contract_address, provider);
        Ok(Self { contract })
    }

    /// Returns value of ETH call to kmsPublicKey.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] in case of transport error.
    pub async fn get_kms_public_key(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // let contract = NoxCompute::new(self.contract_address, &self.provider);
        let protocol_key_bytes: Bytes = self.contract.kmsPublicKey().call().await?;
        Ok(protocol_key_bytes.to_vec())
    }
}
