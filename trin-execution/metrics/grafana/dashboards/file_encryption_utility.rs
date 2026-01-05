
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

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> io::Result<Self> {
        let salt_string = SaltString::b64_encode(salt)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let params = Params {
            rounds: 100_000,
            output_length: 32,
        };
        
        let password_hash = Pbkdf2
            .hash_password_customized(password.as_bytes(), None, None, params, &salt_string)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let key_bytes = password_hash.hash.ok_or_else(|| 
            io::Error::new(io::ErrorKind::InvalidData, "Failed to derive key")
        )?.as_bytes();
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        Ok(Self { key: *key })
    }
    
    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let plaintext = fs::read(input_path)?;
        
        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let cipher = Aes256Gcm::new(&self.key);
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let mut output_data = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        output_data.extend_from_slice(&nonce_bytes);
        output_data.extend_from_slice(&ciphertext);
        
        fs::write(output_path, output_data)?;
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let encrypted_data = fs::read(input_path)?;
        
        if encrypted_data.len() < NONCE_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Encrypted file too short"
            ));
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_LENGTH]);
        let ciphertext = &encrypted_data[NONCE_LENGTH..];
        
        let cipher = Aes256Gcm::new(&self.key);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn process_encryption() -> io::Result<()> {
    let password = "secure_password_123";
    let salt = generate_salt();
    
    let encryptor = FileEncryptor::from_password(password, &salt)?;
    
    let test_data = b"Confidential data requiring encryption";
    let input_file = "test_input.bin";
    let encrypted_file = "encrypted.bin";
    let decrypted_file = "decrypted.bin";
    
    fs::write(input_file, test_data)?;
    
    encryptor.encrypt_file(input_file, encrypted_file)?;
    println!("File encrypted successfully");
    
    encryptor.decrypt_file(encrypted_file, decrypted_file)?;
    println!("File decrypted successfully");
    
    let decrypted_data = fs::read(decrypted_file)?;
    assert_eq!(test_data.as_slice(), decrypted_data.as_slice());
    
    fs::remove_file(input_file)?;
    fs::remove_file(encrypted_file)?;
    fs::remove_file(decrypted_file)?;
    
    Ok(())
}