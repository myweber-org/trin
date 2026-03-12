
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path, is_encrypt: bool) -> Result<(), String> {
        if self.key.is_empty() {
            return Err("Encryption key cannot be empty".to_string());
        }

        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        let mut buffer = [0u8; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = input_file.read(&mut buffer)
                .map_err(|e| format!("Failed to read from input file: {}", e))?;

            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % self.key.len();
            }

            output_file.write_all(&buffer[..bytes_read])
                .map_err(|e| format!("Failed to write to output file: {}", e))?;
        }

        Ok(())
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.xor_transform(data)
    }

    pub fn decrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.xor_transform(data)
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        let mut key_index = 0;

        for &byte in data {
            result.push(byte ^ self.key[key_index]);
            key_index = (key_index + 1) % self.key.len();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let cipher = XORCipher::new("secret_key");
        let original_data = b"Hello, World! This is a test message.";
        
        let encrypted = cipher.encrypt_data(original_data);
        let decrypted = cipher.decrypt_data(&encrypted);
        
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() {
        let cipher = XORCipher::new("test_key_123");
        let original_content = b"Sample file content for encryption testing.";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        cipher.encrypt_file(input_file.path(), output_file.path()).unwrap();
        
        let mut encrypted_content = Vec::new();
        let mut encrypted_file = fs::File::open(output_file.path()).unwrap();
        encrypted_file.read_to_end(&mut encrypted_content).unwrap();
        
        assert_ne!(original_content, encrypted_content.as_slice());
        
        let decrypted_file = NamedTempFile::new().unwrap();
        cipher.decrypt_file(output_file.path(), decrypted_file.path()).unwrap();
        
        let mut decrypted_content = Vec::new();
        let mut decrypted_file_handle = fs::File::open(decrypted_file.path()).unwrap();
        decrypted_file_handle.read_to_end(&mut decrypted_content).unwrap();
        
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        FileEncryptor { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }
}

pub fn generate_key_file(path: &Path) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let mut file = fs::File::create(path)?;
    file.write_all(&key)?;
    Ok(())
}

pub fn load_encryptor_from_key(key_path: &Path) -> io::Result<FileEncryptor> {
    let mut file = fs::File::open(key_path)?;
    let mut key_bytes = [0u8; 32];
    file.read_exact(&mut key_bytes)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    Ok(FileEncryptor { cipher })
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
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
        
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), original_text).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let encrypted_content = fs::read(encrypted_temp.path()).unwrap();
        assert_ne!(encrypted_content, original_text);
        
        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            Some(0x42)
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(decrypted_content, original_text);
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test data for default key";
        
        let input_temp = NamedTempFile::new().unwrap();
        let output_temp = NamedTempFile::new().unwrap();
        
        fs::write(input_temp.path(), test_data).unwrap();
        
        encrypt_file(
            input_temp.path().to_str().unwrap(),
            output_temp.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let encrypted = fs::read(output_temp.path()).unwrap();
        let decrypted: Vec<u8> = encrypted.iter().map(|b| b ^ DEFAULT_KEY).collect();
        
        assert_eq!(decrypted, test_data);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_position: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_position: 0,
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_position];
            result.push(byte ^ key_byte);
            self.key_position = (self.key_position + 1) % self.key.len();
        }
        
        result
    }

    pub fn reset(&mut self) {
        self.key_position = 0;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data = cipher.encrypt(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let decrypted_data = cipher.decrypt(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let mut cipher = XorCipher::new("secret_key");
        let original_data = b"Hello, World! This is a test message.";
        
        let encrypted = cipher.encrypt(original_data);
        cipher.reset();
        let decrypted = cipher.decrypt(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() -> io::Result<()> {
        let test_content = b"Test file content for encryption demonstration.";
        
        let input_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_content)?;
        
        encrypt_file(input_file.path(), encrypted_file.path(), "test_key123")?;
        decrypt_file(encrypted_file.path(), decrypted_file.path(), "test_key123")?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(test_content.to_vec(), decrypted_content);
        
        Ok(())
    }

    #[test]
    fn test_wrong_key_fails() -> io::Result<()> {
        let test_content = b"Sensitive data that should not be readable with wrong key.";
        
        let input_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_content)?;
        
        encrypt_file(input_file.path(), encrypted_file.path(), "correct_key")?;
        decrypt_file(encrypted_file.path(), decrypted_file.path(), "wrong_key")?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_ne!(test_content.to_vec(), decrypted_content);
        
        Ok(())
    }
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

        let mut input_file = File::open(input_path).map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

        let processed_data = self.xor_transform(&buffer);

        let mut output_file = File::create(output_path_obj).map_err(|e| e.to_string())?;
        output_file.write_all(&processed_data).map_err(|e| e.to_string())?;

        if is_encrypt {
            println!("File encrypted successfully: {}", output_path);
        } else {
            println!("File decrypted successfully: {}", output_path);
        }

        Ok(())
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_symmetry() {
        let cipher = XorCipher::new("test_key_123");
        let original_data = b"Hello, this is a secret message!";
        
        let encrypted = cipher.xor_transform(original_data);
        let decrypted = cipher.xor_transform(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let test_content = "Sample content for encryption test\nSecond line\nThird line";
        let test_file = "test_input.txt";
        let encrypted_file = "test_encrypted.bin";
        let decrypted_file = "test_decrypted.txt";
        
        fs::write(test_file, test_content).unwrap();
        
        let cipher = XorCipher::new("secure_password");
        
        cipher.encrypt_file(test_file, encrypted_file).unwrap();
        cipher.decrypt_file(encrypted_file, decrypted_file).unwrap();
        
        let decrypted_content = fs::read_to_string(decrypted_file).unwrap();
        assert_eq!(test_content, decrypted_content);
        
        fs::remove_file(test_file).unwrap_or_default();
        fs::remove_file(encrypted_file).unwrap_or_default();
        fs::remove_file(decrypted_file).unwrap_or_default();
    }
}