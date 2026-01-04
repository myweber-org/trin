use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey};
use rand::RngCore;
use std::error::Error;

#[derive(Debug)]
pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: CipherAlgorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: CipherAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.algorithm {
            CipherAlgorithm::Aes256Gcm => self.encrypt_aes(plaintext, key),
            CipherAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(plaintext, key),
        }
    }

    pub fn decrypt(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.algorithm {
            CipherAlgorithm::Aes256Gcm => self.decrypt_aes(ciphertext, key),
            CipherAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(ciphertext, key),
        }
    }

    fn encrypt_aes(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let cipher = Aes256Gcm::new_from_slice(key)?;
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), plaintext)?;
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    fn decrypt_aes(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if ciphertext.len() < 12 {
            return Err("Invalid ciphertext length".into());
        }
        
        let cipher = Aes256Gcm::new_from_slice(key)?;
        let nonce = &ciphertext[0..12];
        let encrypted_data = &ciphertext[12..];
        
        let plaintext = cipher.decrypt(Nonce::from_slice(nonce), encrypted_data)?;
        Ok(plaintext)
    }

    fn encrypt_chacha(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = cipher.encrypt(&nonce.into(), plaintext)?;
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    fn decrypt_chacha(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if ciphertext.len() < 12 {
            return Err("Invalid ciphertext length".into());
        }
        
        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = &ciphertext[0..12];
        let encrypted_data = &ciphertext[12..];
        
        let plaintext = cipher.decrypt(nonce.into(), encrypted_data)?;
        Ok(plaintext)
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        let plaintext = b"Test encryption data";
        
        let ciphertext = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::ChaCha20Poly1305);
        let key = generate_random_key();
        let plaintext = b"Test encryption data";
        
        let ciphertext = encryptor.encrypt(plaintext, &key).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}