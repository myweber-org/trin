
use sha2::{Digest, Sha256};
use rand::{RngCore, rngs::OsRng};

pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn secure_token() -> String {
    let random_bytes = generate_random_bytes(32);
    hex::encode(sha256_hash(&random_bytes))
}