use std::fs;
use std::io::{self, Read, Write};

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    fn process_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let processed_data: Vec<u8> = buffer
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect();

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&processed_data)?;

        Ok(())
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }

    pub fn decrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.encrypt_data(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_cipher() {
        let cipher = XORCipher::new("secret_key");
        let original_data = b"Hello, World!";
        
        let encrypted = cipher.encrypt_data(original_data);
        assert_ne!(encrypted, original_data);
        
        let decrypted = cipher.decrypt_data(&encrypted);
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_file_encryption() {
        let test_content = b"Test file content for encryption";
        let test_file = "test_input.txt";
        let encrypted_file = "test_encrypted.bin";
        let decrypted_file = "test_decrypted.txt";
        
        fs::write(test_file, test_content).unwrap();
        
        let cipher = XORCipher::new("file_key");
        cipher.encrypt_file(test_file, encrypted_file).unwrap();
        cipher.decrypt_file(encrypted_file, decrypted_file).unwrap();
        
        let decrypted_content = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted_content, test_content);
        
        fs::remove_file(test_file).unwrap();
        fs::remove_file(encrypted_file).unwrap();
        fs::remove_file(decrypted_file).unwrap();
    }
}
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, World! This is a test message.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0x42),
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_file.path()).unwrap();
        assert_ne!(encrypted_content, original_content);
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x42),
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
    }
    
    #[test]
    fn test_default_key() {
        let original_content = b"Test with default key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None,
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None,
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, original_content);
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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let (ciphertext, key, nonce) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let key = Aes256Gcm::generate_key(&mut OsRng);
                let cipher = Aes256Gcm::new(&key);
                let nonce = Nonce::from_slice(b"unique_nonce_");
                let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
                (ciphertext, key.as_slice().to_vec(), nonce.to_vec())
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                let cipher = ChaCha20Poly1305::new(&key);
                let nonce = ChaChaNonce::from_slice(b"unique_nonce_");
                let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?;
                (ciphertext, key.as_slice().to_vec(), nonce.to_vec())
            }
        };

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&ciphertext)?;

        let key_file_path = output_path.with_extension("key");
        let mut key_file = fs::File::create(key_file_path)?;
        key_file.write_all(&key)?;

        let nonce_file_path = output_path.with_extension("nonce");
        let mut nonce_file = fs::File::create(nonce_file_path)?;
        nonce_file.write_all(&nonce)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, key_path: &Path, nonce_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut encrypted_file = fs::File::open(input_path)?;
        let mut ciphertext = Vec::new();
        encrypted_file.read_to_end(&mut ciphertext)?;

        let mut key_file = fs::File::open(key_path)?;
        let mut key = Vec::new();
        key_file.read_to_end(&mut key)?;

        let mut nonce_file = fs::File::open(nonce_path)?;
        let mut nonce = Vec::new();
        nonce_file.read_to_end(&mut nonce)?;

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let key = Key::<Aes256Gcm>::from_slice(&key);
                let cipher = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(&nonce);
                cipher.decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::from_slice(&key);
                let cipher = ChaCha20Poly1305::new(key);
                let nonce = ChaChaNonce::from_slice(&nonce);
                cipher.decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            }
        };

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
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let test_data = b"Test encryption data for AES-256-GCM";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), output_file.path()).unwrap();
        
        let key_path = output_file.path().with_extension("key");
        let nonce_path = output_file.path().with_extension("nonce");
        
        encryptor.decrypt_file(output_file.path(), &key_path, &nonce_path, decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let test_data = b"Test encryption data for ChaCha20Poly1305";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), output_file.path()).unwrap();
        
        let key_path = output_file.path().with_extension("key");
        let nonce_path = output_file.path().with_extension("nonce");
        
        encryptor.decrypt_file(output_file.path(), &key_path, &nonce_path, decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        PasswordHasher, SaltString, PasswordHash, PasswordVerifier
    },
    Pbkdf2
};
use rand::RngCore;
use std::fs;
use std::io::{self, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> io::Result<Self> {
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let password_hash = Pbkdf2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let hash_bytes = password_hash.hash.ok_or_else(|| 
            io::Error::new(io::ErrorKind::InvalidData, "No hash generated")
        )?;
        
        let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid key length"))?;
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
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
        
        let mut output = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);
        
        fs::write(output_path, &output)?;
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let data = fs::read(input_path)?;
        
        if data.len() < NONCE_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, 
                "File too short to contain nonce"
            ));
        }
        
        let nonce = Nonce::from_slice(&data[..NONCE_LENGTH]);
        let ciphertext = &data[NONCE_LENGTH..];
        
        let cipher = Aes256Gcm::new(&self.key);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        fs::write(output_path, &plaintext)?;
        Ok(())
    }
}

pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn verify_password(password: &str, salt: &[u8], stored_hash: &str) -> io::Result<bool> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    let result = Pbkdf2.verify_password(password.as_bytes(), &salt_string, &parsed_hash);
    Ok(result.is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() -> io::Result<()> {
        let password = "secure_password_123";
        let salt = generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt)?;
        
        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        )?;
        
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        )?;
        
        let decrypted_data = fs::read(decrypted_file.path())?;
        assert_eq!(test_data.to_vec(), decrypted_data);
        
        Ok(())
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption_roundtrip() {
        let test_data = b"Hello, XOR encryption!";
        let key = b"secret_key";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}