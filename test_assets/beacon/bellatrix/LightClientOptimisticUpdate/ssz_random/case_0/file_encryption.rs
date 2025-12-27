
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs;
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str, salt: &[u8]) -> Self {
        let mut key = [0u8; 32];
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: key.len(),
        };
        
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key);
        
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        Self { cipher }
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let data = fs::read(input_path)
            .map_err(|e| format!("Failed to read input file: {}", e))?;
        
        let mut nonce = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = self.cipher
            .encrypt(Nonce::from_slice(&nonce), data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&ciphertext);
        
        fs::write(output_path, &output)
            .map_err(|e| format!("Failed to write output file: {}", e))?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let encrypted_data = fs::read(input_path)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Invalid encrypted data: too short".to_string());
        }
        
        let nonce = &encrypted_data[..NONCE_LENGTH];
        let ciphertext = &encrypted_data[NONCE_LENGTH..];
        
        let plaintext = self.cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, &plaintext)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
        
        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_LENGTH] {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let salt = generate_salt();
        let encryptor = FileEncryptor::new("test_password", &salt);
        
        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption should succeed");
        
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption should succeed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}