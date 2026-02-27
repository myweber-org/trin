
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct EncryptionConfig {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            memory_cost: 19456,
            time_cost: 2,
            parallelism: 1,
        }
    }
}

pub struct FileEncryptor {
    config: EncryptionConfig,
}

impl FileEncryptor {
    pub fn new(config: EncryptionConfig) -> Self {
        Self { config }
    }

    pub fn derive_key(&self, password: &str, salt: &SaltString) -> Result<Key<Aes256Gcm>, String> {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                self.config.memory_cost,
                self.config.time_cost,
                self.config.parallelism,
                None,
            ).map_err(|e| format!("Failed to create Argon2 params: {}", e))?,
        );

        let password_hash = argon2
            .hash_password(password.as_bytes(), salt)
            .map_err(|e| format!("Password hashing failed: {}", e))?;

        let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
        if hash_bytes.len() < 32 {
            return Err("Derived key too short".to_string());
        }

        let key_slice = &hash_bytes[..32];
        Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        password: &str,
    ) -> Result<(), String> {
        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let salt = SaltString::generate(&mut OsRng);
        let key = self.derive_key(password, &salt)?;

        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::generate(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        let salt_bytes = salt.as_bytes();
        output_file.write_all(&(salt_bytes.len() as u32).to_le_bytes())
            .map_err(|e| format!("Failed to write salt length: {}", e))?;
        output_file.write_all(salt_bytes)
            .map_err(|e| format!("Failed to write salt: {}", e))?;

        output_file.write_all(nonce.as_slice())
            .map_err(|e| format!("Failed to write nonce: {}", e))?;

        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        password: &str,
    ) -> Result<(), String> {
        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut salt_len_bytes = [0u8; 4];
        input_file.read_exact(&mut salt_len_bytes)
            .map_err(|e| format!("Failed to read salt length: {}", e))?;
        let salt_len = u32::from_le_bytes(salt_len_bytes) as usize;

        let mut salt_bytes = vec![0u8; salt_len];
        input_file.read_exact(&mut salt_bytes)
            .map_err(|e| format!("Failed to read salt: {}", e))?;
        let salt = SaltString::from_b64(&String::from_utf8_lossy(&salt_bytes))
            .map_err(|e| format!("Invalid salt format: {}", e))?;

        let mut nonce_bytes = [0u8; 12];
        input_file.read_exact(&mut nonce_bytes)
            .map_err(|e| format!("Failed to read nonce: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut ciphertext = Vec::new();
        input_file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read ciphertext: {}", e))?;

        let key = self.derive_key(password, &salt)?;
        let cipher = Aes256Gcm::new(&key);

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write plaintext: {}", e))?;

        Ok(())
    }
}

pub fn generate_secure_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !@#$%^&*()-_=+[]{}|;:,.<>?";
    
    let mut rng = OsRng;
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.next_u32() as usize % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect();
    
    password
}