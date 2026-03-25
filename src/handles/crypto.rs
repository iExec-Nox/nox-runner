//! ECIES implementation for operands decryption and results encryption.

use aes_gcm::{
    Aes256Gcm, Nonce,
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
use rsa::{Oaep, RsaPrivateKey, RsaPublicKey, pkcs8::EncodePublicKey};
use sha2::Sha256;
use tracing::info;

const ECIES_CONTEXT: &[u8] = b"ECIES:AES_GCM:v1";

pub struct EciesCiphertext {
    pub ephemeral_pubkey: [u8; 33],
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

pub struct CryptoService {
    private: RsaPrivateKey,
    pub public: String,
    protocol_key: PublicKey,
}

impl CryptoService {
    pub async fn new(protocol_key_bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        let protocol_key = PublicKey::from_sec1_bytes(&protocol_key_bytes)?;

        let key = RsaPrivateKey::new(&mut OsRng, 2048)?;
        let rsa_public_key = RsaPublicKey::from(&key).to_public_key_der()?;

        info!(
            protocol_key = hex::encode_prefixed(&protocol_key_bytes),
            "ECIES crypto service initialized"
        );

        Ok(Self {
            private: key.clone(),
            public: hex::encode_prefixed(rsa_public_key),
            protocol_key,
        })
    }

    pub fn ecies_decrypt(
        &self,
        ciphertext: &str,
        encrypted_shared_secret: &str,
        nonce: &str,
    ) -> Result<Vec<u8>, String> {
        let nonce_bytes =
            hex::decode(nonce).map_err(|_| "Failed to decode nonce hex string".to_string())?;
        let ciphertext_bytes = hex::decode(ciphertext)
            .map_err(|_| "Failed to decode ciphertext hex string".to_string())?;
        let encrypted_shared_secret_bytes = hex::decode(encrypted_shared_secret)
            .map_err(|_| "Failed to decode encrypted shared secret hex string".to_string())?;

        let padding = Oaep::new::<Sha256>();
        let shared_secret = self
            .private
            .decrypt(padding, &encrypted_shared_secret_bytes)
            .map_err(|e| format!("Failed to decrypt shared secret {e}"))?;

        let hkdf = Hkdf::<Sha256>::new(None, &shared_secret);
        let mut aes_key = [0u8; 32];
        hkdf.expand(ECIES_CONTEXT, &mut aes_key)
            .map_err(|e| format!("HKDF expansion failed: {e}"))?;
        let cipher = Aes256Gcm::new(&aes_key.into());

        cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext_bytes.as_slice())
            .map_err(|e| format!("AES 256 GCM decryption failed: {e}"))
    }

    pub fn ecies_encrypt(&self, plaintext: &[u8]) -> Result<EciesCiphertext, String> {
        let ephemeral_secret = EphemeralSecret::random(&mut OsRng);
        let shared_secret = ephemeral_secret.diffie_hellman(&self.protocol_key);
        let hkdf = Hkdf::<Sha256>::new(None, shared_secret.raw_secret_bytes());
        let mut aes_key = [0u8; 32];
        hkdf.expand(ECIES_CONTEXT, &mut aes_key)
            .map_err(|e| format!("HKDF expansion failed: {e}"))?;

        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&aes_key));
        let nonce_arr = GenericArray::from_slice(&nonce);
        let ciphertext = cipher
            .encrypt(nonce_arr, plaintext)
            .map_err(|e| format!("AES 256 GCM encryption failed: {e}"))?;

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
