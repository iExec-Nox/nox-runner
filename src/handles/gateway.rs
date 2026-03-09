//! API to fetch operands from and push results to the Handle Gateway.
//!
//! All operands and results are encrypted with ECIES.
//! See [`super::crypto`] for ECIES related operations.

use alloy_primitives::{Address, U256};
use alloy_signer::SignerSync;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{Eip712Domain, eip712_domain, sol};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tracing::error;

use crate::queue::InputEntry;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("Failed to communicate with Handle Gateway {0}")]
    CommunicationError(#[from] reqwest::Error),
    #[error("Failed to create AUTHORIZATION header")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error(transparent)]
    SignatureError(#[from] alloy_signer::Error),
}

sol! {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct OperandAccessAuthorization {
        address caller;
        string[] operands;
        string rsa_public_key;
        string transaction_hash;
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ResultPublishingAuthorization {
        uint256 chain_id;
        uint256 block_number;
        address caller;
        string transaction_hash;
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NoxComputeRequest {
    payload: OperandAccessAuthorization,
    signature: String,
}

#[derive(Deserialize, Serialize)]
pub struct HandleEntry {
    pub handle: String,
    pub ciphertext: String,
    pub public_key: String,
    pub nonce: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoxComputeResult {
    payload: ResultPublishingAuthorization,
    signature: String,
}

pub struct GatewayClient {
    client: Client,
    url: String,
    signer: PrivateKeySigner,
    domain: Eip712Domain,
}

impl GatewayClient {
    pub async fn new(
        chain_id: u64,
        url: &str,
        signer: PrivateKeySigner,
    ) -> Result<Self, reqwest::Error> {
        let client = Client::builder().build()?;
        let domain = eip712_domain! {
            name: "Handle Gateway",
            version: "1",
            chain_id: chain_id,
        };
        Ok(Self {
            client,
            url: url.to_string(),
            signer,
            domain,
        })
    }

    /// Retrieves handles from the Handle Gateway.
    pub async fn get_handles(
        &self,
        caller: Address,
        transaction_hash: String,
        rsa_public_key: String,
        operands: Vec<String>,
    ) -> Result<Vec<InputEntry>, GatewayError> {
        let url = format!("{}/v0/compute/operands", self.url);
        let payload = OperandAccessAuthorization {
            caller,
            transaction_hash,
            rsa_public_key,
            operands,
        };
        let signature = self
            .signer
            .sign_typed_data_sync(&payload, &self.domain)
            .map_err(GatewayError::SignatureError)?
            .to_string();
        let auth = STANDARD.encode(json!(NoxComputeRequest { payload, signature }).to_string());
        let mut auth_value = header::HeaderValue::from_str(&format!("EIP712 {auth}"))
            .map_err(GatewayError::InvalidHeaderValue)?;
        auth_value.set_sensitive(true);
        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, auth_value)
            .send()
            .await
            .map_err(GatewayError::CommunicationError)?;
        if let Err(err) = response.error_for_status_ref() {
            let status = response.status();
            let error_body = response.text().await?;
            error!("Error {status}: {error_body}");
            return Err(GatewayError::CommunicationError(err));
        }
        response
            .json::<Vec<InputEntry>>()
            .await
            .map_err(GatewayError::CommunicationError)
    }

    /// Push handles associated to a Nox computation to the Handle Gateway.
    pub async fn push_results(
        &self,
        chain_id: u32,
        block_number: u64,
        caller: Address,
        transaction_hash: String,
        handles: Vec<HandleEntry>,
    ) -> Result<(), GatewayError> {
        let url = format!("{}/v0/compute/results", self.url);
        let payload = ResultPublishingAuthorization {
            chain_id: U256::from(chain_id),
            block_number: U256::from(block_number),
            caller,
            transaction_hash,
        };
        let signature = self
            .signer
            .sign_typed_data_sync(&payload, &self.domain)
            .map_err(GatewayError::SignatureError)?
            .to_string();
        let auth = STANDARD.encode(json!(NoxComputeResult { payload, signature }).to_string());
        let mut auth_value = header::HeaderValue::from_str(&format!("EIP712 {auth}"))
            .map_err(GatewayError::InvalidHeaderValue)?;
        auth_value.set_sensitive(true);
        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, auth_value)
            .json(&handles)
            .send()
            .await
            .map_err(GatewayError::CommunicationError)?;
        if let Err(err) = response.error_for_status_ref() {
            let status = response.status();
            let error_body = response.text().await?;
            error!("Error {status}: {error_body}");
            return Err(GatewayError::CommunicationError(err));
        }
        Ok(())
    }
}
