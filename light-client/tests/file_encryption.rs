
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
    InvalidKeyLength,
    InvalidNonceLength,
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

    pub fn generate_key(&self) -> Result<Vec<u8>, EncryptionError> {
        match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let mut key = [0u8; 32];
                OsRng.fill_bytes(&mut key);
                Ok(key.to_vec())
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let mut key = [0u8; 32];
                OsRng.fill_bytes(&mut key);
                Ok(key.to_vec())
            }
        }
    }

    pub fn generate_nonce(&self) -> Result<Vec<u8>, EncryptionError> {
        match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let mut nonce = [0u8; 12];
                OsRng.fill_bytes(&mut nonce);
                Ok(nonce.to_vec())
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let mut nonce = [0u8; 12];
                OsRng.fill_bytes(&mut nonce);
                Ok(nonce.to_vec())
            }
        }
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
        nonce: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut file_data)?;

        let ciphertext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                if key.len() != 32 {
                    return Err(EncryptionError::InvalidKeyLength);
                }
                if nonce.len() != 12 {
                    return Err(EncryptionError::InvalidNonceLength);
                }
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Nonce::from_slice(nonce);
                cipher
                    .encrypt(nonce, file_data.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                if key.len() != 32 {
                    return Err(EncryptionError::InvalidKeyLength);
                }
                if nonce.len() != 12 {
                    return Err(EncryptionError::InvalidNonceLength);
                }
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let nonce = ChaChaNonce::from_slice(nonce);
                cipher
                    .encrypt(nonce, file_data.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
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
        nonce: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut ciphertext = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut ciphertext)?;

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                if key.len() != 32 {
                    return Err(EncryptionError::InvalidKeyLength);
                }
                if nonce.len() != 12 {
                    return Err(EncryptionError::InvalidNonceLength);
                }
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Nonce::from_slice(nonce);
                cipher
                    .decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                if key.len() != 32 {
                    return Err(EncryptionError::InvalidKeyLength);
                }
                if nonce.len() != 12 {
                    return Err(EncryptionError::InvalidNonceLength);
                }
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
                let nonce = ChaChaNonce::from_slice(nonce);
                cipher
                    .decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = encryptor.generate_key().unwrap();
        let nonce = encryptor.generate_nonce().unwrap();

        let test_data = b"Test encryption data for AES-256-GCM";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path(), &key, &nonce)
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path(), &key, &nonce)
            .unwrap();

        let mut decrypted_data = Vec::new();
        let mut decrypted = fs::File::open(decrypted_file.path()).unwrap();
        decrypted.read_to_end(&mut decrypted_data).unwrap();

        assert_eq!(decrypted_data, test_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let key = encryptor.generate_key().unwrap();
        let nonce = encryptor.generate_nonce().unwrap();

        let test_data = b"Test encryption data for ChaCha20Poly1305";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path(), &key, &nonce)
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path(), &key, &nonce)
            .unwrap();

        let mut decrypted_data = Vec::new();
        let mut decrypted = fs::File::open(decrypted_file.path()).unwrap();
        decrypted.read_to_end(&mut decrypted_data).unwrap();

        assert_eq!(decrypted_data, test_data);
    }
}