use alloy_primitives::{Address, U256, hex};
use async_nats::Message;
use chrono::NaiveDateTime;
use serde::Deserialize;
use tracing::{debug, error, info};

use crate::compute::arithmetic::{Operator as ArithmeticOperator, SolidityValue, compute};
use crate::crypto::CryptoService;
use crate::events::{BinaryOperation, EncryptionOperation, Operator, TransactionMessage};
use crate::handle_gateway::{GatewayClient, HandleEntry, TEEComputeResult};
use crate::utils::to_hex_with_prefix;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputEntry {
    handle: String,
    pub ciphertext: String,
    pub encrypted_shared_secret: String,
    pub iv: String,
}

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

    /// Deserialize and handle message received from NATS.
    ///
    /// A valid message should represent confidential operations of a single transaction.
    /// When all result handles have been collected, they are sent to handle gateway in a single batch.
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
                Operator::Add(operation) => {
                    self.compute(event.caller, ArithmeticOperator::Add, operation)
                        .await?
                }
                Operator::Sub(operation) => {
                    self.compute(event.caller, ArithmeticOperator::Sub, operation)
                        .await?
                }
                Operator::Div(operation) => {
                    self.compute(event.caller, ArithmeticOperator::Div, operation)
                        .await?
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
            .map_err(|e| format!("Failed to send encrypted data to handle gateway: {e}"))?;
        Ok(())
    }

    /// Performs an arithmetic computation
    ///
    /// Retrieves operands from handle gateway, decrypts them and returns a result hande.
    async fn compute(
        &self,
        caller: Address,
        operator: ArithmeticOperator,
        operation: BinaryOperation,
    ) -> Result<HandleEntry, String> {
        let operand_handles = vec![operation.left_hand_operand, operation.right_hand_operand];
        let encrypted_operands = self
            .handle_gateway
            .get_handles(
                caller,
                self.crypto_svc.public.clone(),
                operand_handles.clone(),
                vec![operation.result.clone()],
            )
            .await
            .map_err(|e| format!("Failed to fetch operands from handle gateway: {e}"))?;
        let mut operands = Vec::new();
        for encrypted_operand in encrypted_operands {
            match self.decrypt_and_format_operand(encrypted_operand) {
                Ok(operand) => operands.push(operand),
                Err(e) => error!("Operand decryption failure: {e}"),
            }
        }
        if operands.len() != operand_handles.len() {
            return Err(format!(
                "Operands count mismatch [decrypted:{}, expected:{}]",
                operands.len(),
                operand_handles.len()
            ));
        }
        let result = compute(operator, operands[0].clone(), operands[1].clone())?.to_bytes();
        let encrypted_result = self.crypto_svc.ecies_encrypt(&result[30..32])?;
        let handle_entry = HandleEntry {
            handle: operation.result,
            ciphertext: hex::encode(encrypted_result.ciphertext),
            public_key: hex::encode(encrypted_result.ephemeral_pubkey),
            nonce: hex::encode(encrypted_result.nonce),
            created_at: NaiveDateTime::default(),
        };
        Ok(handle_entry)
    }

    /// Decrypts and converts an operand to its alloy-primitives type.
    fn decrypt_and_format_operand(&self, entry: InputEntry) -> Result<SolidityValue, String> {
        let data_bytes = self.crypto_svc.ecies_decrypt(
            &entry.ciphertext,
            &entry.encrypted_shared_secret,
            &entry.iv,
        )?;
        let mut result = [0u8; 32];
        result[32 - data_bytes.len()..32].copy_from_slice(&data_bytes);
        let solidity_type = hex::decode(entry.handle)
            .map_err(|e| format!("Failed to decode handle hex value {e}"))?[30];
        SolidityValue::from_bytes(solidity_type, result)
    }

    /// Encrypt plaintext for storage in handle storage.
    ///
    /// A data size cannot be bigger than 32 bytes at the moment.
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
