use std::{net::SocketAddr, sync::RwLock};

#[derive(Clone)]
pub struct Node {
    addr: String,
}

impl Node {
    fn new(addr: String) -> Self {
        Self { addr }
    }

    pub fn get_addr(&self) -> String {
        self.addr.clone()
    }

    pub fn parse_socket_addr(&self) -> SocketAddr {
        self.addr.parse().unwrap()
    }
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
        let mut inner = self.inner.write().unwrap();
        if inner
            .iter()
            .position(|x| x.get_addr().eq(addr.as_str()))
            .is_none()
        {
            inner.push(Node::new(addr));
        }
    }

    pub fn evict_node(&self, addr: &str) {
        let mut inner = self.inner.write().unwrap();
        if let Some(idx) = inner.iter().position(|x| x.get_addr().eq(addr)) {
            inner.remove(idx);
        }
    }

    pub fn first(&self) -> Option<Node> {
        let inner = self.inner.read().unwrap();
        if let Some(node) = inner.first() {
            return Some(node.clone());
        }
        None
    }

    pub fn get_nodes(&self) -> Vec<Node> {
        self.inner.read().unwrap().to_vec()
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn node_is_known(&self, addr: &str) -> bool {
        let inner = self.inner.read().unwrap();
        if let Some(_) = inner.iter().position(|x| x.get_addr().eq(addr)) {
            return true;
        }
        false
    }
}
