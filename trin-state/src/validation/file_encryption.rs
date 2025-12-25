
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use rand::{rngs::OsRng, RngCore};
use std::fs;
use std::io::{Read, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptedFile {
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
    pub ciphertext: Vec<u8>,
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let params = Params {
        rounds: PBKDF2_ITERATIONS,
        output_length: 32,
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key);
    key
}

pub fn encrypt_file(path: &str, password: &str) -> Result<EncryptedFile, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut salt = [0u8; SALT_LENGTH];
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptedFile {
        salt,
        nonce,
        ciphertext,
    })
}

pub fn decrypt_file(encrypted: &EncryptedFile, password: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    
    cipher
        .decrypt(Nonce::from_slice(&encrypted.nonce), encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn save_encrypted_file(encrypted: &EncryptedFile, output_path: &str) -> Result<(), String> {
    let mut file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    file.write_all(&encrypted.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    file.write_all(&encrypted.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    file.write_all(&encrypted.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn load_encrypted_file(path: &str) -> Result<EncryptedFile, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
    
    if contents.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err("File too short to contain valid encrypted data".to_string());
    }
    
    let salt = contents[..SALT_LENGTH].try_into().unwrap();
    let nonce = contents[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH].try_into().unwrap();
    let ciphertext = contents[SALT_LENGTH + NONCE_LENGTH..].to_vec();
    
    Ok(EncryptedFile {
        salt,
        nonce,
        ciphertext,
    })
}