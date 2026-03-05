
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data
        .into_iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)
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
    
    println!("Enter encryption key (0-255, empty for default):");
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
    
    println!("Encrypt (e) or Decrypt (d)?");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    match choice.trim() {
        "e" | "E" => encrypt_file(input_path, output_path, key),
        "d" | "D" => decrypt_file(input_path, output_path, key),
        _ => {
            eprintln!("Invalid choice");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, World!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data, decrypted_data.as_slice());
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    
    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }
    
    process_file(input_path, output_path, DEFAULT_KEY)?;
    
    println!("File processed successfully with XOR key 0x{:02X}", DEFAULT_KEY);
    Ok(())
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8; 32]) -> Result<(), Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::generate(&mut OsRng);
    
    let mut file_content = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut file_content)?;
    
    let encrypted_data = cipher.encrypt(&nonce, file_content.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(nonce.as_slice())?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8; 32]) -> Result<(), Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    
    let mut encrypted_content = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut encrypted_content)?;
    
    if encrypted_content.len() < 12 {
        return Err("File too short to contain nonce".into());
    }
    
    let (nonce_slice, ciphertext) = encrypted_content.split_at(12);
    let nonce = Nonce::from_slice(nonce_slice);
    
    let decrypted_data = cipher.decrypt(nonce, ciphertext)
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
        let key = [0x42; 32];
        let test_data = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        
        let decrypted_file = NamedTempFile::new().unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), &key).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(decrypted_content, test_data);
    }
}