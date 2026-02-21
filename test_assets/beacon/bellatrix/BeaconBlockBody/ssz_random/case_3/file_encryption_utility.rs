use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn xor_encrypt(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|byte| byte ^ key).collect()
}

pub fn xor_decrypt(data: &[u8], key: u8) -> Vec<u8> {
    xor_encrypt(data, key)
}

pub fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let processed_data = xor_encrypt(&buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;
    
    Ok(())
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let input = Path::new(input_path);
    let output = Path::new(output_path);
    process_file(input, output, DEFAULT_KEY)
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let input = Path::new(input_path);
    let output = Path::new(output_path);
    process_file(input, output, DEFAULT_KEY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_symmetry() {
        let original = b"Hello, World!";
        let encrypted = xor_encrypt(original, DEFAULT_KEY);
        let decrypted = xor_decrypt(&encrypted, DEFAULT_KEY);
        
        assert_eq!(original.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_operations() -> io::Result<()> {
        let test_data = b"Test file content for encryption";
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        )?;
        
        let encrypted_content = fs::read(output_file.path())?;
        assert_ne!(test_data.to_vec(), encrypted_content);
        
        let decrypted_file = NamedTempFile::new()?;
        decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        )?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(test_data.to_vec(), decrypted_content);
        
        Ok(())
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;
    output_file.write_all(&key)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key",
        ));
    }

    let (ciphertext, key_bytes) = encrypted_data.split_at(encrypted_data.len() - 32);
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Test data for encryption";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}