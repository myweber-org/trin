
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output = key.to_vec();
    output.extend_from_slice(nonce);
    output.extend_from_slice(&encrypted_data);
    
    fs::write(output_path, output)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_content = fs::read(input_path)?;
    
    if encrypted_content.len() < 48 {
        return Err("Invalid encrypted file format".into());
    }
    
    let (key_bytes, rest) = encrypted_content.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(12);
    
    let key = key_bytes.try_into()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let decrypted_data = cipher.decrypt(nonce, ciphertext)
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
        let mut plain_file = NamedTempFile::new().unwrap();
        let plain_text = b"Test data for encryption roundtrip";
        plain_file.write_all(plain_text).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encrypt_file(plain_file.path().to_str().unwrap(), 
                    encrypted_file.path().to_str().unwrap()).unwrap();
        
        decrypt_file(encrypted_file.path().to_str().unwrap(),
                    decrypted_file.path().to_str().unwrap()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, plain_text);
    }
}
use std::fs;
use std::io::{self, Read, Write};

fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_cipher(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input> <output> <key>", args[0]);
        std::process::exit(1);
    }

    let key = args[3].as_bytes();
    process_file(&args[1], &args[2], key)?;

    println!("File processed successfully");
    Ok(())
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

pub fn generate_key_from_password(password: &str, length: usize) -> Vec<u8> {
    let mut key = Vec::with_capacity(length);
    let password_bytes = password.as_bytes();
    
    for i in 0..length {
        let byte = password_bytes[i % password_bytes.len()]
            .wrapping_add((i * 7) as u8)
            .rotate_left(3);
        key.push(byte);
    }
    
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Secret data that needs protection!";
        let password = "strong_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_text).unwrap();
        
        let key = generate_key_from_password(password, 32);
        
        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            &key
        ).unwrap();
        
        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            &key
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_key_generation() {
        let password = "test";
        let key1 = generate_key_from_password(password, 16);
        let key2 = generate_key_from_password(password, 16);
        
        assert_eq!(key1.len(), 16);
        assert_eq!(key1, key2);
        
        let different_key = generate_key_from_password("different", 16);
        assert_ne!(key1, different_key);
    }
}
use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_encryption() {
        let test_data = b"Hello, Rust!";
        let key = b"secret";
        let input_path = "test_input.txt";
        let encrypted_path = "test_encrypted.txt";
        let decrypted_path = "test_decrypted.txt";

        fs::write(input_path, test_data).unwrap();

        xor_encrypt_file(input_path, encrypted_path, key).unwrap();
        xor_decrypt_file(encrypted_path, decrypted_path, key).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(decrypted_data, test_data);

        fs::remove_file(input_path).unwrap_or_default();
        fs::remove_file(encrypted_path).unwrap_or_default();
        fs::remove_file(decrypted_path).unwrap_or_default();
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let plaintext = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_data = Vec::new();
    output_data.extend_from_slice(&key);
    output_data.extend_from_slice(nonce);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain valid encrypted data",
        ));
    }
    
    let (key_bytes, rest) = encrypted_data.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_SIZE);
    
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        
        let decrypted_file = NamedTempFile::new().unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}