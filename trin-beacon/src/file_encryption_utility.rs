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
}