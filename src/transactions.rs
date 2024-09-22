use crate::{base58_decode, blockchain::Blockchain, wallet};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const SUBSIDY: i32 = 10;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TXInput {
    /// Bytes representing the id of the transaction that created
    /// the output that this input is sending.
    txid: Vec<u8>,
    /// An index that represents which output of the transaction
    /// with `txid` this input is sending.
    vout: usize,
    /// Bytes that will contain a digital signature of the Transaction
    /// that includes this input.
    signature: Vec<u8>,
    /// Bytes that will contain the public key of the owner of the
    /// funds being sent.
    pub_key: Vec<u8>,
}

impl TXInput {
    pub fn new(txid: &[u8], vout: usize) -> Self {
        Self {
            txid: txid.to_vec(),
            vout,
            signature: Vec::new(),
            pub_key: Vec::new(),
        }
    }

    pub fn get_txid(&self) -> &[u8] {
        self.txid.as_slice()
    }

    pub const fn get_vout(&self) -> usize {
        self.vout
    }

    pub fn get_pub_key(&self) -> &[u8] {
        self.pub_key.as_slice()
    }

    /// Indicates whether the `pub_key` field of the input corresponds to
    /// the specified `pub_key_hash` byte vector.
    pub fn uses_key(&self, pub_key_hash: &[u8]) -> bool {
        let locking_hash = wallet::hash_pub_key(self.pub_key.as_slice());
        locking_hash.eq(pub_key_hash)
    }
}

/// Manages transaction outputs within the blockchain, storing values
/// and public key hashes. Facilitates creation of new outputs, value
/// retrieval, and verification of locked outputs using cryptographic hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TXOutput {
    value: i32,
    pub_key_hash: Vec<u8>,
}

impl TXOutput {
    pub fn new(value: i32, address: &str) -> Self {
        let mut output = Self {
            value,
            pub_key_hash: Vec::new(),
        };
        output.lock(address);
        output
    }

    pub const fn get_value(&self) -> i32 {
        self.value
    }

    pub fn get_pub_key_hash(&self) -> &[u8] {
        self.pub_key_hash.as_slice()
    }

    fn lock(&mut self, address: &str) {
        let payload = base58_decode(address);
        self.pub_key_hash = payload[1..payload.len() - wallet::ADDRESS_CHECK_SUM_LEN].to_vec();
    }

    /// Checks whether the given `pub_key_hash` matches the stored value.
    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash.eq(pub_key_hash)
    }
}

/// Manages transaction creation, validation and signature verifacation
/// in the blockchain. Constructs Coinbase and UTXO transactions, handles
/// transaction signing and verification, and provides methods for serialization
/// and deserialization of transaction data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    id: Vec<u8>,
    vin: Vec<TXInput>,
    vout: Vec<TXOutput>,
}

impl Transaction {
    pub fn new_coinbase_tx(to: &str) -> Self {
        let tx_output = TXOutput::new(SUBSIDY, to);
        let tx_input = TXInput {
            signature: Uuid::new_v4().as_bytes().to_vec(),
            ..Default::default()
        };
        let mut tx = Self {
            id: vec![],
            vin: vec![tx_input],
            vout: vec![tx_output],
        };
        tx.id = tx.hash();
        tx
    }

    pub fn verify(&self, _blockchain: &Blockchain) -> bool {
        true
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].pub_key.is_empty()
    }

    fn hash(&self) -> Vec<u8> {
        let tx_copy = Self {
            id: vec![],
            vin: self.vin.clone(),
            vout: self.vout.clone(),
        };
        crate::sha256_digest(tx_copy.serialize().as_slice())
    }

    pub fn get_id(&self) -> &[u8] {
        self.id.as_slice()
    }

    pub fn get_id_bytes(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn get_vin(&self) -> &[TXInput] {
        self.vin.as_slice()
    }

    pub fn get_vout(&self) -> &[TXOutput] {
        self.vout.as_slice()
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}
