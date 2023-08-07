use super::{Key, StoreEngine, Value};
use anyhow::Result;
use cairo_felt::Felt252;
use sled::Db;
use std::fmt::Debug;
use types::{
    InvokeTransaction, MaybePendingBlockWithTxs, MaybePendingTransactionReceipt, Transaction,
    TransactionReceipt,
};

#[derive(Clone)]
pub struct Store {
    transactions: Db,
    blocks_by_hash: Db,
    blocks_by_height: Db,
    values: Db,
    transaction_receipts: Db,
}

impl Store {
    pub fn new(path: &str) -> Self {
        Self {
            transactions: sled::open(format!("{path}.transactions.db")).unwrap(),
            blocks_by_hash: sled::open(format!("{path}.blocks1.db")).unwrap(),
            blocks_by_height: sled::open(format!("{path}.blocks2.db")).unwrap(),
            values: sled::open(format!("{path}.values.db")).unwrap(),
            transaction_receipts: sled::open(format!("{path}.transaction_receipts.db")).unwrap(),
        }
    }
}

impl StoreEngine for Store {
    fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        let tx_serialized: Vec<u8> = serde_json::to_string(&tx).unwrap().as_bytes().to_vec();
        match tx {
            Transaction::Invoke(InvokeTransaction::V1(invoke_tx)) => {
                let _ = self
                    .transactions
                    .insert(invoke_tx.transaction_hash.to_bytes_be(), tx_serialized);
                Ok(())
            }
            // Currently only InvokeTransactionV1 are supported
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
                let _ = self
                    .blocks_by_height
                    .insert(block_with_txs.block_number.to_be_bytes(), block_serialized);
                Ok(())
            }
            MaybePendingBlockWithTxs::PendingBlock(_) =>
            // Currently only MaybePendingBlockWithTxs::Block is supported
            {
                todo!()
            }
        }
    }

    fn get_block_by_hash(&self, block_hash: Felt252) -> Result<Option<MaybePendingBlockWithTxs>> {
        self.blocks_by_hash
            .get(block_hash.to_bytes_be())?
            .map_or(Ok(None), |value| {
                Ok(Some(serde_json::from_str::<MaybePendingBlockWithTxs>(
                    &String::from_utf8(value.to_vec())?,
                )?))
            })
    }

    fn get_block_by_height(&self, block_height: u64) -> Result<Option<MaybePendingBlockWithTxs>> {
        self.blocks_by_height
            .get(block_height.to_be_bytes())?
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

    fn get_value(&self, key: Key) -> Result<Option<Vec<u8>>> {
        Ok(self.values.get(key)?.map(|value| value.to_vec()))
    }

    fn add_transaction_receipt(
        &mut self,
        transaction_receipt: MaybePendingTransactionReceipt,
    ) -> Result<()> {
        let tx_receipt_serialized = serde_json::to_string(&transaction_receipt)
            .expect("Error serializing tx receipt")
            .as_bytes()
            .to_vec();
        match transaction_receipt {
            MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(tx_receipt)) => {
                let _ = self.transaction_receipts.insert(
                    tx_receipt.transaction_hash.to_bytes_be(),
                    tx_receipt_serialized,
                );
                Ok(())
            }
            // Currently only InvokeTransactionReceipts are supported
            _ => todo!(),
        }
    }

    fn get_transaction_receipt(
        &self,
        tx_hash: Felt252,
    ) -> Result<Option<MaybePendingTransactionReceipt>> {
        self.transaction_receipts
            .get(tx_hash.to_bytes_be())?
            .map_or(Ok(None), |value| {
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
        f.debug_struct("Sled Store").finish()
    }
}
