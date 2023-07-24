use super::{Key, StoreEngine, Value};
use anyhow::Result;
use cairo_felt::Felt252;
use sled::Db;
use std::fmt::Debug;
use types::{InvokeTransaction, MaybePendingBlockWithTxs, Transaction};

#[derive(Clone)]
pub struct Store {
    programs: Db,
    transactions: Db,
    blocks_by_hash: Db,
    blocks_by_height: Db,
    values: Db,
    transaction_receipts: Db,
}

impl Store {
    pub fn new(path: &str) -> Self {
        Self {
            programs: sled::open(format!("{path}.programs.db")).unwrap(),
            transactions: sled::open(format!("{path}.transactions.db")).unwrap(),
            blocks_by_hash: sled::open(format!("{path}.blocks1.db")).unwrap(),
            blocks_by_height: sled::open(format!("{path}.blocks2.db")).unwrap(),
            values: sled::open(format!("{path}.values.db")).unwrap(),
            transaction_receipts: sled::open(format!("{path}.transaction_receipts.db")).unwrap(),
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
            .get(program_id)
            .unwrap()
            .map(|value| value.to_vec())
    }

    fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        let tx_serialized: Vec<u8> = serde_json::to_string(&tx).unwrap().as_bytes().to_vec();
        match tx {
            Transaction::Invoke(InvokeTransaction::V1(invoke_tx)) => {
                let _ = self
                    .transactions
                    .insert(invoke_tx.transaction_hash.to_bytes_be(), tx_serialized);
                Ok(())
            }
            _ => todo!(),
        }
    }

    fn get_transaction(&self, tx_hash: Felt252) -> Result<Option<Transaction>> {
        self.transactions
            .get(tx_hash.to_bytes_be())?
            .map_or(Ok(None), |value| {
                Ok(Some(serde_json::from_str::<Transaction>(
                    &String::from_utf8(value.to_vec())?,
                )?))
            })
    }

    fn add_block(&mut self, block: MaybePendingBlockWithTxs) -> Result<()> {
        let block_serialized: Vec<u8> = serde_json::to_string(&block).unwrap().as_bytes().to_vec();
        match block {
            MaybePendingBlockWithTxs::Block(block_with_txs) => {
                let _ = self.blocks_by_hash.insert(
                    block_with_txs.block_hash.to_bytes_be(),
                    block_serialized.clone(),
                );
                let _ = self.blocks_by_height.insert(
                    block_with_txs.block_number.to_be_bytes().to_vec(),
                    block_serialized,
                );
                Ok(())
            }
            MaybePendingBlockWithTxs::PendingBlock(_) => todo!(),
        }
    }

    fn get_block_by_hash(&self, block_hash: Key) -> Result<Option<MaybePendingBlockWithTxs>> {
        self.blocks_by_hash
            .get(block_hash)?
            .map_or(Ok(None), |value| {
                Ok(Some(serde_json::from_str::<MaybePendingBlockWithTxs>(
                    &String::from_utf8(value.to_vec())?,
                )?))
            })
    }

    fn get_block_by_height(&self, block_height: Key) -> Result<Option<MaybePendingBlockWithTxs>> {
        self.blocks_by_height
            .get(block_height)?
            .map_or(Ok(None), |value| {
                Ok(Some(serde_json::from_str::<MaybePendingBlockWithTxs>(
                    &String::from_utf8(value.to_vec())?,
                )?))
            })
    }

    fn set_value(&mut self, key: Key, value: Value) -> Result<()> {
        let _ = self.values.insert(key, value);
        Ok(())
    }

    fn get_value(&self, key: Key) -> Option<Value> {
        self.values.get(key).unwrap().map(|value| value.to_vec())
    }

    fn add_transaction_receipt(
        &mut self,
        transaction_receipt_id: Key,
        transaction_receipt: Value,
    ) -> Result<()> {
        let _ = self
            .transaction_receipts
            .insert(transaction_receipt_id, transaction_receipt);
        Ok(())
    }

    fn get_transaction_receipt(&self, transaction_receipt_id: Key) -> Option<Value> {
        self.transaction_receipts
            .get(transaction_receipt_id)
            .unwrap()
            .map(|value| value.to_vec())
    }
}

impl Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sled Store").finish()
    }
}
