
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs::{self, File};
use std::io::{Read, Write};

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let nonce = Nonce::from_slice(b"unique_nonce_");
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read encrypted data: {}", e))?;

        let nonce = Nonce::from_slice(b"unique_nonce_");
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }
}

pub fn generate_secure_filename(base_name: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    format!("{}_{:x}.enc", base_name, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        
        let mut temp_input = NamedTempFile::new().unwrap();
        let test_data = b"Test encryption data";
        temp_input.write_all(test_data).unwrap();
        
        let encrypted_path = "test_encrypted.tmp";
        let decrypted_path = "test_decrypted.tmp";
        
        encryptor.encrypt_file(temp_input.path().to_str().unwrap(), encrypted_path)
            .expect("Encryption should succeed");
        
        encryptor.decrypt_file(encrypted_path, decrypted_path)
            .expect("Decryption should succeed");
        
        let mut decrypted_data = Vec::new();
        File::open(decrypted_path)
            .unwrap()
            .read_to_end(&mut decrypted_data)
            .unwrap();
        
        assert_eq!(decrypted_data, test_data);
        
        fs::remove_file(encrypted_path).ok();
        fs::remove_file(decrypted_path).ok();
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W, key: u8) -> io::Result<()> {
    let mut buffer = [0; 1024];
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for byte in buffer[..bytes_read].iter_mut() {
            *byte ^= key;
        }
        
        writer.write_all(&buffer[..bytes_read])?;
    }
    
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Test data for encryption";
        let key = 0x55;
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(key)
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(key)
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data, decrypted_data.as_slice());
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer.iter().map(|byte| byte ^ encryption_key).collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, operation: fn(&str, &str, Option<u8>) -> io::Result<()>, key: Option<u8>) -> io::Result<()> {
    let path = Path::new(dir_path);
    if !path.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Provided path is not a directory"));
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let file_path = entry.path();
        
        if file_path.is_file() {
            let input_str = file_path.to_str().unwrap();
            let output_str = format!("{}.processed", input_str);
            
            operation(input_str, &output_str, key)?;
            println!("Processed: {}", input_str);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, World!";
        let key = Some(0x55);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(input_file.path().to_str().unwrap(), 
                    encrypted_file.path().to_str().unwrap(), 
                    key).unwrap();
        
        decrypt_file(encrypted_file.path().to_str().unwrap(), 
                    decrypted_file.path().to_str().unwrap(), 
                    key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs::{self, File};
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

#[derive(Debug, Clone, Copy)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FileEncryptor {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let (ciphertext, nonce) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes(&plaintext)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&plaintext)?,
        };

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), EncryptionError> {
        let mut file = File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let (nonce, ciphertext) = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                if data.len() < 12 {
                    return Err(EncryptionError::CryptoError("Invalid encrypted data".into()));
                }
                (&data[0..12], &data[12..])
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                if data.len() < 12 {
                    return Err(EncryptionError::CryptoError("Invalid encrypted data".into()));
                }
                (&data[0..12], &data[12..])
            }
        };

        let plaintext = match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(ciphertext, nonce)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(ciphertext, nonce)?,
        };

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }

    fn encrypt_aes(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::generate(&mut OsRng);
        
        cipher.encrypt(&nonce, plaintext)
            .map(|ciphertext| (ciphertext, nonce.to_vec()))
            .map_err(|e| EncryptionError::CryptoError(format!("AES encryption failed: {}", e)))
    }

    fn decrypt_aes(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = Key::<Aes256Gcm>::generate(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(format!("AES decryption failed: {}", e)))
    }

    fn encrypt_chacha(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        let key = ChaChaKey::generate(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = ChaChaNonce::generate(&mut OsRng);
        
        cipher.encrypt(&nonce, plaintext)
            .map(|ciphertext| (ciphertext, nonce.to_vec()))
            .map_err(|e| EncryptionError::CryptoError(format!("ChaCha encryption failed: {}", e)))
    }

    fn decrypt_chacha(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key = ChaChaKey::generate(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = ChaChaNonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::CryptoError(format!("ChaCha decryption failed: {}", e)))
    }
}

pub fn generate_key_file(path: &Path) -> Result<(), EncryptionError> {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    
    let mut file = File::create(path)?;
    file.write_all(&key)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::Aes256Gcm);
        let test_data = b"Hello, World! This is a test message.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(EncryptionAlgorithm::ChaCha20Poly1305);
        let test_data = b"Another test with different algorithm.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}