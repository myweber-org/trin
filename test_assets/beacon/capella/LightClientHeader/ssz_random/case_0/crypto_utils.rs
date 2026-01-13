
use rand::Rng;
use sha2::{Digest, Sha256};

pub fn generate_salt() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut salt = [0u8; 32];
    rng.fill(&mut salt);
    salt
}

pub fn hash_password(password: &str, salt: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    hasher.finalize().to_vec()
}

pub fn verify_password(password: &str, salt: &[u8], hash: &[u8]) -> bool {
    let computed_hash = hash_password(password, salt);
    computed_hash == hash
}