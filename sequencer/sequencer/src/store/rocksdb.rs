use super::{Key, StoreEngine, Value};
use anyhow::{anyhow, Result};
use cairo_felt::Felt252;
use std::fmt::Debug;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::thread;
use tracing::log::error;
use types::{
    InvokeTransaction, MaybePendingBlockWithTxs, MaybePendingTransactionReceipt, Transaction,
    TransactionReceipt,
};

#[derive(Debug)]
enum StoreCommand {
    Put(DbSelector, Key, Value, SyncSender<Result<()>>),
    Get(DbSelector, Key, SyncSender<Result<Option<Value>>>),
}

#[derive(Debug)]
enum DbSelector {
    Transactions,
    BlocksByHash,
    BlocksByHeight,
    Values,
    TransactionReceipts,
}

#[derive(Clone)]
pub struct Store {
    command_sender: Sender<StoreCommand>,
}

impl Store {
    pub fn new(path: &str) -> Result<Self> {
        let transactions = rocksdb::DB::open_default(format!("{path}.transactions.db"))?;
        let blocks_by_hash = rocksdb::DB::open_default(format!("{path}.blocks1.db"))?;
        let blocks_by_height = rocksdb::DB::open_default(format!("{path}.blocks2.db"))?;
        let values = rocksdb::DB::open_default(format!("{path}.values.db"))?;
        let transaction_receipts =
            rocksdb::DB::open_default(format!("{path}.transaction_receipts.db"))?;
        let (command_sender, command_receiver): (Sender<StoreCommand>, Receiver<StoreCommand>) =
            channel();
        thread::spawn(move || {
            while let Ok(command) = command_receiver.recv() {
                match command {
                    StoreCommand::Put(db_selector, id, value, reply_to) => {
                        let db = match db_selector {
                            DbSelector::Transactions => &transactions,
                            DbSelector::BlocksByHash => &blocks_by_hash,
                            DbSelector::BlocksByHeight => &blocks_by_height,
                            DbSelector::Values => &values,
                            DbSelector::TransactionReceipts => &transaction_receipts,
                        };
                        let result = if db.get(id.clone()).unwrap_or(None).is_some() {
                            Err(anyhow!(
                                "Id {} already exists in the store",
                                String::from_utf8_lossy(&id),
                            ))
                        } else {
                            Ok(db
                                .put(id, value)
                                .unwrap_or_else(|e| error!("failed to write to db {}", e)))
                        };

                        reply_to.send(result).unwrap_or_else(|e| error!("{}", e));
                    }
                    StoreCommand::Get(db_selector, id, reply_to) => {
                        let db = match db_selector {
                            DbSelector::Transactions => &transactions,
                            DbSelector::BlocksByHash => &blocks_by_hash,
                            DbSelector::BlocksByHeight => &blocks_by_height,
                            DbSelector::Values => &values,
                            DbSelector::TransactionReceipts => &transaction_receipts,
                        };
                        let result = db.get(id).unwrap_or(None);

                        reply_to
                            .send(Ok(result))
                            .unwrap_or_else(|e| error!("{}", e));
                    }
                };
            }
        });
        Ok(Self { command_sender })
    }
}

