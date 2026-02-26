
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

const IV_SIZE: usize = 16;
const KEY_SIZE: usize = 32;

pub struct FileEncryptor {
    key: [u8; KEY_SIZE],
}

impl FileEncryptor {
    pub fn new() -> Self {
        let mut key = [0u8; KEY_SIZE];
        rand::thread_rng().fill_bytes(&mut key);
        FileEncryptor { key }
    }

    pub fn with_key(key: [u8; KEY_SIZE]) -> Self {
        FileEncryptor { key }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let mut iv = [0u8; IV_SIZE];
        rand::thread_rng().fill_bytes(&mut iv);

        let cipher = Aes256Cbc::new_from_slices(&self.key, &iv)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;

        let ciphertext = cipher.encrypt_vec(&plaintext);

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&iv)
            .map_err(|e| format!("Failed to write IV: {}", e))?;
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        if encrypted_data.len() < IV_SIZE {
            return Err("File too short to contain IV".to_string());
        }

        let iv = &encrypted_data[..IV_SIZE];
        let ciphertext = &encrypted_data[IV_SIZE..];

        let cipher = Aes256Cbc::new_from_slices(&self.key, iv)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;

        let plaintext = cipher.decrypt_vec(ciphertext)
            .map_err(|e| format!("Failed to decrypt: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write plaintext: {}", e))?;

        Ok(())
    }

    pub fn get_key(&self) -> [u8; KEY_SIZE] {
        self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        
        let original_content = b"Hello, this is a secret message!";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let mut decrypted_content = Vec::new();
        let mut decrypted_file_handle = fs::File::open(decrypted_file.path()).unwrap();
        decrypted_file_handle.read_to_end(&mut decrypted_content).unwrap();

        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}