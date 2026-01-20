use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::io::{Read, Write};

const DEFAULT_KEY: &[u8] = b"secret-encryption-key-2024";

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<&[u8]>) -> Result<(), String> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let encrypted_data = xor_cipher(&buffer, encryption_key);
    let encoded_data = general_purpose::STANDARD.encode(encrypted_data);
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(encoded_data.as_bytes())
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<&[u8]>) -> Result<(), String> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut encoded_data = String::new();
    input_file.read_to_string(&mut encoded_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let encrypted_data = general_purpose::STANDARD.decode(encoded_data.trim())
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    let decrypted_data = xor_cipher(&encrypted_data, encryption_key);
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&decrypted_data)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    Ok(())
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
        let data = b"Hello, World!";
        let key = b"testkey";
        
        let encrypted = xor_cipher(data, key);
        let decrypted = xor_cipher(&encrypted, key);
        
        assert_eq!(data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_encryption_roundtrip() {
        let original_content = b"Sample file content for encryption test";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            Some(b"custom-key")
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            Some(b"custom-key")
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}