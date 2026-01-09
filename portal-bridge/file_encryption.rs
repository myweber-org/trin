
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

#[derive(Debug)]
pub enum CryptoError {
    IoError(std::io::Error),
    InvalidData,
    UnsupportedVersion,
}

impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        CryptoError::IoError(err)
    }
}

pub struct FileCrypto;

impl FileCrypto {
    pub fn encrypt_file(
        input_path: &Path,
        output_path: &Path,
        password: &str,
    ) -> Result<(), CryptoError> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);

        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&[0x01])?;
        output_file.write_all(&salt)?;
        output_file.write_all(&iv)?;
        output_file.write_all(&(ciphertext.len() as u32).to_le_bytes())?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        input_path: &Path,
        output_path: &Path,
        password: &str,
    ) -> Result<(), CryptoError> {
        let mut input_file = fs::File::open(input_path)?;
        let mut header = [0u8; 1];
        input_file.read_exact(&mut header)?;

        if header[0] != 0x01 {
            return Err(CryptoError::UnsupportedVersion);
        }

        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        let mut length_bytes = [0u8; 4];
        
        input_file.read_exact(&mut salt)?;
        input_file.read_exact(&mut iv)?;
        input_file.read_exact(&mut length_bytes)?;
        
        let ciphertext_len = u32::from_le_bytes(length_bytes) as usize;
        let mut ciphertext = vec![0u8; ciphertext_len];
        input_file.read_exact(&mut ciphertext)?;

        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

        let cipher = Aes256CbcDec::new(&key.into(), &iv.into());
        let plaintext = cipher
            .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
            .map_err(|_| CryptoError::InvalidData)?;

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
    fn test_encrypt_decrypt() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        FileCrypto::encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption failed");
        
        FileCrypto::decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption failed");
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_wrong_password() {
        let plaintext = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        FileCrypto::encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption failed");
        
        let result = FileCrypto::decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
        );
        
        assert!(matches!(result, Err(CryptoError::InvalidData)));
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = Vec::new();
    result.extend_from_slice(&key);
    result.extend_from_slice(nonce);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < 48 {
        return Err("Invalid ciphertext length".into());
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&ciphertext[..32]);
    let nonce = Nonce::from_slice(&ciphertext[32..44]);
    let encrypted_data = &ciphertext[44..];
    
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher.decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret message for encryption test";
        let encrypted = encrypt_data(original_data).unwrap();
        let decrypted = decrypt_data(&encrypted).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }
}