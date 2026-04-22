//! API to fetch operands from and push results to the Handle Gateway.
//!
//! All operands and results are encrypted with ECIES.
//! See [`super::crypto`] for ECIES related operations.
use std::collections::HashMap;

use alloy_primitives::{Address, FixedBytes, U256, hex};
use alloy_signer::{Signature, SignerSync};
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{SolStruct, eip712_domain, sol};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderValue},
};
use rsa::rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tracing::{error, info, warn};

use crate::queue::{OperandEntry, ResultEntry};

/// EIP-712 domain name for Handle Gateway interactions.
const HANDLE_GATEWAY_EIP712_DOMAIN_NAME: &str = "Handle Gateway";

/// Errors raised in [`GatewayClient`] implementation.
#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("Failed to communicate with Handle Gateway {0}")]
    CommunicationError(#[from] reqwest::Error),
    #[error("Failed to create AUTHORIZATION header")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error(transparent)]
    SignatureError(#[from] alloy_signer::Error),
    #[error("Unknown Handle Gateway: {0}")]
    UnknownHandleGateway(String),
}

sol! {
    /// EIP-712 compatible payload to authorize a Runner to retrieve operands from the Handle Gateway.
    ///
    /// This authorization allows the Handle Gateway to verify the operands are queried by a known Runner.
    #[derive(Serialize)]
    struct OperandAccessAuthorization {
        uint256 chainId;
        uint256 blockNumber;
        address caller;
        string transactionHash;
        string[] operands;
        string rsaPublicKey;
    }

    /// EIP-712 compatible payload to allow a Runner to decrypt operands.
    ///
    /// `encryptedSharedSecret` holds the ECIES shared secret encrypted with
    /// Runner RSA publickey. The shared secret is used to derive with `HKDF`
    /// the `AES-256-GCM` symmetric key, itself allowing to decrypt the operand value.
    #[derive(Deserialize)]
    struct HandleCryptoMaterial {
        string handle;
        string ciphertext;
        string encryptedSharedSecret;
        string iv;
    }

    /// EIP-712 compatible payload to verify operands are sent by a known Handle Gateway.
    ///
    /// The payload wraps a list of [`HandleCryptoMaterial`]s for all handles prepared
    /// for the Runner computation.
    #[derive(Deserialize)]
    struct ComputeOperands {
        HandleCryptoMaterial[] operands;
    }

    /// EIP-712 compatible payload to authorize a Runner to publish results to the Handle Gateway.
    ///
    /// This authorization allows the Handle Gateway to verify the handles are published
    /// by a known Runner.
    #[derive(Serialize)]
    struct ResultPublishingAuthorization {
        uint256 chainId;
        uint256 blockNumber;
        address caller;
        string transactionHash;
    }

    /// EIP-712 compatible payload to verify result handles were sent to a known Handle Gateway.
    #[derive(Deserialize)]
    struct ResultPublishingReport {
        string message;
    }
}

/// Operands retrieved from the Handle Gateway.
///
/// It contains the plain [`ComputeOperands`] EIP-712 data with its signed hash.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeOperandResponse {
    payload: ComputeOperands,
    signature: String,
}

/// Response received from the Handle Gateway when publishing results.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeResultResponse {
    payload: ResultPublishingReport,
    signature: String,
}

pub struct GatewayClient {
    client: Client,
    url: String,
    handle_gateway_addresses: HashMap<u32, Address>,
    signer: PrivateKeySigner,
}

impl GatewayClient {
    pub async fn new(
        url: &str,
        handle_gateway_addresses: HashMap<u32, Address>,
        signer: PrivateKeySigner,
    ) -> Result<Self, reqwest::Error> {
        let client = Client::builder().build()?;
        Ok(Self {
            client,
            url: url.to_string(),
            handle_gateway_addresses,
            signer,
        })
    }

    /// Retrieves handles from the Handle Gateway.
    ///
    /// # Errors
    ///
    /// The operation will fail with:
    /// - [`GatewayError::SignatureError`] if the authorization token payload cannot be signed.
    /// - [`GatewayError::InvalidHeaderValue`] if the authorization header value cannot be created.
    /// - [`GatewayError::CommunicationError`] on communication error with the Handle Gateway.
    pub async fn get_handles(
        &self,
        chain_id: u32,
        block_number: u64,
        caller: Address,
        transaction_hash: String,
        rsa_public_key: String,
        operands: Vec<String>,
    ) -> Result<Vec<OperandEntry>, GatewayError> {
        let salt = self.generate_salt();

        let url = format!("{}/v0/compute/operands", self.url);
        let payload = OperandAccessAuthorization {
            chainId: U256::from(chain_id),
            blockNumber: U256::from(block_number),
            caller,
            transactionHash: transaction_hash,
            rsaPublicKey: rsa_public_key,
            operands,
        };
        let auth_value = self.generate_authorization(chain_id, &payload)?;

        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, auth_value)
            .query(&[("salt", &salt.to_string())])
            .send()
            .await
            .map_err(GatewayError::CommunicationError)?;
        if let Err(err) = response.error_for_status_ref() {
            let status = response.status();
            let error_body = response.text().await?;
            error!("Error {status}: {error_body}");
            return Err(GatewayError::CommunicationError(err));
        }
        let authorization = response
            .json::<ComputeOperandResponse>()
            .await
            .map_err(GatewayError::CommunicationError)?;
        self.recover_and_check_address(
            chain_id,
            &authorization.payload,
            &salt,
            &authorization.signature,
        )?;
        let entries: Vec<OperandEntry> = authorization
            .payload
            .operands
            .into_iter()
            .map(|entry| OperandEntry {
                handle: entry.handle,
                ciphertext: entry.ciphertext,
                encrypted_shared_secret: entry.encryptedSharedSecret,
                iv: entry.iv,
            })
            .collect();

