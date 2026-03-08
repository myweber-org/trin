use rand::{thread_rng, Rng};
use sha2::{Sha256, Digest};

const SALT_LENGTH: usize = 16;
const TOKEN_LENGTH: usize = 32;

pub fn generate_salt() -> String {
    let mut rng = thread_rng();
    (0..SALT_LENGTH)
        .map(|_| rng.gen_range(33..127) as u8 as char)
        .collect()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn generate_api_token() -> String {
    let mut rng = thread_rng();
    (0..TOKEN_LENGTH)
        .map(|_| rng.gen_range(48..123) as u8 as char)
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salt_length() {
        let salt = generate_salt();
        assert_eq!(salt.len(), SALT_LENGTH);
    }

    #[test]
    fn test_hash_consistency() {
        let password = "secure_password";
        let salt = "random_salt";
        let hash1 = hash_password(password, salt);
        let hash2 = hash_password(password, salt);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_token_alphanumeric() {
        let token = generate_api_token();
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
        assert_eq!(token.len(), TOKEN_LENGTH);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::generate(&mut OsRng);
    
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .context("Encryption failed")?;
    
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        anyhow::bail!("Ciphertext too short");
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, encrypted_data)
        .context("Decryption failed")?;
    
    Ok(plaintext)
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}use rand::Rng;
use sha2::{Sha256, Digest};

pub fn generate_salt() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut salt = [0u8; 32];
    rng.fill(&mut salt);
    salt
}

pub fn hash_password(password: &str, salt: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt_length() {
        let salt = generate_salt();
        assert_eq!(salt.len(), 32);
    }

    #[test]
    fn test_hash_password() {
        let password = "secure_password123";
        let salt = generate_salt();
        let hash1 = hash_password(password, &salt);
        let hash2 = hash_password(password, &salt);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_salts_produce_different_hashes() {
        let password = "same_password";
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        let hash1 = hash_password(password, &salt1);
        let hash2 = hash_password(password, &salt2);
        assert_ne!(hash1, hash2);
    }
}