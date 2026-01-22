
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, Params,
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
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, Some(32)).map_err(|e| e.to_string())?,
    );

    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;

    let key_bytes = password_hash.hash.ok_or("Hash generation failed")?.as_bytes();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
    Ok(*key)
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut rng = OsRng;
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
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
) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&plaintext).map_err(|e| e.to_string())?;

    Ok(plaintext)
}

pub fn generate_random_key() -> Result<[u8; 32], String> {
    let mut rng = OsRng;
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    Ok(key)
}

pub fn encrypt_with_key(
    plaintext: &[u8],
    key: &[u8; 32],
) -> Result<(Vec<u8>, [u8; NONCE_SIZE]), String> {
    let mut rng = OsRng;
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce_bytes);

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| e.to_string())?;

    Ok((ciphertext, nonce_bytes))
}

pub fn decrypt_with_key(
    ciphertext: &[u8],
    key: &[u8; 32],
    nonce: &[u8; NONCE_SIZE],
) -> Result<Vec<u8>, String> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Test encryption data";
        let password = "secure_password_123";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(plaintext).unwrap();

        let output_encrypted = NamedTempFile::new().unwrap();
        let output_decrypted = NamedTempFile::new().unwrap();

        let result = encrypt_file(input_file.path(), output_encrypted.path(), password).unwrap();

        let decrypted = decrypt_file(
            output_encrypted.path(),
            output_decrypted.path(),
            password,
            &result.nonce,
            &result.salt,
        ).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_key_based_encryption() {
        let plaintext = b"Key-based encryption test";
        let key = generate_random_key().unwrap();

        let (ciphertext, nonce) = encrypt_with_key(plaintext, &key).unwrap();
        let decrypted = decrypt_with_key(&ciphertext, &key, &nonce).unwrap();

        assert_eq!(decrypted, plaintext);
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

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let encrypted = encrypt_data(&buffer, password)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(&encrypted.salt)?;
    output.write_all(&encrypted.nonce)?;
    output.write_all(&encrypted.ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.len() < SALT_SIZE + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain encryption metadata",
        ));
    }

    let salt = buffer[..SALT_SIZE].try_into().unwrap();
    let nonce = buffer[SALT_SIZE..SALT_SIZE + NONCE_SIZE].try_into().unwrap();
    let ciphertext = buffer[SALT_SIZE + NONCE_SIZE..].to_vec();

    let result = EncryptionResult {
        ciphertext,
        nonce,
        salt,
    };

    let decrypted = decrypt_data(&result, password)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(&decrypted)?;

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
    fn test_wrong_password_fails() {
        let data = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let encrypted = encrypt_data(data, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);

        assert!(result.is_err());
    }

    #[test]
    fn test_file_encryption_roundtrip() {
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        let test_data = b"File encryption test content";
        fs::write(temp_input.path(), test_data).unwrap();

        let password = "file_encryption_password";

        encrypt_file(temp_input.path(), temp_encrypted.path(), password).unwrap();
        decrypt_file(temp_encrypted.path(), temp_decrypted.path(), password).unwrap();

        let decrypted_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
}use aes_gcm::{
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
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        FileEncryptor { cipher }
    }

    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);

        let ciphertext = self
            .cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;

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
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = FileEncryptor::generate_key();
        let encryptor = FileEncryptor::new(&key);

        let test_data = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}