
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{pbkdf2_hmac, Params};
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_LENGTH: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub salt: [u8; SALT_LENGTH],
    pub iv: [u8; IV_LENGTH],
    pub ciphertext: Vec<u8>,
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    let params = Params {
        rounds: PBKDF2_ITERATIONS,
        output_length: KEY_LENGTH,
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key)
        .expect("PBKDF2 should not fail");
    key
}

pub fn encrypt_data(data: &[u8], password: &str) -> io::Result<EncryptionResult> {
    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut iv);
    
    let key = derive_key(password, &salt);
    
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(data);
    
    Ok(EncryptionResult {
        salt,
        iv,
        ciphertext,
    })
}

pub fn decrypt_data(encrypted: &EncryptionResult, password: &str) -> io::Result<Vec<u8>> {
    let key = derive_key(password, &encrypted.salt);
    
    let plaintext = Aes256CbcDec::new(&key.into(), &encrypted.iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted.ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    Ok(plaintext)
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let encrypted = encrypt_data(&buffer, password)?;
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(&encrypted.salt)?;
    output.write_all(&encrypted.iv)?;
    output.write_all(&encrypted.ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    
    let mut salt = [0u8; SALT_LENGTH];
    file.read_exact(&mut salt)?;
    
    let mut iv = [0u8; IV_LENGTH];
    file.read_exact(&mut iv)?;
    
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;
    
    let encrypted = EncryptionResult {
        salt,
        iv,
        ciphertext,
    };
    
    let decrypted = decrypt_data(&encrypted, password)?;
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(&decrypted)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let data = b"Hello, World! This is a secret message.";
        let password = "strong_password_123";
        
        let encrypted = encrypt_data(data, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(data, decrypted.as_slice());
    }
    
    #[test]
    fn test_file_encryption() {
        let original_data = b"Test file content for encryption";
        let password = "file_password";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data, decrypted_data.as_slice());
    }
    
    #[test]
    fn test_wrong_password() {
        let data = b"Sensitive data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_data(data, password).unwrap();
        
        let result = decrypt_data(&encrypted, wrong_password);
        assert!(result.is_err());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::generate(&mut OsRng);
    
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < 12 {
        return Err("Ciphertext too short".into());
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&ciphertext[0..12]);
    let encrypted_data = &ciphertext[12..];
    
    cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn test_encryption_roundtrip() {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        
        let plaintext = b"Secret message for encryption test";
        
        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::rngs::OsRng;
use rand::RngCore;

pub fn encrypt_file(data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".to_string());
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .map(|mut ciphertext| {
            ciphertext.extend_from_slice(&nonce);
            ciphertext
        })
        .map_err(|e| format!("Encryption failed: {}", e))
}

pub fn decrypt_file(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".to_string());
    }

    if ciphertext.len() < 12 {
        return Err("Ciphertext too short".to_string());
    }

    let (data, nonce_bytes) = ciphertext.split_at(ciphertext.len() - 12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), data)
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn generate_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption_roundtrip() {
        let original_text = b"Hello, XOR encryption!";
        let key = b"secret_key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_data);
    }
}