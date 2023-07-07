use super::{Key, Value};
use std::fmt::Debug;
use anyhow::Result;

pub trait StoreEngine: Debug + Send {
    fn add_program(&mut self, program_id: Key, program: Value) -> Result<()>;
    fn get_program(&self, program_id: Key) -> Option<Value>;
    fn add_transaction(&mut self, transaction_id: Key, transaction: Value) -> Result<()>;
    fn get_transaction(&self, transaction_id: Key) -> Option<Value>;
    fn add_block(&mut self, block_id: Key, block: Value) -> Result<()>;
    fn get_block(&self, block_id: Key) -> Option<Value>;
}