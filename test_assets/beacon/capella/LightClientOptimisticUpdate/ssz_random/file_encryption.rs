use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: &[u8] = b"secret-encryption-key-2024";

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<&[u8]>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    let encrypted_data = xor_cipher(&data, encryption_key);
    fs::write(output_path, encrypted_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<&[u8]>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

pub fn process_files() -> io::Result<()> {
    let test_data = b"Hello, this is a secret message!";
    let test_path = "test_data.bin";
    let encrypted_path = "encrypted.bin";
    let decrypted_path = "decrypted.bin";

    fs::write(test_path, test_data)?;
    println!("Original: {:?}", String::from_utf8_lossy(test_data));

    encrypt_file(test_path, encrypted_path, None)?;
    let encrypted = fs::read(encrypted_path)?;
    println!("Encrypted: {:?}", encrypted);

    decrypt_file(encrypted_path, decrypted_path, None)?;
    let decrypted = fs::read(decrypted_path)?;
    println!("Decrypted: {:?}", String::from_utf8_lossy(&decrypted));

    cleanup_files(&[test_path, encrypted_path, decrypted_path])
}

fn cleanup_files(files: &[&str]) -> io::Result<()> {
    for file in files {
        if Path::new(file).exists() {
            fs::remove_file(file)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_cipher_symmetry() {
        let data = b"Test data for encryption";
        let key = b"mykey";
        let encrypted = xor_cipher(data, key);
        let decrypted = xor_cipher(&encrypted, key);
        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original = b"File content to protect";
        let test_file = "test_original.tmp";
        let encrypted_file = "test_encrypted.tmp";
        let decrypted_file = "test_decrypted.tmp";

        fs::write(test_file, original)?;
        encrypt_file(test_file, encrypted_file, Some(b"custom-key"))?;
        decrypt_file(encrypted_file, decrypted_file, Some(b"custom-key"))?;

        let result = fs::read(decrypted_file)?;
        assert_eq!(original, result.as_slice());

        cleanup_files(&[test_file, encrypted_file, decrypted_file])
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
    fs::write(format!("{}.key", output_path), key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    let key_bytes = fs::read(key_path)?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}