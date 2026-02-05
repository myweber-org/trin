
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

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

pub fn process_directory(dir_path: &str, key: Option<u8>, encrypt: bool) -> io::Result<()> {
    let path = Path::new(dir_path);
    
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            
            if file_path.is_file() {
                let input_str = file_path.to_str().unwrap();
                let output_str = if encrypt {
                    format!("{}.enc", input_str)
                } else {
                    input_str.trim_end_matches(".enc").to_string()
                };
                
                if encrypt {
                    encrypt_file(input_str, &output_str, key)?;
                    println!("Encrypted: {} -> {}", input_str, output_str);
                } else {
                    decrypt_file(input_str, &output_str, key)?;
                    println!("Decrypted: {} -> {}", input_str, output_str);
                }
            }
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
        let original_data = b"Hello, World!";
        let key = Some(0x55);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn encrypt_string(text: &str, key: Option<u8>) -> Vec<u8> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    text.bytes()
        .map(|byte| byte ^ encryption_key)
        .collect()
}

pub fn decrypt_string(data: &[u8], key: Option<u8>) -> String {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    data.iter()
        .map(|byte| (byte ^ encryption_key) as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_string_encryption() {
        let original = "Hello, World!";
        let key = Some(0x42);
        
        let encrypted = encrypt_string(original, key);
        let decrypted = decrypt_string(&encrypted, key);
        
        assert_eq!(original, decrypted);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original_content = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), original_content)?;
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(0x77)
        )?;
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x77)
        )?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(original_content.to_vec(), decrypted_content);
        
        Ok(())
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let data = fs::read(input_path)?;
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output)
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain nonce",
        ));
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let key = derive_key(password);
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)
}

fn derive_key(password: &str) -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; 32];
    let password_bytes = password.as_bytes();
    
    for (i, &byte) in password_bytes.iter().enumerate() {
        key_bytes[i % 32] ^= byte;
    }
    
    *Key::<Aes256Gcm>::from_slice(&key_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let plaintext = b"Secret data for encryption test";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> Result<(), String> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let input = Path::new(input_path);
    let output = Path::new(output_path);
    
    if !input.exists() {
        return Err(format!("Input file '{}' does not exist", input_path));
    }
    
    let mut file = match fs::File::open(input) {
        Ok(f) => f,
        Err(e) => return Err(format!("Failed to open input file: {}", e)),
    };
    
    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        return Err(format!("Failed to read input file: {}", e));
    }
    
    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }
    
    let mut output_file = match fs::File::create(output) {
        Ok(f) => f,
        Err(e) => return Err(format!("Failed to create output file: {}", e)),
    };
    
    if let Err(e) = output_file.write_all(&buffer) {
        return Err(format!("Failed to write output file: {}", e));
    }
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> Result<(), String> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, this is a test message for XOR encryption!";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let input_path = input_file.path().to_str().unwrap();
        let encrypted_path = encrypted_file.path().to_str().unwrap();
        let decrypted_path = decrypted_file.path().to_str().unwrap();
        
        assert!(encrypt_file(input_path, encrypted_path, Some(0x55)).is_ok());
        assert!(decrypt_file(encrypted_path, decrypted_path, Some(0x55)).is_ok());
        
        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Testing with default encryption key";
        
        let input_file = NamedTempFile::new().unwrap();
        let mut input_path = input_file.path().to_path_buf();
        input_path.set_extension("orig");
        let input_path_str = input_path.to_str().unwrap();
        
        fs::write(input_path_str, test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        let encrypted_path = encrypted_file.path().to_str().unwrap();
        let decrypted_path = decrypted_file.path().to_str().unwrap();
        
        assert!(encrypt_file(input_path_str, encrypted_path, None).is_ok());
        assert!(decrypt_file(encrypted_path, decrypted_path, None).is_ok());
        
        let result_data = fs::read(decrypted_path).unwrap();
        assert_eq!(test_data.to_vec(), result_data);
    }
}