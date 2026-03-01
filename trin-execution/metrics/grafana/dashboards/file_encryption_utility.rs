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
    input_path: &str,
    output_path: &str,
    password: &str,
) -> io::Result<EncryptionResult> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

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
    input_path: &str,
    output_path: &str,
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

        let enc_result = encrypt_file(
            input_file.path().to_str().unwrap(),
            output_encrypted.path().to_str().unwrap(),
            password,
        ).unwrap();

        decrypt_file(
            output_encrypted.path().to_str().unwrap(),
            output_decrypted.path().to_str().unwrap(),
            password,
            &enc_result.nonce,
            &enc_result.salt,
        ).unwrap();

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
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> io::Result<Self> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Salt error: {}", e))
        })?;

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Hashing error: {}", e))
            })?;

        let key_bytes = password_hash.hash.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Failed to generate hash")
        })?;

        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let mut rng = OsRng;
        let nonce = Nonce::generate(&mut rng);

        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Encryption failed: {}", e)))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(nonce.as_slice())?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Decryption failed: {}", e)))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }
}

pub fn generate_random_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = generate_random_salt();

        let encryptor = FileEncryptor::from_password(password, &salt).unwrap();

        let original_content = b"Secret data that needs protection";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();

        assert_eq!(decrypted_content, original_content);
    }

    #[test]
    fn test_wrong_password_fails() {
        let salt = generate_random_salt();
        let encryptor1 = FileEncryptor::from_password("password1", &salt).unwrap();
        let encryptor2 = FileEncryptor::from_password("password2", &salt).unwrap();

        let original_content = b"Test data";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        encryptor1
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();

        let decrypted_file = NamedTempFile::new().unwrap();
        let result = encryptor2.decrypt_file(encrypted_file.path(), decrypted_file.path());

        assert!(result.is_err());
    }
}