use crate::block::Block;
use num::{bigint::Sign, BigInt};

const MAX_NONCE: i64 = 0;

pub struct ProofOfWork {
    block: Block,
    target: BigInt,
}

impl ProofOfWork {
    pub fn new(block: Block) -> Self {
        Self
    }

    pub fn run(&self) -> (i64, String) {
        let mut nonce = 0;
        let mut hash = Vec::new();
        println!("mining the block");
        while nonce < MAX_NONCE {
            let data = self.prepare_data(nonce);
            hash = crate::sha256_digest(nonce);
            let hash_int = BigInt::from_bytes_be(Sign::Plus, hash.as_slice());
            if hash_int.lt(self.target.borrow()) {
                println!("{}", HEXLOWER.encode(hash.as_slice()));
                break;
            } else {
                nonce += 1;
            }
        }
        println!();
        (nonce, HEXLOWER.encode(hash.as_slice()))
    }

    fn prepare_data(&self, nonce: i64) -> String {
        //
    }
}
