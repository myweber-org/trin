
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
        self.key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[self.key_index];
                self.key_index = (self.key_index + 1) % self.key.len();
            }

            dest_file.write_all(&buffer[..bytes_read])?;
        }

        dest_file.flush()?;
        Ok(())
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
        let key = "strong_secret_key_123!";
        let original_data = b"Hello, this is a secret message!";
        
        let mut cipher = XorCipher::new(key);
        let mut encrypted = original_data.to_vec();
        
        for i in 0..encrypted.len() {
            encrypted[i] ^= cipher.key[i % cipher.key.len()];
        }
        
        let mut decrypted = encrypted.clone();
        cipher.key_index = 0;
        for i in 0..decrypted.len() {
            decrypted[i] ^= cipher.key[i % cipher.key.len()];
        }
        
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() {
        let key = "test_encryption_key";
        let mut cipher = XorCipher::new(key);
        
        let original_content = "Sensitive data that needs protection";
        let source_file = NamedTempFile::new().unwrap();
        let dest_file = NamedTempFile::new().unwrap();
        
        fs::write(source_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(source_file.path(), dest_file.path()).unwrap();
        
        let encrypted_content = fs::read(dest_file.path()).unwrap();
        assert_ne!(original_content.as_bytes(), encrypted_content.as_slice());
        
        let mut cipher2 = XorCipher::new(key);
        let decrypted_file = NamedTempFile::new().unwrap();
        cipher2.decrypt_file(dest_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read_to_string(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content);
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("valid_long_key").is_ok());
        assert!(validate_key("short").is_err());
        assert!(validate_key("").is_err());
    }
}