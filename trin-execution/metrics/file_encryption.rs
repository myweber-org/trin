
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in &mut buffer {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, this is a secret message!";
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();

        fs::write(input_temp.path(), original_content).unwrap();

        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            Some(0xAA),
        ).unwrap();

        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            Some(0xAA),
        ).unwrap();

        let decrypted_content = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_default_key() {
        let original_content = b"Test with default key";
        let input_temp = NamedTempFile::new().unwrap();
        let encrypted_temp = NamedTempFile::new().unwrap();
        let decrypted_temp = NamedTempFile::new().unwrap();

        fs::write(input_temp.path(), original_content).unwrap();

        encrypt_file(
            input_temp.path().to_str().unwrap(),
            encrypted_temp.path().to_str().unwrap(),
            None,
        ).unwrap();

        decrypt_file(
            encrypted_temp.path().to_str().unwrap(),
            decrypted_temp.path().to_str().unwrap(),
            None,
        ).unwrap();

        let decrypted_content = fs::read(decrypted_temp.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use std::fs;
use std::io::{self, Read, Write};
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

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            self.xor_transform(&mut processed_buffer, &mut key_index);

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    fn xor_transform(&self, data: &mut [u8], key_index: &mut usize) {
        for byte in data.iter_mut() {
            *byte ^= self.key[*key_index];
            *key_index = (*key_index + 1) % self.key.len();
        }
    }
}

pub fn validate_key(key: &str) -> Result<(), &'static str> {
    if key.is_empty() {
        return Err("Encryption key cannot be empty");
    }
    if key.len() < 8 {
        return Err("Encryption key must be at least 8 characters long");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let cipher = XORCipher::new("secret_key_123");
        let test_data = b"Hello, World!";
        
        let mut encrypted = test_data.to_vec();
        let mut key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);
        
        key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);
        
        assert_eq!(encrypted, test_data);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = XORCipher::new("test_password");
        
        let mut source_file = NamedTempFile::new()?;
        source_file.write_all(b"Test file content")?;
        
        let dest_file = NamedTempFile::new()?;
        
        cipher.encrypt_file(source_file.path(), dest_file.path())?;
        
        let mut encrypted_content = Vec::new();
        fs::File::open(dest_file.path())?.read_to_end(&mut encrypted_content)?;
        
        assert_ne!(encrypted_content, b"Test file content");
        Ok(())
    }

    #[test]
    fn test_validate_key() {
        assert!(validate_key("").is_err());
        assert!(validate_key("short").is_err());
        assert!(validate_key("valid_long_key").is_ok());
    }
}