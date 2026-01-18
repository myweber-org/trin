use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8; 32]) -> Result<(), Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    
    let mut input_file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;
    
    let nonce = Nonce::generate(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())?;
    
    let mut output_file = File::create(output_path)?;
    output_file.write_all(nonce.as_slice())?;
    output_file.write_all(&ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8; 32]) -> Result<(), Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    
    let mut input_file = File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err("Invalid encrypted file format".into());
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&plaintext)?;
    
    Ok(())
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let test_data = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.as_slice(), decrypted_data);
    }
}