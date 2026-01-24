
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