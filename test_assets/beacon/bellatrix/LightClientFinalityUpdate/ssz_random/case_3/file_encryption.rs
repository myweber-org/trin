
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
        
        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), result);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_file_interactive() -> io::Result<()> {
    println!("Enter input file path:");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input_path = input.trim();
    
    println!("Enter output file path:");
    let mut output = String::new();
    io::stdin().read_line(&mut output)?;
    let output_path = output.trim();
    
    println!("Enter encryption key (0-255, empty for default):");
    let mut key_input = String::new();
    io::stdin().read_line(&mut key_input)?;
    
    let key = key_input.trim().parse::<u8>().ok();
    
    println!("Encrypt (e) or Decrypt (d)?");
    let mut mode = String::new();
    io::stdin().read_line(&mut mode)?;
    
    match mode.trim().to_lowercase().as_str() {
        "e" => encrypt_file(input_path, output_path, key),
        "d" => decrypt_file(input_path, output_path, key),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid mode selection"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, World!";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), original_data).unwrap();
        
        encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut content = fs::read(input_path)?;
    xor_cipher(&mut content, key);
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
    fn test_xor_cipher_symmetry() {
        let original = b"Hello, World!";
        let mut data = original.to_vec();
        let key = 0xCC;

        xor_cipher(&mut data, key);
        assert_ne!(data.as_slice(), original);

        xor_cipher(&mut data, key);
        assert_eq!(data.as_slice(), original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        let test_data = b"Secret data for encryption test";
        fs::write(input_file.path(), test_data)?;

        encrypt_file(input_file.path(), output_file.path(), Some(0xDD))?;
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted.as_slice(), test_data);

        decrypt_file(output_file.path(), decrypted_file.path(), Some(0xDD))?;
        let decrypted = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted.as_slice(), test_data);

        Ok(())
    }
}