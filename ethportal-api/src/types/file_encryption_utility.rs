use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_index: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_index: 0,
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
            let key_byte = self.key[self.key_index];
            result.push(byte ^ key_byte);
            self.key_index = (self.key_index + 1) % self.key.len();
        }
        
        result
    }

    pub fn reset(&mut self) {
        self.key_index = 0;
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original_data = b"Hello, World! This is a test message.";
        
        let mut cipher = XorCipher::new(key);
        let encrypted = cipher.encrypt(original_data);
        
        cipher.reset();
        let decrypted = cipher.decrypt(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let test_data = b"Test file content for encryption";
        let key = "test_key_123";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        encrypt_file(input_file.path(), output_file.path(), key)?;
        decrypt_file(output_file.path(), decrypted_file.path(), key)?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(test_data.to_vec(), decrypted_content);
        
        Ok(())
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted data length".into());
        }

        let (nonce_slice, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_slice);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e).into())
    }
}

pub fn process_file_encryption() -> Result<(), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    let test_data = b"Confidential file content for encryption demonstration";

    println!("Original data length: {} bytes", test_data.len());

    let encrypted = encryptor.encrypt_data(test_data)?;
    println!("Encrypted data length: {} bytes", encrypted.len());

    let decrypted = encryptor.decrypt_data(&encrypted)?;
    println!("Decrypted data length: {} bytes", decrypted.len());

    assert_eq!(test_data.to_vec(), decrypted);
    println!("Encryption/decryption cycle completed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_cycle() {
        let encryptor = FileEncryptor::new();
        let data = b"Test payload for cryptographic validation";
        
        let encrypted = encryptor.encrypt_data(data).unwrap();
        let decrypted = encryptor.decrypt_data(&encrypted).unwrap();
        
        assert_eq!(data.to_vec(), decrypted);
        assert_ne!(data, encrypted.as_slice());
    }

    #[test]
    fn test_different_keys_produce_different_output() {
        let encryptor1 = FileEncryptor::new();
        let encryptor2 = FileEncryptor::new();
        let data = b"Same input, different keys";
        
        let encrypted1 = encryptor1.encrypt_data(data).unwrap();
        let encrypted2 = encryptor2.encrypt_data(data).unwrap();
        
        assert_ne!(encrypted1, encrypted2);
    }
}