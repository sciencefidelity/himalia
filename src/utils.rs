use crypto::digest::Digest;
use ring::digest::{Context, SHA256};
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING};
use std::iter::repeat;
use std::time::{SystemTime, UNIX_EPOCH};

/// Retrieves the current timestamp as an integer representing milliseconds since the Unix epoch.
pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_millis()
        .try_into()
        .unwrap()
}

/// Performs a SHA-256 hash operation on the input.
pub fn sha256_digest(data: &[u8]) -> Vec<u8> {
    let mut context = Context::new(&SHA256);
    context.update(data);
    let digest = context.finish();
    digest.as_ref().to_vec()
}

/// Calculates the RIPEMD-160 hash of the input.
pub fn ripemd160_digest(data: &[u8]) -> Vec<u8> {
    let mut ripemd160 = crypto::ripemd160::Ripemd160::new();
    ripemd160.input(data);
    let mut buf: Vec<u8> = repeat(0).take(ripemd160.output_bytes()).collect();
    ripemd160.result(&mut buf);
    buf
}

/// Encodes a slice of bytes using the Base58 encoding scheme.
pub fn base58_encode(data: &[u8]) -> String {
    bs58::encode(data).into_string()
}

/// Decodes a Base58 encoded string back into it's original byte representation.
pub fn base58_decode(data: &str) -> Vec<u8> {
    bs58::decode(data).into_vec().unwrap()
}

/// Generates a new ECDSA key pair returning the private key as bytes.
pub fn new_key_pair() -> Vec<u8> {
    let rng = SystemRandom::new();
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
    pkcs8.as_ref().to_vec()
}

/// Signs the provided `message` using ECDSA P-256 SHA-256 algorithm.
pub fn ecdsa_p256_sha256_sign_digest(pkcs8: &[u8], message: &[u8]) -> Vec<u8> {
    let key_pair = EcdsaKeyPair::from_pkcs8(
        &ECDSA_P256_SHA256_FIXED_SIGNING,
        pkcs8,
        &SystemRandom::new(),
    )
    .unwrap();
    let rng = SystemRandom::new();
    key_pair.sign(&rng, message).unwrap().as_ref().to_vec()
}

/// Verifies an ECDSA P-256 SHA-256 signature against a provided `message` using  the corresponding
/// `public_key` value.
pub fn ecdsa_p256_sha256_sign_verify(public_key: &[u8], signature: &[u8], message: &[u8]) -> bool {
    let peer_public_key =
        ring::signature::UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, public_key);
    let result = peer_public_key.verify(message, signature.as_ref());
    result.is_ok()
}
