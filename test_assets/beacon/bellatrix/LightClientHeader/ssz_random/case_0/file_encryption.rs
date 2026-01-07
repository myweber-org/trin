
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::from_slice(b"unique_nonce_");
        let ciphertext = self.cipher.encrypt(nonce, plaintext)?;
        Ok(ciphertext)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::from_slice(b"unique_nonce_");
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}

pub fn process_file_data(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    let encrypted = encryptor.encrypt(data)?;
    let decrypted = encryptor.decrypt(&encrypted)?;
    
    if data != decrypted.as_slice() {
        return Err("Decryption failed: data mismatch".into());
    }
    
    Ok((encrypted, decrypted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Confidential file content";
        let result = process_file_data(test_data);
        
        assert!(result.is_ok());
        let (encrypted, decrypted) = result.unwrap();
        
        assert_ne!(encrypted, test_data);
        assert_eq!(decrypted, test_data);
    }
}