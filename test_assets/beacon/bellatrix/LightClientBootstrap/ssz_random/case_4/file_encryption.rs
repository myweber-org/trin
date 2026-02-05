
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut file_data)?;

        let (ciphertext, nonce) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes(&file_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&file_data, key)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut encrypted_data = Vec::new();
        let mut file = fs::File::open(input_path)?;
        file.read_to_end(&mut encrypted_data)?;

        let (nonce, ciphertext) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let nonce_size = 12;
                if encrypted_data.len() < nonce_size {
                    return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
                }
                (&encrypted_data[..nonce_size], &encrypted_data[nonce_size..])
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let nonce_size = 12;
                if encrypted_data.len() < nonce_size {
                    return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
                }
                (&encrypted_data[..nonce_size], &encrypted_data[nonce_size..])
            }
        };

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(ciphertext, key, nonce)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(ciphertext, key, nonce)?,
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn encrypt_aes(&self, data: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        cipher.encrypt(&nonce, data)
            .map(|ciphertext| (ciphertext, nonce.to_vec()))
            .map_err(|e| EncryptionError::CryptoError(format!("Encryption failed: {}", e)))
    }

    fn decrypt_aes(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(format!("Decryption failed: {}", e)))
    }

    fn encrypt_chacha(
        &self,
        data: &[u8],
        key: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        cipher.encrypt(&nonce, data)
            .map(|ciphertext| (ciphertext, nonce.to_vec()))
            .map_err(|e| EncryptionError::CryptoError(format!("Encryption failed: {}", e)))
    }

    fn decrypt_chacha(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaChaNonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(format!("Decryption failed: {}", e)))
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        
        let test_data = b"Hello, this is a secret message!";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let mut decrypted_data = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_data)
            .unwrap();
            
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let key = generate_random_key();
        
        let test_data = b"Another secret message for testing";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let mut decrypted_data = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_data)
            .unwrap();
            
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const KEY_SIZE: usize = 32;
const IV_SIZE: usize = 16;

pub fn generate_key() -> [u8; KEY_SIZE] {
    let mut key = [0u8; KEY_SIZE];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), String> {
    if key.len() != KEY_SIZE {
        return Err(format!("Key must be {} bytes", KEY_SIZE));
    }

    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut iv = [0u8; IV_SIZE];
    rand::thread_rng().fill_bytes(&mut iv);

    let ciphertext = Aes256CbcEnc::new(key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    output_file.write_all(&iv)
        .map_err(|e| format!("Failed to write IV: {}", e))?;
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), String> {
    if key.len() != KEY_SIZE {
        return Err(format!("Key must be {} bytes", KEY_SIZE));
    }

    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    if encrypted_data.len() < IV_SIZE {
        return Err("File too short to contain IV".to_string());
    }

    let (iv, ciphertext) = encrypted_data.split_at(IV_SIZE);
    
    let plaintext = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_key();
        let test_data = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LEN],
    pub iv: [u8; IV_LEN],
}

pub fn encrypt_data(password: &str, plaintext: &[u8]) -> Result<EncryptionResult, &'static str> {
    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);
    
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);
    
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        iv,
    })
}

pub fn decrypt_data(password: &str, result: &EncryptionResult) -> Result<Vec<u8>, &'static str> {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &result.salt, PBKDF2_ITERATIONS, &mut key);
    
    let plaintext = Aes256CbcDec::new(&key.into(), &result.iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&result.ciphertext)
        .map_err(|_| "Decryption failed: invalid padding or corrupted data")?;
    
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let plaintext = b"Secret message to encrypt";
        
        let encrypted = encrypt_data(password, plaintext).unwrap();
        let decrypted = decrypt_data(password, &encrypted).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let plaintext = b"Test data";
        
        let encrypted = encrypt_data(password, plaintext).unwrap();
        let result = decrypt_data(wrong_password, &encrypted);
        
        assert!(result.is_err());
    }
}