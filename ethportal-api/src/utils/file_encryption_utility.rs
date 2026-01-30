use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let mut key = [0u8; 32];
    let params = Params {
        rounds: PBKDF2_ITERATIONS,
        output_length: key.len(),
    };
    
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key)
        .map_err(|e| format!("PBKDF2 error: {}", e))?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key).clone())
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce: nonce_bytes,
    })
}

pub fn decrypt_data(encrypted: &EncryptionResult, password: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&encrypted.nonce);
    
    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let encrypted = encrypt_data(&buffer, password)?;
    
    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output.write_all(&encrypted.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output.write_all(&encrypted.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output.write_all(&encrypted.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut salt = [0u8; SALT_LENGTH];
    file.read_exact(&mut salt)
        .map_err(|e| format!("Failed to read salt: {}", e))?;
    
    let mut nonce = [0u8; NONCE_LENGTH];
    file.read_exact(&mut nonce)
        .map_err(|e| format!("Failed to read nonce: {}", e))?;
    
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read ciphertext: {}", e))?;
    
    let encrypted = EncryptionResult {
        ciphertext,
        salt,
        nonce,
    };
    
    let decrypted = decrypt_data(&encrypted, password)?;
    
    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&decrypted)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
    
    Ok(())
}

pub fn interactive_encrypt() -> Result<(), String> {
    println!("Enter file path to encrypt:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let input_path = Path::new(input_path.trim());
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)
        .map_err(|e| format!("Failed to read output path: {}", e))?;
    let output_path = Path::new(output_path.trim());
    
    println!("Enter encryption password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password)
        .map_err(|e| format!("Failed to read password: {}", e))?;
    let password = password.trim();
    
    encrypt_file(input_path, output_path, password)?;
    println!("File encrypted successfully!");
    Ok(())
}

pub fn interactive_decrypt() -> Result<(), String> {
    println!("Enter file path to decrypt:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let input_path = Path::new(input_path.trim());
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)
        .map_err(|e| format!("Failed to read output path: {}", e))?;
    let output_path = Path::new(output_path.trim());
    
    println!("Enter decryption password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password)
        .map_err(|e| format!("Failed to read password: {}", e))?;
    let password = password.trim();
    
    decrypt_file(input_path, output_path, password)?;
    println!("File decrypted successfully!");
    Ok(())
}