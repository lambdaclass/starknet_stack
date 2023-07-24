use super::{Key, StoreEngine, Value};
use anyhow::{anyhow, Result};
use cairo_felt::Felt252;
use std::fmt::Debug;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::thread;
use tracing::log::error;
use types::{InvokeTransaction, MaybePendingBlockWithTxs, Transaction};

#[derive(Debug)]
enum StoreCommand {
    Put(DbSelector, Key, Value, SyncSender<Result<()>>),
    Get(DbSelector, Key, SyncSender<Result<Option<Value>>>),
}

#[derive(Debug)]
enum DbSelector {
    Programs,
    Transactions,
}

#[derive(Clone)]
pub struct Store {
    command_sender: Sender<StoreCommand>,
}

impl Store {
    pub fn new(path: &str) -> Result<Self> {
        let programs = rocksdb::DB::open_default(format!("{path}.programs.db"))?;
        let transactions = rocksdb::DB::open_default(format!("{path}.transactions.db"))?;
        let (command_sender, command_receiver): (Sender<StoreCommand>, Receiver<StoreCommand>) =
            channel();
        thread::spawn(move || {
            while let Ok(command) = command_receiver.recv() {
                match command {
                    StoreCommand::Put(db_selector, id, value, reply_to) => {
                        let db = match db_selector {
                            DbSelector::Programs => &programs,
                            DbSelector::Transactions => &transactions,
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
                            DbSelector::Programs => &programs,
                            DbSelector::Transactions => &transactions,
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
    fn add_program(&mut self, program_id: Key, program: Value) -> Result<()> {
        let (reply_sender, reply_receiver) = sync_channel(0);

        self.command_sender.send(StoreCommand::Put(
            DbSelector::Programs,
            program_id,
            program,
            reply_sender,
        ))?;

        reply_receiver.recv()?
    }

    fn get_program(&self, program_id: Key) -> Option<Value> {
        let (reply_sender, reply_receiver) = sync_channel(0);

        self.command_sender
            .send(StoreCommand::Get(
                DbSelector::Programs,
                program_id,
                reply_sender,
            ))
            .unwrap();

        // TODO: properly handle errors
        reply_receiver.recv().expect("error").expect("Other error")
    }

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
        f.debug_struct("RocksDB Store").finish()
    }
}

unsafe impl Sync for Store {}
