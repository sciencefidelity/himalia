use crate::blockchain::Blockchain;

const UTXO_TREE: &str = "chainstate";

pub struct UTXOSet {
    blockchain: Blockchain,
}

impl UTXOSet {
    pub const fn new(blockchain: Blockchain) -> Self {
        Self { blockchain }
    }

    pub fn reindex(&self) {
        let db = self.blockchain.get_db();
        let _utxo_tree = db.open_tree(UTXO_TREE);
    }
}
