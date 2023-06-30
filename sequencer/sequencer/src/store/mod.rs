use self::in_memory::Store as InMemoryStore;
use self::rocksdb::Store as RocksDBStore;
use self::sled::Store as SledStore;
use anyhow::Result;
use std::fmt::Debug;

pub mod in_memory;
pub mod rocksdb;
pub mod sled;

pub(crate) type Key = Vec<u8>;
pub(crate) type Value = Vec<u8>;

pub trait StoreEngine: Debug {
    fn add_program(&mut self, program_id: Key, program: Value) -> Result<()>;
    fn get_program(&self, program_id: Key) -> Option<Value>;
    fn add_transaction(&mut self, transaction_id: Key, transaction: Value) -> Result<()>;
    fn get_transaction(&self, transaction_id: Key) -> Option<Value>;
}

#[derive(Debug)]
pub struct Store {
    engine: Box<dyn StoreEngine>,
}

#[allow(dead_code)]
pub enum EngineType {
    RocksDB,
    Sled,
    InMemory,
}

// TODO remove once it's being used
#[allow(dead_code)]
impl Store {
    pub fn new(engine_type: EngineType) -> Self {
        match engine_type {
            EngineType::RocksDB => Self {
                engine: Box::new(
                    RocksDBStore::new("rocks").expect("could not create rocksdb store"),
                ),
            },
            EngineType::Sled => Self {
                engine: Box::new(SledStore::new("sled")),
            },
            EngineType::InMemory => Self {
                engine: Box::new(InMemoryStore::new()),
            },
        }
    }

    pub fn add_program(&mut self, program_id: Key, program: Value) -> Result<()> {
        self.engine.add_program(program_id, program)
    }

    pub fn get_program(&self, program_id: Key) -> Option<Value> {
        self.engine.get_program(program_id)
    }

    pub fn add_transaction(&mut self, transaction_id: Key, transaction: Value) -> Result<()> {
        self.engine.add_transaction(transaction_id, transaction)
    }

    pub fn get_transaction(&self, transaction_id: Key) -> Option<Value> {
        self.engine.get_transaction(transaction_id)
    }
}
