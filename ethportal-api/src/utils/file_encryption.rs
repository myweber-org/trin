
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
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)?;
        
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
        let encrypted_data = &ciphertext[12..];
        
        let plaintext = self.cipher.decrypt(nonce, encrypted_data)?;
        Ok(plaintext)
    }
}

pub fn process_file_encryption(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    let encrypted = encryptor.encrypt(data)?;
    let decrypted = encryptor.decrypt(&encrypted)?;
    
    Ok((encrypted, decrypted))
}