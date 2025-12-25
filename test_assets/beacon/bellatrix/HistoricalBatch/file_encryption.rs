use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let ciphertext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes(&plaintext, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&plaintext, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)?;

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(&ciphertext, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(&ciphertext, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn encrypt_aes(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let nonce = Nonce::generate(&mut OsRng);
        cipher
            .encrypt(&nonce, plaintext)
            .map(|mut ct| {
                let mut result = nonce.to_vec();
                result.append(&mut ct);
                result
            })
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn decrypt_aes(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid ciphertext length".into()));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let nonce = Nonce::from_slice(&ciphertext[..12]);
        cipher
            .decrypt(nonce, &ciphertext[12..])
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn encrypt_chacha(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let nonce = chacha20poly1305::Nonce::generate(&mut OsRng);
        cipher
            .encrypt(&nonce, plaintext)
            .map(|mut ct| {
                let mut result = nonce.to_vec();
                result.append(&mut ct);
                result
            })
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn decrypt_chacha(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid ciphertext length".into()));
        }

        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let nonce = chacha20poly1305::Nonce::from_slice(&ciphertext[..12]);
        cipher
            .decrypt(nonce, &ciphertext[12..])
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        
        let plaintext = b"Hello, this is a secret message!";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(plaintext).unwrap();
        
        let mut encrypted_file = NamedTempFile::new().unwrap();
        let mut decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path(), &key)
            .unwrap();
        
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path(), &key)
            .unwrap();
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted_content);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let key = generate_random_key();
        
        let plaintext = b"Another secret message for testing";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(plaintext).unwrap();
        
        let mut encrypted_file = NamedTempFile::new().unwrap();
        let mut decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path(), &key)
            .unwrap();
        
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path(), &key)
            .unwrap();
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted_content);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_position: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_position: 0,
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_position];
            result.push(byte ^ key_byte);
            
            self.key_position = (self.key_position + 1) % self.key.len();
        }
        
        result
    }

    pub fn reset(&mut self) {
        self.key_position = 0;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data = cipher.encrypt(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let decrypted_data = cipher.decrypt(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let mut cipher = XorCipher::new(key);
        
        let original_data = b"Hello, World! This is a test message.";
        let encrypted = cipher.encrypt(original_data);
        
        cipher.reset();
        let decrypted = cipher.decrypt(&encrypted);
        
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let original_content = b"Sample file content for encryption test.";
        
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(original_content)?;
        let input_path = input_file.path();
        
        let mut encrypted_file = NamedTempFile::new()?;
        let encrypted_path = encrypted_file.path();
        
        encrypt_file(input_path, encrypted_path, key)?;
        
        let mut decrypted_file = NamedTempFile::new()?;
        let decrypted_path = decrypted_file.path();
        
        decrypt_file(encrypted_path, decrypted_path, key)?;
        
        let decrypted_content = fs::read(decrypted_path)?;
        assert_eq!(original_content, decrypted_content.as_slice());
        
        Ok(())
    }
}