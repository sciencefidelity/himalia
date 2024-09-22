use crate::block::Block;
use crate::transactions::{TXOutput, Transaction};
use data_encoding::HEXLOWER;
use sled::transaction::TransactionResult;
use sled::{Db, Tree};
use std::collections::HashMap;
use std::env::current_dir;
use std::sync::{Arc, RwLock};

const TIP_BLOCK_HASH_KEY: &str = "tip_block_hash";
const BLOCKS_TREE: &str = "blocks";

#[derive(Clone)]
pub struct Blockchain {
    tip_hash: Arc<RwLock<String>>,
    db: Db,
}

impl Blockchain {
    /// Create a new `Blockchain` instance by initializing a new database connection
    /// and creating the genesis block.
    pub fn create(genesis_address: &str) -> Self {
        let db = sled::open(current_dir().unwrap().join("data")).unwrap();
        let blocks_tree = db.open_tree(BLOCKS_TREE).unwrap();
        let data = blocks_tree.get(TIP_BLOCK_HASH_KEY).unwrap();
        let tip_hash = data.map_or_else(
            || {
                let coinbase_tx = Transaction::new_coinbase_tx(genesis_address);
                let block = Block::generate_genesis(&coinbase_tx);
                Self::update_blocks_tree(&blocks_tree, &block);
                String::from(block.get_hash())
            },
            |data| String::from_utf8(data.to_vec()).unwrap(),
        );
        Self {
            tip_hash: Arc::new(RwLock::new(tip_hash)),
            db,
        }
    }

    /// Update the `blocks_tree` database tree with the new `Block` object.
    fn update_blocks_tree(blocks_tree: &Tree, block: &Block) {
        let block_hash = block.get_hash();
        let _: TransactionResult<(), ()> = blocks_tree.transaction(|tx_db| {
            let _ = tx_db.insert(block_hash, block.clone());
            let _ = tx_db.insert(TIP_BLOCK_HASH_KEY, block_hash);
            Ok(())
        });
    }

    /// Initialize the new `Blockchain` instance by initiating a new instance
    /// of the database and retrieving the latest block hash.
    ///
    /// # Panics
    ///
    /// Will panic if the `Blockchain` instance, is not found.
    /// `Blockchain::create()` must be called before calling this method.
    pub fn new() -> Self {
        let db = sled::open(current_dir().unwrap().join("data")).unwrap();
        let blocks_tree = db.open_tree(BLOCKS_TREE).unwrap();
        let tip_bytes = blocks_tree
            .get(TIP_BLOCK_HASH_KEY)
            .unwrap()
            .expect("No existing blockchain found. Create one first.");
        let tip_hash = String::from_utf8(tip_bytes.to_vec()).unwrap();
        Self {
            tip_hash: Arc::new(RwLock::new(tip_hash)),
            db,
        }
    }

    pub const fn get_db(&self) -> &Db {
        &self.db
    }

    pub fn get_tip_hash(&self) -> String {
        self.tip_hash.read().unwrap().clone()
    }

    pub fn set_tip_hash(&self, new_tip_hash: &str) {
        let mut tip_hash = self.tip_hash.write().unwrap();
        *tip_hash = String::from(new_tip_hash);
    }

    /// Mine a block. Create a new block and incorporate it into the blockchain.
    pub fn mine_block(&self, transactions: &[Transaction]) -> Block {
        for transaction in transactions {
            assert!(transaction.verify(self), "ERROR: Invalid transaction");
        }
        let best_height = self.get_best_height();

        let block = Block::new(self.get_tip_hash(), transactions, best_height + 1);
        let block_hash = block.get_hash();

        let blocks_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        Self::update_blocks_tree(&blocks_tree, &block);
        self.set_tip_hash(block_hash);
        block
    }

    pub fn iterator(&self) -> Iterator {
        Iterator::new(self.get_tip_hash(), self.db.clone())
    }

