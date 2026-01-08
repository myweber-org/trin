use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};

const NONCE_SIZE: usize = 12;

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let encryption_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(encryption_key);
    let nonce = Nonce::from_slice(&generate_nonce());

    cipher
        .encrypt(nonce, plaintext)
        .context("Encryption failed")
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let decryption_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(decryption_key);
    let nonce = Nonce::from_slice(&ciphertext[..NONCE_SIZE]);

    cipher
        .decrypt(nonce, &ciphertext[NONCE_SIZE..])
        .context("Decryption failed")
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}