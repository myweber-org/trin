
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::{RngCore, rngs::OsRng};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let iv = generate_iv();
    let cipher = Aes256Cbc::new_from_slices(key, &iv)
        .map_err(|e| format!("Cipher initialization failed: {}", e))?;

    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let ciphertext = cipher.encrypt_vec(&plaintext);

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&iv)
        .map_err(|e| format!("Failed to write IV: {}", e))?;
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut file_content = Vec::new();
    input_file.read_to_end(&mut file_content)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    if file_content.len() < 16 {
        return Err("File too short to contain IV".to_string());
    }

    let iv = &file_content[..16];
    let ciphertext = &file_content[16..];

    let cipher = Aes256Cbc::new_from_slices(key, iv)
        .map_err(|e| format!("Cipher initialization failed: {}", e))?;

    let plaintext = cipher.decrypt_vec(ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;

    Ok(())
}

fn generate_iv() -> [u8; 16] {
    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);
    iv
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn key_from_password(password: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    let password_bytes = password.as_bytes();
    let len = password_bytes.len().min(32);
    key[..len].copy_from_slice(&password_bytes[..len]);
    
    for i in len..32 {
        key[i] = (i as u8).wrapping_add(key[i % len]);
    }
    
    key
}