use std::fs;
use std::io::{self, Read, Write};
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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.xor_transform(&buffer);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.encrypt_file(input_path, output_path)
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn process_file_encryption(
    source_path: &str,
    dest_path: &str,
    key: &str,
    encrypt: bool,
) -> Result<(), String> {
    let cipher = XorCipher::new(key);
    let source = Path::new(source_path);
    let dest = Path::new(dest_path);

    if !source.exists() {
        return Err("Source file does not exist".to_string());
    }

    if encrypt {
        cipher
            .encrypt_file(source, dest)
            .map_err(|e| format!("Encryption failed: {}", e))
    } else {
        cipher
            .decrypt_file(source, dest)
            .map_err(|e| format!("Decryption failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XorCipher::new("test_key_123");
        let original_data = b"Hello, this is a secret message!";

        let encrypted = cipher.xor_transform(original_data);
        let decrypted = cipher.xor_transform(&encrypted);

        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_roundtrip() {
        let key = "strong_password";
        let cipher = XorCipher::new(key);

        let original_content = b"Confidential data: 42";
        let mut temp_input = NamedTempFile::new().unwrap();
        temp_input.write_all(original_content).unwrap();

        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        cipher
            .encrypt_file(temp_input.path(), temp_encrypted.path())
            .unwrap();
        cipher
            .decrypt_file(temp_encrypted.path(), temp_decrypted.path())
            .unwrap();

        let restored_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_content.to_vec(), restored_content);
    }
}use std::fs;
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

    pub fn encrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&mut self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        self.key_position = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[self.key_position];
                self.key_position = (self.key_position + 1) % self.key.len();
            }

            dest_file.write_all(&buffer[..bytes_read])?;
        }

        dest_file.flush()?;
        Ok(())
    }
}

pub fn validate_key(key: &str) -> Result<(), &'static str> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty");
    }
    if key.len() < 4 {
        return Err("Encryption key must be at least 4 characters long");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secure_encryption_key_2024";
        let mut cipher = XorCipher::new(key);

        let original_data = b"Hello, this is a secret message!";
        
        let mut temp_input = NamedTempFile::new().unwrap();
        temp_input.write_all(original_data).unwrap();
        
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        cipher.encrypt_file(temp_input.path(), temp_encrypted.path()).unwrap();
        
        let mut cipher2 = XorCipher::new(key);
        cipher2.decrypt_file(temp_encrypted.path(), temp_decrypted.path()).unwrap();

        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_key_validation() {
        assert!(validate_key("valid_key").is_ok());
        assert!(validate_key("").is_err());
        assert!(validate_key("abc").is_err());
    }
}