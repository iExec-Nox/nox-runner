use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use alloy_primitives::hex;
use hkdf::Hkdf;
use k256::{
    PublicKey,
    ecdh::EphemeralSecret,
    elliptic_curve::{
        rand_core::{OsRng, RngCore},
        sec1::ToEncodedPoint,
    },
};
use reqwest::Client;
use serde::Deserialize;
use sha2::Sha256;
use tracing::debug;

use crate::utils::strip_0x_prefix;

const ECIES_CONTEXT: &[u8] = b"ECIES:AES_GCM:v1";

pub struct EciesCiphertext {
    pub ephemeral_pubkey: [u8; 33],
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KmsPublicKeyResponse {
    public_key: String,
}

pub struct CryptoService {
    protocol_key: PublicKey,
}

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
        let protocol_key = PublicKey::from_sec1_bytes(&bytes)?;

        Ok(Self { protocol_key })
    }

    pub fn ecies_encrypt(&self, plaintext: &[u8]) -> Result<EciesCiphertext, String> {
        let ephemeral_secret = EphemeralSecret::random(&mut OsRng);
        let shared_secret = ephemeral_secret.diffie_hellman(&self.protocol_key);
        let hkdf = Hkdf::<Sha256>::new(None, shared_secret.raw_secret_bytes());
        let mut aes_key = [0u8; 32];
        hkdf.expand(ECIES_CONTEXT, &mut aes_key)
            .map_err(|e| format!("HKDF expansion failed {e}"))?;

        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&aes_key));
        let nonce_arr = GenericArray::from_slice(&nonce);
        let ciphertext = cipher
            .encrypt(nonce_arr, plaintext)
            .map_err(|e| format!("AES GCM encryption failed {e}"))?;

        let ephemeral_pubkey = self.encode_pubkey_compressed(&ephemeral_secret);

        Ok(EciesCiphertext {
            ephemeral_pubkey,
            nonce,
            ciphertext,
        })
    }

    /// Encode an EC public key as compressed SEC1 format (33 bytes).
    fn encode_pubkey_compressed(&self, secret: &EphemeralSecret) -> [u8; 33] {
        let encoded = secret.public_key().to_encoded_point(true);
        let bytes = encoded.as_bytes();
        let mut result = [0u8; 33];
        result.copy_from_slice(bytes);
        result
    }
}
