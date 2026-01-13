
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: CipherAlgorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: CipherAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let data = self.read_file(input_path)?;
        let (ciphertext, nonce) = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => self.aes_encrypt(&data)?,
            CipherAlgorithm::ChaCha20Poly1305 => self.chacha_encrypt(&data)?,
        };
        
        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&nonce)
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let (nonce, ciphertext) = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                if buffer.len() < 12 {
                    return Err("File too short for AES-GCM".to_string());
                }
                (&buffer[0..12], &buffer[12..])
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                if buffer.len() < 12 {
                    return Err("File too short for ChaCha20-Poly1305".to_string());
                }
                (&buffer[0..12], &buffer[12..])
            }
        };
        
        let plaintext = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => self.aes_decrypt(ciphertext, nonce)?,
            CipherAlgorithm::ChaCha20Poly1305 => self.chacha_decrypt(ciphertext, nonce)?,
        };
        
        fs::write(output_path, plaintext)
            .map_err(|e| format!("Failed to write output file: {}", e))?;
        
        Ok(())
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        fs::read(path).map_err(|e| format!("Failed to read file: {}", e))
    }

    fn aes_encrypt(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::generate(&mut OsRng);
        
        cipher.encrypt(&nonce, data)
            .map(|ciphertext| (ciphertext, nonce.to_vec()))
            .map_err(|e| format!("AES encryption failed: {}", e))
    }

    fn aes_decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("AES decryption failed: {}", e))
    }

    fn chacha_encrypt(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
        let key = ChaChaKey::generate(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = ChaChaNonce::generate(&mut OsRng);
        
        cipher.encrypt(&nonce, data)
            .map(|ciphertext| (ciphertext, nonce.to_vec()))
            .map_err(|e| format!("ChaCha20 encryption failed: {}", e))
    }

    fn chacha_decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
        let key = ChaChaKey::generate(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = ChaChaNonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("ChaCha20 decryption failed: {}", e))
    }
}

pub fn validate_file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_roundtrip() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let test_data = b"Test encryption data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }

    #[test]
    fn test_chacha_encryption_roundtrip() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::ChaCha20Poly1305);
        let test_data = b"Another test for ChaCha";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
}