    /// Navigates through the blockchain, identifying UTXOs by inspecting each
    /// transaction within each block.
    pub fn find_utxo(&self) -> HashMap<String, Vec<TXOutput>> {
        let mut utxo: HashMap<String, Vec<TXOutput>> = HashMap::new();
        let mut spent_txos: HashMap<String, Vec<usize>> = HashMap::new();

        let mut iterator = self.iterator();
        loop {
            let option = iterator.next();
            if option.is_none() {
                break;
            }
            let block = option.unwrap();
            'outer: for tx in block.get_transactions() {
                let txid_hex = HEXLOWER.encode(tx.get_id());
                for (idx, out) in tx.get_vout().iter().enumerate() {
                    if let Some(outs) = spent_txos.get(txid_hex.as_str()) {
                        for spend_out_idx in outs {
                            if idx.eq(spend_out_idx) {
                                continue 'outer;
                            }
                        }
                    }
                    if utxo.contains_key(txid_hex.as_str()) {
                        utxo.get_mut(txid_hex.as_str()).unwrap().push(out.clone());
                    } else {
                        utxo.insert(txid_hex.clone(), vec![out.clone()]);
                    }
                }
                if tx.is_coinbase() {
                    continue;
                }

                for txin in tx.get_vin() {
                    let txid_hex = HEXLOWER.encode(txin.get_txid());
                    if spent_txos.contains_key(txid_hex.as_str()) {
                        spent_txos
                            .get_mut(txid_hex.as_str())
                            .unwrap()
                            .push(txin.get_vout());
                    } else {
                        spent_txos.insert(txid_hex, vec![txin.get_vout()]);
                    }
                }
            }
        }
        utxo
    }

    /// Searches the blockchain for a specific transaction by its ID.
    pub fn find_transaction(&self, txid: &[u8]) -> Option<Transaction> {
        let mut iterator = self.iterator();
        loop {
            let option = iterator.next();
            if option.is_none() {
                break;
            }
            let block = option.unwrap();
            for transaction in block.get_transactions() {
                if txid.eq(transaction.get_id()) {
                    return Some(transaction.clone());
                }
            }
        }
        None
    }

    /// Add a new block to the `Blockchain` struct after it's been mined.
    pub fn add_block(&self, block: &Block) {
        let block_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        if block_tree.get(block.get_hash()).unwrap().is_some() {
            return;
        }
        let _: TransactionResult<(), ()> = block_tree.transaction(|tx_db| {
            let _ = tx_db.insert(block.get_hash(), block.serialize()).unwrap();
            let tip_block_bytes = tx_db
                .get(self.get_tip_hash())
                .unwrap()
                .expect("The tip hash is not valid");
            let tip_block = Block::deserialize(tip_block_bytes.as_ref());
            if block.get_height() > tip_block.get_height() {
                let _ = tx_db.insert(TIP_BLOCK_HASH_KEY, block.get_hash()).unwrap();
                self.set_tip_hash(block.get_hash());
            }
            Ok(())
        });
    }

    /// Returns the height of the block with the highest height in blockchain.
    pub fn get_best_height(&self) -> usize {
        let block_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        let tip_block_bytes = block_tree
            .get(self.get_tip_hash())
            .unwrap()
            .expect("The tip hash is valid");
        let tip_block = Block::deserialize(tip_block_bytes.as_ref());
        tip_block.get_height()
    }

    /// Retrieve the block bytes for the database corresponding to the hash
    /// and deserialize them into a `Block` struct.
    pub fn get_block(&self, block_hash: &[u8]) -> Option<Block> {
        let block_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        if let Some(block_bytes) = block_tree.get(block_hash).unwrap() {
            return Some(Block::deserialize(&block_bytes));
        }
        None
    }

    /// Returns a list of block hashes in the blockchain.
    pub fn get_block_hashes(&self) -> Vec<Vec<u8>> {
        let mut iterator = self.iterator();
        let mut blocks = vec![];
        loop {
            let option = iterator.next();
            if option.is_none() {
                break;
            }
            let block = option.unwrap();
            blocks.push(block.get_hash_bytes());
        }
        blocks
    }
}

// TODO: implement Iterator for Block.
pub struct Iterator {
    db: Db,
    current_hash: String,
}

impl Iterator {
    const fn new(tip_hash: String, db: Db) -> Self {
        Self {
            current_hash: tip_hash,
            db,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Block> {
        let block_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        let data = block_tree.get(self.current_hash.clone()).unwrap();
        data.as_ref()?;
        let block = Block::deserialize(data.unwrap().to_vec().as_slice());
        self.current_hash = block.get_pre_block_hash();
        Some(block)
    }
}
