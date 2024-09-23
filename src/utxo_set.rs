use crate::{block::Block, blockchain::Blockchain, transactions::TXOutput};
use data_encoding::HEXLOWER;
use std::collections::HashMap;

const UTXO_TREE: &str = "chainstate";

/// Manages UTXOs (Unspent Transactional Outputs) in the blockchain. Facilitates
/// functionalities such as finding spendable outputs, reindexing outputs, updating
/// outputs after block confirmation, and counting transactions within the blockchain.
pub struct UTXOSet {
    blockchain: Blockchain,
}

impl UTXOSet {
    pub const fn new(blockchain: Blockchain) -> Self {
        Self { blockchain }
    }

    pub const fn get_blockchain(&self) -> &Blockchain {
        &self.blockchain
    }

    /// Identifies spendable outputs for a given public key and required amount.
    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &[u8],
        amount: i32,
    ) -> (i32, HashMap<String, Vec<usize>>) {
        let mut unspent_outputs: HashMap<String, Vec<usize>> = HashMap::new();
        let mut accumulated = 0;
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();
        for item in &utxo_tree {
            let (k, v) = item.unwrap();
            let txid_hex = HEXLOWER.encode(k.to_vec().as_slice());
            let outs: Vec<TXOutput> = bincode::deserialize(v.to_vec().as_slice())
                .expect("unable to deserialize TXOutput");
            for (idx, out) in outs.iter().enumerate() {
                if out.is_locked_with_key(pub_key_hash) && accumulated < amount {
                    accumulated += out.get_value();
                    if unspent_outputs.contains_key(txid_hex.as_str()) {
                        unspent_outputs
                            .get_mut(txid_hex.as_str())
                            .unwrap()
                            .push(idx);
                    } else {
                        unspent_outputs.insert(txid_hex.clone(), vec![idx]);
                    }
                }
            }
        }
        (accumulated, unspent_outputs)
    }

    /// Finds all UTXOs associated with a provided public hash.
    pub fn find_utxo(&self, pub_key_hash: &[u8]) -> Vec<TXOutput> {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();
        let mut utxos = Vec::new();
        for item in &utxo_tree {
            let (_, v) = item.unwrap();
            let outs: Vec<TXOutput> = bincode::deserialize(v.to_vec().as_slice())
                .expect("unable to deserialize TXOutput");
            for out in &outs {
                if out.is_locked_with_key(pub_key_hash) {
                    utxos.push(out.clone());
                }
            }
        }
        utxos
    }

    pub fn count_transactions(&self) -> i32 {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();
        utxo_tree.len().try_into().unwrap()
    }

    /// Reindexes the UTXO tree by clearing it and rebuilding it from the
    /// blockchain's transaction outputs.
    pub fn reindex(&self) {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();
        utxo_tree.clear().unwrap();
        let utxo_map = self.blockchain.find_utxo();
        for (txid_hex, outs) in &utxo_map {
            let txid = HEXLOWER.decode(txid_hex.as_bytes()).unwrap();
            let value = bincode::serialize(outs).unwrap();
            utxo_tree.insert(txid.as_slice(), value).unwrap();
        }
    }

    /// Updates the UTXO set after a block confirmation.
    #[allow(clippy::similar_names)]
    pub fn update(&self, block: &Block) {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();
        for tx in block.get_transactions() {
            if !tx.is_coinbase() {
                for vin in tx.get_vin() {
                    let mut updated_outs = Vec::new();
                    let outs_bytes = utxo_tree.get(vin.get_txid()).unwrap().unwrap();
                    let outs: Vec<TXOutput> = bincode::deserialize(outs_bytes.as_ref())
                        .expect("unable to deserialize TXOutput");
                    for (idx, out) in outs.iter().enumerate() {
                        if idx != vin.get_vout() {
                            updated_outs.push(out.clone());
                        }
                    }
                    if updated_outs.is_empty() {
                        utxo_tree.remove(vin.get_txid()).unwrap();
                    } else {
                        let out_bytes = bincode::serialize(&updated_outs)
                            .expect("unable to serialize TXOutput");
                        utxo_tree.insert(vin.get_txid(), out_bytes).unwrap();
                    }
                }
            }
            let mut new_outputs = Vec::new();
            for out in tx.get_vout() {
                new_outputs.push(out.clone());
            }
            let outs_bytes =
                bincode::serialize(&new_outputs).expect("unable to serialize TXOutput");
            let _ = utxo_tree.insert(tx.get_id(), outs_bytes).unwrap();
        }
    }
}
