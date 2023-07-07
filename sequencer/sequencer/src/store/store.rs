use super::{Key, StoreEngine, Value};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
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

    fn add_block(&mut self, block_id: Key, block: Value) -> Result<()> {
        let (reply_sender, reply_receiver) = sync_channel(0);

        self.command_sender.send(StoreCommand::Put(
            DbSelector::Programs,
            transaction_id,
            transaction,
            reply_sender,
        ))?;

        reply_receiver.recv()?
    }

    fn get_block(&self, block_id: Key) -> Option<Value> {
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
}

impl Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocksDB Store").finish()
    }
}

unsafe impl Sync for Store {}