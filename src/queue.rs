//! Handle a [`TransactionMessage`] received through NATS.

use alloy_primitives::{Address, FixedBytes};
use serde::Deserialize;
use tracing::{error, info};

use crate::events::{
    ArithmeticOperation, BooleanOperation, BurnOperation, EncryptionOperation, MintOperation,
    Operator, SelectOperation, TransactionMessage, TransferOperation,
};
use crate::handles::{
    cache::HandlesCache,
    crypto::CryptoService,
    gateway::{GatewayClient, HandleEntry},
};
use crate::utils::to_hex_with_prefix;
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputEntry {
    handle: String,
    pub ciphertext: String,
    pub encrypted_shared_secret: String,
    pub iv: String,
}

/// Struct to deal with all events of a message corresponding to a given transaction on-chain.
///
/// Call modules and methods from [`super::compute`] to perform actual computations.
///
/// At the exception of the [`Self::encrypt_plaintext`] methods, all methods have the same following workflow:
/// * Fetches operands from the Handle Gateway and decrypts them
/// * Performs a computation on plaintext operands and produces plaintext results
/// * Encrypts each plaintext result and associates it to its corresponding result handle in an [`HandleEntry`]
/// * Collects all produced [`entries`](HandleEntry) and publishes them to the Handle Gateway
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
        transaction_message: TransactionMessage,
    ) -> Result<(), String> {
        let mut tx_result_entries = Vec::new();
        for event in transaction_message.events {
            let transaction_hash = transaction_message.transaction_hash.clone();
            info!(
                transaction_hash,
                log_index = event.log_index,
                operator = ?event.operator,
                "Received event"
            );
            let event_result_entries = match event.operator {
                Operator::PlaintextToEncrypted(operation) => self.encrypt_plaintext(operation)?,
                Operator::Add(operation) => {
                    self.compute(
                        event.caller,
                        transaction_hash,
                        ArithmeticOperator::Add,
                        operation,
                    )
                    .await?
                }
                Operator::Sub(operation) => {
                    self.compute(
                        event.caller,
                        transaction_hash,
                        ArithmeticOperator::Sub,
                        operation,
                    )
                    .await?
                }
                Operator::Mul(operation) => {
                    self.compute(
                        event.caller,
                        transaction_hash,
                        ArithmeticOperator::Mul,
                        operation,
                    )
                    .await?
                }
                Operator::Div(operation) => {
                    self.compute(
                        event.caller,
                        transaction_hash,
                        ArithmeticOperator::Div,
                        operation,
                    )
                    .await?
                }
                Operator::SafeAdd(operation) => {
                    self.safe_compute(
                        event.caller,
                        transaction_hash,
                        ArithmeticOperator::Add,
                        operation,
                    )
                    .await?
                }
                Operator::SafeSub(operation) => {
                    self.safe_compute(
                        event.caller,
                        transaction_hash,
                        ArithmeticOperator::Sub,
                        operation,
                    )
                    .await?
                }
                Operator::SafeMul(operation) => {
                    self.safe_compute(
                        event.caller,
                        transaction_hash,
                        ArithmeticOperator::Mul,
                        operation,
                    )
                    .await?
                }
                Operator::SafeDiv(operation) => {
                    self.safe_compute(
                        event.caller,
                        transaction_hash,
                        ArithmeticOperator::Div,
                        operation,
                    )
                    .await?
                }
                Operator::Eq(operation) => {
                    self.compare(
                        event.caller,
                        transaction_hash,
                        BooleanOperator::Eq,
                        operation,
                    )
                    .await?
                }
                Operator::Ne(operation) => {
                    self.compare(
                        event.caller,
                        transaction_hash,
                        BooleanOperator::Ne,
                        operation,
                    )
                    .await?
                }
                Operator::Ge(operation) => {
                    self.compare(
                        event.caller,
                        transaction_hash,
                        BooleanOperator::Ge,
                        operation,
                    )
                    .await?
                }
                Operator::Gt(operation) => {
                    self.compare(
                        event.caller,
                        transaction_hash,
                        BooleanOperator::Gt,
                        operation,
                    )
                    .await?
                }
                Operator::Le(operation) => {
                    self.compare(
                        event.caller,
                        transaction_hash,
                        BooleanOperator::Le,
                        operation,
                    )
                    .await?
                }
                Operator::Lt(operation) => {
                    self.compare(
                        event.caller,
                        transaction_hash,
                        BooleanOperator::Lt,
                        operation,
                    )
                    .await?
                }
                Operator::Select(operation) => {
                    self.select(event.caller, transaction_hash, operation)
                        .await?
                }
                Operator::Transfer(operation) => {
                    self.transfer(event.caller, transaction_hash, operation)
                        .await?
                }
                Operator::Mint(operation) => {
                    self.mint(event.caller, transaction_hash, operation).await?
                }
                Operator::Burn(operation) => {
                    self.burn(event.caller, transaction_hash, operation).await?
                }
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
                transaction_message.transaction_hash,
                tx_result_entries,
            )
            .await
            .map_err(|e| format!("Failed to send encrypted data to handle gateway: {e}"))?;
        Ok(())
    }

    /// Encrypts plaintext for storage in handle storage.
    ///
    /// A plaintext value is provided as bytes32 in the [`EncryptionOperation`] event.
    /// See PlaintextToEncrypted event in [`INoxCompute`](https://github.com/iExec-Nox/nox-protocol-contracts/blob/main/contracts/interfaces/INoxCompute.sol) interface.
    fn encrypt_plaintext(
        &mut self,
        operation: EncryptionOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let value_bytes: FixedBytes<32> = operation
            .value
            .parse()
            .map_err(|e| format!("Failed to parse input as bytes32: {e}"))?;
        let value = SolidityValue::from_bytes(operation.tee_type, value_bytes.0)?;
        self.format_and_encrypt_result(operation.handle, value)
            .map(|entry| vec![entry])
    }

    /// Performs a comparison between 2 handles representing a same numeric type and
    /// returns a new handle representing a boolean.
    ///
    /// Comparisons are implemented in [`compare`]
    async fn compare(
        &mut self,
        caller: Address,
        transaction_hash: String,
        operator: BooleanOperator,
        operation: BooleanOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let operand_handles = vec![operation.left_hand_operand, operation.right_hand_operand];
        let operands = self
            .fetch_operands(caller, transaction_hash, operand_handles)
            .await?;
        let result = compare(operator, operands[0].clone(), operands[1].clone())?;
        self.format_and_encrypt_result(operation.result, SolidityValue::Boolean(result))
            .map(|entry| vec![entry])
    }

    /// Performs an arithmetic computation.
    ///
    /// Arithmetic operations are implemented in [`compute`].
    async fn compute(
        &mut self,
        caller: Address,
        transaction_hash: String,
        operator: ArithmeticOperator,
        operation: ArithmeticOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let operand_handles = vec![operation.left_hand_operand, operation.right_hand_operand];
        let operands = self
            .fetch_operands(caller, transaction_hash, operand_handles)
            .await?;
        let result = compute(operator, operands[0].clone(), operands[1].clone())?;
        self.format_and_encrypt_result(operation.result, result)
            .map(|entry| vec![entry])
    }

    /// Performs a safe arithmetic operation.
    ///
    /// Safe arithmetic operations are implemented in [`safe_compute`].
    async fn safe_compute(
        &mut self,
        caller: Address,
        transaction_hash: String,
        operator: ArithmeticOperator,
        operation: SafeArithmeticOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let operand_handles = vec![operation.left_hand_operand, operation.right_hand_operand];
        let operands = self
            .fetch_operands(caller, transaction_hash, operand_handles)
            .await?;
        let (success, result) = safe_compute(operator, operands[0].clone(), operands[1].clone())?;
        Ok(vec![
            self.format_and_encrypt_result(operation.success, SolidityValue::Boolean(success))?,
            self.format_and_encrypt_result(operation.result, result)?,
        ])
    }

    /// Returns one between 2 handles depending on a condition.
    ///
    /// This is equivalent to if { ... } else { ... } or a ternary operator.
    ///
    /// The operation is implemented in [`select`].
    async fn select(
        &mut self,
        caller: Address,
        transaction_hash: String,
        operation: SelectOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let operand_handles = vec![operation.condition, operation.if_true, operation.if_false];
        let operands = self
            .fetch_operands(caller, transaction_hash, operand_handles)
            .await?;
        let result = select(
            operands[0].clone(),
            operands[1].clone(),
            operands[2].clone(),
        )?;
        self.format_and_encrypt_result(operation.result, result)
            .map(|entry| vec![entry])
    }

    /// Confidential tokens transfer operation.
    ///
    /// Performs the equivalent of an ERC20 transfer on handles representing uint256 values.
    ///
    /// The operation is implemented in [`transfer`].
    async fn transfer(
        &mut self,
        caller: Address,
        transaction_hash: String,
        operation: TransferOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let operand_handles = vec![
            operation.balance_from,
            operation.balance_to,
            operation.amount,
        ];
        let operands = self
            .fetch_operands(caller, transaction_hash, operand_handles)
            .await?;
        let (success, new_balance_from, new_balance_to) = transfer(
            operands[0].clone(),
            operands[1].clone(),
            operands[2].clone(),
        )?;
        Ok(vec![
            self.format_and_encrypt_result(operation.success, success)?,
            self.format_and_encrypt_result(operation.new_balance_from, new_balance_from)?,
            self.format_and_encrypt_result(operation.new_balance_to, new_balance_to)?,
        ])
    }

    /// Confidential tokens mint operation.
    ///
    /// Performs the equivalent of an ERC20 mint on handles representing uint256 values.
    ///
    /// The operation is implemented in [`mint`].
    async fn mint(
        &mut self,
        caller: Address,
        transaction_hash: String,
        operation: MintOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let operand_handles = vec![
            operation.balance_to,
            operation.amount,
            operation.total_supply,
        ];
        let operands = self
            .fetch_operands(caller, transaction_hash, operand_handles)
            .await?;
        let (success, new_balance_to, new_total_supply) = mint(
            operands[0].clone(),
            operands[1].clone(),
            operands[2].clone(),
        )?;
        Ok(vec![
            self.format_and_encrypt_result(operation.success, success)?,
            self.format_and_encrypt_result(operation.new_balance_to, new_balance_to)?,
            self.format_and_encrypt_result(operation.new_total_supply, new_total_supply)?,
        ])
    }

    /// Confidential tokens burn operation.
    ///
    /// Performs the equivalent of an ERC20 burn on handles representing uint256 values.
    ///
    /// The operation is implemented in [`burn`].
    async fn burn(
        &mut self,
        caller: Address,
        transaction_hash: String,
        operation: BurnOperation,
    ) -> Result<Vec<HandleEntry>, String> {
        let operand_handles = vec![
            operation.balance_from,
            operation.amount,
            operation.total_supply,
        ];
        let operands = self
            .fetch_operands(caller, transaction_hash, operand_handles)
            .await?;
        let (success, new_balance_from, new_total_supply) = burn(
            operands[0].clone(),
            operands[1].clone(),
            operands[2].clone(),
        )?;
        Ok(vec![
            self.format_and_encrypt_result(operation.success, success)?,
            self.format_and_encrypt_result(operation.new_balance_from, new_balance_from)?,
            self.format_and_encrypt_result(operation.new_total_supply, new_total_supply)?,
        ])
    }

    /// Fetches operands from the handle gateway for a required computation.
    ///
    /// The handle gateway checks at the same time that results handles do not exist.
    /// To avoid multiple decryptions of the same operands, they are stored in cache for
    /// the lifetime of the current transaction computation.
    async fn fetch_operands(
        &mut self,
        caller: Address,
        transaction_hash: String,
        operand_handles: Vec<String>,
    ) -> Result<Vec<SolidityValue>, String> {
        let handles_expected_count = operand_handles.len();
        let missing_operand_handles = self
            .handles_cache
            .find_handles_not_in_cache(operand_handles.clone());
        let encrypted_operands = self
            .handle_gateway
            .get_handles(
                caller,
                transaction_hash,
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
    fn decrypt_and_format_operand(&self, entry: &InputEntry) -> Result<SolidityValue, String> {
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
        handle: String,
        value: SolidityValue,
    ) -> Result<HandleEntry, String> {
        let solidity_type = get_solidity_type_from_handle(&handle)?;
        let solidity_type_size = get_solidity_type_size(solidity_type)?;
        let result = value.to_bytes();
        self.handles_cache.add_handle(&handle, value);
        let encrypted_result = self
            .crypto_svc
            .ecies_encrypt(&result[(32 - solidity_type_size)..32])?;
        let handle_entry = HandleEntry {
            handle,
            ciphertext: to_hex_with_prefix(&encrypted_result.ciphertext),
            public_key: to_hex_with_prefix(&encrypted_result.ephemeral_pubkey),
            nonce: to_hex_with_prefix(&encrypted_result.nonce),
        };
        Ok(handle_entry)
    }
}
