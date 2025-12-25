use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use sha2::{Sha256, Digest};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub fn derive_key(password: &str, salt: &[u8]) -> Key<Aes256Gcm> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    *Key::<Aes256Gcm>::from_slice(&result)
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let salt: [u8; SALT_SIZE] = OsRng.gen();
    let key = derive_key(password, &salt);
    
    let cipher = Aes256Gcm::new(&key);
    let nonce: [u8; NONCE_SIZE] = OsRng.gen();
    
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output_file.write_all(&nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
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

    if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err("File too short to contain valid encrypted data".to_string());
    }

    let (salt_data, rest) = encrypted_data.split_at(SALT_SIZE);
    let (nonce_data, ciphertext) = rest.split_at(NONCE_SIZE);
    
    let salt: [u8; SALT_SIZE] = salt_data.try_into()
        .map_err(|_| "Invalid salt size")?;
    let nonce: [u8; NONCE_SIZE] = nonce_data.try_into()
        .map_err(|_| "Invalid nonce size")?;

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher.decrypt(Nonce::from_slice(&nonce), ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Test data for encryption and decryption";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let mut input = input_file.reopen().unwrap();
        input.write_all(original_content).unwrap();
        
        let password = "secure_password_123";
        
        encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption should succeed");
        
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption should succeed");
        
        let mut decrypted_content = Vec::new();
        let mut decrypted = decrypted_file.reopen().unwrap();
        decrypted.read_to_end(&mut decrypted_content).unwrap();
        
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}