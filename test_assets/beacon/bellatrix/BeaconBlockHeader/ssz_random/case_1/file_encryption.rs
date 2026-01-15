
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data = xor_encrypt(&input_data, key);
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let decrypted_data = xor_encrypt(&input_data, key);
    fs::write(output_path, decrypted_data)?;
    Ok(())
}

fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
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
    fn test_xor_encryption_roundtrip() {
        let original_text = b"Hello, Rust encryption!";
        let key = b"secret_key";
        
        let encrypted = xor_encrypt(original_text, key);
        let decrypted = xor_encrypt(&encrypted, key);
        
        assert_eq!(original_text.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let temp_input = NamedTempFile::new()?;
        let temp_encrypted = NamedTempFile::new()?;
        let temp_decrypted = NamedTempFile::new()?;
        
        let test_data = b"Test file content for encryption";
        fs::write(temp_input.path(), test_data)?;
        
        let key = b"test_key_123";
        
        xor_encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap(),
            key,
        )?;
        
        xor_decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap(),
            key,
        )?;
        
        let decrypted_data = fs::read(temp_decrypted.path())?;
        assert_eq!(test_data.to_vec(), decrypted_data);
        
        Ok(())
    }
}