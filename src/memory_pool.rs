use std::{collections::HashMap, sync::RwLock};

use data_encoding::HEXLOWER;

use crate::transactions::Transaction;

/// A mempool. Serves as a holding area for pending transactions awaiting
/// validation and inclusion in a block on the [Blockchain] network.
/// Stores unconfirmed transactions, acting as a temporary repository before
/// miners select and verify them for block inclusion.
#[derive(Default)]
pub struct MemoryPool(RwLock<HashMap<String, Transaction>>);

impl MemoryPool {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }

    /// Checks whether a [Transaction] with a specific id exists within the [`MemoryPool`].
    pub fn contains(&self, txid_hex: &str) -> bool {
        self.0.read().unwrap().contains_key(txid_hex)
    }

    /// Inserts a new [Transaction] into the [`MemoryPool`].
    pub fn add(&self, tx: Transaction) {
        let txid_hex = HEXLOWER.encode(tx.get_id());
        self.0.write().unwrap().insert(txid_hex, tx);
    }

    /// Attempts to retrieve a [Transaction] from the [`MemoryPool`] matching
    /// the given transaction id.
    pub fn get(&self, txid_hex: &str) -> Option<Transaction> {
        if let Some(tx) = self.0.read().unwrap().get(txid_hex) {
            return Some(tx.clone());
        }
        None
    }

    /// Removes a [Transaction] from the [`MemoryPool`] matching the given
    /// transaction ID.
    pub fn remove(&self, txid_hex: &str) {
        let mut inner = self.0.write().unwrap();
        inner.remove(txid_hex);
    }

    /// Retrieves all [Transaction]s stored in the [`MemoryPool`].
    pub fn get_all(&self) -> Vec<Transaction> {
        let mut txs = vec![];
        for (_, v) in self.0.read().unwrap().iter() {
            txs.push(v.clone());
        }
        txs
    }

    pub fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// For tracking [Block]s that are in transit during a P2P networking protocol.
#[derive(Default)]
pub struct BlockInTransit(RwLock<Vec<Vec<u8>>>);

impl BlockInTransit {
    pub const fn new() -> Self {
        Self(RwLock::new(Vec::new()))
    }

    pub fn add_blocks(&self, blocks: &[Vec<u8>]) {
        let mut inner = self.0.write().unwrap();
        for hash in blocks {
            inner.push(hash.clone());
        }
    }

    pub fn first(&self) -> Option<Vec<u8>> {
        if let Some(block_hash) = self.0.read().unwrap().first() {
            return Some(block_hash.clone());
        }
        None
    }

    /// Deletes a specific [Block] identified by its hash from [`BlockInTransit`].
    pub fn remove(&self, block_hash: &[u8]) {
        let mut inner = self.0.write().unwrap();
        if let Some(idx) = inner.iter().position(|x| x.eq(block_hash)) {
            inner.remove(idx);
        }
    }

    pub fn clear(&self) {
        let mut inner = self.0.write().unwrap();
        inner.clear();
    }

    pub fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
