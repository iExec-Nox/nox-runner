//! Handle a [`TransactionMessage`] received through NATS.

use alloy_primitives::{FixedBytes, hex, utils::Keccak256};
use axum_prometheus::metrics::counter;
use serde::{Deserialize, Serialize};
use strum::VariantNames;
use tracing::{error, info};

use crate::events::{
    ArithmeticOperation, BooleanOperation, BurnOperation, EncryptionOperation, MintOperation,
    Operator, SelectOperation, TransactionMessage, TransactionMetadata, TransferOperation,
};
use crate::handles::{cache::HandlesCache, crypto::CryptoService, gateway::GatewayClient};
use crate::{
    compute::{
        SolidityValue,
        arithmetic::{Operator as ArithmeticOperator, compute, safe_compute},
        boolean::{Operator as BooleanOperator, compare, select},
        get_solidity_type_from_handle, get_solidity_type_size,
        token::{burn, mint, transfer},
    },
    events::SafeArithmeticOperation,
};

/// Handles operands fetched from the Handle Gateway for an event computation.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperandEntry {
    pub handle: String,
    pub ciphertext: String,
    pub encrypted_shared_secret: String,
    pub iv: String,
}

/// Handles results sent to the Handle Gateway when publishing results.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultEntry {
    pub handle: String,
    pub handle_value_tag: String,
    pub ciphertext: String,
    pub public_key: String,
    pub nonce: String,
}

/// Struct to deal with all events of a message corresponding to a given transaction on-chain.
///
/// Call modules and methods from [`super::compute`] to perform actual computations.
///
/// At the exception of the [`Self::encrypt_plaintext`] methods, all methods have the same following workflow:
/// * Fetches operands from the Handle Gateway and decrypts them
/// * Performs a computation on plaintext operands and produces plaintext results
/// * Encrypts each plaintext result and associates it to its corresponding result handle in an [`ResultEntry`]
/// * Collects all produced [`entries`](ResultEntry) and publishes them to the Handle Gateway
///
/// The [`Self::encrypt_plaintext`] method only differs because it does not fetch operands and starts at
/// the third bullet point by directly encrypting the plaintext value.
pub struct QueueService {
    handles_cache: HandlesCache,
    crypto_svc: CryptoService,
    handle_gateway: GatewayClient,
}

impl QueueService {
    pub fn new(crypto_svc: CryptoService, handle_gateway: GatewayClient) -> Self {
        Self {
            handles_cache: HandlesCache::new(),
            crypto_svc,
            handle_gateway,
        }
    }

    /// Initializes Prometheus metrics counters.
    ///
    /// This method needs to be called after the observability harness has been configured.
    pub fn init_metrics(&self, chain_id: String) {
        for operator in Operator::VARIANTS {
            counter!("nox_runner.operation", "chain_id" => chain_id.clone(), "operator" => *operator).absolute(0);
        }
    }

    /// Clears previous cache to free memory and allocate a new fresh empty cache.
    pub fn reset_cache(&mut self) {
        self.handles_cache = HandlesCache::new();
    }

