
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        rand_core::RngCore,
        PasswordHasher, SaltString
    },
    Params, Pbkdf2
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let key_material = Pbkdf2.hash_password_customized(
            password.as_bytes(),
            None,
            None,
            Params {
                rounds: PBKDF2_ITERATIONS,
                output_length: 32,
            },
            salt.as_salt(),
        )?;
        
        let key_hash = key_material.ok_or("Key derivation failed")?;
        let key_bytes = key_hash.hash.ok_or("No hash generated")?.as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        
        Ok(Self {
            cipher: Aes256Gcm::new(key),
        })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let plaintext = fs::read(input_path)?;
        
        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_data = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        output_data.extend_from_slice(&nonce_bytes);
        output_data.extend_from_slice(&ciphertext);
        
        fs::write(output_path, output_data)?;
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_data = fs::read(input_path)?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err("Invalid encrypted file format".into());
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_LENGTH]);
        let ciphertext = &encrypted_data[NONCE_LENGTH..];
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

pub fn process_encryption() -> Result<(), Box<dyn std::error::Error>> {
    let password = "secure_passphrase_123";
    let encryptor = FileEncryptor::new(password)?;
    
    let test_data = b"Confidential document content";
    let input_file = "test_document.txt";
    let encrypted_file = "document.enc";
    let decrypted_file = "document_decrypted.txt";
    
    fs::write(input_file, test_data)?;
    
    encryptor.encrypt_file(Path::new(input_file), Path::new(encrypted_file))?;
    println!("File encrypted successfully");
    
    let encryptor2 = FileEncryptor::new(password)?;
    encryptor2.decrypt_file(Path::new(encrypted_file), Path::new(decrypted_file))?;
    println!("File decrypted successfully");
    
    let restored_data = fs::read(decrypted_file)?;
    assert_eq!(test_data.to_vec(), restored_data);
    
    fs::remove_file(input_file)?;
    fs::remove_file(encrypted_file)?;
    fs::remove_file(decrypted_file)?;
    
    Ok(())
}