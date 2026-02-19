
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(key: &[u8; 32]) -> Result<Self, String> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Ok(FileEncryptor { cipher })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let mut rng = OsRng;
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        let nonce = Nonce::from_slice(&rng.gen::<[u8; NONCE_SIZE]>());

        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(nonce.as_slice())
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".to_string());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write plaintext: {}", e))?;

        Ok(())
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_random_key();
        let encryptor = FileEncryptor::new(&key).unwrap();

        let original_content = b"Test data for encryption and decryption";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let mut decrypted_content = Vec::new();
        let mut decrypted_file_handle = fs::File::open(decrypted_file.path()).unwrap();
        decrypted_file_handle.read_to_end(&mut decrypted_content).unwrap();

        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, Some(32)).map_err(|e| e.to_string())?,
    );

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Invalid hash length")?;

    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;

    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new(&key);
    let encrypted_data = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&encrypted_data)
        .map_err(|e| e.to_string())?;

    Ok(EncryptionResult {
        encrypted_data,
        salt,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    salt: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let nonce = Nonce::from_slice(nonce);

    let cipher = Aes256Gcm::new(&key);
    let decrypted_data = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    fs::write(output_path, &decrypted_data).map_err(|e| e.to_string())?;

    Ok(decrypted_data)
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
        fs::write(input_file.path(), plaintext).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        let decrypted_file = NamedTempFile::new().unwrap();
        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &result.salt,
            &result.nonce,
        )
        .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "correct_password";

        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), plaintext).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        let decrypted_file = NamedTempFile::new().unwrap();
        let wrong_password_result = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            "wrong_password",
            &result.salt,
            &result.nonce,
        );

        assert!(wrong_password_result.is_err());
    }
}