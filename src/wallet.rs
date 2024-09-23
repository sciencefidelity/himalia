use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};
use serde::{Deserialize, Serialize};

const VERSION: u8 = 0x00;
pub const ADDRESS_CHECK_SUM_LEN: usize = 4;

/// Functionality for creating and managing wallet addresses in the blockchain system.
#[derive(Clone, Serialize, Deserialize)]
pub struct Wallet {
    pkcs8: Vec<u8>,
    public_key: Vec<u8>,
}

impl Wallet {
    /// Generates a new [Wallet] instance by creating a new cryptographic key pair,
    /// and extracting the public key.
    pub fn new() -> Self {
        let pkcs8 = crate::new_key_pair();
        let key_pair = EcdsaKeyPair::from_pkcs8(
            &ECDSA_P256_SHA256_FIXED_SIGNING,
            pkcs8.as_ref(),
            &SystemRandom::new(),
        )
        .unwrap();
        let public_key = key_pair.public_key().as_ref().to_vec();
        Self { pkcs8, public_key }
    }

    /// Constructs an address from the [Wallet]'s public key in a Base58 format.
    pub fn get_address(&self) -> String {
        let pub_key_hash = hash_pub_key(self.public_key.as_slice());
        let mut payload: Vec<u8> = Vec::new();
        payload.push(VERSION);
        payload.extend(pub_key_hash.as_slice());
        let checksum = checksum(payload.as_slice());
        payload.extend(checksum.as_slice());
        crate::base58_encode(payload.as_slice())
    }

    /// Retrieves the raw bytes representing the associated public key.
    pub fn get_public_key(&self) -> &[u8] {
        self.public_key.as_slice()
    }

    /// Retrieves the raw bytes of the PKCS #8 representation of the public key.
    pub fn get_pksc8(&self) -> &[u8] {
        self.pkcs8.as_slice()
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

/// Hashes the given public key using SHA-256 and then RIPEMD-160 hash functions.
pub fn hash_pub_key(pub_key: &[u8]) -> Vec<u8> {
    let pub_key_sha256 = crate::sha256_digest(pub_key);
    crate::ripemd160_digest(pub_key_sha256.as_slice())
}

/// Generates a checksum for a payload by applying a double SHA256 hash and
/// extracting the first byte.
fn checksum(payload: &[u8]) -> Vec<u8> {
    let first_sha = crate::sha256_digest(payload);
    let second_sha = crate::sha256_digest(first_sha.as_slice());
    second_sha[0..ADDRESS_CHECK_SUM_LEN].to_vec()
}

/// Validates the integrity of an address by decoding it, separating its components,
/// and recomputing the checksum.
pub fn validate_address(address: &str) -> bool {
    let payload = crate::base58_decode(address);
    let actual_checksum = payload[payload.len() - ADDRESS_CHECK_SUM_LEN..].to_vec();
    let version = payload[0];
    let pub_key_hash = payload[1..payload.len() - ADDRESS_CHECK_SUM_LEN].to_vec();
    let mut target_vec = Vec::new();
    target_vec.push(version);
    target_vec.extend(pub_key_hash);
    let target_checksum = checksum(target_vec.as_slice());
    actual_checksum.eq(target_checksum.as_slice())
}

/// Converts a public key hash into a Base58 encoded address.
pub fn convert_address(pub_hash_key: &[u8]) -> String {
    let mut payload: Vec<u8> = vec![];
    payload.push(VERSION);
    payload.extend(pub_hash_key);
    let checksum = checksum(payload.as_slice());
    payload.extend(checksum.as_slice());
    crate::base58_encode(payload.as_slice())
}
