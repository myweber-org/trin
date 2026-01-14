use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short")?;

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

    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new(&key);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&ciphertext).map_err(|e| e.to_string())?;

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
) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let nonce = Nonce::from_slice(nonce);

    let cipher = Aes256Gcm::new(&key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&plaintext).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data for encryption test";
        let password = "strong_password_123";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(plaintext).unwrap();

        let output_encrypted = NamedTempFile::new().unwrap();
        let output_decrypted = NamedTempFile::new().unwrap();

        let result = encrypt_file(input_file.path(), output_encrypted.path(), password).unwrap();

        decrypt_file(
            output_encrypted.path(),
            output_decrypted.path(),
            password,
            &result.nonce,
            &result.salt,
        )
        .unwrap();

        let mut decrypted_content = Vec::new();
        File::open(output_decrypted.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();

        assert_eq!(decrypted_content, plaintext);
    }

    #[test]
    fn test_key_derivation() {
        let password = "test_password";
        let mut salt = [0u8; SALT_SIZE];
        ArgonRng.fill_bytes(&mut salt);

        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();
        let key3 = derive_key("different_password", &salt).unwrap();

        assert_eq!(key1.as_slice(), key2.as_slice());
        assert_ne!(key1.as_slice(), key3.as_slice());
    }
}