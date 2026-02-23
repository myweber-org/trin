
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub struct FileEncryptor {
    key: [u8; 32],
    iv: [u8; 16],
}

impl FileEncryptor {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut key);
        rand::thread_rng().fill_bytes(&mut iv);
        Self { key, iv }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let ciphertext = Aes256CbcEnc::new(&self.key.into(), &self.iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        let plaintext = Aes256CbcDec::new(&self.key.into(), &self.iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }

    pub fn save_key_iv(&self, path: &str) -> Result<(), String> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.key);
        data.extend_from_slice(&self.iv);
        
        fs::write(path, &data)
            .map_err(|e| format!("Failed to save key/IV: {}", e))?;
        Ok(())
    }

    pub fn load_key_iv(path: &str) -> Result<Self, String> {
        let data = fs::read(path)
            .map_err(|e| format!("Failed to read key/IV file: {}", e))?;
        
        if data.len() != 48 {
            return Err("Invalid key/IV file format".to_string());
        }

        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        key.copy_from_slice(&data[0..32]);
        iv.copy_from_slice(&data[32..48]);

        Ok(Self { key, iv })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();

        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let input_data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

pub fn process_file_interactive() -> io::Result<()> {
    println!("File Encryption Utility");
    println!("=======================");
    
    let mut input_path = String::new();
    print!("Enter input file path: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();
    
    let mut output_path = String::new();
    print!("Enter output file path: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();
    
    let mut key_input = String::new();
    print!("Enter encryption key (0-255, empty for default): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut key_input)?;
    
    let key = if key_input.trim().is_empty() {
        None
    } else {
        match key_input.trim().parse::<u8>() {
            Ok(k) => Some(k),
            Err(_) => {
                eprintln!("Invalid key. Using default key.");
                None
            }
        }
    };
    
    println!("Choose operation:");
    println!("1. Encrypt");
    println!("2. Decrypt");
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    match choice.trim() {
        "1" => {
            xor_encrypt_file(input_path, output_path, key)?;
            println!("File encrypted successfully.");
        }
        "2" => {
            xor_decrypt_file(input_path, output_path, key)?;
            println!("File decrypted successfully.");
        }
        _ => {
            eprintln!("Invalid choice.");
            return Ok(());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_encryption_decryption() {
        let test_data = b"Hello, World! This is a test file.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0xAA)
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0xAA)
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
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

pub fn generate_key_from_passphrase(passphrase: &str) -> Vec<u8> {
    let mut key = passphrase.as_bytes().to_vec();
    if key.len() < 16 {
        key.extend(vec![0u8; 16 - key.len()]);
    }
    key.truncate(16);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Secret data that needs protection";
        let key = b"my_secret_key_123";
        
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
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use rand::RngCore;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
    InvalidKeyLength,
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let data = fs::read(input_path)?;
        
        let encrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes(&data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&data, key)?,
        };
        
        fs::write(output_path, encrypted_data)?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path, key: &[u8]) -> Result<(), EncryptionError> {
        let encrypted_data = fs::read(input_path)?;
        
        let decrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(&encrypted_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(&encrypted_data, key)?,
        };
        
        fs::write(output_path, decrypted_data)?;
        Ok(())
    }

    fn encrypt_aes(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    fn decrypt_aes(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }

        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = &data[..12];
        let ciphertext = &data[12..];
        
        let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        Ok(plaintext)
    }

    fn encrypt_chacha(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }

        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = cipher.encrypt(ChaChaNonce::from_slice(&nonce), data)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    fn decrypt_chacha(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }

        if data.len() < 12 {
            return Err(EncryptionError::CryptoError("Invalid encrypted data".to_string()));
        }

        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        
        let nonce = &data[..12];
        let ciphertext = &data[12..];
        
        let plaintext = cipher.decrypt(ChaChaNonce::from_slice(nonce), ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
        
        Ok(plaintext)
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
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
        let test_data = b"Hello, AES encryption!";
        
        let encrypted = encryptor.encrypt_aes(test_data, &key).unwrap();
        let decrypted = encryptor.decrypt_aes(&encrypted, &key).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let key = generate_random_key();
        let test_data = b"Hello, ChaCha20Poly1305 encryption!";
        
        let encrypted = encryptor.encrypt_chacha(test_data, &key).unwrap();
        let decrypted = encryptor.decrypt_chacha(&encrypted, &key).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = generate_random_key();
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let test_data = b"Test file encryption and decryption";
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}