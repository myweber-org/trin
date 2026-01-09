use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, key: Option<u8>, encrypt: bool) -> io::Result<()> {
    let dir_entries = fs::read_dir(dir_path)?;
    let encryption_key = key.unwrap_or(DEFAULT_KEY);

    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let input_path = path.to_str().unwrap();
            let output_path = format!("{}.processed", input_path);
            
            if encrypt {
                encrypt_file(input_path, &output_path, Some(encryption_key))?;
            } else {
                decrypt_file(input_path, &output_path, Some(encryption_key))?;
            }
            
            println!("Processed: {}", input_path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data to protect";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(original_content).unwrap();
        
        let input_path = temp_file.path().to_str().unwrap();
        let encrypted_path = format!("{}.enc", input_path);
        let decrypted_path = format!("{}.dec", input_path);
        
        encrypt_file(input_path, &encrypted_path, Some(0xAA)).unwrap();
        decrypt_file(&encrypted_path, &decrypted_path, Some(0xAA)).unwrap();
        
        let decrypted_content = fs::read(decrypted_path).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
    InvalidKeyLength,
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Algorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: Algorithm,
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(algorithm: Algorithm, key: &[u8]) -> Result<Self, EncryptionError> {
        match algorithm {
            Algorithm::Aes256Gcm if key.len() != 32 => return Err(EncryptionError::InvalidKeyLength),
            Algorithm::ChaCha20Poly1305 if key.len() != 32 => return Err(EncryptionError::InvalidKeyLength),
            _ => {}
        }
        
        Ok(Self {
            algorithm,
            key: key.to_vec(),
        })
    }
    
    pub fn generate_key(algorithm: Algorithm) -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;
        
        let ciphertext = match self.algorithm {
            Algorithm::Aes256Gcm => self.encrypt_aes(&file_data)?,
            Algorithm::ChaCha20Poly1305 => self.encrypt_chacha(&file_data)?,
        };
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut ciphertext = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut ciphertext)?;
        
        let plaintext = match self.algorithm {
            Algorithm::Aes256Gcm => self.decrypt_aes(&ciphertext)?,
            Algorithm::ChaCha20Poly1305 => self.decrypt_chacha(&ciphertext)?,
        };
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
    
    fn encrypt_aes(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let mut ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.append(&mut ciphertext);
        
        Ok(result)
    }
    
    fn decrypt_aes(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 12 {
            return Err(EncryptionError::CryptoError("Ciphertext too short".to_string()));
        }
        
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = Nonce::from_slice(&ciphertext[0..12]);
        let encrypted_data = &ciphertext[12..];
        
        cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }
    
    fn encrypt_chacha(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = ChaChaKey::from_slice(&self.key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);
        
        let mut ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.append(&mut ciphertext);
        
        Ok(result)
    }
    
    fn decrypt_chacha(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 12 {
            return Err(EncryptionError::CryptoError("Ciphertext too short".to_string()));
        }
        
        let key = ChaChaKey::from_slice(&self.key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let nonce = ChaChaNonce::from_slice(&ciphertext[0..12]);
        let encrypted_data = &ciphertext[12..];
        
        cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_aes_encryption_decryption() {
        let key = FileEncryptor::generate_key(Algorithm::Aes256Gcm);
        let encryptor = FileEncryptor::new(Algorithm::Aes256Gcm, &key).unwrap();
        
        let test_data = b"Hello, this is a test message for AES encryption!";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_data = Vec::new();
        fs::File::open(decrypted_file.path()).unwrap().read_to_end(&mut decrypted_data).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_chacha_encryption_decryption() {
        let key = FileEncryptor::generate_key(Algorithm::ChaCha20Poly1305);
        let encryptor = FileEncryptor::new(Algorithm::ChaCha20Poly1305, &key).unwrap();
        
        let test_data = b"Hello, this is a test message for ChaCha20Poly1305 encryption!";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_data = Vec::new();
        fs::File::open(decrypted_file.path()).unwrap().read_to_end(&mut decrypted_data).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}