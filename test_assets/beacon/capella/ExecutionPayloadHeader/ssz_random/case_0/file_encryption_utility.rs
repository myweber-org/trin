
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2, Params
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, Some(32)).map_err(|e| e.to_string())?
    );
    
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;
    
    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Invalid hash length")?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str
) -> Result<EncryptionResult, String> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| e.to_string())?
        .read_to_end(&mut file_data)
        .map_err(|e| e.to_string())?;

    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let ciphertext = cipher.encrypt(nonce, file_data.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| e.to_string())?;
    output_file.write_all(&ciphertext)
        .map_err(|e| e.to_string())?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE]
) -> Result<Vec<u8>, String> {
    let mut encrypted_data = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| e.to_string())?
        .read_to_end(&mut encrypted_data)
        .map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| e.to_string())?;

    fs::File::create(output_path)
        .map_err(|e| e.to_string())?
        .write_all(&plaintext)
        .map_err(|e| e.to_string())?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption and decryption data";
        let password = "secure_password_123";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let output_encrypted = NamedTempFile::new().unwrap();
        let output_decrypted = NamedTempFile::new().unwrap();
        
        let enc_result = encrypt_file(
            input_file.path(),
            output_encrypted.path(),
            password
        ).unwrap();
        
        let decrypted = decrypt_file(
            output_encrypted.path(),
            output_decrypted.path(),
            password,
            &enc_result.nonce,
            &enc_result.salt
        ).unwrap();
        
        assert_eq!(decrypted, test_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub enum EncryptionError {
    IoError(std::io::Error),
    CryptoError(String),
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::IoError(err)
    }
}

pub struct FileEncryptor {
    algorithm: EncryptionAlgorithm,
}

pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;

        let encrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.aes_encrypt(&file_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_encrypt(&file_data, key)?,
        };

        fs::File::create(output_path)?.write_all(&encrypted_data)?;
        Ok(())
    }

    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        key: &[u8],
    ) -> Result<(), EncryptionError> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;

        let decrypted_data = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.aes_decrypt(&encrypted_data, key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.chacha_decrypt(&encrypted_data, key)?,
        };

        fs::File::create(output_path)?.write_all(&decrypted_data)?;
        Ok(())
    }

    fn aes_encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError(
                "AES-256-GCM requires 32-byte key".to_string(),
            ));
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::generate(&mut OsRng);

        cipher
            .encrypt(&nonce, data)
            .map(|mut ciphertext| {
                ciphertext.splice(0..0, nonce.iter().copied());
                ciphertext
            })
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn aes_decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError(
                "AES-256-GCM requires 32-byte key".to_string(),
            ));
        }

        if data.len() < 12 {
            return Err(EncryptionError::CryptoError(
                "Encrypted data too short".to_string(),
            ));
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn chacha_encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError(
                "ChaCha20Poly1305 requires 32-byte key".to_string(),
            ));
        }

        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let nonce = ChaChaNonce::generate(&mut OsRng);

        cipher
            .encrypt(&nonce, data)
            .map(|mut ciphertext| {
                ciphertext.splice(0..0, nonce.iter().copied());
                ciphertext
            })
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }

    fn chacha_decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::CryptoError(
                "ChaCha20Poly1305 requires 32-byte key".to_string(),
            ));
        }

        if data.len() < 12 {
            return Err(EncryptionError::CryptoError(
                "Encrypted data too short".to_string(),
            ));
        }

        let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(key));
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = ChaChaNonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_roundtrip() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let test_data = b"Hello, AES encryption!";
        let key = [0x42u8; 32];

        let encrypted = encryptor.aes_encrypt(test_data, &key).unwrap();
        let decrypted = encryptor.aes_decrypt(&encrypted, &key).unwrap();

        assert_eq!(test_data, decrypted.as_slice());
    }

    #[test]
    fn test_chacha_encryption_roundtrip() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let test_data = b"Hello, ChaCha encryption!";
        let key = [0x42u8; 32];

        let encrypted = encryptor.chacha_encrypt(test_data, &key).unwrap();
        let decrypted = encryptor.chacha_decrypt(&encrypted, &key).unwrap();

        assert_eq!(test_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let key = [0x42u8; 32];

        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), b"Test file content").unwrap();

        encryptor
            .encrypt_file(input_file.path(), output_file.path(), &key)
            .unwrap();
        encryptor
            .decrypt_file(output_file.path(), decrypted_file.path(), &key)
            .unwrap();

        let original_content = fs::read(input_file.path()).unwrap();
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();

        assert_eq!(original_content, decrypted_content);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_position: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_position: 0,
        }
    }

    pub fn encrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;
        
        let mut buffer = [0u8; 4096];
        
        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            let processed_data = self.process_chunk(&buffer[..bytes_read]);
            dest_file.write_all(&processed_data)?;
        }
        
        self.reset_key_position();
        Ok(())
    }

    fn process_chunk(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .map(|byte| {
                let key_byte = self.key[self.key_position];
                self.key_position = (self.key_position + 1) % self.key.len();
                byte ^ key_byte
            })
            .collect()
    }

    fn reset_key_position(&mut self) {
        self.key_position = 0;
    }
}

pub fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty".to_string());
    }
    
    if key.len() < 8 {
        return Err("Encryption key should be at least 8 characters long".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let test_data = b"Hello, World! This is a test message.";
        let key = "SuperSecretKey123!";
        
        let mut cipher = XorCipher::new(key);
        
        let encrypted: Vec<u8> = cipher.process_chunk(test_data);
        cipher.reset_key_position();
        let decrypted: Vec<u8> = cipher.process_chunk(&encrypted);
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let original_content = b"Sample file content for encryption testing.";
        let key = "AnotherSecretKey456!";
        
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let mut cipher = XorCipher::new(key);
        
        cipher.encrypt_file(source_file.path(), encrypted_file.path()).unwrap();
        cipher.reset_key_position();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();
        
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("ValidKey123").is_ok());
        assert!(validate_key("Short").is_err());
        assert!(validate_key("").is_err());
    }
}