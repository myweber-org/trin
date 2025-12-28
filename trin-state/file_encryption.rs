
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".into());
        }
        
        let (nonce_slice, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_slice);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output = File::create(output_path)?;
        output.write_all(&plaintext)?;
        
        Ok(())
    }
    
    pub fn encrypt_directory(&self, dir_path: &Path, output_dir: &Path) -> Result<usize, Box<dyn std::error::Error>> {
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }
        
        let mut encrypted_count = 0;
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let output_path = output_dir.join(path.file_name().unwrap());
                self.encrypt_file(&path, &output_path)?;
                encrypted_count += 1;
            }
        }
        
        Ok(encrypted_count)
    }
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encrypt_decrypt() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();
        
        let original_content = b"Secret data that needs protection";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        encryptor.encrypt_file(temp_file.path(), encrypted_file.path()).unwrap();
        
        let decrypted_file = NamedTempFile::new().unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_content = Vec::new();
        File::open(decrypted_file.path()).unwrap()
            .read_to_end(&mut decrypted_content).unwrap();
        
        assert_eq!(decrypted_content, original_content);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let encryptor1 = FileEncryptor::new("password1").unwrap();
        let encryptor2 = FileEncryptor::new("password2").unwrap();
        
        let original_content = b"Test data";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        encryptor1.encrypt_file(temp_file.path(), encrypted_file.path()).unwrap();
        
        let decrypted_file = NamedTempFile::new().unwrap();
        let result = encryptor2.decrypt_file(encrypted_file.path(), decrypted_file.path());
        
        assert!(result.is_err());
    }
}