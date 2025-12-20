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

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &str, output_path: &str, is_encrypt: bool) -> Result<(), String> {
        let input_path_obj = Path::new(input_path);
        let output_path_obj = Path::new(output_path);

        if !input_path_obj.exists() {
            return Err(format!("Input file does not exist: {}", input_path));
        }

        let mut input_file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let processed_data = self.xor_process(&buffer);

        let mut output_file = File::create(output_path_obj)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    fn xor_process(&self, data: &[u8]) -> Vec<u8> {
        let key_length = self.key.len();
        if key_length == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_length])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("secret_key");
        let original_data = b"Hello, this is a test message for XOR encryption!";
        
        let encrypted = cipher.xor_process(original_data);
        let decrypted = cipher.xor_process(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_empty_key() {
        let cipher = XORCipher::new("");
        let data = b"Test data";
        
        let processed = cipher.xor_process(data);
        assert_eq!(data.to_vec(), processed);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let cipher = XORCipher::new("my_secure_password");
        
        let original_content = b"Confidential information that needs protection.";
        
        let input_temp_file = NamedTempFile::new().unwrap();
        let encrypted_temp_file = NamedTempFile::new().unwrap();
        let decrypted_temp_file = NamedTempFile::new().unwrap();
        
        fs::write(input_temp_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(
            input_temp_file.path().to_str().unwrap(),
            encrypted_temp_file.path().to_str().unwrap()
        ).unwrap();
        
        cipher.decrypt_file(
            encrypted_temp_file.path().to_str().unwrap(),
            decrypted_temp_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_temp_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}