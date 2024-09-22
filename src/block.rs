use crate::{current_timestamp, sha256_digest};
use crate::{proof_of_work::ProofOfWork, transactions::Transaction};
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
    /// Create a new `Block` struct for incorporation into the blockchain.
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

    /// Generate the first block in the blockchain.
    pub fn generate_genesis(transaction: &Transaction) -> Self {
        let transactions = vec![transaction.clone()];
        Self::new(String::from("None"), &transactions, 0)
    }

    /// Deserialize a `Block` object from a slice of bytes.
    pub fn deserialize(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }

    /// Serialize a slice of bytes from a reference to a block.
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    /// Get the list of transactions.
    pub fn get_transactions(&self) -> &[Transaction] {
        self.transactions.as_slice()
    }

    /// Returns a cloned copy of the `pre_block_hash` string.
    pub fn get_pre_block_hash(&self) -> String {
        self.pre_block_hash.clone()
    }

    /// Get the hash of the transaction.
    pub fn get_hash(&self) -> &str {
        self.hash.as_str()
    }

    /// Returns a vector of bytes representing the hash string held
    /// within the struct instance.
    pub fn get_hash_bytes(&self) -> Vec<u8> {
        self.hash.as_bytes().to_vec()
    }

    /// Return the timestamp held within the struct instance.
    pub const fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Return the height of the block.
    pub const fn get_height(&self) -> usize {
        self.height
    }

    /// Hash the transaction IDs using SHA-256 and return the hash
    /// a vector of bytes.
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
