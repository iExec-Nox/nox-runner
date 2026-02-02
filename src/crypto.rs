use alloy_primitives::hex;
use k256::PublicKey;
use reqwest::Client;
use serde::Deserialize;
use tracing::debug;

use crate::utils::strip_0x_prefix;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KmsPublicKeyResponse {
    public_key: String,
}

pub struct CryptoService {}

impl CryptoService {
    pub async fn new(kms_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder().build()?;
        let base = kms_url.trim_end_matches('/');
        let url = format!("{base}/v0/public-key");
        debug!("Fetching KMS public key from {url}");

        let response = client.get(&url).send().await?.error_for_status()?;

        let hex_protocol_key = response.json::<KmsPublicKeyResponse>().await?.public_key;

        let trimmed = strip_0x_prefix(&hex_protocol_key);
        let bytes = hex::decode(trimmed)?;
        let _protocol_key = PublicKey::from_sec1_bytes(&bytes)?;

        Ok(Self {})
    }
}
