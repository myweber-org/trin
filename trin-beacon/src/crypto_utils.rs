use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};

const NONCE_SIZE: usize = 12;

pub fn encrypt_aes256_gcm(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&OsRng.fill_bytes(NONCE_SIZE));
    
    cipher
        .encrypt(nonce, plaintext)
        .context("Encryption failed")
        .map(|mut ciphertext| {
            let mut result = nonce.to_vec();
            result.append(&mut ciphertext);
            result
        })
}

pub fn decrypt_aes256_gcm(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < NONCE_SIZE {
        anyhow::bail!("Ciphertext too short");
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_bytes, encrypted_data) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    cipher
        .decrypt(nonce, encrypted_data)
        .context("Decryption failed")
}