
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

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
            .map_err(|e| e.to_string())?
            .hash
            .ok_or("Failed to generate hash")?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.as_bytes()[..32]);
        
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut file_data)
            .map_err(|e| e.to_string())?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let encrypted_data = cipher
            .encrypt(nonce, file_data.as_ref())
            .map_err(|e| e.to_string())?;
        
        let mut output = Vec::new();
        output.extend_from_slice(nonce.as_slice());
        output.extend_from_slice(&encrypted_data);
        
        fs::File::create(output_path)
            .map_err(|e| e.to_string())?
            .write_all(&output)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut encrypted_data)
            .map_err(|e| e.to_string())?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_SIZE]);
        let ciphertext = &encrypted_data[NONCE_SIZE..];
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let decrypted_data = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())?;
        
        fs::File::create(output_path)
            .map_err(|e| e.to_string())?
            .write_all(&decrypted_data)
            .map_err(|e| e.to_string())?;
        
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

pub fn encrypt_directory(
    encryptor: &FileEncryptor,
    dir_path: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    if !dir_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    
    for entry in fs::read_dir(dir_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.is_file() {
            let output_path = output_dir.join(
                path.file_name()
                    .ok_or("Invalid file name")?
                    .to_string_lossy()
                    .to_string()
                    + ".enc",
            );
            
            encryptor.encrypt_file(&path, &output_path)?;
        }
    }
    
    Ok(())
}