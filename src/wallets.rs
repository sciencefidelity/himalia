use crate::wallet::Wallet;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::{collections::HashMap, env::current_dir};

pub const WALLET_FILE: &str = "wallet.dat";

pub struct Wallets(HashMap<String, Wallet>);

impl Wallets {
    pub fn new() -> Self {
        let mut wallets = Self(HashMap::new());
        wallets.load_from_file();
        wallets
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.0.insert(address.clone(), wallet);
        self.save_to_file();
        address
    }

    pub fn get_addresses(&self) -> Vec<String> {
        let mut addresses = vec![];
        for address in self.0.keys() {
            addresses.push(address.clone());
        }
        addresses
    }

    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.0.get(address)
    }

    pub fn load_from_file(&mut self) {
        let path = current_dir().unwrap().join(WALLET_FILE);
        if !path.exists() {
            return;
        }
        let mut file = File::open(path).unwrap();
        let metadata = file.metadata().expect("unable to read metadata");
        let mut buf = vec![0; usize::try_from(metadata.len()).unwrap()];
        let _ = file.read(&mut buf).expect("buffer overflow");
        self.0 = bincode::deserialize(&buf[..]).expect("unable to deserialize file data");
    }

    fn save_to_file(&self) {
        let path = current_dir().unwrap().join(WALLET_FILE);
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .expect("unable to open wallet.dat");
        let mut writer = BufWriter::new(file);
        let wallets_bytes = bincode::serialize(&self.0).expect("unable to serialize wallets");
        writer.write_all(wallets_bytes.as_slice()).unwrap();
        let _ = writer.flush();
    }
}

impl Default for Wallets {
    fn default() -> Self {
        Self::new()
    }
}
