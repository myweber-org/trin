use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::RngCore;
use sha2::{Sha256, Digest};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_LENGTH: usize = 32;

pub struct FileCrypto;

impl FileCrypto {
    pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let mut salt = [0u8; SALT_LENGTH];
        let mut iv = [0u8; IV_LENGTH];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);

        let key = Self::derive_key(password, &salt);
        
        let cipher = Aes256Cbc::new_from_slices(&key, &iv)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;
        
        let ciphertext = cipher.encrypt_vec(&plaintext);
        
        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&salt)
            .map_err(|e| format!("Failed to write salt: {}", e))?;
        output_file.write_all(&iv)
            .map_err(|e| format!("Failed to write IV: {}", e))?;
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        if encrypted_data.len() < SALT_LENGTH + IV_LENGTH {
            return Err("File too short to contain salt and IV".to_string());
        }

        let salt = &encrypted_data[0..SALT_LENGTH];
        let iv = &encrypted_data[SALT_LENGTH..SALT_LENGTH + IV_LENGTH];
        let ciphertext = &encrypted_data[SALT_LENGTH + IV_LENGTH..];

        let key = Self::derive_key(password, salt);
        
        let cipher = Aes256Cbc::new_from_slices(&key, iv)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;
        
        let plaintext = cipher.decrypt_vec(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write plaintext: {}", e))?;

        Ok(())
    }

    fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        
        let mut key = [0u8; KEY_LENGTH];
        key.copy_from_slice(&hasher.finalize());
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        let password = "strong_password_123";
        
        FileCrypto::encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");
        
        FileCrypto::decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption should succeed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        FileCrypto::encrypt_file(input_file.path(), encrypted_file.path(), "correct_password")
            .expect("Encryption should succeed");
        
        let result = FileCrypto::decrypt_file(encrypted_file.path(), decrypted_file.path(), "wrong_password");
        assert!(result.is_err());
    }
}