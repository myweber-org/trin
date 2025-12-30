use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::error::Error;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn generate_key() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut key = [0u8; 32];
    rng.fill(&mut key);
    key
}

pub fn generate_iv() -> [u8; 16] {
    let mut rng = rand::thread_rng();
    let mut iv = [0u8; 16];
    rng.fill(&mut iv);
    iv
}

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Result<String, Box<dyn Error>> {
    let ciphertext = Aes256CbcEnc::new(key.into(), iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    Ok(hex::encode(ciphertext))
}

pub fn decrypt_data(ciphertext_hex: &str, key: &[u8; 32], iv: &[u8; 16]) -> Result<Vec<u8>, Box<dyn Error>> {
    let ciphertext = hex::decode(ciphertext_hex)?;
    let plaintext = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let iv = generate_iv();
        let original_data = b"Secret message for encryption test";

        let encrypted = encrypt_data(original_data, &key, &iv).unwrap();
        let decrypted = decrypt_data(&encrypted, &key, &iv).unwrap();

        assert_eq!(original_data.to_vec(), decrypted);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < 12 {
        return Err("Invalid ciphertext length".into());
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_slice, encrypted_data) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce_slice);
    
    let plaintext = cipher.decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

pub fn generate_key() -> Vec<u8> {
    let key: [u8; 32] = OsRng.gen();
    key.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let original_data = b"Secret message for encryption test";
        
        let encrypted = encrypt_data(original_data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key();
        let key2 = generate_key();
        let data = b"Test data";
        
        let encrypted = encrypt_data(data, &key1).unwrap();
        let result = decrypt_data(&encrypted, &key2);
        
        assert!(result.is_err());
    }
}