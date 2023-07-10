use super::{Key, StoreEngine, Value};
use anyhow::Result;
use std::{collections::HashMap, fmt::Debug};

#[derive(Clone)]
pub struct Store {
    programs: HashMap<Key, Value>,
    transactions: HashMap<Key, Value>,
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

    fn add_transaction(&mut self, transaction_id: Key, transaction: Value) -> Result<()> {
        self.transactions.insert(transaction_id, transaction);
        Ok(())
    }

    fn get_transaction(&self, transaction_id: Key) -> Option<Value> {
        self.transactions.get(&transaction_id).cloned()
    }

    fn add_block(&mut self, _block_id: Key, _block: Value) -> Result<()> {
        todo!()
    }

    fn get_block(&self, _block_id: Key) -> Option<Value> {
        todo!()
    }
}

impl Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("In Memory Store").finish()
    }
}
