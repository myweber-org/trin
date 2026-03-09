
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new_from_password(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let hash_bytes = password_hash.hash.unwrap().as_bytes();
        
        let key = Key::<Aes256Gcm>::from_slice(&hash_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(FileEncryptor { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;
        
        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted file format".into());
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;
        
        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }
}

pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let parsed_hash = PasswordHash::new(stored_hash)?;
    let argon2 = Argon2::default();
    
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn encrypt_string(text: &str, key: Option<u8>) -> Vec<u8> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    text.bytes()
        .map(|byte| byte ^ encryption_key)
        .collect()
}

pub fn decrypt_string(data: &[u8], key: Option<u8>) -> String {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    data.iter()
        .map(|byte| (byte ^ encryption_key) as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_string_encryption() {
        let original = "Hello, World!";
        let encrypted = encrypt_string(original, None);
        let decrypted = decrypt_string(&encrypted, None);
        
        assert_ne!(original.as_bytes(), &encrypted);
        assert_eq!(original, decrypted);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original_content = b"Secret data for encryption test";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), original_content)?;
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(0x42),
        )?;
        
        let encrypted_content = fs::read(output_file.path())?;
        assert_ne!(original_content, encrypted_content.as_slice());
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x42),
        )?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(original_content, decrypted_content.as_slice());
        
        Ok(())
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let key_bytes = password_hash.hash.ok_or("Failed to derive key")?.as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Nonce::from_slice(&generate_nonce());
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce"
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::from_password(password).unwrap();
        
        let original_content = b"Secret data that needs protection";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}