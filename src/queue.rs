use alloy_primitives::{Address, U256};
use chrono::NaiveDateTime;
use serde::Deserialize;
use tracing::{error, info};

use crate::crypto::CryptoService;
use crate::events::{ArithmeticOperation, EncryptionOperation, Operator, TransactionMessage};
use crate::handle_gateway::{GatewayClient, HandleEntry, TEEComputeResult};
use crate::utils::to_hex_with_prefix;
use crate::{
    compute::{
        arithmetic::{Operator as ArithmeticOperator, SolidityValue, compute, safe_compute},
        get_solidity_type_from_handle, get_solidity_type_size,
    },
    events::SafeArithmeticOperation,
};

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

    /// Handle message representing a transaction received from NATS.
    ///
    /// A valid message should represent confidential operations of a single transaction.
    /// When all result handles have been collected, they are sent to handle gateway in a single batch.
    pub async fn handle_message(
        &self,
        transaction_message: TransactionMessage,
    ) -> Result<(), String> {
        let mut tx_result_entries = Vec::new();
        for event in transaction_message.events {
            info!(
                transaction_hash = transaction_message.transaction_hash,
                log_index = event.log_index,
                operator = ?event.operator,
                "Received event"
            );
            let event_result_entries = match event.operator {
                Operator::PlaintextToEncrypted(operation) => {
                    self.do_plaintext_to_encrypted(operation)?
                }
                Operator::Add(operation) => {
                    self.compute(event.caller, ArithmeticOperator::Add, operation)
                        .await?
                }
                Operator::Sub(operation) => {
                    self.compute(event.caller, ArithmeticOperator::Sub, operation)
                        .await?
                }
                Operator::Mul(operation) => {
                    self.compute(event.caller, ArithmeticOperator::Mul, operation)
                        .await?
                }
                Operator::Div(operation) => {
                    self.compute(event.caller, ArithmeticOperator::Div, operation)
                        .await?
                }
                Operator::SafeAdd(operation) => {
                    self.safe_compute(event.caller, ArithmeticOperator::Add, operation)
                        .await?
                }
                Operator::SafeSub(operation) => {
                    self.safe_compute(event.caller, ArithmeticOperator::Sub, operation)
                        .await?
                }
                Operator::SafeMul(operation) => {
                    self.safe_compute(event.caller, ArithmeticOperator::Mul, operation)
                        .await?
                }
                Operator::SafeDiv(operation) => {
                    self.safe_compute(event.caller, ArithmeticOperator::Div, operation)
                        .await?
                }
            };
            for entry in event_result_entries {
                tx_result_entries.push(entry);
            }
        }
        let request = TEEComputeResult {
            chain_id: transaction_message.chain_id,
            block_number: transaction_message.block_number,
            caller: transaction_message.caller,
            transaction_hash: transaction_message.transaction_hash,
            handles: tx_result_entries,
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
        operation: ArithmeticOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let operand_handles = vec![operation.left_hand_operand, operation.right_hand_operand];
        let result_handles = vec![operation.result.clone()];
        let operands = self
            .fetch_operands(caller, operand_handles, result_handles)
            .await?;
        let result = compute(operator, operands[0].clone(), operands[1].clone())?.to_bytes();
        self.format_and_encrypt_result(operation.result, result)
            .map(|entry| vec![entry])
    }

    async fn safe_compute(
        &self,
        caller: Address,
        operator: ArithmeticOperator,
        operation: SafeArithmeticOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let operand_handles = vec![operation.left_hand_operand, operation.right_hand_operand];
        let result_handles = vec![operation.success.clone(), operation.result.clone()];
        let operands = self
            .fetch_operands(caller, operand_handles, result_handles)
            .await?;
        let (success, result) = safe_compute(operator, operands[0].clone(), operands[1].clone())?;
        let mut success_bytes = [0u8; 32];
        if success {
            success_bytes[31] = 1;
        }
        let mut handles = Vec::<HandleEntry>::new();
        handles.push(self.format_and_encrypt_result(operation.success, success_bytes)?);
        handles.push(self.format_and_encrypt_result(operation.result, result.to_bytes())?);
        Ok(handles)
    }

    async fn fetch_operands(
        &self,
        caller: Address,
        operand_handles: Vec<String>,
        result_handles: Vec<String>,
    ) -> Result<Vec<SolidityValue>, String> {
        let encrypted_operands = self
            .handle_gateway
            .get_handles(
                caller,
                self.crypto_svc.public.clone(),
                operand_handles.clone(),
                result_handles.clone(),
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
        Ok(operands)
    }

    /// Decrypts and converts an operand to its alloy-primitives type.
    fn decrypt_and_format_operand(&self, entry: InputEntry) -> Result<SolidityValue, String> {
        let data_bytes = self.crypto_svc.ecies_decrypt(
            &entry.ciphertext,
            &entry.encrypted_shared_secret,
            &entry.iv,
        )?;
        let mut result = [0u8; 32];
        result[(32 - data_bytes.len())..32].copy_from_slice(&data_bytes);
        let solidity_type = get_solidity_type_from_handle(&entry.handle)?;
        SolidityValue::from_bytes(solidity_type, result)
    }

    /// Encrypt plaintext for storage in handle storage.
    ///
    /// A data size cannot be bigger than 32 bytes at the moment.
    fn do_plaintext_to_encrypted(
        &self,
        operation: EncryptionOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let solidity_type_size = get_solidity_type_size(operation.tee_type)?;
        let plaintext_bytes: U256 = operation
            .value
            .parse()
            .map_err(|e| format!("Failed to parse input as uint256: {e}"))?;
        if solidity_type_size < plaintext_bytes.byte_len() {
            let message = format!(
                "plaintext size {} exceeds TEE type size {solidity_type_size}",
                plaintext_bytes.byte_len()
            );
            error!(message);
            return Err(message);
        }
        self.format_and_encrypt_result(operation.handle, plaintext_bytes.to_be_bytes())
            .map(|entry| vec![entry])
    }

    /// Formats and encrypts result from a 32-byte value to a valid solidity type size
    fn format_and_encrypt_result(
        &self,
        handle: String,
        result: [u8; 32],
    ) -> Result<HandleEntry, String> {
        let solidity_type = get_solidity_type_from_handle(&handle)?;
        let solidity_type_size = get_solidity_type_size(solidity_type)?;
        let encrypted_result = self
            .crypto_svc
            .ecies_encrypt(&result[(32 - solidity_type_size)..32])?;
        let handle_entry = HandleEntry {
            handle,
            ciphertext: to_hex_with_prefix(&encrypted_result.ciphertext),
            public_key: to_hex_with_prefix(&encrypted_result.ephemeral_pubkey),
            nonce: to_hex_with_prefix(&encrypted_result.nonce),
            created_at: NaiveDateTime::default(),
        };
        Ok(handle_entry)
    }
}
