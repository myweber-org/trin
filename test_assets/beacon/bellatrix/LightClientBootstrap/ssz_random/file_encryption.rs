
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, World! This is a test file.";
        
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), original_content).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            Some(0x42),
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_temp.path()).unwrap();
        assert_ne!(encrypted_content, original_content);
        
        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            Some(0x42),
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }
    
    #[test]
    fn test_default_key() {
        let original_content = b"Testing default key functionality";
        
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), original_content).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            None,
        ).unwrap();
        
        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            None,
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};
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
        let data = fs::read(input_path)?;
        let nonce = Nonce::generate(&mut OsRng);
        
        let encrypted_data = self.cipher
            .encrypt(&nonce, data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(&nonce)?;
        output.write_all(&encrypted_data)?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let encrypted = fs::read(input_path)?;
        if encrypted.len() < 12 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "File too short"));
        }
        
        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let decrypted_data = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, decrypted_data)?;
        Ok(())
    }
}

pub fn generate_key_file(path: &Path) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    fs::write(path, key.as_slice())?;
    Ok(())
}
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
        source_path: &Path,
        dest_path: &Path,
        password: &str,
    ) -> Result<(), CryptoError> {
        let mut source_file = fs::File::open(source_path)?;
        let mut plaintext = Vec::new();
        source_file.read_to_end(&mut plaintext)?;

        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);

        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut dest_file = fs::File::create(dest_path)?;
        dest_file.write_all(&[0x01])?;
        dest_file.write_all(&salt)?;
        dest_file.write_all(&iv)?;
        dest_file.write_all(&(ciphertext.len() as u32).to_be_bytes())?;
        dest_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        source_path: &Path,
        dest_path: &Path,
        password: &str,
    ) -> Result<(), CryptoError> {
        let mut source_file = fs::File::open(source_path)?;
        let mut header = [0u8; 1];
        source_file.read_exact(&mut header)?;

        if header[0] != 0x01 {
            return Err(CryptoError::UnsupportedVersion);
        }

        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        source_file.read_exact(&mut salt)?;
        source_file.read_exact(&mut iv)?;

        let mut length_bytes = [0u8; 4];
        source_file.read_exact(&mut length_bytes)?;
        let ciphertext_len = u32::from_be_bytes(length_bytes) as usize;

        let mut ciphertext = vec![0u8; ciphertext_len];
        source_file.read_exact(&mut ciphertext)?;

        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

        let cipher = Aes256CbcDec::new(&key.into(), &iv.into());
        let plaintext = cipher
            .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
            .map_err(|_| CryptoError::InvalidData)?;

        let mut dest_file = fs::File::create(dest_path)?;
        dest_file.write_all(&plaintext)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_cycle() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";

        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), plaintext).unwrap();

        FileCrypto::encrypt_file(source_file.path(), encrypted_file.path(), password).unwrap();
        FileCrypto::decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, plaintext);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), plaintext).unwrap();

        FileCrypto::encrypt_file(source_file.path(), encrypted_file.path(), password).unwrap();
        let result = FileCrypto::decrypt_file(encrypted_file.path(), decrypted_file.path(), wrong_password);

        assert!(matches!(result, Err(CryptoError::InvalidData)));
    }
}