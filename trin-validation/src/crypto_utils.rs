use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};

const NONCE_SIZE: usize = 12;

pub fn encrypt_aes256_gcm(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_SIZE]>());
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .context("Encryption failed")?;
    
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_aes256_gcm(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < NONCE_SIZE {
        anyhow::bail!("Ciphertext too short");
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_slice, encrypted_data) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_slice);
    
    let plaintext = cipher
        .decrypt(nonce, encrypted_data)
        .context("Decryption failed")?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_aes256_gcm_roundtrip() {
        let key = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let plaintext = b"Secret message for encryption";
        
        let ciphertext = encrypt_aes256_gcm(plaintext, &key).unwrap();
        let decrypted = decrypt_aes256_gcm(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_decryption_fails_with_wrong_key() {
        let key1 = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let key2 = hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        let plaintext = b"Test message";
        
        let ciphertext = encrypt_aes256_gcm(plaintext, &key1).unwrap();
        let result = decrypt_aes256_gcm(&ciphertext, &key2);
        
        assert!(result.is_err());
    }
}