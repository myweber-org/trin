
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
    InvalidFileSize,
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut file_data)?;

        let encrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes(&file_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&file_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let mut encrypted_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut encrypted_data)?;

        let decrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(&encrypted_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(&encrypted_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&decrypted_data)?;
        Ok(())
    }

    fn encrypt_aes(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError("AES-256-GCM requires 32-byte key".into()));
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);

        let encrypted = cipher.encrypt(Nonce::from_slice(&nonce), data)
            .map_err(|e| EncryptionError::CryptoError(format!("Encryption failed: {}", e)))?;

        let mut result = Vec::with_capacity(nonce.len() + encrypted.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&encrypted);
        Ok(result)
    }

    fn decrypt_aes(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError("AES-256-GCM requires 32-byte key".into()));
        }

        if data.len() < 12 {
            return Err(EncryptionError::InvalidFileSize);
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = &data[..12];
        let ciphertext = &data[12..];

        let decrypted = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| EncryptionError::CryptoError(format!("Decryption failed: {}", e)))?;

        Ok(decrypted)
    }

    fn encrypt_chacha(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError("ChaCha20Poly1305 requires 32-byte key".into()));
        }

        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);

        let encrypted = cipher.encrypt(ChaChaNonce::from_slice(&nonce), data)
            .map_err(|e| EncryptionError::CryptoError(format!("Encryption failed: {}", e)))?;

        let mut result = Vec::with_capacity(nonce.len() + encrypted.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&encrypted);
        Ok(result)
    }

    fn decrypt_chacha(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError("ChaCha20Poly1305 requires 32-byte key".into()));
        }

        if data.len() < 12 {
            return Err(EncryptionError::InvalidFileSize);
        }

        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = &data[..12];
        let ciphertext = &data[12..];

        let decrypted = cipher.decrypt(ChaChaNonce::from_slice(nonce), ciphertext)
            .map_err(|e| EncryptionError::CryptoError(format!("Decryption failed: {}", e)))?;

        Ok(decrypted)
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let test_data = b"Hello, this is a secret message!";
        let key = generate_random_key();
        
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        
        let encrypted = encryptor.encrypt_aes(test_data, &key).unwrap();
        let decrypted = encryptor.decrypt_aes(&encrypted, &key).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let test_data = b"Another secret message for testing";
        let key = generate_random_key();
        
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        
        let encrypted = encryptor.encrypt_chacha(test_data, &key).unwrap();
        let decrypted = encryptor.decrypt_chacha(&encrypted, &key).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_operations() {
        let key = generate_random_key();
        let test_content = b"File encryption test content";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_content).unwrap();
        
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        
        encryptor.encrypt_file(input_file.path(), output_file.path(), &key).unwrap();
        encryptor.decrypt_file(output_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct FileCrypto {
    cipher: Aes256Gcm,
}

impl FileCrypto {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| e.to_string())?;

        let mut output = File::create(output_path).map_err(|e| e.to_string())?;
        output.write_all(&nonce).map_err(|e| e.to_string())?;
        output.write_all(&ciphertext).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(|e| e.to_string())?;

        if data.len() < 12 {
            return Err("Invalid encrypted file format".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())?;

        let mut output = File::create(output_path).map_err(|e| e.to_string())?;
        output.write_all(&plaintext).map_err(|e| e.to_string())?;

        Ok(())
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_key();
        let crypto = FileCrypto::new(&key);

        let test_data = b"Secret data for encryption test";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        crypto
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();
        crypto
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}