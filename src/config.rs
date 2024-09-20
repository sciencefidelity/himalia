use once_cell::sync::Lazy;

pub static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(|| Config::new());

pub struct Config;

impl Config {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_node_addr(&self) -> String {
        String::new()
    }
}
