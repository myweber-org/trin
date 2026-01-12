use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::{
    fs,
    io::{Read, Write},
    path::Path
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor;

impl FileEncryptor {
    pub fn encrypt_file(
        input_path: &Path,
        output_path: &Path,
        password: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_material = password_hash.hash.ok_or("Hash generation failed")?;

        let key = Key::<Aes256Gcm>::from_slice(&key_material.as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(salt.as_bytes())?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        input_path: &Path,
        output_path: &Path,
        password: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < SALT_SIZE {
            return Err("Invalid encrypted file format".into());
        }

        let salt_bytes = &encrypted_data[..SALT_SIZE];
        let ciphertext = &encrypted_data[SALT_SIZE..];
        let salt = SaltString::from_b64(std::str::from_utf8(salt_bytes)?)?;

        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_material = password_hash.hash.ok_or("Hash generation failed")?;

        let key = Key::<Aes256Gcm>::from_slice(&key_material.as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

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
        let original_content = b"Test data for encryption and decryption";
        let password = "secure_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        FileEncryptor::encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password
        ).unwrap();

        FileEncryptor::decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password
        ).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_wrong_password_fails() {
        let content = b"Sensitive data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), content).unwrap();

        FileEncryptor::encrypt_file(
            input_file.path(),
            encrypted_file.path(),
            password
        ).unwrap();

        let result = FileEncryptor::decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password
        );

        assert!(result.is_err());
    }
}