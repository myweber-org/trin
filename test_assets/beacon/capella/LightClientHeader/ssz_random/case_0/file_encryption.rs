
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
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        xor_cipher(&mut data, 0xAA);
        assert_eq!(data, vec![0xAA, 0x55, 0xFF, 0x00]);
        xor_cipher(&mut data, 0xAA);
        assert_eq!(data, vec![0x00, 0xFF, 0x55, 0xAA]);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original = b"Secret data";
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        fs::write(input_file.path(), original)?;
        encrypt_file(input_file.path(), output_file.path(), Some(0xCC))?;
        decrypt_file(output_file.path(), decrypted_file.path(), Some(0xCC))?;

        let restored = fs::read(decrypted_file.path())?;
        assert_eq!(original.as_slice(), &restored);
        Ok(())
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn encrypt_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    let key_bytes = key.as_bytes();
    let mut content = fs::read(input_path)?;
    
    xor_cipher(&mut content, key_bytes);
    
    fs::write(output_path, &content)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_symmetry() {
        let key = b"secret";
        let mut data = b"hello world".to_vec();
        let original = data.clone();
        
        xor_cipher(&mut data, key);
        assert_ne!(data, original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key";
        let content = b"confidential data";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), content)?;
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            key,
        )?;
        
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, content);
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        )?;
        
        let decrypted = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted, content);
        
        Ok(())
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