        Ok(entries)
    }

    /// Push handles associated to a Nox computation to the Handle Gateway.
    ///
    /// # Errors
    ///
    /// The operation will fail with:
    /// - [`GatewayError::SignatureError`] if the authorization token payload cannot be signed.
    /// - [`GatewayError::InvalidHeaderValue`] if the authorization header value cannot be created.
    /// - [`GatewayError::CommunicationError`] on communication error with the Handle Gateway.
    pub async fn push_results(
        &self,
        chain_id: u32,
        block_number: u64,
        caller: Address,
        transaction_hash: String,
        handles: Vec<ResultEntry>,
    ) -> Result<(), GatewayError> {
        let salt = self.generate_salt();

        let url = format!("{}/v0/compute/results", self.url);
        let payload = ResultPublishingAuthorization {
            chainId: U256::from(chain_id),
            blockNumber: U256::from(block_number),
            caller,
            transactionHash: transaction_hash.clone(),
        };
        let auth_value = self.generate_authorization(chain_id, &payload)?;

        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, auth_value)
            .query(&[("salt", &salt.to_string())])
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
        let authorization = response
            .json::<ComputeResultResponse>()
            .await
            .map_err(GatewayError::CommunicationError)?;
        self.recover_and_check_address(
            chain_id,
            &authorization.payload,
            &salt,
            &authorization.signature,
        )?;
        info!(message = authorization.payload.message, transaction_hash);
        Ok(())
    }

    /// Generates value for AUTHORIZATION header
    ///
    /// # Errors
    ///
    /// The operation will fail with:
    /// - [`GatewayError::SignatureError`] if the authorization token payload cannot be signed.
    /// - [`GatewayError::InvalidHeaderValue`] if the authorization header value cannot be created.
    fn generate_authorization<P>(
        &self,
        chain_id: u32,
        payload: &P,
    ) -> Result<HeaderValue, GatewayError>
    where
        P: Serialize + SolStruct,
    {
        let domain = eip712_domain! {
            name: HANDLE_GATEWAY_EIP712_DOMAIN_NAME,
            version: "1",
            chain_id: u64::from(chain_id),
        };
        let signature = self
            .signer
            .sign_typed_data_sync(payload, &domain)
            .map_err(GatewayError::SignatureError)?
            .to_string();
        let auth =
            STANDARD.encode(json!({ "payload": payload, "signature": signature }).to_string());
        let mut auth_value = HeaderValue::from_str(&format!("EIP712 {auth}"))
            .map_err(GatewayError::InvalidHeaderValue)?;
        auth_value.set_sensitive(true);
        Ok(auth_value)
    }

    /// Generates 32 bytes session salt for a single interaction with the Handle Gateway.
    fn generate_salt(&self) -> FixedBytes<32> {
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        FixedBytes::<32>::from(salt)
    }

    /// Recovers the address used to sign an authorization token and verifies it against an expected address.
    ///
    /// # Errors
    ///
    /// The method will return [`GatewayError::UnknownHandleGateway`] in the following situations:
    /// - The `signature` is not encoded as a valid hex value.
    /// - The signature bytes can not be converted to a `Signature`.
    /// - No address can be recovered from the provided `hash`.
    /// - There is a mismatch between the recovered address and the expected one.
    fn recover_and_check_address<P>(
        &self,
        chain_id: u32,
        payload: &P,
        salt: &FixedBytes<32>,
        signature: &str,
    ) -> Result<(), GatewayError>
    where
        P: SolStruct,
    {
        let domain = eip712_domain! {
            name: HANDLE_GATEWAY_EIP712_DOMAIN_NAME,
            version: "1",
            chain_id: u64::from(chain_id),
            salt: *salt,
        };
        let hash = payload.eip712_signing_hash(&domain);
        let signature_bytes = hex::decode(signature)
            .map_err(|e| GatewayError::UnknownHandleGateway(e.to_string()))?;
        let signature = Signature::from_raw(&signature_bytes)
            .map_err(|e| GatewayError::UnknownHandleGateway(e.to_string()))?;
        let recovered_address = signature
            .recover_address_from_prehash(&hash)
            .map_err(|e| GatewayError::UnknownHandleGateway(e.to_string()))?;
        if recovered_address != self.handle_gateway_addresses[&chain_id] {
            warn!(
                user = self.handle_gateway_addresses[&chain_id].to_string(),
                recovered = recovered_address.to_string(),
                "recovered address mismatch",
            );
            return Err(GatewayError::UnknownHandleGateway(
                "invalid signature".to_string(),
            ));
        }
        Ok(())
    }
}
