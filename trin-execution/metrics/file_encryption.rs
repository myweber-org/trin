
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(b"unique nonce");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, &encrypted_data)?;
    
    let key_path = format!("{}.key", output_path);
    fs::write(key_path, &key)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    let key = fs::read(key_path)?;
    
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(b"unique nonce");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    
    Ok(())
}
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &str, output_path: &str, is_encrypt: bool) -> Result<(), String> {
        let input_path_obj = Path::new(input_path);
        let output_path_obj = Path::new(output_path);

        if !input_path_obj.exists() {
            return Err(format!("Input file does not exist: {}", input_path));
        }

        let mut input_file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let processed_data = self.xor_process(&buffer);

        let mut output_file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    fn xor_process(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen::<u8>()).collect()
}

pub fn save_key_to_file(key: &[u8], file_path: &str) -> Result<(), String> {
    let hex_string: String = key.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();

    fs::write(file_path, hex_string)
        .map_err(|e| format!("Failed to save key file: {}", e))
}

pub fn load_key_from_file(file_path: &str) -> Result<Vec<u8>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;

    let mut key = Vec::new();
    for i in (0..content.len()).step_by(2) {
        if i + 1 < content.len() {
            let byte_str = &content[i..i + 2];
            let byte = u8::from_str_radix(byte_str, 16)
                .map_err(|e| format!("Invalid hex in key file: {}", e))?;
            key.push(byte);
        }
    }

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let cipher = XorCipher::new("secret_key");
        let data = b"Hello, World!";
        let encrypted = cipher.xor_process(data);
        let decrypted = cipher.xor_process(&encrypted);
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let test_data = b"Test file content for encryption";
        let cipher = XorCipher::new("test_key");

        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        cipher.encrypt_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();

        cipher.decrypt_file(
            output_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
}