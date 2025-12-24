
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs;
use std::io::{self, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let params = Params {
        rounds: PBKDF2_ITERATIONS,
        output_length: 32,
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key);
    key
}

pub fn encrypt_file_data(data: &[u8], password: &str) -> io::Result<EncryptionResult> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let key_material = derive_key(password, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
    
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    let nonce_obj = Nonce::from_slice(&nonce);
    
    let ciphertext = cipher.encrypt(nonce_obj, data)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce,
    })
}

pub fn decrypt_file_data(encrypted: &EncryptionResult, password: &str) -> io::Result<Vec<u8>> {
    let key_material = derive_key(password, &encrypted.salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
    
    let nonce_obj = Nonce::from_slice(&encrypted.nonce);
    
    cipher.decrypt(nonce_obj, encrypted.ciphertext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let data = fs::read(input_path)?;
    let result = encrypt_file_data(&data, password)?;
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(&result.salt)?;
    output.write_all(&result.nonce)?;
    output.write_all(&result.ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain encrypted data"
        ));
    }
    
    let salt = &encrypted_data[0..SALT_LENGTH];
    let nonce = &encrypted_data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH];
    let ciphertext = &encrypted_data[SALT_LENGTH + NONCE_LENGTH..];
    
    let result = EncryptionResult {
        ciphertext: ciphertext.to_vec(),
        salt: salt.try_into().unwrap(),
        nonce: nonce.try_into().unwrap(),
    };
    
    let decrypted = decrypt_file_data(&result, password)?;
    fs::write(output_path, decrypted)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Secret document content";
        let password = "strong_password_123";
        
        let encrypted = encrypt_file_data(test_data, password).unwrap();
        let decrypted = decrypt_file_data(&encrypted, password).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let test_data = b"Secret document content";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_file_data(test_data, password).unwrap();
        let result = decrypt_file_data(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_file_operations() {
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_content = b"Test file content for encryption";
        fs::write(input_file.path(), test_content).unwrap();
        
        let password = "file_password";
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }
}