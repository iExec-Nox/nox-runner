use alloy_primitives::U256;
use async_nats::Message;
use chrono::NaiveDateTime;
use tracing::{debug, error, info};

use crate::crypto::CryptoService;
use crate::events::{EncryptionOperation, Operator, TransactionMessage};
use crate::handle_gateway::{GatewayClient, HandleEntry, TEEComputeResult};
use crate::utils::to_hex_with_prefix;

pub struct QueueService {
    crypto_svc: CryptoService,
    handle_gateway: GatewayClient,
}

impl QueueService {
    pub fn new(crypto_svc: CryptoService, handle_gateway: GatewayClient) -> Self {
        Self {
            crypto_svc,
            handle_gateway,
        }
    }

    pub async fn handle_message(&self, message: Message) -> Result<(), String> {
        debug!("Received message {:?}", message);
        let transaction_message = serde_json::from_slice::<TransactionMessage>(&message.payload)
            .map_err(|e| format!("Failed to deserialize message: {e}"))?;
        let mut result_entries = Vec::new();
        for event in transaction_message.events {
            info!(
                event.log_index,
                caller = event.caller.to_string(),
                "Received event"
            );
            let result_entry = match event.operator {
                Operator::PlaintextToEncrypted(operation) => {
                    self.do_plaintext_to_encrypted(operation).await?
                }
            };
            result_entries.push(result_entry);
        }
        let request = TEEComputeResult {
            chain_id: transaction_message.chain_id,
            block_number: transaction_message.block_number,
            caller: transaction_message.caller,
            transaction_hash: transaction_message.transaction_hash,
            handles: result_entries,
        };
        self.handle_gateway
            .push_results(request)
            .await
            .map_err(|e| format!("Failed to send encrypted data to gateway: {e}"))?;
        Ok(())
    }

    /// Encrypt plaintext, data cannot be bigger than 32 bytes
    async fn do_plaintext_to_encrypted(
        &self,
        operation: EncryptionOperation,
    ) -> Result<HandleEntry, String> {
        let tee_type_size = match operation.tee_type {
            0_u8 => 1_u8,
            1_u8 => 20_u8,
            2_u8..4_u8 => 32,
            v @ 4_u8..36_u8 => v - 3_u8,
            v @ 36..68_u8 => v - 35_u8,
            v @ 68_u8..100_u8 => v - 67_u8,
            v => {
                let message = format!("Unsupported TEE type for encryption ({v})");
                error!(message);
                return Err(message);
            }
        };
        let plaintext_bytes: U256 = operation
            .value
            .parse()
            .map_err(|e| format!("Failed to parse input as uint256: {e}"))?;
        if usize::from(tee_type_size) < plaintext_bytes.byte_len() {
            let message = format!(
                "plaintext size {} exceeds TEE type size {tee_type_size}",
                plaintext_bytes.byte_len()
            );
            error!(message);
            return Err(message);
        }
        let encrypted_result = self
            .crypto_svc
            .ecies_encrypt(&plaintext_bytes.to_be_bytes_trimmed_vec())?;
        Ok(HandleEntry {
            handle: operation.handle,
            ciphertext: to_hex_with_prefix(&encrypted_result.ciphertext),
            public_key: to_hex_with_prefix(&encrypted_result.ephemeral_pubkey),
            nonce: to_hex_with_prefix(&encrypted_result.nonce),
            created_at: NaiveDateTime::default(),
        })
    }
}
