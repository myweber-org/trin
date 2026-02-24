
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

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
    println!("File processed successfully with key: 0x{:02X}", DEFAULT_KEY);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_cipher_symmetry() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xCC;
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let input_content = b"Hello, World!";
        let input_file = NamedTempFile::new()?;
        fs::write(input_file.path(), input_content)?;
        
        let output_file = NamedTempFile::new()?;
        
        process_file(input_file.path(), output_file.path(), DEFAULT_KEY)?;
        
        let encrypted_content = fs::read(output_file.path())?;
        assert_ne!(encrypted_content, input_content);
        
        let mut double_encrypted = encrypted_content.clone();
        xor_cipher(&mut double_encrypted, DEFAULT_KEY);
        assert_eq!(double_encrypted, input_content);
        
        Ok(())
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: &[u8] = b"secret-passphrase-123";

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<&[u8]>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    let encrypted_data = xor_cipher(&data, key);
    fs::write(output_path, encrypted_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<&[u8]>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    let key_len = key.len();
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key_len])
        .collect()
}

pub fn process_files() -> io::Result<()> {
    let test_data = b"Hello, this is a secret message!";
    let test_file = "test_original.txt";
    let encrypted_file = "test_encrypted.bin";
    let decrypted_file = "test_decrypted.txt";

    fs::write(test_file, test_data)?;
    
    println!("Encrypting file...");
    encrypt_file(test_file, encrypted_file, None)?;
    
    println!("Decrypting file...");
    decrypt_file(encrypted_file, decrypted_file, None)?;
    
    let restored_data = fs::read(decrypted_file)?;
    
    if test_data == &restored_data[..] {
        println!("Encryption/decryption successful!");
        
        fs::remove_file(test_file)?;
        fs::remove_file(encrypted_file)?;
        fs::remove_file(decrypted_file)?;
    } else {
        eprintln!("Error: Data mismatch after decryption");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_cipher_symmetry() {
        let data = b"Test data for encryption";
        let key = b"my-key";
        
        let encrypted = xor_cipher(data, key);
        let decrypted = xor_cipher(&encrypted, key);
        
        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_different_keys() {
        let data = b"Sensitive information";
        let key1 = b"key-one";
        let key2 = b"key-two";
        
        let encrypted1 = xor_cipher(data, key1);
        let encrypted2 = xor_cipher(data, key2);
        
        assert_ne!(encrypted1, encrypted2);
    }
}use std::fs;
use std::io::{self, Read, Write};

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    
    let mut buffer = [0u8; 1024];
    let key_len = key.len();
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for i in 0..bytes_read {
            buffer[i] ^= key[i % key_len];
        }
        
        output_file.write_all(&buffer[..bytes_read])?;
    }
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, Rust!";
        let key = b"secret";
        
        fs::write("test_input.txt", test_data).unwrap();
        
        encrypt_file("test_input.txt", "test_encrypted.txt", key).unwrap();
        decrypt_file("test_encrypted.txt", "test_decrypted.txt", key).unwrap();
        
        let decrypted = fs::read("test_decrypted.txt").unwrap();
        assert_eq!(decrypted, test_data);
        
        fs::remove_file("test_input.txt").unwrap_or_default();
        fs::remove_file("test_encrypted.txt").unwrap_or_default();
        fs::remove_file("test_decrypted.txt").unwrap_or_default();
    }
}