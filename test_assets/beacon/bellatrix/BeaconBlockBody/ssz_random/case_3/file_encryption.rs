
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let input_data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, this is a secret message!";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), original_text).unwrap();
        
        xor_encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap(),
            Some(0xCC),
        ).unwrap();
        
        xor_decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap(),
            Some(0xCC),
        ).unwrap();
        
        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test data for default key";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_output = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), test_data).unwrap();
        
        xor_encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
            None,
        ).unwrap();
        
        let encrypted = fs::read(temp_output.path()).unwrap();
        assert_ne!(test_data.to_vec(), encrypted);
        
        let decrypted: Vec<u8> = encrypted.iter().map(|byte| byte ^ DEFAULT_KEY).collect();
        assert_eq!(test_data.to_vec(), decrypted);
    }
}use std::fs;
use std::io::{self, Read, Write};

const BUFFER_SIZE: usize = 8192;

pub fn xor_cipher(input: &[u8], key: &[u8]) -> Vec<u8> {
    input
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

pub fn process_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;

    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        let processed = xor_cipher(&buffer[..bytes_read], key);
        output_file.write_all(&processed)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_cipher_symmetry() {
        let data = b"Hello, World!";
        let key = b"secret";
        let encrypted = xor_cipher(data, key);
        let decrypted = xor_cipher(&encrypted, key);
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_empty_data() {
        let data = b"";
        let key = b"key";
        let result = xor_cipher(data, key);
        assert!(result.is_empty());
    }
}