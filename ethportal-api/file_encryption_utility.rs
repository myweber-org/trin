use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: &[u8] = b"secret-encryption-key-2024";

pub struct FileCipher {
    key: Vec<u8>,
}

impl FileCipher {
    pub fn new(key: Option<&[u8]>) -> Self {
        let key = key.unwrap_or(DEFAULT_KEY).to_vec();
        FileCipher { key }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path, is_encrypt: bool) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0u8; 4096];
        let key_len = self.key.len();
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % key_len;
            }

            dest_file.write_all(&buffer[..bytes_read])?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_string(&self, input: &str) -> Vec<u8> {
        self.process_bytes(input.as_bytes())
    }

    pub fn decrypt_string(&self, encrypted: &[u8]) -> String {
        let decrypted = self.process_bytes(encrypted);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn process_bytes(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let cipher = FileCipher::new(Some(b"test-key"));
        let original = "Hello, World! This is a test message.";
        
        let encrypted = cipher.encrypt_string(original);
        let decrypted = cipher.decrypt_string(&encrypted);
        
        assert_ne!(encrypted, original.as_bytes());
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_file_operations() -> io::Result<()> {
        let cipher = FileCipher::new(Some(b"file-test-key"));
        let content = b"Sample file content for encryption test";
        
        let source_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(source_file.path(), content)?;
        
        cipher.encrypt_file(source_file.path(), encrypted_file.path())?;
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path())?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted_content, content);
        
        Ok(())
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let nonce = generate_nonce();
    
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut input_file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err("File too short to contain nonce".to_string());
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;
    
    Ok(())
}

fn derive_key(password: &str) -> aes_gcm::Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    let password_bytes = password.as_bytes();
    
    for i in 0..key.len() {
        key[i] = password_bytes[i % password_bytes.len()].wrapping_add(i as u8);
    }
    
    aes_gcm::Key::<Aes256Gcm>::from_slice(&key).clone()
}

fn generate_nonce() -> Nonce {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    Nonce::from_slice(&nonce).clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let original_content = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let result = decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            wrong_password
        );
        
        assert!(result.is_err());
    }
}