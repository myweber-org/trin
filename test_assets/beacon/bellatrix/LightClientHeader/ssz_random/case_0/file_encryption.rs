
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::error::Error;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::from_slice(b"unique_nonce_");
        let ciphertext = self.cipher.encrypt(nonce, plaintext)?;
        Ok(ciphertext)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Nonce::from_slice(b"unique_nonce_");
        let plaintext = self.cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}

pub fn process_file_data(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let encryptor = FileEncryptor::new();
    let encrypted = encryptor.encrypt(data)?;
    let decrypted = encryptor.decrypt(&encrypted)?;
    
    if data != decrypted.as_slice() {
        return Err("Decryption failed: data mismatch".into());
    }
    
    Ok((encrypted, decrypted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Confidential file content";
        let result = process_file_data(test_data);
        
        assert!(result.is_ok());
        let (encrypted, decrypted) = result.unwrap();
        
        assert_ne!(encrypted, test_data);
        assert_eq!(decrypted, test_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let data = fs::read(input_path)?;
        let nonce = Nonce::generate(&mut OsRng);
        
        let encrypted_data = self.cipher
            .encrypt(&nonce, data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(&nonce)?;
        output.write_all(&encrypted_data)?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let data = fs::read(input_path)?;
        
        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce"
            ));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let decrypted_data = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, decrypted_data)?;
        Ok(())
    }
}

pub fn generate_key_file(path: &Path) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    fs::write(path, key.as_slice())?;
    Ok(())
}
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
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(FileEncryptor { key })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&ciphertext)?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut input_file = File::open(input_path)?;
        let mut ciphertext = Vec::new();
        input_file.read_to_end(&mut ciphertext)?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        
        Ok(())
    }

    pub fn generate_key_file(&self, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let encoded_key = hex::encode(self.key);
        fs::write(output_path, encoded_key)?;
        Ok(())
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();
        
        let original_content = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_key_generation() {
        let key = generate_random_key();
        assert_eq!(key.len(), 32);
        
        let key_file = NamedTempFile::new().unwrap();
        let encryptor = FileEncryptor {
            key,
        };
        
        encryptor.generate_key_file(key_file.path()).unwrap();
        let saved_key = fs::read_to_string(key_file.path()).unwrap();
        assert_eq!(saved_key, hex::encode(key));
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_crypt(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut content = fs::read(input_path)?;
    xor_crypt(&mut content, key);
    fs::write(output_path, content)
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_crypt() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        xor_crypt(&mut data, 0xAA);
        assert_eq!(data, vec![0xAA, 0x55, 0xFF, 0x00]);
        xor_crypt(&mut data, 0xAA);
        assert_eq!(data, vec![0x00, 0xFF, 0x55, 0xAA]);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        let original_data = b"Secret data for encryption test";
        fs::write(input_file.path(), original_data)?;

        encrypt_file(input_file.path(), output_file.path(), Some(0xCC))?;
        decrypt_file(output_file.path(), decrypted_file.path(), Some(0xCC))?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted_content, original_data);

        Ok(())
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, encrypted_data)?;
    
    let key_path = format!("{}.key", output_path);
    fs::write(key_path, key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    let key_data = fs::read(key_path)?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_data);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Test encryption data";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let key_path = format!("{}.key", encrypted_file.path().to_str().unwrap());
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            &key_path,
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}