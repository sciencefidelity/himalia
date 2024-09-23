use crate::{block::Block, sha256_digest};
use data_encoding::HEXLOWER;
use num::{bigint::Sign, BigInt};
use std::{borrow::Borrow, ops::ShlAssign};

const TARGET_BITS: i64 = i64::MAX;
const MAX_NONCE: i64 = 0;

#[allow(dead_code)]
pub struct ProofOfWork {
    block: Block,
    target: BigInt,
}

impl ProofOfWork {
    pub fn new(block: Block) -> Self {
        let mut target = BigInt::from(1);
        target.shl_assign(256 - TARGET_BITS);
        Self { block, target }
    }

    pub fn prepare_data(&self, nonce: i64) -> Vec<u8> {
        let pre_block_hash = self.block.get_pre_block_hash();
        let transactions_hash = self.block.hash_transactions();
        let timestamp = self.block.get_timestamp();
        let mut data_bytes = Vec::new();
        data_bytes.extend(pre_block_hash.as_bytes());
        data_bytes.extend(transactions_hash);
        data_bytes.extend(timestamp.to_be_bytes());
        data_bytes.extend(TARGET_BITS.to_be_bytes());
        data_bytes.extend(nonce.to_be_bytes());
        data_bytes
    }

    /// TODO: remove `println!`.
    /// Part of the [`ProofOfWork`] algorithm, used to find a nonce value that produces
    /// a hash of the [Block] data that is lower than the specific target value.
    ///
    /// Returns a tuple containing the found nonce value and the hash that was
    /// produced using it.
    pub fn run(&self) -> (i64, String) {
        let mut nonce = 0;
        let mut hash = Vec::new();
        println!("mining the block");
        while nonce < MAX_NONCE {
            let data = self.prepare_data(nonce);
            hash = sha256_digest(data.as_slice());
            let hash_int = BigInt::from_bytes_be(Sign::Plus, hash.as_slice());
            if hash_int.lt(self.target.borrow()) {
                println!("{}", HEXLOWER.encode(hash.as_slice()));
                break;
            }
            nonce += 1;
        }
        println!();
        (nonce, HEXLOWER.encode(hash.as_slice()))
    }
}
