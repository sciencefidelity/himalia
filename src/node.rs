use std::sync::RwLock;

pub struct Node {
    addr: String,
}

pub struct Nodes {
    inner: RwLock<Vec<Node>>,
}

impl Nodes {
    pub fn new() -> Nodes {
        Nodes {
            inner: RwLock::new(vec![]),
        }
    }

    pub fn add_node(&self, addr: String) {
        //
    }

    pub fn evict_node(&self, addr: &str) {
        //
    }
}
