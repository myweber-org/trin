
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &[u8]) -> Self {
        let salt: [u8; SALT_LENGTH] = OsRng.gen();
        let mut key = [0u8; 32];
        
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: key.len(),
        };
        
        pbkdf2_hmac::<Sha256>(password, &salt, params.rounds, &mut key);
        
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        Self { cipher }
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?
            .read_to_end(&mut file_data)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_LENGTH]>());
        
        let ciphertext = self.cipher
            .encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(nonce.as_slice())
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?
            .read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read encrypted data: {}", e))?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, plaintext)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = b"secure_password_123";
        let encryptor = FileEncryptor::new(password);
        
        let test_data = b"Hello, this is a secret message!";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption should succeed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption should succeed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use sha2::{Sha256, Digest};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let key_hash = hasher.finalize();
        
        let key = Key::<Aes256Gcm>::from_slice(&key_hash);
        let cipher = Aes256Gcm::new(key);
        
        FileEncryptor { cipher }
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file_content = Vec::new();
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        file.read_to_end(&mut file_content)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let encrypted_data = self.cipher.encrypt(nonce, file_content.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&nonce_bytes)
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output.write_all(&encrypted_data)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut encrypted_data = Vec::new();
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let decrypted_data = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&decrypted_data)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new("secure_password");
        let test_data = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption should succeed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption should succeed");
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
}