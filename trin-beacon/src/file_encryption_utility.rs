use std::fs::{File, read, write};
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        self.apply_xor(data)
    }

    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        self.apply_xor(data)
    }

    fn apply_xor(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> Result<(), String> {
    let cipher = XorCipher::new(key);
    
    let content = read(input_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let encrypted = cipher.encrypt(&content);
    
    write(output_path, &encrypted).map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> Result<(), String> {
    let cipher = XorCipher::new(key);
    
    let content = read(input_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let decrypted = cipher.decrypt(&content);
    
    write(output_path, &decrypted).map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XorCipher::new("secret_key");
        let original = b"Hello, World!";
        
        let encrypted = cipher.encrypt(original);
        let decrypted = cipher.decrypt(&encrypted);
        
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let key = "test_key_123";
        let original_content = b"Sample file content for encryption test";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(input_file.path(), output_file.path(), key).unwrap();
        decrypt_file(output_file.path(), decrypted_file.path(), key).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let (ciphertext, key, nonce) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let key = Aes256Gcm::generate_key(&mut OsRng);
                let cipher = Aes256Gcm::new(&key);
                let nonce = Nonce::from_slice(b"unique_nonce_");
                let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
                    .map_err(|e| format!("Encryption failed: {}", e))?;
                (ciphertext, key.to_vec(), nonce.to_vec())
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                let cipher = ChaCha20Poly1305::new(&key);
                let nonce = ChaChaNonce::from_slice(b"unique_nonce_");
                let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
                    .map_err(|e| format!("Encryption failed: {}", e))?;
                (ciphertext, key.to_vec(), nonce.to_vec())
            }
        };

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        let key_file_path = output_path.with_extension("key");
        let mut key_file = fs::File::create(&key_file_path)
            .map_err(|e| format!("Failed to create key file: {}", e))?;
        
        key_file.write_all(&key)
            .map_err(|e| format!("Failed to write key: {}", e))?;
        key_file.write_all(&nonce)
            .map_err(|e| format!("Failed to write nonce: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, key_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut encrypted_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut ciphertext = Vec::new();
        encrypted_file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read ciphertext: {}", e))?;

        let mut key_file = fs::File::open(key_path)
            .map_err(|e| format!("Failed to open key file: {}", e))?;
        
        let mut key_nonce = Vec::new();
        key_file.read_to_end(&mut key_nonce)
            .map_err(|e| format!("Failed to read key/nonce: {}", e))?;

        let (key_bytes, nonce_bytes) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                if key_nonce.len() != 32 + 12 {
                    return Err("Invalid key/nonce length for AES-256-GCM".to_string());
                }
                (&key_nonce[0..32], &key_nonce[32..])
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                if key_nonce.len() != 32 + 12 {
                    return Err("Invalid key/nonce length for ChaCha20Poly1305".to_string());
                }
                (&key_nonce[0..32], &key_nonce[32..])
            }
        };

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let key = Key::<Aes256Gcm>::from_slice(key_bytes);
                let cipher = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(nonce_bytes);
                cipher.decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| format!("Decryption failed: {}", e))?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::from_slice(key_bytes);
                let cipher = ChaCha20Poly1305::new(key);
                let nonce = ChaChaNonce::from_slice(nonce_bytes);
                cipher.decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| format!("Decryption failed: {}", e))?
            }
        };

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write plaintext: {}", e))?;

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
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let key_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        let test_data = b"Test encryption data";
        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(input_file.path(), output_file.path()).unwrap();
        encryptor.decrypt_file(
            output_file.path(),
            &output_file.path().with_extension("key"),
            decrypted_file.path()
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        let test_data = b"Test ChaCha20-Poly1305 encryption";
        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(input_file.path(), output_file.path()).unwrap();
        encryptor.decrypt_file(
            output_file.path(),
            &output_file.path().with_extension("key"),
            decrypted_file.path()
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}