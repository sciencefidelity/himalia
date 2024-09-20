use std::sync::RwLock;

pub struct MemoryPool;

impl MemoryPool {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct BlockInTransit {
    inner: RwLock<Vec<Vec<u8>>>,
}

impl BlockInTransit {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(vec![]),
        }
    }
}
