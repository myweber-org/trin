use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    Ok((ciphertext, nonce.to_vec()))
}

pub fn decrypt_data(ciphertext: &[u8], nonce: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret message for encryption test";
        let (ciphertext, nonce) = encrypt_data(original_data).unwrap();
        
        let key = Aes256Gcm::generate_key(&mut OsRng).to_vec();
        let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from_slice(&key));
        let nonce_obj = Nonce::from_slice(&nonce);
        
        let encrypted = cipher.encrypt(nonce_obj, original_data).unwrap();
        let decrypted = decrypt_data(&encrypted, &nonce, &key).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }
}
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer.iter()
        .map(|byte| byte ^ key)
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
    use std::fs;
    
    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, World!";
        let test_file = "test_input.txt";
        let encrypted_file = "test_encrypted.bin";
        let decrypted_file = "test_decrypted.txt";
        
        fs::write(test_file, test_data).unwrap();
        
        encrypt_file(test_file, encrypted_file, Some(0x42)).unwrap();
        decrypt_file(encrypted_file, decrypted_file, Some(0x42)).unwrap();
        
        let decrypted_content = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted_content, test_data);
        
        fs::remove_file(test_file).unwrap_or_default();
        fs::remove_file(encrypted_file).unwrap_or_default();
        fs::remove_file(decrypted_file).unwrap_or_default();
    }
}