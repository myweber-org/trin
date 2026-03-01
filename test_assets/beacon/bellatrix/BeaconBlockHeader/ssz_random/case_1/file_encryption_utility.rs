
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, ParamsBuilder,
};
use std::error::Error;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptedData {
    ciphertext: Vec<u8>,
    nonce: [u8; NONCE_SIZE],
    salt: [u8; SALT_SIZE],
}

pub fn encrypt_file_data(
    plaintext: &[u8],
    password: &str,
) -> Result<EncryptedData, Box<dyn Error>> {
    let salt = SaltString::generate(&mut ArgonRng);
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        ParamsBuilder::new()
            .output_len(32)
            .p_cost(4)
            .m_cost(8192)
            .t_cost(3)
            .build()?,
    );

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;

    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut salt_bytes = [0u8; SALT_SIZE];
    salt_bytes.copy_from_slice(salt.as_bytes());

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce.to_vec().try_into().unwrap(),
        salt: salt_bytes,
    })
}

pub fn decrypt_file_data(
    encrypted: &EncryptedData,
    password: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let salt_str = SaltString::from_b64(&base64::encode(encrypted.salt))?;
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        ParamsBuilder::new()
            .output_len(32)
            .p_cost(4)
            .m_cost(8192)
            .t_cost(3)
            .build()?,
    );

    let password_hash = argon2.hash_password(password.as_bytes(), &salt_str)?;
    let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;

    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted.nonce);

    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Confidential document content";
        let password = "StrongPass123!";

        let encrypted = encrypt_file_data(test_data, password).unwrap();
        let decrypted = decrypt_file_data(&encrypted, password).unwrap();

        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let test_data = b"Sensitive information";
        let password = "CorrectPassword";

        let encrypted = encrypt_file_data(test_data, password).unwrap();
        let result = decrypt_file_data(&encrypted, "WrongPassword");

        assert!(result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(nonce.as_slice())?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }

        let (nonce_slice, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_slice);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    key.as_slice().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret data for encryption test";

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