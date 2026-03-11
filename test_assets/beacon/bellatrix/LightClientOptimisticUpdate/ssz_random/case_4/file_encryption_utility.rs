use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, argon2::password_hash::Error> {
    let salt_str = SaltString::encode_b64(salt)?;
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_str)?;
    
    let hash_bytes = password_hash.hash.unwrap().as_bytes();
    let key_slice = &hash_bytes[..32];
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> io::Result<EncryptionResult> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let mut rng = OsRng;
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let key = derive_key(password, &salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    let cipher = Aes256Gcm::new(&key);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;

    let key = derive_key(password, salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(plaintext).unwrap();
        
        let output_encrypted = NamedTempFile::new().unwrap();
        let output_decrypted = NamedTempFile::new().unwrap();
        
        let result = encrypt_file(input_file.path(), output_encrypted.path(), password)
            .expect("Encryption failed");
        
        decrypt_file(
            output_encrypted.path(),
            output_decrypted.path(),
            password,
            &result.nonce,
            &result.salt,
        ).expect("Decryption failed");
        
        let mut decrypted_content = Vec::new();
        fs::File::open(output_decrypted.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();
        
        assert_eq!(decrypted_content, plaintext);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short")?;

    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_data(result: &EncryptionResult, password: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &result.salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&result.nonce);

    cipher
        .decrypt(nonce, result.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    let result = encrypt_data(&buffer, password)?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&result.ciphertext)
        .map_err(|e| e.to_string())?;

    Ok(result)
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let result = EncryptionResult {
        ciphertext,
        nonce: *nonce,
        salt: *salt,
    };

    let plaintext = decrypt_data(&result, password)?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&plaintext)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let data = b"Secret data for encryption test";
        let password = "strong_password_123";

        let encrypted = encrypt_data(data, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let temp_input = NamedTempFile::new().unwrap();
        let temp_output = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        let test_data = b"File encryption test content";
        fs::write(temp_input.path(), test_data).unwrap();

        let password = "file_encryption_password";
        let result = encrypt_file(temp_input.path(), temp_output.path(), password).unwrap();

        decrypt_file(
            temp_output.path(),
            temp_decrypted.path(),
            password,
            &result.nonce,
            &result.salt,
        )
        .unwrap();

        let decrypted_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }

    #[test]
    fn test_wrong_password_fails() {
        let data = b"Sensitive information";
        let password = "correct_password";

        let encrypted = encrypt_data(data, password).unwrap();
        let wrong_password = "wrong_password";

        assert!(decrypt_data(&encrypted, wrong_password).is_err());
    }
}