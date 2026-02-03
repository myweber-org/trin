use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let key_bytes = key.as_bytes();
    let mut content = fs::read(input_path)?;
    
    xor_cipher(&mut content, key_bytes);
    
    fs::write(output_path, content)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input> <output> <key>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    let key = &args[3];
    
    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }
    
    process_file(input_path, output_path, key)?;
    println!("File processed successfully");
    Ok(())
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let original = b"Hello, World!";
        let mut data = original.to_vec();
        let key = 0x42;

        xor_cipher(&mut data, key);
        assert_ne!(data.as_slice(), original);

        xor_cipher(&mut data, key);
        assert_eq!(data.as_slice(), original);
    }

    #[test]
    fn test_file_encryption_decryption() -> io::Result<()> {
        let mut input_file = NamedTempFile::new()?;
        write!(input_file, "Secret data 123")?;

        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        let key = Some(0x7F);

        encrypt_file(input_file.path(), encrypted_file.path(), key)?;
        decrypt_file(encrypted_file.path(), decrypted_file.path(), key)?;

        let original = fs::read(input_file.path())?;
        let restored = fs::read(decrypted_file.path())?;

        assert_eq!(original, restored);
        Ok(())
    }
}