use super::{Key, StoreEngine, Value};
use anyhow::{anyhow, Result};
use std::fmt::Debug;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::thread;
use tracing::log::error;

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

    fn add_transaction(&mut self, transaction_id: Key, transaction: Value) -> Result<()> {
        let (reply_sender, reply_receiver) = sync_channel(0);

        self.command_sender.send(StoreCommand::Put(
            DbSelector::Programs,
            transaction_id,
            transaction,
            reply_sender,
        ))?;

        reply_receiver.recv()?
    }

    fn get_transaction(&self, transaction_id: Key) -> Option<Value> {
        let (reply_sender, reply_receiver) = sync_channel(0);

        self.command_sender
            .send(StoreCommand::Get(
                DbSelector::Transactions,
                transaction_id,
                reply_sender,
            ))
            .unwrap();

        // TODO: properly handle errors
        reply_receiver.recv().expect("error").expect("Other error")
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
        f.debug_struct("RocksDB Store").finish()
    }
}

unsafe impl Sync for Store {}
