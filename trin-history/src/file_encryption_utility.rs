
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        PasswordHasher, SaltString, PasswordHash, PasswordVerifier
    },
    Pbkdf2
};
use rand_core::RngCore;
use std::fs;
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, String> {
        let salt = SaltString::generate(&mut OsRng);
        let key = Self::derive_key(password, salt.as_str())?;
        let cipher = Aes256Gcm::new(&key);
        Ok(Self { cipher })
    }

    fn derive_key(password: &str, salt: &str) -> Result<Key<Aes256Gcm>, String> {
        let password_hash = Pbkdf2
            .hash_password(password.as_bytes(), salt)
            .map_err(|e| format!("Key derivation failed: {}", e))?;

        let hash = PasswordHash::new(&password_hash.hash.unwrap().to_string())
            .map_err(|e| format!("Hash parsing failed: {}", e))?;

        let key_bytes = hash.hash.unwrap().as_bytes();
        if key_bytes.len() != 32 {
            return Err("Derived key must be 32 bytes".to_string());
        }

        Ok(*Key::<Aes256Gcm>::from_slice(key_bytes))
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let plaintext = fs::read(input_path)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let mut nonce = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce);
        let nonce = Nonce::from_slice(&nonce);

        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_data = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        output_data.extend_from_slice(nonce);
        output_data.extend_from_slice(&ciphertext);

        fs::write(output_path, output_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let encrypted_data = fs::read(input_path)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Encrypted data too short".to_string());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::write(output_path, plaintext)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;

        Ok(())
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();

        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_key_generation() {
        let key = generate_random_key();
        assert_eq!(key.len(), 32);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let salt = SaltString::generate(&mut ArgonRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;

    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_LENGTH]);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(salt.as_bytes())?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < SALT_LENGTH {
        return Err("File too short to contain salt".into());
    }

    let (salt_bytes, ciphertext) = encrypted_data.split_at(SALT_LENGTH);
    let salt = SaltString::from_b64(std::str::from_utf8(salt_bytes)?)?;

    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;

    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_LENGTH]);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("File Encryption Utility");
    println!("1. Encrypt file");
    println!("2. Decrypt file");

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    match choice {
        "1" => {
            println!("Enter input file path:");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            println!("Enter output file path:");
            let mut output = String::new();
            io::stdin().read_line(&mut output)?;
            let output = output.trim();

            println!("Enter password:");
            let mut password = String::new();
            io::stdin().read_line(&mut password)?;
            let password = password.trim();

            encrypt_file(input, output, password)?;
            println!("Encryption completed successfully.");
        }
        "2" => {
            println!("Enter input file path:");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            println!("Enter output file path:");
            let mut output = String::new();
            io::stdin().read_line(&mut output)?;
            let output = output.trim();

            println!("Enter password:");
            let mut password = String::new();
            io::stdin().read_line(&mut password)?;
            let password = password.trim();

            decrypt_file(input, output, password)?;
            println!("Decryption completed successfully.");
        }
        _ => println!("Invalid choice."),
    }

    Ok(())
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    let nonce = generate_nonce();

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&nonce).map_err(|e| format!("Failed to write nonce: {}", e))?;
    output.write_all(&ciphertext).map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    if data.len() < NONCE_SIZE {
        return Err("File too short to contain nonce".to_string());
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&plaintext).map_err(|e| format!("Failed to write plaintext: {}", e))?;

    Ok(())
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    let bytes = password.as_bytes();
    for (i, &byte) in bytes.iter().cycle().take(32).enumerate() {
        key[i] = byte;
    }
    *Key::<Aes256Gcm>::from_slice(&key)
}

fn generate_nonce() -> Nonce {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    Nonce::from_slice(&nonce).clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();
        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        let result = decrypt_file(encrypted_file.path(), decrypted_file.path(), wrong_password);
        assert!(result.is_err());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;
    output_file.write_all(&key)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key",
        ));
    }

    let (ciphertext, key_bytes) = encrypted_data.split_at(encrypted_data.len() - 32);
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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
        let original_content = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}