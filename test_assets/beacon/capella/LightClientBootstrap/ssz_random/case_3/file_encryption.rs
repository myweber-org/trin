
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
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World! This is a test message.";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), original_text).unwrap();
        
        encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let encrypted_content = fs::read(temp_encrypted.path()).unwrap();
        assert_ne!(encrypted_content, original_text);
        
        decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let decrypted_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
    
    #[test]
    fn test_default_key() {
        let original_text = b"Test with default key";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_output = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), original_text).unwrap();
        
        encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let encrypted_content = fs::read(temp_output.path()).unwrap();
        assert_ne!(encrypted_content, original_text);
        
        let mut temp_final = NamedTempFile::new().unwrap();
        decrypt_file(
            temp_output.path().to_str().unwrap(),
            temp_final.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let final_content = fs::read(temp_final.path()).unwrap();
        assert_eq!(final_content, original_text);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;
    output_file.write_all(&key)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key and ciphertext",
        ));
    }

    let key_slice = &data[data.len() - 32..];
    let ciphertext = &data[..data.len() - 32];

    let key = Key::<Aes256Gcm>::from_slice(key_slice);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}