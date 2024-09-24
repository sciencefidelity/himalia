use std::{net::SocketAddr, sync::RwLock};

/// Represents network nodes in the blockchain.
#[derive(Clone)]
pub struct Node {
    addr: String,
}

impl Node {
    const fn new(addr: String) -> Self {
        Self { addr }
    }

    pub fn get_addr(&self) -> String {
        self.addr.clone()
    }

    pub fn parse_socket_addr(&self) -> SocketAddr {
        self.addr.parse().unwrap()
    }
}

#[derive(Default)]
pub struct Nodes(RwLock<Vec<Node>>);

impl Nodes {
    pub const fn new() -> Self {
        Self(RwLock::new(vec![]))
    }

    /// Adds a new [Node] to the collection with the given address only
    /// if the address is not already in the collection.
    pub fn add_node(&self, addr: String) {
        let mut inner = self.0.write().unwrap();
        if !inner.iter().any(|x| x.get_addr().eq(addr.as_str())) {
            inner.push(Node::new(addr));
        }
    }

    pub fn evict_node(&self, addr: &str) {
        let mut inner = self.0.write().unwrap();
        if let Some(idx) = inner.iter().position(|x| x.get_addr().eq(addr)) {
            inner.remove(idx);
        }
    }

    pub fn first(&self) -> Option<Node> {
        if let Some(node) = self.0.read().unwrap().first() {
            return Some(node.clone());
        }
        None
    }

    pub fn get_nodes(&self) -> Vec<Node> {
        self.0.read().unwrap().to_vec()
    }

    pub fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if a [Node] with the given address in in the collection.
    pub fn node_is_known(&self, addr: &str) -> bool {
        self.0.read().unwrap().iter().any(|x| x.get_addr().eq(addr))
    }
}
