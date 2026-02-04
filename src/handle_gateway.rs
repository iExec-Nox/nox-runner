use alloy_primitives::Address;
use chrono::NaiveDateTime;
use reqwest::Client;
use serde::Serialize;
use tracing::error;

#[derive(Serialize)]
pub struct HandleEntry {
    pub handle: String,
    pub ciphertext: String,
    pub public_key: String,
    pub nonce: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize)]
pub struct TEEComputeResult {
    pub chain_id: u32,
    pub block_number: u64,
    pub caller: Address,
    pub transaction_hash: String,
    pub handles: Vec<HandleEntry>,
}
pub struct GatewayClient {
    client: Client,
    url: String,
}

impl GatewayClient {
    pub async fn new(url: &str) -> Result<Self, reqwest::Error> {
        let client = Client::builder().build()?;
        Ok(Self {
            client,
            url: url.to_string(),
        })
    }

    pub async fn push_results(&self, data: TEEComputeResult) -> Result<(), reqwest::Error> {
        let url = format!("{}/v0/compute/results", self.url);
        let response = self.client.post(&url).json(&data).send().await?;
        if let Err(err) = response.error_for_status_ref() {
            let status = response.status();
            let error_body = response.text().await?;
            error!("Error {status}: {error_body}");
            return Err(err);
        }
        Ok(())
    }
}
