
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

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

pub fn process_file() -> io::Result<()> {
    let input = "secret.txt";
    let encrypted = "secret.enc";
    let decrypted = "secret_decrypted.txt";
    
    if !Path::new(input).exists() {
        let sample_data = b"This is confidential information.";
        fs::write(input, sample_data)?;
        println!("Created sample file: {}", input);
    }
    
    encrypt_file(input, encrypted, Some(0xCC))?;
    println!("File encrypted: {}", encrypted);
    
    decrypt_file(encrypted, decrypted, Some(0xCC))?;
    println!("File decrypted: {}", decrypted);
    
    let original = fs::read(input)?;
    let restored = fs::read(decrypted)?;
    
    if original == restored {
        println!("Encryption/decryption successful!");
    } else {
        eprintln!("Error: Decrypted content doesn't match original!");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_encryption() {
        let test_data = b"Hello, World!";
        let key = 0x55;
        
        let encrypted: Vec<u8> = test_data.iter()
            .map(|byte| byte ^ key)
            .collect();
        
        let decrypted: Vec<u8> = encrypted.iter()
            .map(|byte| byte ^ key)
            .collect();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let mut input_file = NamedTempFile::new()?;
        let mut output_file = NamedTempFile::new()?;
        let mut result_file = NamedTempFile::new()?;
        
        let test_content = b"Test file content";
        input_file.write_all(test_content)?;
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(0x77)
        )?;
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            result_file.path().to_str().unwrap(),
            Some(0x77)
        )?;
        
        let result_content = fs::read(result_file.path())?;
        assert_eq!(test_content, result_content.as_slice());
        
        Ok(())
    }
}