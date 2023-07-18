use crate::rpc::{
    serializable_types::FeltParam, BlockHashAndNumber, BlockId, BroadcastedDeclareTransaction,
    BroadcastedDeployAccountTransaction, BroadcastedInvokeTransaction, BroadcastedTransaction,
    ContractClass, DeclareTransactionResult, DeployAccountTransactionResult, EventFilterWithPage,
    EventsPage, FeeEstimate, FunctionCall, InvokeTransaction, InvokeTransactionReceipt,
    InvokeTransactionResult, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs,
    MaybePendingTransactionReceipt, StarknetRpcApiServer, StateUpdate, SyncStatusType, Transaction,
};
use cairo_felt::Felt252;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{error::ErrorCode, ErrorObject},
};
use log::{error, info};
use sequencer::store::Store;

pub struct StarknetBackend {
    pub(crate) store: Store,
}

#[async_trait]
#[allow(unused_variables)]
impl StarknetRpcApiServer for StarknetBackend {
    fn block_number(&self) -> RpcResult<u64> {
        Ok(self.store.get_height().expect("Heigh not found"))
    }

    fn block_hash_and_number(&self) -> RpcResult<BlockHashAndNumber> {
        unimplemented!();
    }

    fn get_block_transaction_count(&self, block_id: BlockId) -> RpcResult<u128> {
        unimplemented!();
    }

    /// get the storage at a given address and key and at a given block
    fn get_storage_at(
        &self,
        contract_address: FeltParam,
        key: FeltParam,
        block_id: BlockId,
    ) -> RpcResult<Felt252> {
        unimplemented!();
    }

    fn call(&self, request: FunctionCall, block_id: BlockId) -> RpcResult<Vec<String>> {
        unimplemented!();
    }

    /// Get the contract class at a given contract address for a given block id
    fn get_class_at(
        &self,
        block_id: BlockId,
        contract_address: FeltParam,
    ) -> RpcResult<ContractClass> {
        unimplemented!();
    }

    /// Get the contract class hash in the given block for the contract deployed at the given
    /// address
    ///
    /// # Arguments
    ///
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag
    /// * `contract_address` - The address of the contract whose class hash will be returned
    ///
    /// # Returns
    ///
    /// * `class_hash` - The class hash of the given contract
    fn get_class_hash_at(
        &self,
        block_id: BlockId,
        contract_address: FeltParam,
    ) -> RpcResult<Felt252> {
        unimplemented!();
    }

    /// Implementation of the `syncing` RPC Endpoint.
    async fn syncing(&self) -> RpcResult<SyncStatusType> {
        unimplemented!();
    }

    /// Get the contract class definition in the given block associated with the given hash.
    fn get_class(&self, block_id: BlockId, class_hash: FeltParam) -> RpcResult<ContractClass> {
        unimplemented!();
    }

    /// Returns the specified block with transaction hashes.
    fn get_block_with_tx_hashes(
        &self,
        block_id: BlockId,
    ) -> RpcResult<MaybePendingBlockWithTxHashes> {
        unimplemented!();
    }

    /// Get the nonce associated with the given address at the given block
    fn get_nonce(&self, block_id: BlockId, contract_address: FeltParam) -> RpcResult<Felt252> {
        unimplemented!();
    }

    /// Get block information with full transactions given the block id
    fn get_block_with_txs(&self, block_id: BlockId) -> RpcResult<MaybePendingBlockWithTxs> {
        let block = match block_id {
            BlockId::Number(0) => self.store.get_block_by_height(1),
            BlockId::Number(height) => {
                info!("block number requested is {}", &height);
                self.store.get_block_by_height(height)
            }
            BlockId::Hash(hash) => self.store.get_block_by_hash(hash.to_bytes_be()),
            BlockId::Latest => self
                .store
                .get_block_by_height(self.store.get_height().expect("Height not found")),
            _ => todo!(),
        };
        block
            .map(|option| option.expect("Block not found"))
            .map_err(|e| {
                error!("error {}", e);
                ErrorObject::from(ErrorCode::InternalError)
            })
    }

    /// Returns the chain id.
    fn chain_id(&self) -> RpcResult<Felt252> {
        unimplemented!();
    }