    /// Handle message representing all events associated to a transaction received from NATS.
    ///
    /// A valid message represents all confidential operations of a single transaction.
    /// When all result handles have been collected, they are sent to the Handle Gateway in a single operation
    /// in order to preserve transaction integrity as on a blockchain network.
    ///
    /// At the end of the transaction, before publishing handles to the Handle Gateway, the cache is cleared.
    pub async fn handle_message(
        &mut self,
        transaction_message: &TransactionMessage,
    ) -> Result<(), String> {
        let mut tx_result_entries = Vec::new();
        let metadata = transaction_message.get_metadata();
        let chain_id = metadata.chain_id.to_string();
        for event in &transaction_message.events {
            info!(
                chain_id = chain_id,
                transaction_hash = metadata.transaction_hash,
                log_index = event.log_index,
                operator = ?event.operator,
                "Received event"
            );
            counter!("nox_runner.operation", "chain_id" => chain_id.clone(), "operator" => event.operator.as_str()).increment(1);
            let event_result_entries = match &event.operator {
                Operator::WrapAsPublicHandle(operation) => {
                    self.encrypt_plaintext(&metadata, operation)?
                }
                Operator::Add(operation) => {
                    self.compute(&metadata, ArithmeticOperator::Add, operation)
                        .await?
                }
                Operator::Sub(operation) => {
                    self.compute(&metadata, ArithmeticOperator::Sub, operation)
                        .await?
                }
                Operator::Mul(operation) => {
                    self.compute(&metadata, ArithmeticOperator::Mul, operation)
                        .await?
                }
                Operator::Div(operation) => {
                    self.compute(&metadata, ArithmeticOperator::Div, operation)
                        .await?
                }
                Operator::SafeAdd(operation) => {
                    self.safe_compute(&metadata, ArithmeticOperator::Add, operation)
                        .await?
                }
                Operator::SafeSub(operation) => {
                    self.safe_compute(&metadata, ArithmeticOperator::Sub, operation)
                        .await?
                }
                Operator::SafeMul(operation) => {
                    self.safe_compute(&metadata, ArithmeticOperator::Mul, operation)
                        .await?
                }
                Operator::SafeDiv(operation) => {
                    self.safe_compute(&metadata, ArithmeticOperator::Div, operation)
                        .await?
                }
                Operator::Eq(operation) => {
                    self.compare(&metadata, BooleanOperator::Eq, operation)
                        .await?
                }
                Operator::Ne(operation) => {
                    self.compare(&metadata, BooleanOperator::Ne, operation)
                        .await?
                }
                Operator::Ge(operation) => {
                    self.compare(&metadata, BooleanOperator::Ge, operation)
                        .await?
                }
                Operator::Gt(operation) => {
                    self.compare(&metadata, BooleanOperator::Gt, operation)
                        .await?
                }
                Operator::Le(operation) => {
                    self.compare(&metadata, BooleanOperator::Le, operation)
                        .await?
                }
                Operator::Lt(operation) => {
                    self.compare(&metadata, BooleanOperator::Lt, operation)
                        .await?
                }
                Operator::Select(operation) => self.select(&metadata, operation).await?,
                Operator::Transfer(operation) => self.transfer(&metadata, operation).await?,
                Operator::Mint(operation) => self.mint(&metadata, operation).await?,
                Operator::Burn(operation) => self.burn(&metadata, operation).await?,
            };
            for entry in event_result_entries {
                tx_result_entries.push(entry);
            }
        }
        self.handle_gateway
            .push_results(
                transaction_message.chain_id,
                transaction_message.block_number,
                transaction_message.caller,
                &transaction_message.transaction_hash,
                tx_result_entries,
            )
            .await
            .map_err(|e| format!("Failed to send encrypted data to handle gateway: {e}"))?;
        Ok(())
    }

    /// Encrypts plaintext for storage in handle storage.
    ///
    /// A plaintext value is provided as bytes32 in the [`EncryptionOperation`] event.
    /// See WrapAsPublicHandle event in [`INoxCompute`](https://github.com/iExec-Nox/nox-protocol-contracts/blob/main/contracts/interfaces/INoxCompute.sol) interface.
    fn encrypt_plaintext(
        &mut self,
        metadata: &TransactionMetadata,
        operation: &EncryptionOperation,
    ) -> Result<Vec<ResultEntry>, String> {
        let value_bytes: FixedBytes<32> = operation
            .value
            .parse()
            .map_err(|e| format!("Failed to parse input as bytes32: {e}"))?;
        let value = SolidityValue::from_bytes(operation.tee_type, value_bytes.0)?;
        self.format_and_encrypt_result(metadata, &operation.handle, value)
            .map(|entry| vec![entry])
    }

    /// Performs a comparison between 2 handles representing a same numeric type and
    /// returns a new handle representing a boolean.
    ///
    /// Comparisons are implemented in [`compare`]
    async fn compare(
        &mut self,
        metadata: &TransactionMetadata,
        operator: BooleanOperator,
        operation: &BooleanOperation,
    ) -> Result<Vec<ResultEntry>, String> {
        let operand_handles: [&str; 2] =
            [&operation.left_hand_operand, &operation.right_hand_operand];
        let operands = self.fetch_operands(metadata, &operand_handles).await?;
        let result = compare(operator, operands[0].clone(), operands[1].clone())?;
        self.format_and_encrypt_result(metadata, &operation.result, SolidityValue::Boolean(result))
            .map(|entry| vec![entry])
    }

