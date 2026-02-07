
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

fn main() -> io::Result<()> {
    let key = b"secret_key";
    let original_file = "test_data.txt";
    let encrypted_file = "encrypted.bin";
    let decrypted_file = "decrypted.txt";
    
    fs::write(original_file, "Sensitive information that needs protection.")?;
    
    println!("Encrypting file...");
    xor_encrypt_file(original_file, encrypted_file, key)?;
    
    println!("Decrypting file...");
    xor_decrypt_file(encrypted_file, decrypted_file, key)?;
    
    let decrypted_content = fs::read_to_string(decrypted_file)?;
    println!("Decrypted content: {}", decrypted_content);
    
    fs::remove_file(original_file)?;
    fs::remove_file(encrypted_file)?;
    fs::remove_file(decrypted_file)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_encryption_roundtrip() {
        let test_data = b"Test data for encryption";
        let key = b"test_key";
        let original_file = "test_original.tmp";
        let encrypted_file = "test_encrypted.tmp";
        let decrypted_file = "test_decrypted.tmp";
        
        fs::write(original_file, test_data).unwrap();
        xor_encrypt_file(original_file, encrypted_file, key).unwrap();
        xor_decrypt_file(encrypted_file, decrypted_file, key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
        
        fs::remove_file(original_file).unwrap_or_default();
        fs::remove_file(encrypted_file).unwrap_or_default();
        fs::remove_file(decrypted_file).unwrap_or_default();
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0u8; 4096];
        let key_len = self.key.len();
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % key_len;
            }

            dest_file.write_all(&buffer[..bytes_read])?;
        }

        Ok(())
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.process_data(data)
    }

    pub fn decrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.process_data(data)
    }

    fn process_data(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        let mut result = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            result.push(byte ^ self.key[i % key_len]);
        }

        result
    }
}

pub fn calculate_checksum(data: &[u8]) -> u32 {
    let mut checksum: u32 = 0;
    for &byte in data {
        checksum = checksum.wrapping_add(byte as u32);
    }
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("secret_key");
        let original_data = b"Hello, World! This is test data.";
        
        let encrypted = cipher.encrypt_data(original_data);
        let decrypted = cipher.decrypt_data(&encrypted);
        
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = XORCipher::new("test_key_123");
        
        let mut source_file = NamedTempFile::new()?;
        let test_data = b"Sample file content for encryption test";
        source_file.write_all(test_data)?;
        
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        cipher.encrypt_file(source_file.path(), encrypted_file.path())?;
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path())?;
        
        let decrypted_data = fs::read(decrypted_file.path())?;
        assert_eq!(test_data, decrypted_data.as_slice());
        
        Ok(())
    }

    #[test]
    fn test_checksum_calculation() {
        let data = b"Test data for checksum";
        let checksum = calculate_checksum(data);
        
        assert_eq!(checksum, 2189);
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
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, World!";
        let test_file = "test_input.txt";
        let encrypted_file = "test_encrypted.bin";
        let decrypted_file = "test_decrypted.txt";
        
        fs::write(test_file, test_data).unwrap();
        
        encrypt_file(test_file, encrypted_file, Some(0x42)).unwrap();
        decrypt_file(encrypted_file, decrypted_file, Some(0x42)).unwrap();
        
        let decrypted_content = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted_content, test_data);
        
        fs::remove_file(test_file).unwrap();
        fs::remove_file(encrypted_file).unwrap();
        fs::remove_file(decrypted_file).unwrap();
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test data";
        let test_file = "test_default.txt";
        let encrypted_file = "test_default_enc.bin";
        let decrypted_file = "test_default_dec.txt";
        
        fs::write(test_file, test_data).unwrap();
        
        encrypt_file(test_file, encrypted_file, None).unwrap();
        decrypt_file(encrypted_file, decrypted_file, None).unwrap();
        
        let decrypted_content = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted_content, test_data);
        
        fs::remove_file(test_file).unwrap();
        fs::remove_file(encrypted_file).unwrap();
        fs::remove_file(decrypted_file).unwrap();
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_file(
    plaintext: &[u8],
    key: &[u8; 32],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(ciphertext)
}

pub fn decrypt_file(
    ciphertext: &[u8],
    key: &[u8; 32],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let original_data = b"Secret file content";
        
        let encrypted = encrypt_file(original_data, &key).unwrap();
        let decrypted = decrypt_file(&encrypted, &key).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }
}