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