//! Starknet RPC API trait and types
//!
//! Starkware maintains [a description of the Starknet API](https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_api_openrpc.json)
//! using the openRPC specification.

use cairo_felt::Felt252;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;

pub mod types;
pub use types::*;

/// Starknet rpc interface.
#[rpc(server, namespace = "starknet")]
pub trait StarknetRpcApi {
    /// Get the most recent accepted block number
    #[method(name = "blockNumber")]
    fn block_number(&self) -> RpcResult<u64>;

    /// Get the most recent accepted block hash and number
    #[method(name = "blockHashAndNumber")]
    fn block_hash_and_number(&self) -> RpcResult<BlockHashAndNumber>;

    /// Get the number of transactions in a block given a block id
    #[method(name = "getBlockTransactionCount")]
    fn get_block_transaction_count(&self, block_id: BlockId) -> RpcResult<u128>;

    /// Get the value of the storage at the given address and key, at the given block id
    #[method(name = "getStorageAt")]
    fn get_storage_at(
        &self,
        contract_address: Felt252,
        key: Felt252,
        block_id: BlockId,
    ) -> RpcResult<Felt252>;

    /// Call a contract function at a given block id
    #[method(name = "call")]
    fn call(&self, request: FunctionCall, block_id: BlockId) -> RpcResult<Vec<String>>;

    /// Get the contract class at a given contract address for a given block id
    #[method(name = "getClassAt")]
    fn get_class_at(
        &self,
        block_id: BlockId,
        contract_address: Felt252,
    ) -> RpcResult<ContractClass>;

    /// Get the contract class hash in the given block for the contract deployed at the given
    /// address
    #[method(name = "getClassHashAt")]
    fn get_class_hash_at(&self, block_id: BlockId, contract_address: Felt252)
        -> RpcResult<Felt252>;

    /// Get an object about the sync status, or false if the node is not syncing
    #[method(name = "syncing")]
    async fn syncing(&self) -> RpcResult<SyncStatusType>;

    /// Get the contract class definition in the given block associated with the given hash
    #[method(name = "getClass")]
    fn get_class(&self, block_id: BlockId, class_hash: Felt252) -> RpcResult<ContractClass>;

    /// Get block information with transaction hashes given the block id
    #[method(name = "getBlockWithTxHashes")]
    fn get_block_with_tx_hashes(
        &self,
        block_id: BlockId,
    ) -> RpcResult<MaybePendingBlockWithTxHashes>;

    /// Get the nonce associated with the given address at the given block
    #[method(name = "getNonce")]
    fn get_nonce(&self, block_id: BlockId, contract_address: Felt252) -> RpcResult<Felt252>;

    /// Get block information with full transactions given the block id
    #[method(name = "getBlockWithTxs")]
    fn get_block_with_txs(&self, block_id: BlockId) -> RpcResult<MaybePendingBlockWithTxs>;

    /// Get the chain id
    #[method(name = "chainId")]
    fn chain_id(&self) -> RpcResult<Felt252>;

    /// Add an Invoke Transaction to invoke a contract function
    #[method(name = "addInvokeTransaction")]
    async fn add_invoke_transaction(
        &self,
        invoke_transaction: BroadcastedInvokeTransaction,
    ) -> RpcResult<InvokeTransactionResult>;

    /// Add a Deploy Account Transaction
    #[method(name = "addDeployAccountTransaction")]
    async fn add_deploy_account_transaction(
        &self,
        deploy_account_transaction: BroadcastedDeployAccountTransaction,
    ) -> RpcResult<DeployAccountTransactionResult>;

    /// Estimate the fee associated with transaction
    #[method(name = "estimateFee")]
    async fn estimate_fee(
        &self,
        request: BroadcastedTransaction,
        block_id: BlockId,
    ) -> RpcResult<FeeEstimate>;

    /// Get the details of a transaction by a given block id and index
    #[method(name = "getTransactionByBlockIdAndIndex")]
    fn get_transaction_by_block_id_and_index(
        &self,
        block_id: BlockId,
        index: usize,
    ) -> RpcResult<Transaction>;

    /// Get the information about the result of executing the requested block
    #[method(name = "getStateUpdate")]
    fn get_state_update(&self, block_id: BlockId) -> RpcResult<StateUpdate>;

    /// Returns the transactions in the transaction pool, recognized by this sequencer
    #[method(name = "pendingTransactions")]
    async fn pending_transactions(&self) -> RpcResult<Vec<Transaction>>;

    /// Returns all events matching the given filter
    #[method(name = "getEvents")]
    async fn get_events(&self, filter: EventFilterWithPage) -> RpcResult<EventsPage>;

    /// Submit a new transaction to be added to the chain
    #[method(name = "addDeclareTransaction")]
    async fn add_declare_transaction(
        &self,
        declare_transaction: BroadcastedDeclareTransaction,
    ) -> RpcResult<DeclareTransactionResult>;

    /// Returns the information about a transaction by transaction hash.
    #[method(name = "getTransactionByHash")]
    fn get_transaction_by_hash(&self, transaction_hash: Felt252) -> RpcResult<Transaction>;

    /// Returns the receipt of a transaction by transaction hash.
    #[method(name = "getTransactionReceipt")]
    fn get_transaction_receipt(
        &self,
        transaction_hash: Felt252,
    ) -> RpcResult<MaybePendingTransactionReceipt>;
}
