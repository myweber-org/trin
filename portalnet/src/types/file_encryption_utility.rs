use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    let params = Params {
        rounds: PBKDF2_ITERATIONS,
        output_length: key.len(),
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key);
    *Key::<Aes256Gcm>::from_slice(&key)
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    let nonce_obj = Nonce::from_slice(&nonce);
    
    cipher.encrypt(nonce_obj, data)
        .map(|ciphertext| EncryptionResult {
            ciphertext,
            salt,
            nonce,
        })
        .map_err(|e| format!("Encryption failed: {}", e))
}

pub fn decrypt_data(encrypted: &EncryptionResult, password: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&encrypted.nonce);
    
    cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let encrypted = encrypt_data(&buffer, password)?;
    
    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output.write_all(&encrypted.salt)
        .and_then(|_| output.write_all(&encrypted.nonce))
        .and_then(|_| output.write_all(&encrypted.ciphertext))
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut salt = [0u8; SALT_LENGTH];
    file.read_exact(&mut salt)
        .map_err(|e| format!("Failed to read salt: {}", e))?;
    
    let mut nonce = [0u8; NONCE_LENGTH];
    file.read_exact(&mut nonce)
        .map_err(|e| format!("Failed to read nonce: {}", e))?;
    
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read ciphertext: {}", e))?;
    
    let encrypted = EncryptionResult {
        ciphertext,
        salt,
        nonce,
    };
    
    let decrypted = decrypt_data(&encrypted, password)?;
    
    fs::write(output_path, &decrypted)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let data = b"Secret data to encrypt";
        let password = "strong_password_123";
        
        let encrypted = encrypt_data(data, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_operations() {
        let original_data = b"File content for encryption test";
        let password = "test_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let data = b"Sensitive information";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_data(data, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
}