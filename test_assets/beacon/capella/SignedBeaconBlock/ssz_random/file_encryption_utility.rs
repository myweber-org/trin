use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short")?;
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<(), String> {
    let mut file_data = fs::read(input_path).map_err(|e| e.to_string())?;
    
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let encrypted_data = cipher
        .encrypt(nonce, file_data.as_ref())
        .map_err(|e| e.to_string())?;
    
    let mut output = Vec::with_capacity(SALT_SIZE + NONCE_SIZE + encrypted_data.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&encrypted_data);
    
    fs::write(output_path, &output).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<(), String> {
    let encrypted_data = fs::read(input_path).map_err(|e| e.to_string())?;
    
    if encrypted_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err("File too short".to_string());
    }
    
    let salt = &encrypted_data[..SALT_SIZE];
    let nonce_bytes = &encrypted_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &encrypted_data[SALT_SIZE + NONCE_SIZE..];
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let decrypted_data = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())?;
    
    fs::write(output_path, &decrypted_data).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() -> Result<(), String> {
    println!("File Encryption Utility");
    println!("1. Encrypt file");
    println!("2. Decrypt file");
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).map_err(|e| e.to_string())?;
    
    match choice.trim() {
        "1" => {
            println!("Enter input file path:");
            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
            
            println!("Enter output file path:");
            let mut output = String::new();
            io::stdin().read_line(&mut output).map_err(|e| e.to_string())?;
            
            println!("Enter password:");
            let mut password = String::new();
            io::stdin().read_line(&mut password).map_err(|e| e.to_string())?;
            
            encrypt_file(input.trim(), output.trim(), password.trim())?;
            println!("Encryption completed successfully");
        }
        "2" => {
            println!("Enter input file path:");
            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
            
            println!("Enter output file path:");
            let mut output = String::new();
            io::stdin().read_line(&mut output).map_err(|e| e.to_string())?;
            
            println!("Enter password:");
            let mut password = String::new();
            io::stdin().read_line(&mut password).map_err(|e| e.to_string())?;
            
            decrypt_file(input.trim(), output.trim(), password.trim())?;
            println!("Decryption completed successfully");
        }
        _ => return Err("Invalid choice".to_string()),
    }
    
    Ok(())
}