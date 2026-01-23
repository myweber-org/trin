use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub struct EncryptionManager {
    cipher: Aes256Gcm,
}

impl EncryptionManager {
    pub fn new() -> Self {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if data.len() < 12 {
            return Err("Invalid encrypted data length".into());
        }
        
        let (nonce_slice, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_slice);
        
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let manager = EncryptionManager::new();
        let original = b"Secret message for encryption test";
        
        let encrypted = manager.encrypt(original).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn test_tampered_data() {
        let manager = EncryptionManager::new();
        let original = b"Test data";
        
        let mut encrypted = manager.encrypt(original).unwrap();
        encrypted[20] ^= 0xFF;
        
        let result = manager.decrypt(&encrypted);
        assert!(result.is_err());
    }
}