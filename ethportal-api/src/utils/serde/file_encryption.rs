
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, String> {
        let mut key_bytes = [0u8; 32];
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: 32,
        };
        
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key_bytes)
            .map_err(|e| format!("Key derivation failed: {}", e))?;
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        Ok(Self { key: *key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_LENGTH));
        
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(nonce.as_slice())
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err("File too short to contain nonce".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(&self.key);
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        let mut output = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&plaintext)
            .map_err(|e| format!("Failed to write plaintext: {}", e))?;
        
        Ok(())
    }
}

fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn generate_salt() -> Vec<u8> {
    generate_random_bytes(SALT_LENGTH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let salt = generate_salt();
        let password = "secure_password_123";
        let encryptor = FileEncryptor::from_password(password, &salt).unwrap();
        
        let original_content = b"Secret data that needs protection";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_content = Vec::new();
        File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();
        
        assert_eq!(decrypted_content, original_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output = key.to_vec();
    output.extend_from_slice(nonce);
    output.extend_from_slice(&encrypted_data);
    
    fs::write(output_path, output)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_content = fs::read(input_path)?;
    
    if encrypted_content.len() < 48 {
        return Err("Invalid encrypted file format".into());
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&encrypted_content[0..32]);
    let nonce = Nonce::from_slice(&encrypted_content[32..44]);
    let ciphertext = &encrypted_content[44..];
    
    let cipher = Aes256Gcm::new(key);
    let decrypted_data = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_encryption_roundtrip() {
        let original_content = b"Test data for encryption roundtrip";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encrypt_file(input_file.path().to_str().unwrap(), 
                    encrypted_file.path().to_str().unwrap()).unwrap();
        
        decrypt_file(encrypted_file.path().to_str().unwrap(),
                    decrypted_file.path().to_str().unwrap()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}