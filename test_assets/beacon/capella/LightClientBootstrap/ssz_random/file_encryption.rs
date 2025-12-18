use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(key.into());
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

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if ciphertext.len() < 12 {
            return Err("Invalid ciphertext length".into());
        }
        
        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let plaintext = self
            .cipher
            .decrypt(nonce, &ciphertext[12..])
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        Ok(plaintext)
    }
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let encryptor = FileEncryptor::new(&key);
        let message = b"Secret data to protect";
        
        let encrypted = encryptor.encrypt(message).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        
        assert_eq!(message.to_vec(), decrypted);
    }

    #[test]
    fn test_tampered_ciphertext() {
        let key = generate_key();
        let encryptor = FileEncryptor::new(&key);
        let message = b"Test message";
        
        let mut encrypted = encryptor.encrypt(message).unwrap();
        encrypted[20] ^= 0xFF;
        
        assert!(encryptor.decrypt(&encrypted).is_err());
    }
}