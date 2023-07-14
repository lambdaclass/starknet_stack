use self::in_memory::Store as InMemoryStore;
use self::rocksdb::Store as RocksDBStore;
use self::sled::Store as SledStore;
use anyhow::Result;
use cairo_felt::Felt252;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub mod in_memory;
pub mod rocksdb;
pub mod sled;
//pub mod store;

pub(crate) type Key = Vec<u8>;
pub(crate) type Value = Vec<u8>;

// TODO: add tests

const BLOCK_HEIGHT: &str = "height";
pub trait StoreEngine: Debug + Send {
    fn add_program(&mut self, program_id: Key, program: Value) -> Result<()>;
    fn get_program(&self, program_id: Key) -> Option<Value>;
    fn add_transaction(&mut self, transaction_id: Key, transaction: Value) -> Result<()>;
    fn get_transaction(&self, transaction_id: Key) -> Option<Value>;
    fn add_block(&mut self, block_id: Key, block: Value) -> Result<()>;
    fn get_block(&self, block_id: Key) -> Option<Value>;
    fn set_value(&mut self, key: Key, value: Value) -> Result<()>;
    fn get_value(&self, key: Key) -> Option<Value>;
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
        let mut store = match engine_type {
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
        };
        store.init();
        store
    }

    fn init(&mut self) {
        if self.get_height().is_none() {
            _ = self.set_height(0);
        }
    }

    // TODO: we might want this API to return types objects instead of bytes
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

    pub fn get_block(&self, block_height: u64) -> Option<Value> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_block(block_height.to_be_bytes().to_vec())
    }

    pub fn get_block_by_hash(&self, hash: Felt252) -> Option<Value> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_block(hash.to_be_bytes().to_vec())
    }

    pub fn set_height(&mut self, value: u64) -> Result<()> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .set_value(BLOCK_HEIGHT.into(), value.to_be_bytes().to_vec())
    }

    pub fn get_height(&self) -> Option<u64> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_value(BLOCK_HEIGHT.into())
            .map(|value| u64::from_be_bytes(value.as_slice()[..8].try_into().unwrap()))
    }
}
