use self::in_memory::Store as InMemoryStore;
use self::rocksdb::Store as RocksDBStore;
use self::sled::Store as SledStore;
use anyhow::Result;
use cairo_felt::Felt252;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use types::{MaybePendingBlockWithTxs, MaybePendingTransactionReceipt, Transaction};

pub mod in_memory;
pub mod rocksdb;
pub mod sled;

pub(crate) type Key = Vec<u8>;
pub(crate) type Value = Vec<u8>;

const BLOCK_HEIGHT: &str = "height";
pub trait StoreEngine: Debug + Send {
    fn add_transaction(&mut self, transaction: Transaction) -> Result<()>;
    fn get_transaction(&self, tx_hash: Felt252) -> Result<Option<Transaction>>;
    fn add_block(&mut self, block: MaybePendingBlockWithTxs) -> Result<()>;
    fn get_block_by_hash(&self, block_hash: Felt252) -> Result<Option<MaybePendingBlockWithTxs>>;
    fn get_block_by_height(&self, block_height: u64) -> Result<Option<MaybePendingBlockWithTxs>>;
    fn set_value(&mut self, key: Key, value: Value) -> Result<()>;
    fn get_value(&self, key: Key) -> Result<Option<Value>>;
    fn add_transaction_receipt(
        &mut self,
        transaction_receipt: MaybePendingTransactionReceipt,
    ) -> Result<()>;
    fn get_transaction_receipt(
        &self,
        transaction_id: Felt252,
    ) -> Result<Option<MaybePendingTransactionReceipt>>;
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
    pub fn new(path: &str, engine_type: EngineType) -> Result<Self> {
        let mut store = match engine_type {
            EngineType::RocksDB => Self {
                engine: Arc::new(Mutex::new(
                    RocksDBStore::new(&format!("{path}.rocksdb"))
                        .expect("could not create rocksdb store"),
                )),
            },
            EngineType::Sled => Self {
                engine: Arc::new(Mutex::new(SledStore::new(&format!("{path}.sled"))?)),
            },
            EngineType::InMemory => Self {
                engine: Arc::new(Mutex::new(InMemoryStore::new()?)),
            },
        };
        store.init();
        Ok(store)
    }

    fn init(&mut self) {
        if self.get_height().is_none() {
            _ = self.set_height(0);
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .add_transaction(transaction)
    }

    pub fn get_transaction(&self, tx_hash: Felt252) -> Result<Option<Transaction>> {
        self.engine.clone().lock().unwrap().get_transaction(tx_hash)
    }

    pub fn add_block(&mut self, block: MaybePendingBlockWithTxs) -> Result<()> {
        self.engine.clone().lock().unwrap().add_block(block)
    }

    pub fn get_block_by_height(
        &self,
        block_height: u64,
    ) -> Result<Option<MaybePendingBlockWithTxs>> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_block_by_height(block_height)
    }

