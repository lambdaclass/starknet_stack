use self::in_memory::Store as InMemoryStore;
use self::rocksdb::Store as RocksDBStore;
use self::sled::Store as SledStore;
use anyhow::Result;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub mod in_memory;
pub mod rocksdb;
pub mod sled;
//pub mod store;

pub(crate) type Key = Vec<u8>;
pub(crate) type Value = Vec<u8>;

pub trait StoreEngine: Debug + Send {
    fn add_program(&mut self, program_id: Key, program: Value) -> Result<()>;
    fn get_program(&self, program_id: Key) -> Option<Value>;
    fn add_transaction(&mut self, transaction_id: Key, transaction: Value) -> Result<()>;
    fn get_transaction(&self, transaction_id: Key) -> Option<Value>;
    fn add_block(&mut self, block_id: Key, block: Value) -> Result<()>;
    fn get_block(&self, block_id: Key) -> Option<Value>;
}

#[derive(Debug, Clone)]
pub struct Store {
    engine: Arc<Mutex<dyn StoreEngine>>,
}

#[allow(dead_code)]
pub enum EngineType {
    RocksDB,
    Sled,
    InMemory,
}

impl Store {
    pub fn new(path: &str, engine_type: EngineType) -> Self {
        match engine_type {
            EngineType::RocksDB => Self {
                engine: Arc::new(Mutex::new(
                    RocksDBStore::new("rocks").expect("could not create rocksdb store"),
                )),
            },
            EngineType::Sled => Self {
                engine: Arc::new(Mutex::new(SledStore::new(&format!("{path}.sled")))),
            },
            EngineType::InMemory => Self {
                engine: Arc::new(Mutex::new(InMemoryStore::new())),
            },
        }
    }

    pub fn add_program(&mut self, program_id: Key, program: Value) -> Result<()> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .add_program(program_id, program)
    }

    pub fn get_program(&self, program_id: Key) -> Option<Value> {
        self.engine.clone().lock().unwrap().get_program(program_id)
    }

    pub fn add_transaction(&mut self, transaction_id: Key, transaction: Value) -> Result<()> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .add_transaction(transaction_id, transaction)
    }

    pub fn get_transaction(&self, transaction_id: Key) -> Option<Value> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_transaction(transaction_id)
    }

    pub fn add_block(&mut self, block_id: Key, block: Value) -> Result<()> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .add_block(block_id, block)
    }

    pub fn get_block(&self, block_id: Key) -> Option<Value> {
        self.engine.clone().lock().unwrap().get_block(block_id)
    }
}
