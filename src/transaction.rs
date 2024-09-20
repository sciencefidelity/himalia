use serde::{Deserialize, Serialize};

use crate::blockchain::Blockchain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    id: Vec<u8>,
    vin: Vec<TXInput>,
    vout: Vec<TXOutput>,
}

impl Transaction {
    pub fn get_id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn new_coinbase_tx(_genesis_address: &str) -> Self {
        Self {
            id: vec![],
            vin: vec![],
            vout: vec![],
        }
    }

    pub fn verify(&self, _blockchain: &Blockchain) -> bool {
        true
    }

    pub fn is_coinbase(&self) -> bool {
        true
    }

    pub fn get_vin(&self) -> Vec<TXInput> {
        self.vin.clone()
    }

    pub fn get_vout(&self) -> Vec<TXOutput> {
        self.vout.clone()
    }

    pub fn serialize(&self) -> Vec<u8> {
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TXInput {
    txid: Vec<u8>,
    vout: usize,
    signature: Vec<u8>,
    pub_key: Vec<u8>,
}

impl TXInput {
    pub fn get_txid(&self) -> Vec<u8> {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> usize {
        self.vout
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TXOutput {
    value: i32,
    pub_key_hash: Vec<u8>,
}
