
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

    pub fn encrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        self.key_position = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let processed_data = self.process_chunk(&buffer[..bytes_read]);
            dest_file.write_all(&processed_data)?;
        }

        Ok(())
    }

    fn process_chunk(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_position];
            result.push(byte ^ key_byte);
            
            self.key_position = (self.key_position + 1) % self.key.len();
        }
        
        result
    }
}

pub fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty".to_string());
    }
    
    if key.len() < 8 {
        return Err("Encryption key should be at least 8 characters".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "strong_encryption_key_123!";
        let original_text = b"Secret data that needs protection";
        
        let mut cipher = XorCipher::new(key);
        let encrypted = cipher.process_chunk(original_text);
        
        cipher.key_position = 0;
        let decrypted = cipher.process_chunk(&encrypted);
        
        assert_eq!(original_text.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let key = "test_key_890";
        let test_data = b"Sample content for encryption test";
        
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(test_data).unwrap();
        
        let dest_file = NamedTempFile::new().unwrap();
        
        let mut cipher = XorCipher::new(key);
        cipher.encrypt_file(source_file.path(), dest_file.path()).unwrap();
        
        cipher.key_position = 0;
        let mut decrypted_cipher = XorCipher::new(key);
        let verification_file = NamedTempFile::new().unwrap();
        decrypted_cipher.decrypt_file(dest_file.path(), verification_file.path()).unwrap();
        
        let mut verified_data = Vec::new();
        fs::File::open(verification_file.path())
            .unwrap()
            .read_to_end(&mut verified_data)
            .unwrap();
        
        assert_eq!(test_data.to_vec(), verified_data);
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("valid_key_long_enough").is_ok());
    }
}