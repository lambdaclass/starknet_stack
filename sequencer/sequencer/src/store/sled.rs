use super::{Key, StoreEngine, Value};
use anyhow::Result;
use sled::Db;
use std::fmt::Debug;

pub struct Store {
    programs: Db,
    transactions: Db,
}

impl Store {
    pub fn new(path: &str) -> Self {
        Self {
            programs: sled::open(format!("{path}.programs.db")).unwrap(),
            transactions: sled::open(format!("{path}.transactions.db")).unwrap(),
        }
    }
}

impl StoreEngine for Store {
    fn add_program(&mut self, program_id: Key, program: Value) -> Result<()> {
        let _ = self.programs.insert(program_id, program);
        Ok(())
    }

    fn get_program(&self, program_id: Key) -> Option<Value> {
        self.programs
            .get(&program_id)
            .unwrap()
            .map(|value| value.to_vec())
    }

    fn add_transaction(&mut self, transaction_id: Key, transaction: Value) -> Result<()> {
        let _ = self.transactions.insert(transaction_id, transaction);
        Ok(())
    }

    fn get_transaction(&self, transaction_id: Key) -> Option<Value> {
        self.transactions
            .get(&transaction_id)
            .unwrap()
            .map(|value| value.to_vec())
    }
}

impl Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sled Store").finish()
    }
}
