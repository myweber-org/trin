use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> std::io::Result<()> {
        let mut input_file = File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.process_bytes(&buffer);

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> std::io::Result<()> {
        self.encrypt_file(input_path, output_path)
    }

    fn process_bytes(&self, data: &[u8]) -> Vec<u8> {
        let key_length = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_length])
            .collect()
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("secret_key");
        let original_data = b"Hello, XOR encryption!";
        
        let encrypted = cipher.process_bytes(original_data);
        let decrypted = cipher.process_bytes(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> std::io::Result<()> {
        let cipher = XORCipher::new("test_key");
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        let test_content = b"File encryption test data";
        fs::write(input_file.path(), test_content)?;

        cipher.encrypt_file(input_file.path(), output_file.path())?;
        cipher.decrypt_file(output_file.path(), decrypted_file.path())?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(test_content.to_vec(), decrypted_content);

        Ok(())
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;
const TAG_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let argon2 = Argon2::default();
    let mut output_key_material = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output_key_material)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(output_key_material)
}

pub fn encrypt_data(
    plaintext: &[u8],
    password: &str,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);

    let key_material = derive_key(password, &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_data(
    encrypted_result: &EncryptionResult,
    password: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let key_material = derive_key(password, &encrypted_result.salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_material);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted_result.nonce);

    let plaintext = cipher
        .decrypt(nonce, encrypted_result.ciphertext.as_ref())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok(plaintext)
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let result = encrypt_data(&buffer, password)?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&result.salt)?;
    output_file.write_all(&result.nonce)?;
    output_file.write_all(&result.ciphertext)?;

    Ok(())
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.len() < SALT_SIZE + NONCE_SIZE + TAG_SIZE {
        return Err("File too small to be valid encrypted data".into());
    }

    let salt = &buffer[0..SALT_SIZE];
    let nonce = &buffer[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &buffer[SALT_SIZE + NONCE_SIZE..];

    let encrypted_result = EncryptionResult {
        ciphertext: ciphertext.to_vec(),
        nonce: nonce.try_into().unwrap(),
        salt: salt.try_into().unwrap(),
    };

    let plaintext = decrypt_data(&encrypted_result, password)?;

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

        let encrypted = encrypt_data(plaintext, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let temp_input = NamedTempFile::new().unwrap();
        let temp_output = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        let test_data = b"File encryption test content";
        fs::write(temp_input.path(), test_data).unwrap();

        let password = "file_encryption_password";

        encrypt_file(temp_input.path(), temp_output.path(), password).unwrap();
        decrypt_file(temp_output.path(), temp_decrypted.path(), password).unwrap();

        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Sensitive information";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let encrypted = encrypt_data(plaintext, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);

        assert!(result.is_err());
    }
}