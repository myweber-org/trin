
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, String> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| e.to_string())?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| e.to_string())?;
        
        let mut output = File::create(output_path).map_err(|e| e.to_string())?;
        output.write_all(nonce.as_slice()).map_err(|e| e.to_string())?;
        output.write_all(&ciphertext).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data).map_err(|e| e.to_string())?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("File too short to contain nonce".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())?;
        
        let mut output = File::create(output_path).map_err(|e| e.to_string())?;
        output.write_all(&plaintext).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    pub fn generate_salt() -> [u8; SALT_SIZE] {
        generate_random_bytes(SALT_SIZE)
    }
}

fn generate_random_bytes(size: usize) -> [u8; SALT_SIZE] {
    let mut bytes = [0u8; SALT_SIZE];
    if size <= SALT_SIZE {
        let mut rng = OsRng;
        rng.fill_bytes(&mut bytes[..size]);
    }
    bytes
}

pub fn secure_delete_file(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    let file_size = metadata.len() as usize;
    
    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    
    let random_data = generate_random_bytes(file_size);
    file.write_all(&random_data).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    
    fs::remove_file(path).map_err(|e| e.to_string())?;
    
    Ok(())
}