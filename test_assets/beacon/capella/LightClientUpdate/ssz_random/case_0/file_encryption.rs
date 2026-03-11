
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let data = fs::read(input_path)?;
        let nonce = Nonce::from_slice(b"unique_nonce_");
        
        let encrypted_data = self.cipher
            .encrypt(nonce, data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, encrypted_data)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let encrypted_data = fs::read(input_path)?;
        let nonce = Nonce::from_slice(b"unique_nonce_");
        
        let decrypted_data = self.cipher
            .decrypt(nonce, encrypted_data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        fs::write(output_path, decrypted_data)
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    key.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret confidential data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), result);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;

pub enum CipherType {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    cipher_type: CipherType,
}

impl FileEncryptor {
    pub fn new(cipher_type: CipherType) -> Self {
        Self { cipher_type }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.cipher_type {
            CipherType::Aes256Gcm => {
                let key = Aes256Gcm::generate_key(&mut OsRng);
                let cipher = Aes256Gcm::new(&key);
                let nonce = Nonce::from_slice(&[0u8; 12]);
                let ciphertext = cipher.encrypt(nonce, plaintext)?;
                Ok([key.as_slice(), &ciphertext].concat())
            }
            CipherType::ChaCha20Poly1305 => {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                let cipher = ChaCha20Poly1305::new(&key);
                let nonce = ChaChaNonce::from_slice(&[0u8; 12]);
                let ciphertext = cipher.encrypt(nonce, plaintext)?;
                Ok([key.as_slice(), &ciphertext].concat())
            }
        }
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.cipher_type {
            CipherType::Aes256Gcm => {
                let (key_bytes, encrypted) = ciphertext.split_at(32);
                let key = Key::<Aes256Gcm>::from_slice(key_bytes);
                let cipher = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(&[0u8; 12]);
                cipher.decrypt(nonce, encrypted).map_err(|e| e.into())
            }
            CipherType::ChaCha20Poly1305 => {
                let (key_bytes, encrypted) = ciphertext.split_at(32);
                let key = ChaChaKey::from_slice(key_bytes);
                let cipher = ChaCha20Poly1305::new(key);
                let nonce = ChaChaNonce::from_slice(&[0u8; 12]);
                cipher.decrypt(nonce, encrypted).map_err(|e| e.into())
            }
        }
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption() {
        let encryptor = FileEncryptor::new(CipherType::Aes256Gcm);
        let plaintext = b"Secret message for AES";
        let ciphertext = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha_encryption() {
        let encryptor = FileEncryptor::new(CipherType::ChaCha20Poly1305);
        let plaintext = b"Secret message for ChaCha";
        let ciphertext = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_key_generation() {
        let key1 = generate_random_key();
        let key2 = generate_random_key();
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
        assert_ne!(key1, key2);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Nonce::generate(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }

        let (nonce_slice, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_slice);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> std::io::Result<()> {
    let mut input_file = File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> std::io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn generate_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World! This is a test message.";
        let key = b"secret_key";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_text).unwrap();

        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_content);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce as ChaChaNonce};
use rand::RngCore;
use std::error::Error;

const AES_NONCE_SIZE: usize = 12;
const CHACHA_NONCE_SIZE: usize = 12;

pub enum CipherType {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_data(
    plaintext: &[u8],
    key: &[u8],
    cipher_type: CipherType,
) -> Result<EncryptionResult, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            
            let cipher = Aes256Gcm::new_from_slice(key)?;
            let mut nonce_bytes = [0u8; AES_NONCE_SIZE];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
            })
        }
        
        CipherType::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            
            let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
            let mut nonce_bytes = [0u8; CHACHA_NONCE_SIZE];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = ChaChaNonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher.encrypt(nonce, plaintext)?;
            
            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
            })
        }
    }
}

pub fn decrypt_data(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
    cipher_type: CipherType,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match cipher_type {
        CipherType::Aes256Gcm => {
            if key.len() != 32 {
                return Err("AES-256-GCM requires 32-byte key".into());
            }
            if nonce.len() != AES_NONCE_SIZE {
                return Err(format!("AES-256-GCM requires {}-byte nonce", AES_NONCE_SIZE).into());
            }
            
            let cipher = Aes256Gcm::new_from_slice(key)?;
            let nonce = Nonce::from_slice(nonce);
            cipher.decrypt(nonce, ciphertext).map_err(|e| e.into())
        }
        
        CipherType::ChaCha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 requires 32-byte key".into());
            }
            if nonce.len() != CHACHA_NONCE_SIZE {
                return Err(format!("ChaCha20Poly1305 requires {}-byte nonce", CHACHA_NONCE_SIZE).into());
            }
            
            let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
            let nonce = ChaChaNonce::from_slice(nonce);
            cipher.decrypt(nonce, ciphertext).map_err(|e| e.into())
        }
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aes_encryption_decryption() {
        let key = generate_random_key();
        let plaintext = b"Test secret message";
        
        let encrypted = encrypt_data(plaintext, &key, CipherType::Aes256Gcm).unwrap();
        let decrypted = decrypt_data(&encrypted.ciphertext, &key, &encrypted.nonce, CipherType::Aes256Gcm).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_chacha_encryption_decryption() {
        let key = generate_random_key();
        let plaintext = b"Another secret message";
        
        let encrypted = encrypt_data(plaintext, &key, CipherType::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt_data(&encrypted.ciphertext, &key, &encrypted.nonce, CipherType::ChaCha20Poly1305).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_wrong_key_fails() {
        let key = generate_random_key();
        let wrong_key = generate_random_key();
        let plaintext = b"Secret data";
        
        let encrypted = encrypt_data(plaintext, &key, CipherType::Aes256Gcm).unwrap();
        let result = decrypt_data(&encrypted.ciphertext, &wrong_key, &encrypted.nonce, CipherType::Aes256Gcm);
        
        assert!(result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        FileEncryptor { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret data for encryption test";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
}