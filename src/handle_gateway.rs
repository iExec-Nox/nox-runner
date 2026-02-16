//! API to fetch operands from and push results to the Handle Gateway.
//!
//! All operands and results are encrypted with ECIES.
//! See [`super::crypto`] for ECIES related operations.

use alloy_primitives::Address;
use chrono::NaiveDateTime;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::queue::InputEntry;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TEEComputeRequest {
    caller: Address,
    rsa_public_key: String,
    operands: Vec<String>,
    results: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct HandleEntry {
    pub handle: String,
    pub ciphertext: String,
    pub public_key: String,
    pub nonce: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
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

    pub async fn get_handles(
        &self,
        caller: Address,
        rsa_public_key: String,
        operands: Vec<String>,
        results: Vec<String>,
    ) -> Result<Vec<InputEntry>, reqwest::Error> {
        let url = format!("{}/v0/compute/operands", self.url);
        let request = TEEComputeRequest {
            caller,
            rsa_public_key,
            operands,
            results,
        };
        let response = self
            .client
            .get(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;
        let data = response.json::<Vec<InputEntry>>().await?;
        Ok(data)
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