    /// Performs an arithmetic computation.
    ///
    /// Arithmetic operations are implemented in [`compute`].
    async fn compute(
        &mut self,
        metadata: &TransactionMetadata,
        operator: ArithmeticOperator,
        operation: &ArithmeticOperation,
    ) -> Result<Vec<ResultEntry>, String> {
        let operand_handles: [&str; 2] =
            [&operation.left_hand_operand, &operation.right_hand_operand];
        let operands = self.fetch_operands(metadata, &operand_handles).await?;
        let result = compute(operator, operands[0].clone(), operands[1].clone())?;
        self.format_and_encrypt_result(metadata, &operation.result, result)
            .map(|entry| vec![entry])
    }

    /// Performs a safe arithmetic operation.
    ///
    /// Safe arithmetic operations are implemented in [`safe_compute`].
    async fn safe_compute(
        &mut self,
        metadata: &TransactionMetadata,
        operator: ArithmeticOperator,
        operation: &SafeArithmeticOperation,
    ) -> Result<Vec<ResultEntry>, String> {
        let operand_handles: [&str; 2] =
            [&operation.left_hand_operand, &operation.right_hand_operand];
        let operands = self.fetch_operands(metadata, &operand_handles).await?;
        let (success, result) = safe_compute(operator, operands[0].clone(), operands[1].clone())?;
        Ok(vec![
            self.format_and_encrypt_result(
                metadata,
                &operation.success,
                SolidityValue::Boolean(success),
            )?,
            self.format_and_encrypt_result(metadata, &operation.result, result)?,
        ])
    }

    /// Returns one between 2 handles depending on a condition.
    ///
    /// This is equivalent to if { ... } else { ... } or a ternary operator.
    ///
    /// The operation is implemented in [`select`].
    async fn select(
        &mut self,
        metadata: &TransactionMetadata,
        operation: &SelectOperation,
    ) -> Result<Vec<ResultEntry>, String> {
        let operand_handles: [&str; 3] = [
            &operation.condition,
            &operation.if_true,
            &operation.if_false,
        ];
        let operands = self.fetch_operands(metadata, &operand_handles).await?;
        let result = select(
            operands[0].clone(),
            operands[1].clone(),
            operands[2].clone(),
        )?;
        self.format_and_encrypt_result(metadata, &operation.result, result)
            .map(|entry| vec![entry])
    }

    /// Confidential tokens transfer operation.
    ///
    /// Performs the equivalent of an ERC20 transfer on handles representing uint256 values.
    ///
    /// The operation is implemented in [`transfer`].
    async fn transfer(
        &mut self,
        metadata: &TransactionMetadata,
        operation: &TransferOperation,
    ) -> Result<Vec<ResultEntry>, String> {
        let operand_handles: [&str; 3] = [
            &operation.balance_from,
            &operation.balance_to,
            &operation.amount,
        ];
        let operands = self.fetch_operands(metadata, &operand_handles).await?;
        let (success, new_balance_from, new_balance_to) = transfer(
            operands[0].clone(),
            operands[1].clone(),
            operands[2].clone(),
        )?;
        Ok(vec![
            self.format_and_encrypt_result(metadata, &operation.success, success)?,
            self.format_and_encrypt_result(
                metadata,
                &operation.new_balance_from,
                new_balance_from,
            )?,
            self.format_and_encrypt_result(metadata, &operation.new_balance_to, new_balance_to)?,
        ])
    }

    /// Confidential tokens mint operation.
    ///
    /// Performs the equivalent of an ERC20 mint on handles representing uint256 values.
    ///
    /// The operation is implemented in [`mint`].
    async fn mint(
        &mut self,
        metadata: &TransactionMetadata,
        operation: &MintOperation,
    ) -> Result<Vec<ResultEntry>, String> {
        let operand_handles: [&str; 3] = [
            &operation.balance_to,
            &operation.amount,
            &operation.total_supply,
        ];
        let operands = self.fetch_operands(metadata, &operand_handles).await?;
        let (success, new_balance_to, new_total_supply) = mint(
            operands[0].clone(),
            operands[1].clone(),
            operands[2].clone(),
        )?;
        Ok(vec![
            self.format_and_encrypt_result(metadata, &operation.success, success)?,
            self.format_and_encrypt_result(metadata, &operation.new_balance_to, new_balance_to)?,
            self.format_and_encrypt_result(
                metadata,
                &operation.new_total_supply,
                new_total_supply,
            )?,
        ])
    }

