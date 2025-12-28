use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &str, dest_path: &str) -> Result<(), String> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &str, dest_path: &str) -> Result<(), String> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &str, dest_path: &str, _is_encrypt: bool) -> Result<(), String> {
        let source = Path::new(source_path);
        let dest = Path::new(dest_path);

        if !source.exists() {
            return Err(format!("Source file not found: {}", source_path));
        }

        let mut source_file = File::open(source).map_err(|e| e.to_string())?;
        let mut dest_file = File::create(dest).map_err(|e| e.to_string())?;

        let mut buffer = Vec::new();
        source_file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

        let processed_data = self.xor_transform(&buffer);
        dest_file.write_all(&processed_data).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
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

pub fn calculate_file_hash(path: &str) -> Result<String, String> {
    let content = fs::read(path).map_err(|e| e.to_string())?;
    let hash = sha256::digest(&content);
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let cipher = XORCipher::new("secret_key");
        let test_data = b"Hello, World!";
        let encrypted = cipher.xor_transform(test_data);
        let decrypted = cipher.xor_transform(&encrypted);
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let cipher = XORCipher::new("test_key");
        
        let mut source_file = NamedTempFile::new().unwrap();
        let dest_file = NamedTempFile::new().unwrap();
        
        let test_content = b"Test file content for encryption";
        source_file.write_all(test_content).unwrap();
        
        let source_path = source_file.path().to_str().unwrap();
        let dest_path = dest_file.path().to_str().unwrap();
        
        cipher.encrypt_file(source_path, dest_path).unwrap();
        cipher.decrypt_file(dest_path, source_path).unwrap();
        
        let mut final_content = Vec::new();
        source_file.read_to_end(&mut final_content).unwrap();
        
        assert_eq!(test_content.to_vec(), final_content);
    }
}