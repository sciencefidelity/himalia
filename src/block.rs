use crate::{current_timestamp, sha256_digest};
use crate::{proof_of_work::ProofOfWork, transaction::Transaction};
use serde::{Deserialize, Serialize};
use sled::IVec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    timestamp: i64,
    pre_block_hash: String,
    hash: String,
    transactions: Vec<Transaction>,
    nonce: i64,
    height: usize,
}

impl Block {
    #[must_use]
    pub fn new(pre_block_hash: String, transactions: &[Transaction], height: usize) -> Self {
        let mut block = Self {
            timestamp: current_timestamp(),
            pre_block_hash,
            hash: String::new(),
            transactions: transactions.to_vec(),
            nonce: 0,
            height,
        };
        let pow = ProofOfWork::new(block.clone());
        (block.nonce, block.hash) = pow.run();
        block
    }

    #[must_use]
    pub fn generate_genesis(transaction: &Transaction) -> Self {
        let transactions = vec![transaction.clone()];
        Self::new(String::from("None"), &transactions, 0)
    }

    /// # Panics
    ///
    /// Will panic if `bincode::deserialize` fails.
    #[must_use]
    pub fn deserialize(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).expect("failed to deserialize bytes")
    }

    /// # Panics
    ///
    /// Will panic if `bincode::serialize` fails.
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("failed to serialize bytes")
    }

    #[must_use]
    pub fn get_transactions(&self) -> &[Transaction] {
        self.transactions.as_slice()
    }

    #[must_use]
    pub fn get_pre_block_hash(&self) -> String {
        self.pre_block_hash.clone()
    }

    #[must_use]
    pub fn get_hash(&self) -> &str {
        self.hash.as_str()
    }

    #[must_use]
    pub fn get_hash_bytes(&self) -> Vec<u8> {
        self.hash.as_bytes().to_vec()
    }

    #[must_use]
    pub const fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    #[must_use]
    pub const fn get_height(&self) -> usize {
        self.height
    }

    #[must_use]
    pub fn hash_transactions(&self) -> Vec<u8> {
        let mut txhashs = vec![];
        for transaction in &self.transactions {
            txhashs.extend(transaction.get_id());
        }
        sha256_digest(txhashs.as_slice())
    }
}

// TODO: implement `TryFrom`
#[allow(clippy::fallible_impl_from)]
impl From<Block> for IVec {
    fn from(b: Block) -> Self {
        let bytes = bincode::serialize(&b).expect("failed to serialize bytes");
        Self::from(bytes)
    }
}
