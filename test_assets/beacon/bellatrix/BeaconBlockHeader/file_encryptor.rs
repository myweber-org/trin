
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(key: &str) -> Self {
        FileEncryptor {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path, is_encrypt: bool) -> Result<(), String> {
        if self.key.is_empty() {
            return Err("Encryption key cannot be empty".to_string());
        }

        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        let mut buffer = [0u8; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = input_file.read(&mut buffer)
                .map_err(|e| format!("Failed to read from input file: {}", e))?;

            if bytes_read == 0 {
                break;
            }

            let processed_chunk: Vec<u8> = buffer[..bytes_read]
                .iter()
                .map(|&byte| {
                    let result = byte ^ self.key[key_index];
                    key_index = (key_index + 1) % self.key.len();
                    result
                })
                .collect();

            output_file.write_all(&processed_chunk)
                .map_err(|e| format!("Failed to write to output file: {}", e))?;
        }

        Ok(())
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        let mut result = Vec::with_capacity(text.len());
        let mut key_index = 0;

        for byte in text.bytes() {
            result.push(byte ^ self.key[key_index]);
            key_index = (key_index + 1) % self.key.len();
        }

        result
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let mut result = String::with_capacity(data.len());
        let mut key_index = 0;

        for &byte in data {
            result.push((byte ^ self.key[key_index]) as char);
            key_index = (key_index + 1) % self.key.len();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption_decryption() {
        let encryptor = FileEncryptor::new("secret_key");
        let original_text = "Hello, World!";
        
        let encrypted = encryptor.encrypt_string(original_text);
        let decrypted = encryptor.decrypt_string(&encrypted);
        
        assert_eq!(original_text, decrypted);
        assert_ne!(original_text.as_bytes(), encrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let encryptor = FileEncryptor::new("test_key_123");
        let test_content = "This is a test file content for encryption.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read_to_string(decrypted_file.path()).unwrap();
        assert_eq!(test_content, decrypted_content);
    }

    #[test]
    fn test_empty_key() {
        let encryptor = FileEncryptor::new("");
        let test_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        let result = encryptor.encrypt_file(test_file.path(), output_file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }
}