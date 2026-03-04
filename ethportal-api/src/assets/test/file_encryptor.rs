
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

        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let processed_data = self.xor_cipher(&buffer, is_encrypt);

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    fn xor_cipher(&self, data: &[u8], is_encrypt: bool) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = self.key[i % self.key.len()];
            result.push(byte ^ key_byte);
        }

        if !is_encrypt {
            if let Some(pos) = result.iter().rposition(|&x| x != 0) {
                result.truncate(pos + 1);
            }
        }

        result
    }

    pub fn generate_key(length: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..length).map(|_| rng.gen::<u8>()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, this is a secret message!";
        let key = "super-secret-key";
        let encryptor = FileEncryptor::new(key);

        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();

        fs::write(input_temp.path(), original_text).unwrap();

        encryptor.encrypt_file(input_temp.path(), encrypted_temp.path()).unwrap();
        encryptor.decrypt_file(encrypted_temp.path(), decrypted_temp.path()).unwrap();

        let decrypted_data = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(decrypted_data, original_text);
    }

    #[test]
    fn test_empty_key() {
        let encryptor = FileEncryptor::new("");
        let temp_file = NamedTempFile::new().unwrap();
        
        let result = encryptor.encrypt_file(temp_file.path(), temp_file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }
}