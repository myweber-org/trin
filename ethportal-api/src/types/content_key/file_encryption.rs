
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

    pub fn encrypt_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_index];
            result.push(byte ^ key_byte);
            self.key_index = (self.key_index + 1) % self.key.len();
        }
        
        result
    }

    pub fn decrypt_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.encrypt_bytes(data)
    }

    pub fn encrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut buffer = Vec::new();
        source_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.encrypt_bytes(&buffer);
        
        let mut dest_file = fs::File::create(dest_path)?;
        dest_file.write_all(&encrypted_data)?;
        
        Ok(())
    }

    pub fn decrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.encrypt_file(source_path, dest_path)
    }
}

pub fn process_file_operation(
    operation: &str,
    input_file: &str,
    output_file: &str,
    key: &str,
) -> Result<(), String> {
    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);

    if !input_path.exists() {
        return Err(format!("Input file '{}' does not exist", input_file));
    }

    let mut cipher = XorCipher::new(key);

    match operation {
        "encrypt" => cipher
            .encrypt_file(input_path, output_path)
            .map_err(|e| e.to_string()),
        "decrypt" => cipher
            .decrypt_file(input_path, output_path)
            .map_err(|e| e.to_string()),
        _ => Err("Invalid operation. Use 'encrypt' or 'decrypt'".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original_data = b"Hello, World! This is a test message.";
        
        let mut cipher1 = XorCipher::new(key);
        let encrypted = cipher1.encrypt_bytes(original_data);
        
        let mut cipher2 = XorCipher::new(key);
        let decrypted = cipher2.decrypt_bytes(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let key = "test_key_123";
        let test_content = b"Sample file content for encryption testing.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_content).unwrap();
        
        let mut cipher = XorCipher::new(key);
        cipher
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();
        
        let mut cipher2 = XorCipher::new(key);
        cipher2
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }
}