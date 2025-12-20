
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::error::Error;

pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_file_data(plaintext: &[u8]) -> Result<EncryptedData, Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let ciphertext = cipher.encrypt(&nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptedData {
        ciphertext,
        nonce: nonce.to_vec(),
    })
}

pub fn decrypt_file_data(
    encrypted: &EncryptedData,
    key: &Key<Aes256Gcm>
) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted.nonce);
    
    let plaintext = cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes_gcm::KeyInit;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret file content that needs protection";
        
        let encrypted = encrypt_file_data(test_data).unwrap();
        let key = Aes256Gcm::generate_key(&mut OsRng);
        
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        let decrypted = cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
            .expect("Decryption should succeed with correct key");
        
        assert_eq!(decrypted, test_data);
    }
}