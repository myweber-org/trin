use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use sha2::{Sha256, Digest};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Self {
        let salt = Self::generate_salt();
        let key = Self::derive_key(password, &salt);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    fn generate_salt() -> [u8; SALT_SIZE] {
        let mut salt = [0u8; SALT_SIZE];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    fn derive_key(password: &str, salt: &[u8]) -> Key<Aes256Gcm> {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        let result = hasher.finalize();
        *Key::<Aes256Gcm>::from_slice(&result)
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read encrypted data: {}", e))?;

        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        let plaintext = self.cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }
}

pub fn process_encryption(password: &str, input_file: &str, output_file: &str) -> Result<(), String> {
    let encryptor = FileEncryptor::new(password);
    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);

    if !input_path.exists() {
        return Err("Input file does not exist".to_string());
    }

    encryptor.encrypt_file(input_path, output_path)
}

pub fn process_decryption(password: &str, input_file: &str, output_file: &str) -> Result<(), String> {
    let encryptor = FileEncryptor::new(password);
    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);

    if !input_path.exists() {
        return Err("Encrypted file does not exist".to_string());
    }

    encryptor.decrypt_file(input_path, output_path)
}