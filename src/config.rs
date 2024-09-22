use once_cell::sync::Lazy;
use std::{collections::HashMap, env, sync::RwLock};

pub static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(Config::new);
static DEFAULT_NODE_ADDR: &str = "127.0.0.1:2001";
const NODE_ADDRESS_KEY: &str = "NODE_ADDRESS";
const MINING_ADDRESS_KEY: &str = "MINING_ADDRESS";

#[derive(Default)]
pub struct Config(RwLock<HashMap<String, String>>);

impl Config {
    pub fn new() -> Self {
        let node_addr = env::var("NODE_ADDRESS").unwrap_or_else(|_| DEFAULT_NODE_ADDR.to_owned());
        let map = HashMap::from([(String::from(NODE_ADDRESS_KEY), node_addr)]);
        Self(RwLock::new(map))
    }

    pub fn get_node_addr(&self) -> String {
        let inner = self.0.read().unwrap();
        inner.get(NODE_ADDRESS_KEY).unwrap().clone()
    }

    pub fn set_mining_addr(&self, addr: String) {
        let mut inner = self.0.write().unwrap();
        inner.insert(String::from(MINING_ADDRESS_KEY), addr);
    }

    pub fn get_mining_addr(&self) -> Option<String> {
        if let Some(addr) = self.0.read().unwrap().get(MINING_ADDRESS_KEY) {
            return Some(addr.clone());
        }
        None
    }

    pub fn is_miner(&self) -> bool {
        let inner = self.0.read().unwrap();
        inner.contains_key(MINING_ADDRESS_KEY)
    }
}
