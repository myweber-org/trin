use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<EncryptionResult, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes: [u8; 12] = OsRng.fill(&mut [0u8; 12])?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext)?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_encryption_roundtrip() {
        let key = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let plaintext = b"Secret message for encryption test";
        
        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted.ciphertext, &key, &encrypted.nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_invalid_decryption() {
        let key = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let wrong_key = hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        let plaintext = b"Test data";
        
        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let result = decrypt_data(&encrypted.ciphertext, &wrong_key, &encrypted.nonce);
        
        assert!(result.is_err());
    }
}