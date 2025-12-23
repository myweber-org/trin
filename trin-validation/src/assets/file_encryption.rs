
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use pbkdf2::{
    password_hash::{PasswordHasher, SaltString},
    Pbkdf2,
};
use std::fs::{self, File};
use std::io::{Read, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let salt = SaltString::generate(&mut OsRng);
    let key = derive_key(password, salt.as_str())?;

    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_LENGTH]);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(salt.as_str().as_bytes()).map_err(|e| format!("Failed to write salt: {}", e))?;
    output.write_all(&ciphertext).map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    if data.len() < SALT_LENGTH {
        return Err("Invalid encrypted file format".to_string());
    }

    let salt_str = std::str::from_utf8(&data[..SALT_LENGTH])
        .map_err(|_| "Invalid salt encoding".to_string())?;
    let ciphertext = &data[SALT_LENGTH..];

    let key = derive_key(password, salt_str)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_LENGTH]);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&plaintext).map_err(|e| format!("Failed to write plaintext: {}", e))?;

    Ok(())
}

fn derive_key(password: &str, salt: &str) -> Result<Key<Aes256Gcm>, String> {
    let salt = SaltString::from_b64(salt).map_err(|e| format!("Invalid salt: {}", e))?;
    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    let hash_bytes = password_hash.hash.ok_or("No hash generated".to_string())?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short".to_string())?;

    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}