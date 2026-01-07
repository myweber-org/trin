
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(key: &str) -> Self {
        FileEncryptor {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut encrypted_buffer = buffer[..bytes_read].to_vec();
            for byte in &mut encrypted_buffer {
                *byte ^= self.key[key_index];
                key_index = (key_index + 1) % self.key.len();
            }

            dest_file.write_all(&encrypted_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.encrypt_file(source_path, dest_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new("secret_key");
        let original_content = b"Hello, this is a test message for encryption!";

        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), original_content).unwrap();

        encryptor
            .encrypt_file(source_file.path(), encrypted_file.path())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct FileEncryptor {
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(key: &str) -> Self {
        Self {
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

            if !is_encrypt {
                for i in 0..bytes_read {
                    buffer[i] ^= self.key[key_index];
                    key_index = (key_index + 1) % self.key.len();
                }
            }

            output_file.write_all(&buffer[..bytes_read])
                .map_err(|e| format!("Failed to write to output file: {}", e))?;
        }

        Ok(())
    }

    pub fn encrypt_string(&self, input: &str) -> Vec<u8> {
        self.process_bytes(input.as_bytes(), true)
    }

    pub fn decrypt_string(&self, encrypted: &[u8]) -> String {
        let decrypted = self.process_bytes(encrypted, false);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn process_bytes(&self, data: &[u8], is_encrypt: bool) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        let mut key_index = 0;

        for &byte in data {
            let processed = byte ^ self.key[key_index];
            result.push(processed);
            key_index = (key_index + 1) % self.key.len();
        }

        if !is_encrypt {
            let mut final_result = Vec::with_capacity(result.len());
            key_index = 0;
            for &byte in &result {
                let processed = byte ^ self.key[key_index];
                final_result.push(processed);
                key_index = (key_index + 1) % self.key.len();
            }
            final_result
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption_decryption() {
        let encryptor = FileEncryptor::new("secret_key");
        let original = "Hello, World!";
        
        let encrypted = encryptor.encrypt_string(original);
        let decrypted = encryptor.decrypt_string(&encrypted);
        
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_file_encryption_decryption() {
        let encryptor = FileEncryptor::new("test_key");
        let test_content = b"Test file content for encryption";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }
}