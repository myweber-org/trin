use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
    nonce: [u8; NONCE_SIZE],
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8; SALT_SIZE]) -> io::Result<Self> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Salt encoding failed: {}", e))
        })?;
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Hashing failed: {}", e)))?;
        
        let key_bytes = password_hash.hash.ok_or_else(|| 
            io::Error::new(io::ErrorKind::InvalidData, "Hash generation failed")
        )?.as_bytes();
        
        if key_bytes.len() < 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, 
                "Derived key too short for AES-256"
            ));
        }
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        
        Ok(Self {
            cipher,
            nonce,
        })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let plaintext = fs::read(input_path)?;
        
        let ciphertext = self.cipher.encrypt(
            Nonce::from_slice(&self.nonce), 
            plaintext.as_ref()
        ).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData, 
            format!("Encryption failed: {}", e)
        ))?;
        
        let mut output_data = Vec::with_capacity(SALT_SIZE + NONCE_SIZE + ciphertext.len());
        output_data.extend_from_slice(&self.nonce);
        output_data.extend_from_slice(&ciphertext);
        
        fs::write(output_path, &output_data)?;
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let encrypted_data = fs::read(input_path)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, 
                "Encrypted file too short"
            ));
        }
        
        let (nonce_slice, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_slice);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(
                io::ErrorKind::InvalidData, 
                format!("Decryption failed: {}", e)
            ))?;
        
        fs::write(output_path, &plaintext)?;
        Ok(())
    }
    
    pub fn generate_salt() -> [u8; SALT_SIZE] {
        let mut salt = [0u8; SALT_SIZE];
        OsRng.fill_bytes(&mut salt);
        salt
    }
}

pub fn process_encryption(
    password: &str,
    input_file: &str,
    output_file: &str,
    encrypt: bool
) -> io::Result<()> {
    let salt = FileEncryptor::generate_salt();
    let encryptor = FileEncryptor::from_password(password, &salt)?;
    
    let input_path = Path::new(input_file);
    let output_path = Path::new(output_file);
    
    if encrypt {
        encryptor.encrypt_file(input_path, output_path)?;
        println!("File encrypted successfully");
    } else {
        encryptor.decrypt_file(input_path, output_path)?;
        println!("File decrypted successfully");
    }
    
    Ok(())
}