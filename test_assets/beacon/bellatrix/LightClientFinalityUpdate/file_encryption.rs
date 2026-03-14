use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let data = fs::read(input_path)?;
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(nonce_bytes.as_slice());
    output.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output)
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; 32];
    let password_bytes = password.as_bytes();
    
    for (i, byte) in password_bytes.iter().enumerate() {
        key_bytes[i % 32] ^= byte;
    }
    
    *Key::<Aes256Gcm>::from_slice(&key_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret data for encryption test";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.as_slice(), decrypted_data.as_slice());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce as ChaChaNonce};
use rand::RngCore;
use std::error::Error;

#[derive(Debug)]
pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub algorithm: CipherAlgorithm,
}

pub fn encrypt_data(
    plaintext: &[u8],
    key: &[u8],
    algorithm: CipherAlgorithm,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match algorithm {
        CipherAlgorithm::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            
            let cipher = Aes256Gcm::new_from_slice(key)?;
            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
                algorithm: CipherAlgorithm::Aes256Gcm,
            })
        }
        
        CipherAlgorithm::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            
            let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = ChaChaNonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
                algorithm: CipherAlgorithm::ChaCha20Poly1305,
            })
        }
    }
}

pub fn decrypt_data(
    encrypted: &EncryptionResult,
    key: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    match encrypted.algorithm {
        CipherAlgorithm::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            
            let cipher = Aes256Gcm::new_from_slice(key)?;
            let nonce = Nonce::from_slice(&encrypted.nonce);
            cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
                .map_err(|e| e.into())
        }
        
        CipherAlgorithm::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            
            let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
            let nonce = ChaChaNonce::from_slice(&encrypted.nonce);
            cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
                .map_err(|e| e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aes_encryption_decryption() {
        let plaintext = b"Secret message for AES";
        let key = [0x42u8; 32];
        
        let encrypted = encrypt_data(
            plaintext,
            &key,
            CipherAlgorithm::Aes256Gcm
        ).unwrap();
        
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_chacha_encryption_decryption() {
        let plaintext = b"Secret message for ChaCha";
        let key = [0x24u8; 32];
        
        let encrypted = encrypt_data(
            plaintext,
            &key,
            CipherAlgorithm::ChaCha20Poly1305
        ).unwrap();
        
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}