use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8; 32]) -> Result<(), String> {
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let encryption_result = encrypt_data(&plaintext, key)?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&encryption_result.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    
    output_file.write_all(&encryption_result.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8; 32]) -> Result<(), String> {
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    if encrypted_data.len() < NONCE_SIZE {
        return Err("Encrypted data too short".to_string());
    }

    let nonce = &encrypted_data[..NONCE_SIZE];
    let ciphertext = &encrypted_data[NONCE_SIZE..];

    let plaintext = decrypt_data(ciphertext, key, nonce)?;
    
    let mut output_file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(())
}

fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<EncryptionResult, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    
    let nonce_obj = Nonce::from_slice(&nonce);
    
    let ciphertext = cipher.encrypt(nonce_obj, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce,
    })
}

fn decrypt_data(ciphertext: &[u8], key: &[u8; 32], nonce: &[u8]) -> Result<Vec<u8>, String> {
    if nonce.len() != NONCE_SIZE {
        return Err(format!("Invalid nonce length: expected {}, got {}", NONCE_SIZE, nonce.len()));
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_obj = Nonce::from_slice(nonce);
    
    cipher.decrypt(nonce_obj, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn save_key_to_file(key: &[u8; 32], path: &Path) -> Result<(), String> {
    let hex_key = hex::encode(key);
    fs::write(path, hex_key)
        .map_err(|e| format!("Failed to save key: {}", e))
}

pub fn load_key_from_file(path: &Path) -> Result<[u8; 32], String> {
    let hex_key = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    
    let bytes = hex::decode(hex_key.trim())
        .map_err(|e| format!("Invalid hex format: {}", e))?;
    
    if bytes.len() != 32 {
        return Err(format!("Invalid key length: expected 32 bytes, got {}", bytes.len()));
    }
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}