impl StoreEngine for Store {
    fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        let (reply_sender, reply_receiver) = sync_channel(0);
        let tx_serialized: Vec<u8> = serde_json::to_string(&tx).unwrap().as_bytes().to_vec();
        match tx {
            Transaction::Invoke(InvokeTransaction::V1(invoke_tx)) => {
                self.command_sender.send(StoreCommand::Put(
                    DbSelector::Transactions,
                    invoke_tx.transaction_hash.to_bytes_be(),
                    tx_serialized,
                    reply_sender,
                ))?;
                reply_receiver.recv()?
            }
            _ => todo!(),
        }
    }

    fn get_transaction(&self, tx_hash: Felt252) -> Result<Option<Transaction>> {
        let (reply_sender, reply_receiver) = sync_channel(0);

        self.command_sender
            .send(StoreCommand::Get(
                DbSelector::Transactions,
                tx_hash.to_bytes_be(),
                reply_sender,
            ))
            .unwrap();

        // TODO: properly handle errors
        reply_receiver.recv()??.map_or(Ok(None), |value| {
            Ok(Some(serde_json::from_str::<Transaction>(
                &String::from_utf8(value.to_vec())?,
            )?))
        })
    }

    fn add_block(&mut self, block: MaybePendingBlockWithTxs) -> Result<()> {
        let (reply_sender_by_hash, reply_receiver_by_hash) = sync_channel(0);
        let (reply_sender_by_height, reply_receiver_by_height) = sync_channel(0);

        let block_serialized: Vec<u8> = serde_json::to_string(&block).unwrap().as_bytes().to_vec();
        match block {
            MaybePendingBlockWithTxs::Block(block_with_txs) => {
                self.command_sender.send(StoreCommand::Put(
                    DbSelector::BlocksByHash,
                    block_with_txs.block_hash.to_bytes_be(),
                    block_serialized.clone(),
                    reply_sender_by_hash,
                ))?;
                self.command_sender.send(StoreCommand::Put(
                    DbSelector::BlocksByHeight,
                    block_with_txs.block_number.to_be_bytes().to_vec(),
                    block_serialized.clone(),
                    reply_sender_by_height,
                ))?;
                reply_receiver_by_hash
                    .recv()
                    .and(reply_receiver_by_height.recv())?
            }
            MaybePendingBlockWithTxs::PendingBlock(_) => todo!(),
        }
    }

    fn get_block_by_hash(&self, block_hash: Felt252) -> Result<Option<MaybePendingBlockWithTxs>> {
        let (reply_sender, reply_receiver) = sync_channel(0);

        self.command_sender
            .send(StoreCommand::Get(
                DbSelector::BlocksByHash,
                block_hash.to_bytes_be(),
                reply_sender,
            ))
            .unwrap();

        // TODO: properly handle errors
        reply_receiver.recv()??.map_or(Ok(None), |value| {
            Ok(Some(serde_json::from_str::<MaybePendingBlockWithTxs>(
                &String::from_utf8(value.to_vec())?,
            )?))
        })
    }

    fn get_block_by_height(&self, block_height: u64) -> Result<Option<MaybePendingBlockWithTxs>> {
        let (reply_sender, reply_receiver) = sync_channel(0);

        self.command_sender
            .send(StoreCommand::Get(
                DbSelector::BlocksByHash,
                block_height.to_be_bytes().to_vec(),
                reply_sender,
            ))
            .unwrap();

        // TODO: properly handle errors
        reply_receiver.recv()??.map_or(Ok(None), |value| {
            Ok(Some(serde_json::from_str::<MaybePendingBlockWithTxs>(
                &String::from_utf8(value.to_vec())?,
            )?))
        })
    }

    fn set_value(&mut self, key: Key, value: Value) -> Result<()> {
        let (reply_sender, reply_receiver) = sync_channel(0);
        self.command_sender.send(StoreCommand::Put(
            DbSelector::Values,
            key,
            value,
            reply_sender,
        ))?;
        reply_receiver.recv()?
    }

    fn get_value(&self, key: Key) -> Option<Value> {
        let (reply_sender, reply_receiver) = sync_channel(0);

        self.command_sender
            .send(StoreCommand::Get(DbSelector::Values, key, reply_sender))
            .unwrap();

        // TODO: properly handle errors
        reply_receiver.recv().unwrap().unwrap()
    }

    fn add_transaction_receipt(
        &mut self,
        transaction_receipt: MaybePendingTransactionReceipt,
    ) -> Result<()> {
        let (reply_sender, reply_receiver) = sync_channel(0);
        let tx_receipt_serialized = serde_json::to_string(&transaction_receipt)
            .expect("Error serializing tx receipt")
            .as_bytes()
            .to_vec();
        match transaction_receipt {
            MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(tx_receipt)) => {
                self.command_sender.send(StoreCommand::Put(
                    DbSelector::TransactionReceipts,
                    tx_receipt.transaction_hash.to_bytes_be(),
                    tx_receipt_serialized,
                    reply_sender,
                ))?;
                reply_receiver.recv()?
            }
            // Currently only InvokeTransactionReceipts are supported
            _ => todo!(),
        }
    }

    fn get_transaction_receipt(
        &self,
        transaction_id: Felt252,
    ) -> Result<Option<MaybePendingTransactionReceipt>> {
        let (reply_sender, reply_receiver) = sync_channel(0);

        self.command_sender
            .send(StoreCommand::Get(
                DbSelector::TransactionReceipts,
                transaction_id.to_bytes_be(),
                reply_sender,
            ))
            .unwrap();

        // TODO: properly handle errors
        reply_receiver.recv()??.map_or(Ok(None), |value| {
            Ok(Some(
                serde_json::from_str::<MaybePendingTransactionReceipt>(&String::from_utf8(
                    value.to_vec(),
                )?)?,
            ))
        })
    }
}

impl Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocksDB Store").finish()
    }
}

unsafe impl Sync for Store {}
