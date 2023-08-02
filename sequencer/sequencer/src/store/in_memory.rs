use super::{Key, StoreEngine, Value};
use anyhow::Result;
use cairo_felt::Felt252;
use std::{collections::HashMap, fmt::Debug};
use types::{
    InvokeTransaction, MaybePendingBlockWithTxs, MaybePendingTransactionReceipt, Transaction,
    TransactionReceipt,
};

#[derive(Clone, Default)]
pub struct Store {
    transactions: HashMap<Felt252, Transaction>,
    blocks_by_hash: HashMap<Felt252, MaybePendingBlockWithTxs>,
    blocks_by_height: HashMap<u64, MaybePendingBlockWithTxs>,
    transaction_receipts: HashMap<Felt252, MaybePendingTransactionReceipt>,
    values: HashMap<Key, Value>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            blocks_by_hash: HashMap::new(),
            blocks_by_height: HashMap::new(),
            transaction_receipts: HashMap::new(),
            values: HashMap::new(),
        }
    }
}

impl StoreEngine for Store {
    fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        match tx.clone() {
            Transaction::Invoke(InvokeTransaction::V1(invoke_tx)) => {
                let _ = self.transactions.insert(invoke_tx.transaction_hash, tx);
                Ok(())
            }
            _ => todo!(),
        }
    }

    fn get_transaction(&self, tx_hash: Felt252) -> Result<Option<Transaction>> {
        Ok(self.transactions.get(&tx_hash).cloned())
    }

    fn add_block(&mut self, _block: MaybePendingBlockWithTxs) -> Result<()> {
        todo!()
    }

    fn get_block_by_hash(&self, block_hash: Felt252) -> Result<Option<MaybePendingBlockWithTxs>> {
        Ok(self.blocks_by_hash.get(&block_hash).cloned())
    }

    fn get_block_by_height(&self, block_height: u64) -> Result<Option<MaybePendingBlockWithTxs>> {
        Ok(self.blocks_by_height.get(&block_height).cloned())
    }

    fn set_value(&mut self, key: Key, value: Value) -> Result<()> {
        let _ = self.values.insert(key, value);
        Ok(())
    }

    fn get_value(&self, key: Key) -> Option<Value> {
        self.values.get(&key).cloned()
    }

    fn add_transaction_receipt(
        &mut self,
        _transaction_receipt: MaybePendingTransactionReceipt,
    ) -> Result<()> {
        todo!()
    }

    fn get_transaction_receipt(
        &self,
        tx_hash: Felt252,
    ) -> Result<Option<MaybePendingTransactionReceipt>> {
        Ok(self.transaction_receipts.get(&tx_hash).cloned())
    }
}

impl Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("In Memory Store").finish()
    }
}
