use crate::{proof_of_work::ProofOfWork, transaction::Transaction};
use serde::{Deserialize, Serialize};

pub fn current_timestamp() -> i64 {
    0
}

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

    pub fn deserialize(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap().to_vec()
    }
}
