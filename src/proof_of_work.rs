use crate::{block::Block, sha256_digest};
use data_encoding::HEXLOWER;
use num::{bigint::Sign, BigInt};
use std::borrow::Borrow;

const MAX_NONCE: i64 = 0;

#[allow(dead_code)]
pub struct ProofOfWork {
    block: Block,
    target: BigInt,
}

impl ProofOfWork {
    pub fn new(block: Block) -> Self {
        Self {
            block,
            target: BigInt::new(Sign::Plus, vec![0]),
        }
    }

    /// TODO: remove `println!`.
    /// Part of the proof-of-work algorithm, used to find a nonce value that produces
    /// a hash of the block data that is lower than the specific target value.
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

    const fn prepare_data(&self, _nonce: i64) -> Vec<u8> {
        vec![]
    }
}
