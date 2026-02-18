
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, encrypted_data)?;
    fs::write(format!("{}.key", output_path), key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    let key_bytes = fs::read(key_path)?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let mut plaintext_file = NamedTempFile::new().unwrap();
        write!(plaintext_file, "Sensitive data: {}", "Test content").unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        let key_file = NamedTempFile::new().unwrap();
        
        let plain_path = plaintext_file.path().to_str().unwrap();
        let enc_path = encrypted_file.path().to_str().unwrap();
        let dec_path = decrypted_file.path().to_str().unwrap();
        let key_path = key_file.path().to_str().unwrap();
        
        encrypt_file(plain_path, enc_path).unwrap();
        fs::rename(format!("{}.key", enc_path), key_path).unwrap();
        
        decrypt_file(enc_path, key_path, dec_path).unwrap();
        
        let original = fs::read_to_string(plain_path).unwrap();
        let decrypted = fs::read_to_string(dec_path).unwrap();
        
        assert_eq!(original, decrypted);
    }
}
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> Result<(), String> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path, is_encrypt: bool) -> Result<(), String> {
        if self.key.is_empty() {
            return Err("Encryption key cannot be empty".to_string());
        }

        let mut source_file = File::open(source_path)
            .map_err(|e| format!("Failed to open source file: {}", e))?;

        let mut buffer = Vec::new();
        source_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read source file: {}", e))?;

        let processed_data: Vec<u8> = buffer.iter()
            .enumerate()
            .map(|(i, &byte)| {
                let key_byte = self.key[i % self.key.len()];
                byte ^ key_byte
            })
            .collect();

        let mut dest_file = File::create(dest_path)
            .map_err(|e| format!("Failed to create destination file: {}", e))?;

        dest_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write processed data: {}", e))?;

        Ok(())
    }

    pub fn encrypt_string(&self, text: &str) -> Vec<u8> {
        text.as_bytes()
            .iter()
            .enumerate()
            .map(|(i, &byte)| {
                let key_byte = self.key[i % self.key.len()];
                byte ^ key_byte
            })
            .collect()
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        let decrypted_bytes: Vec<u8> = data.iter()
            .enumerate()
            .map(|(i, &byte)| {
                let key_byte = self.key[i % self.key.len()];
                byte ^ key_byte
            })
            .collect();

        String::from_utf8_lossy(&decrypted_bytes).to_string()
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen::<u8>()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption_decryption() {
        let cipher = XorCipher::new("secret_key");
        let original_text = "Hello, World! This is a test message.";
        
        let encrypted = cipher.encrypt_string(original_text);
        let decrypted = cipher.decrypt_string(&encrypted);
        
        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let cipher = XorCipher::new("another_secret");
        let test_content = b"Sample file content for encryption testing.";
        
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(test_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        cipher.encrypt_file(source_file.path(), encrypted_file.path()).unwrap();
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_content = Vec::new();
        File::open(decrypted_file.path()).unwrap()
            .read_to_end(&mut decrypted_content).unwrap();
        
        assert_eq!(test_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_empty_key() {
        let cipher = XorCipher::new("");
        let temp_file = NamedTempFile::new().unwrap();
        
        let result = cipher.encrypt_file(temp_file.path(), temp_file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }
}
use std::fs;
use std::io::{self, Read, Write};

fn xor_encrypt_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt_decrypt(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input_file> <output_file> <key>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let key = args[3].as_bytes();

    process_file(input_path, output_path, key)?;
    println!("File processed successfully: {} -> {}", input_path, output_path);

    Ok(())
}