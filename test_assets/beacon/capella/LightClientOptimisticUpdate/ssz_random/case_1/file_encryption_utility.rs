use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use sha2::{Sha256, Digest};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const SALT: &[u8] = b"static_salt_for_kdf";

pub fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(SALT);
    let result = hasher.finalize();
    *Key::<Aes256Gcm>::from_slice(&result)
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;
    
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())?;
    
    let mut output = File::create(output_path)?;
    output.write_all(&nonce)?;
    output.write_all(&ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut file = File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    let nonce_size = 12;
    if data.len() < nonce_size {
        return Err("File too short".into());
    }
    
    let (nonce_bytes, ciphertext) = data.split_at(nonce_size);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    
    let mut output = File::create(output_path)?;
    output.write_all(&plaintext)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret data for encryption test";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}