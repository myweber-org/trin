use std::fs::{File, read, write};
use std::io::{Read, Write};
use std::path::Path;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), String> {
    let data = read(input_path).map_err(|e| format!("Failed to read input file: {}", e))?;
    let encrypted = xor_cipher(&data, key);
    write(output_path, encrypted).map_err(|e| format!("Failed to write output file: {}", e))?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> Result<(), String> {
    encrypt_file(input_path, output_path, key)
}

fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let data = b"Test data for encryption";
        let key = b"secret";
        
        let encrypted = xor_cipher(data, key);
        let decrypted = xor_cipher(&encrypted, key);
        
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let key = b"test_key_123";
        
        let test_data = b"Sample content for file encryption test";
        std::fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            key,
        ).unwrap();
        
        let encrypted = std::fs::read(output_file.path()).unwrap();
        assert_ne!(test_data, encrypted.as_slice());
        
        decrypt_file(
            output_file.path().to_str().unwrap(),
            input_file.path().to_str().unwrap(),
            key,
        ).unwrap();
        
        let decrypted = std::fs::read(input_file.path()).unwrap();
        assert_eq!(test_data, decrypted.as_slice());
    }
}