    /// Confidential tokens burn operation.
    ///
    /// Performs the equivalent of an ERC20 burn on handles representing uint256 values.
    ///
    /// The operation is implemented in [`burn`].
    async fn burn(
        &mut self,
        metadata: &TransactionMetadata,
        operation: &BurnOperation,
    ) -> Result<Vec<ResultEntry>, String> {
        let operand_handles: [&str; 3] = [
            &operation.balance_from,
            &operation.amount,
            &operation.total_supply,
        ];
        let operands = self.fetch_operands(metadata, &operand_handles).await?;
        let (success, new_balance_from, new_total_supply) = burn(
            operands[0].clone(),
            operands[1].clone(),
            operands[2].clone(),
        )?;
        Ok(vec![
            self.format_and_encrypt_result(metadata, &operation.success, success)?,
            self.format_and_encrypt_result(
                metadata,
                &operation.new_balance_from,
                new_balance_from,
            )?,
            self.format_and_encrypt_result(
                metadata,
                &operation.new_total_supply,
                new_total_supply,
            )?,
        ])
    }

    /// Fetches operands from the handle gateway for a required computation.
    ///
    /// The handle gateway checks at the same time that results handles do not exist.
    /// To avoid multiple decryptions of the same operands, they are stored in cache for
    /// the lifetime of the current transaction computation.
    async fn fetch_operands(
        &mut self,
        metadata: &TransactionMetadata,
        operand_handles: &[&str],
    ) -> Result<Vec<SolidityValue>, String> {
        let handles_expected_count = operand_handles.len();
        let missing_operand_handles = self
            .handles_cache
            .find_handles_not_in_cache(operand_handles);
        let encrypted_operands = self
            .handle_gateway
            .get_handles(
                metadata.chain_id,
                metadata.block_number,
                metadata.caller,
                metadata.transaction_hash.clone(),
                self.crypto_svc.public.clone(),
                missing_operand_handles,
            )
            .await
            .map_err(|e| format!("Failed to fetch operands from handle gateway: {e}"))?;
        for encrypted_operand in encrypted_operands {
            match self.decrypt_and_format_operand(&encrypted_operand) {
                Ok(operand) => self
                    .handles_cache
                    .add_handle(&encrypted_operand.handle, operand),
                Err(e) => error!("Operand decryption failure: {e}"),
            }
        }
        let operands = self.handles_cache.read_handles(operand_handles);
        let operands_decrypted_count = operands.len();
        if operands_decrypted_count != handles_expected_count {
            return Err(format!(
                "Operands count mismatch [decrypted:{}, expected:{}]",
                operands_decrypted_count, handles_expected_count
            ));
        }
        Ok(operands)
    }

    /// Decrypts and converts an operand to its alloy-primitives type.
    fn decrypt_and_format_operand(&self, entry: &OperandEntry) -> Result<SolidityValue, String> {
        info!(handle = entry.handle, "decrypting operand");
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

    /// Formats and encrypts result from a 32-byte value to a valid solidity type size.
    ///
    /// In the scope of a given transaction, all results are written to cache in case they
    /// are required as operands for another event.
    fn format_and_encrypt_result(
        &mut self,
        metadata: &TransactionMetadata,
        handle: &str,
        value: SolidityValue,
    ) -> Result<ResultEntry, String> {
        let solidity_type = get_solidity_type_from_handle(handle)?;
        let solidity_type_size = get_solidity_type_size(solidity_type)?;
        let result_bytes = value.to_bytes();

        let handle_bytes =
            hex::decode(handle).map_err(|e| format!("Failed to decode {handle} to bytes: {e}"))?;
        let mut hasher = Keccak256::new();
        hasher.update(&handle_bytes);
        hasher.update(result_bytes);
        let handle_value_tag_bytes = hasher.finalize();

        self.handles_cache.add_handle(handle, value);
        let encrypted_result = self.crypto_svc.ecies_encrypt(
            metadata.chain_id,
            &result_bytes[(32 - solidity_type_size)..32],
        )?;
        let handle_entry = ResultEntry {
            handle: handle.to_string(),
            handle_value_tag: hex::encode_prefixed(handle_value_tag_bytes),
            ciphertext: hex::encode_prefixed(encrypted_result.ciphertext),
            public_key: hex::encode_prefixed(encrypted_result.ephemeral_pubkey),
            nonce: hex::encode_prefixed(encrypted_result.nonce),
        };
        Ok(handle_entry)
    }
}
