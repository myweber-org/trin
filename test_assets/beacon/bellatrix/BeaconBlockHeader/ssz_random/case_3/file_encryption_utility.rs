
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

    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_position];
            result.push(byte ^ key_byte);
            
            self.key_position = (self.key_position + 1) % self.key.len();
        }
        
        result
    }

    pub fn reset(&mut self) {
        self.key_position = 0;
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
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let mut cipher = XorCipher::new(key);
        
        let original_data = b"Hello, World! This is a test message.";
        let encrypted = cipher.encrypt(original_data);
        
        cipher.reset();
        let decrypted = cipher.decrypt(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let key = "test_password_123";
        let original_content = b"Sample file content for encryption testing.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), key).unwrap();
        
        let encrypted_content = fs::read(encrypted_file.path()).unwrap();
        assert_ne!(original_content, encrypted_content.as_slice());
        
        decrypt_file(encrypted_file.path(), decrypted_file.path(), key).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}