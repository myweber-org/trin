use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(Key::<Aes256Gcm>::from_slice(&key).clone())
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce,
    })
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
    salt: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let result = encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        let decrypted = decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
            &result.salt,
            &result.nonce,
        ).unwrap();
        
        assert_eq!(decrypted, test_data);
    }
}