    pub fn get_block_by_hash(
        &self,
        block_hash: Felt252,
    ) -> Result<Option<MaybePendingBlockWithTxs>> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_block_by_hash(block_hash)
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
            .map_or(None, |result| {
                result.map(|value| u64::from_be_bytes(value.as_slice()[..8].try_into().unwrap()))
            })
    }

    pub fn add_transaction_receipt(
        &mut self,
        transaction_receipt: MaybePendingTransactionReceipt,
    ) -> Result<()> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .add_transaction_receipt(transaction_receipt)
    }

    pub fn get_transaction_receipt(
        &self,
        transaction_id: Felt252,
    ) -> Result<Option<MaybePendingTransactionReceipt>> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_transaction_receipt(transaction_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};
    use test_context::{test_context, TestContext};
    use types::{InvokeTransaction, InvokeTransactionV1};

    struct DbTestContext {}

    impl TestContext for DbTestContext {
        fn setup() -> DbTestContext {
            DbTestContext {}
        }

        fn teardown(self) {
            // Removes all test databases from filesystem
            for entry in fs::read_dir(env::current_dir().unwrap()).unwrap() {
                if entry
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    .starts_with("test.")
                {
                    fs::remove_dir_all(entry.unwrap().path()).unwrap();
                }
            }
        }
    }

    #[test]
    fn test_in_memory_store() {
        let store = Store::new("test", EngineType::InMemory).unwrap();
        test_store_tx(store.clone());
        test_store_height(store);
    }

    #[test_context(DbTestContext)]
    #[test]
    fn test_sled_store(_ctx: &mut DbTestContext) {
        let store = Store::new("test", EngineType::Sled).unwrap();
        test_store_tx(store.clone());
        test_store_height(store);
    }

    #[test]
    fn test_rocksdb_store() {
        let store = Store::new("test", EngineType::RocksDB).unwrap();
        test_store_tx(store.clone());
        test_store_height(store);
    }

    fn test_store_height(mut store: Store) {
        // Test height starts in 0
        assert_eq!(Some(0u64), store.get_height());

        // Set height to an arbitrary number
        let _ = store.set_height(25u64).unwrap();

        // Test value has been persisted
        assert_eq!(Some(25u64), store.get_height());
    }

    fn test_store_tx(mut store: Store) {
        let tx_hash = Felt252::new(123123);
        let tx_fee = Felt252::new(89853483);
        let tx_signature = vec![Felt252::new(183728913)];
        let tx_nonce = Felt252::new(5);
        let tx_sender_address = Felt252::new(91232018);
        let tx_calldata = vec![Felt252::new(10), Felt252::new(0)];

        let tx = new_transaction(
            tx_hash.clone(),
            tx_fee.clone(),
            tx_signature.clone(),
            tx_nonce.clone(),
            tx_sender_address.clone(),
            tx_calldata.clone(),
        );
        let _ = store.add_transaction(tx);

        let stored_tx = store.get_transaction(tx_hash.clone()).unwrap().unwrap();
        let (
            stored_tx_hash,
            stored_tx_fee,
            stored_tx_signature,
            stored_tx_nonce,
            stored_tx_sender_address,
            stored_tx_calldata,
        ) = get_tx_data(stored_tx);
        assert_eq!(tx_hash, stored_tx_hash);
        assert_eq!(tx_fee, stored_tx_fee);
        assert_eq!(tx_signature, stored_tx_signature);
        assert_eq!(tx_nonce, stored_tx_nonce);
        assert_eq!(tx_sender_address, stored_tx_sender_address);
        assert_eq!(tx_calldata, stored_tx_calldata);
    }

    fn new_transaction(
        tx_hash: Felt252,
        tx_fee: Felt252,
        tx_signature: Vec<Felt252>,
        tx_nonce: Felt252,
        tx_sender_address: Felt252,
        tx_calldata: Vec<Felt252>,
    ) -> Transaction {
        let invoke_tx_v1 = InvokeTransactionV1 {
            transaction_hash: tx_hash,
            max_fee: tx_fee,
            signature: tx_signature,
            nonce: tx_nonce,
            sender_address: tx_sender_address,
            calldata: tx_calldata,
        };
        Transaction::Invoke(InvokeTransaction::V1(invoke_tx_v1))
    }

    fn get_tx_data(
        tx: Transaction,
    ) -> (
        Felt252,
        Felt252,
        Vec<Felt252>,
        Felt252,
        Felt252,
        Vec<Felt252>,
    ) {
        match tx {
            Transaction::Invoke(InvokeTransaction::V1(invoke_tx_v1)) => (
                invoke_tx_v1.transaction_hash,
                invoke_tx_v1.max_fee,
                invoke_tx_v1.signature,
                invoke_tx_v1.nonce,
                invoke_tx_v1.sender_address,
                invoke_tx_v1.calldata,
            ),
            _ => todo!(),
        }
    }
}
