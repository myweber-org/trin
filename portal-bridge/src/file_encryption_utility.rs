
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        FileEncryptor { cipher }
    }

    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt_data(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if ciphertext.len() < 12 {
            return Err("Invalid ciphertext length".into());
        }

        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let encrypted_data = &ciphertext[12..];

        self.cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|e| format!("Decryption failed: {}", e).into())
    }
}

pub fn process_file_encryption(input: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    let encrypted = encryptor.encrypt_data(input)?;
    let decrypted = encryptor.decrypt_data(&encrypted)?;

    if input != decrypted.as_slice() {
        return Err("Encryption/decryption mismatch".into());
    }

    Ok((encrypted, decrypted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret file content that needs protection";
        let result = process_file_encryption(test_data);
        
        assert!(result.is_ok());
        let (encrypted, decrypted) = result.unwrap();
        
        assert_ne!(encrypted, test_data);
        assert_eq!(decrypted, test_data);
    }

    #[test]
    fn test_empty_data() {
        let test_data = b"";
        let result = process_file_encryption(test_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_large_data() {
        let test_data = vec![0x42; 1024 * 1024];
        let result = process_file_encryption(&test_data);
        assert!(result.is_ok());
    }
}