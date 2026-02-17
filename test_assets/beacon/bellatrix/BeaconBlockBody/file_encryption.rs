
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_files() -> io::Result<()> {
    let test_data = b"Hello, this is a secret message!";
    let test_file = "test_original.txt";
    let encrypted_file = "test_encrypted.bin";
    let decrypted_file = "test_decrypted.txt";
    
    fs::write(test_file, test_data)?;
    
    println!("Encrypting file...");
    encrypt_file(test_file, encrypted_file, Some(0xAA))?;
    
    println!("Decrypting file...");
    decrypt_file(encrypted_file, decrypted_file, Some(0xAA))?;
    
    let decrypted_content = fs::read(decrypted_file)?;
    println!("Decrypted content matches original: {}", 
             decrypted_content == test_data);
    
    cleanup_files(&[test_file, encrypted_file, decrypted_file])?;
    
    Ok(())
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
    fn test_xor_encryption() {
        let data = vec![0x00, 0xFF, 0x55, 0xAA];
        let key = 0xCC;
        
        let encrypted: Vec<u8> = data.iter().map(|byte| byte ^ key).collect();
        let decrypted: Vec<u8> = encrypted.iter().map(|byte| byte ^ key).collect();
        
        assert_eq!(data, decrypted);
    }
    
    #[test]
    fn test_file_encryption_roundtrip() -> io::Result<()> {
        let original = "test_roundtrip.txt";
        let encrypted = "test_roundtrip.enc";
        let decrypted = "test_roundtrip.dec";
        
        let test_content = b"Roundtrip test data";
        fs::write(original, test_content)?;
        
        encrypt_file(original, encrypted, Some(0x77))?;
        decrypt_file(encrypted, decrypted, Some(0x77))?;
        
        let result = fs::read(decrypted)?;
        assert_eq!(test_content.as_slice(), result.as_slice());
        
        cleanup_files(&[original, encrypted, decrypted])?;
        Ok(())
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
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
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();
    
    println!("Enter operation (encrypt/decrypt):");
    let mut operation = String::new();
    io::stdin().read_line(&mut operation)?;
    let operation = operation.trim().to_lowercase();
    
    println!("Enter encryption key (0-255, press Enter for default):");
    let mut key_input = String::new();
    io::stdin().read_line(&mut key_input)?;
    let key_input = key_input.trim();
    
    let key = if key_input.is_empty() {
        None
    } else {
        match key_input.parse::<u8>() {
            Ok(k) => Some(k),
            Err(_) => {
                eprintln!("Invalid key, using default");
                None
            }
        }
    };
    
    if operation == "encrypt" {
        encrypt_file(input_path, output_path, key)?;
        println!("File encrypted successfully");
    } else if operation == "decrypt" {
        decrypt_file(input_path, output_path, key)?;
        println!("File decrypted successfully");
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid operation. Use 'encrypt' or 'decrypt'"
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, World!";
        let key = Some(0xAA);
        
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
    
    #[test]
    fn test_default_key() {
        let original_data = b"Test data";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}