    /// Add an Invoke Transaction to invoke a contract function
    ///
    /// # Arguments
    ///
    /// * `invoke tx` - <https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#invoke_transaction>
    ///
    /// # Returns
    ///
    /// * `transaction_hash` - transaction hash corresponding to the invocation
    async fn add_invoke_transaction(
        &self,
        invoke_transaction: BroadcastedInvokeTransaction,
    ) -> RpcResult<InvokeTransactionResult> {
        unimplemented!();
    }

    /// Add an Deploy Account Transaction
    ///
    /// # Arguments
    ///
    /// * `deploy account transaction` - <https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#deploy_account_transaction>
    ///
    /// # Returns
    ///
    /// * `transaction_hash` - transaction hash corresponding to the invocation
    /// * `contract_address` - address of the deployed contract account
    async fn add_deploy_account_transaction(
        &self,
        deploy_account_transaction: BroadcastedDeployAccountTransaction,
    ) -> RpcResult<DeployAccountTransactionResult> {
        unimplemented!();
    }

    /// Estimate the fee associated with transaction
    ///
    /// # Arguments
    ///
    /// * `request` - starknet transaction request
    /// * `block_id` - hash of the requested block, number (height), or tag
    ///
    /// # Returns
    ///
    /// * `fee_estimate` - fee estimate in gwei
    async fn estimate_fee(
        &self,
        request: BroadcastedTransaction,
        block_id: BlockId,
    ) -> RpcResult<FeeEstimate> {
        unimplemented!();
    }

    // Returns the details of a transaction by a given block id and index
    fn get_transaction_by_block_id_and_index(
        &self,
        block_id: BlockId,
        index: usize,
    ) -> RpcResult<Transaction> {
        unimplemented!();
    }

    /// Get the information about the result of executing the requested block
    fn get_state_update(&self, block_id: BlockId) -> RpcResult<StateUpdate> {
        unimplemented!();
    }

    /// Returns the transactions in the transaction pool, recognized by this sequencer
    async fn pending_transactions(&self) -> RpcResult<Vec<Transaction>> {
        unimplemented!();
    }

    /// Returns all events matching the given filter
    async fn get_events(&self, filter: EventFilterWithPage) -> RpcResult<EventsPage> {
        unimplemented!();
    }

    /// Submit a new declare transaction to be added to the chain
    ///
    /// # Arguments
    ///
    /// * `declare_transaction` - the declare transaction to be added to the chain
    ///
    /// # Returns
    ///
    /// * `declare_transaction_result` - the result of the declare transaction
    async fn add_declare_transaction(
        &self,
        declare_transaction: BroadcastedDeclareTransaction,
    ) -> RpcResult<DeclareTransactionResult> {
        unimplemented!();
    }

    /// Returns a transaction details from its hash.
    ///
    /// If the transaction is in the transactions pool,
    /// it considers the transaction hash as not found.
    /// Consider using `pending_transaction` for that purpose.
    ///
    /// # Arguments
    ///
    /// * `transaction_hash` - Transaction hash corresponding to the transaction.
    fn get_transaction_by_hash(&self, transaction_hash: FeltParam) -> RpcResult<Transaction> {
        // necessary destructuring so that we can use a hex felt as a param
        self.store
            .get_transaction(transaction_hash.0)
            .map(|option| option.expect("Transaction not found"))
            .map_err(|e| {
                error!("error {}", e);
                ErrorObject::from(ErrorCode::InternalError)
            })
    }

    /// Returns the receipt of a transaction by transaction hash.
    ///
    /// # Arguments
    ///
    /// * `transaction_hash` - Transaction hash corresponding to the transaction.
    fn get_transaction_receipt(
        &self,
        transaction_hash: FeltParam,
    ) -> RpcResult<MaybePendingTransactionReceipt> {
        // necessary destructuring so that we can use a hex felt as a param
        let deserialized_tx = self.get_transaction_by_hash(transaction_hash);

        match deserialized_tx {
            Ok(Transaction::Invoke(InvokeTransaction::V1(tx))) => {
                let invoke_tx_receipt = InvokeTransactionReceipt {
                    transaction_hash: tx.transaction_hash,
                    actual_fee: tx.max_fee,
                    status: crate::rpc::TransactionStatus::AcceptedOnL2,
                    block_hash: Felt252::new(12315),
                    block_number: 24123u64,
                    messages_sent: vec![],
                    events: vec![],
                };

                Ok(MaybePendingTransactionReceipt::Receipt(
                    crate::rpc::TransactionReceipt::Invoke(invoke_tx_receipt),
                ))
            }
            _ => todo!("Transaction receipts or transaction not found"),
        }
    }
}
