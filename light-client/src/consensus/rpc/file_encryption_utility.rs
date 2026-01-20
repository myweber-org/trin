use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::error::Error;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<EncryptionResult, Box<dyn Error>> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".into());
    }

    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce_bytes: [u8; 12] = OsRng.fill(&mut [0u8; 12])?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext)?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

pub fn decrypt_data(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".into());
    }
    if nonce.len() != 12 {
        return Err("Nonce must be 12 bytes".into());
    }

    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0x42u8; 32];
        let plaintext = b"Secret message for encryption test";

        let result = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&result.ciphertext, &key, &result.nonce).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0x42u8; 16];
        let plaintext = b"Test";

        let result = encrypt_data(plaintext, &short_key);
        assert!(result.is_err());
    }
}