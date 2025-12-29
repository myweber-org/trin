use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn encrypt_data(key: &[u8; 32], plaintext: &[u8]) -> io::Result<EncryptionResult> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_data(key: &[u8; 32], nonce: &[u8; NONCE_SIZE], ciphertext: &[u8]) -> io::Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn encrypt_file(key: &[u8; 32], input_path: &Path, output_path: &Path) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let result = encrypt_data(key, &buffer)?;
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&result.nonce)?;
    output_file.write_all(&result.ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(key: &[u8; 32], input_path: &Path, output_path: &Path) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    if buffer.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }
    
    let nonce: [u8; NONCE_SIZE] = buffer[..NONCE_SIZE].try_into().unwrap();
    let ciphertext = &buffer[NONCE_SIZE..];
    
    let plaintext = decrypt_data(key, &nonce, ciphertext)?;
    
    fs::write(output_path, plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let test_data = b"Secret data that needs protection";
        
        let encrypted = encrypt_data(&key, test_data).unwrap();
        let decrypted = decrypt_data(&key, &encrypted.nonce, &encrypted.ciphertext).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let key = generate_key();
        let test_content = b"File content to encrypt";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_content).unwrap();
        
        encrypt_file(&key, input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(&key, encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }
}