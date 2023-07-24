use super::{Key, StoreEngine, Value};
use anyhow::Result;
use cairo_felt::Felt252;
use std::{collections::HashMap, fmt::Debug};
use types::{InvokeTransaction, MaybePendingBlockWithTxs, Transaction};

#[derive(Clone)]
pub struct Store {
    programs: HashMap<Key, Value>,
    transactions: HashMap<Felt252, Transaction>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            programs: HashMap::new(),
            transactions: HashMap::new(),
        }
    }
}

impl StoreEngine for Store {
    fn add_program(&mut self, program_id: Key, program: Value) -> Result<()> {
        self.programs.insert(program_id, program);
        Ok(())
    }

    fn get_program(&self, program_id: Key) -> Option<Value> {
        self.programs.get(&program_id).cloned()
    }

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

    fn get_block_by_hash(&self, _block_hash: Key) -> Result<Option<MaybePendingBlockWithTxs>> {
        todo!()
    }

    fn get_block_by_height(&self, _block_height: Key) -> Result<Option<MaybePendingBlockWithTxs>> {
        todo!()
    }

    fn set_value(&mut self, _key: Key, _value: Value) -> Result<()> {
        todo!()
    }

    fn get_value(&self, _key: Key) -> Option<Value> {
        todo!()
    }

    fn add_transaction_receipt(
        &mut self,
        _transaction_id: Key,
        _transaction_receipt: Value,
    ) -> Result<()> {
        todo!()
    }

    fn get_transaction_receipt(&self, _transaction_id: Key) -> Option<Value> {
        todo!()
    }
}

impl Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("In Memory Store").finish()
    }
}
