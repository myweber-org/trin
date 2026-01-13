
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

fn main() -> io::Result<()> {
    let key = b"secret_key";
    let original_file = "test_data.txt";
    let encrypted_file = "encrypted.bin";
    let decrypted_file = "decrypted.txt";
    
    fs::write(original_file, "Sensitive information that needs protection.")?;
    
    println!("Encrypting file...");
    xor_encrypt_file(original_file, encrypted_file, key)?;
    
    println!("Decrypting file...");
    xor_decrypt_file(encrypted_file, decrypted_file, key)?;
    
    let decrypted_content = fs::read_to_string(decrypted_file)?;
    println!("Decrypted content: {}", decrypted_content);
    
    fs::remove_file(original_file)?;
    fs::remove_file(encrypted_file)?;
    fs::remove_file(decrypted_file)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_encryption_roundtrip() {
        let test_data = b"Test data for encryption";
        let key = b"test_key";
        let original_file = "test_original.tmp";
        let encrypted_file = "test_encrypted.tmp";
        let decrypted_file = "test_decrypted.tmp";
        
        fs::write(original_file, test_data).unwrap();
        xor_encrypt_file(original_file, encrypted_file, key).unwrap();
        xor_decrypt_file(encrypted_file, decrypted_file, key).unwrap();
        
        let decrypted_data = fs::read(decrypted_file).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
        
        fs::remove_file(original_file).unwrap_or_default();
        fs::remove_file(encrypted_file).unwrap_or_default();
        fs::remove_file(decrypted_file).unwrap_or_default();
    }
}