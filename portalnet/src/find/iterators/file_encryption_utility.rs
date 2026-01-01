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
        
        let mut buffer = [0u8; 4096];
        
        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            let processed_data = self.process_chunk(&buffer[..bytes_read]);
            dest_file.write_all(&processed_data)?;
        }
        
        self.reset_key_position();
        Ok(())
    }

    fn process_chunk(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .map(|&byte| {
                let key_byte = self.key[self.key_position];
                self.key_position = (self.key_position + 1) % self.key.len();
                byte ^ key_byte
            })
            .collect()
    }

    fn reset_key_position(&mut self) {
        self.key_position = 0;
    }
}

pub fn validate_key(key: &str) -> Result<(), &'static str> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty");
    }
    if key.len() < 8 {
        return Err("Encryption key must be at least 8 characters long");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let test_data = b"Hello, World! This is a test message.";
        let key = "SuperSecretKey123!";
        
        let mut cipher = XorCipher::new(key);
        let encrypted: Vec<u8> = cipher.process_chunk(test_data);
        
        cipher.reset_key_position();
        let decrypted: Vec<u8> = cipher.process_chunk(&encrypted);
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let original_content = b"Sample file content for encryption test";
        let key = "TestEncryptionKey456";
        
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let mut cipher = XorCipher::new(key);
        
        cipher.encrypt_file(source_file.path(), encrypted_file.path()).unwrap();
        cipher.reset_key_position();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();
        
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("ValidKey123").is_ok());
        assert!(validate_key("Short").is_err());
        assert!(validate_key("").is_err());
    }
}