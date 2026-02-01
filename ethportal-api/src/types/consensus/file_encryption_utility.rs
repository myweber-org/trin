use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub struct FileEncryptor {
    key: u8,
}

impl FileEncryptor {
    pub fn new(key: Option<u8>) -> Self {
        FileEncryptor {
            key: key.unwrap_or(DEFAULT_KEY),
        }
    }

    pub fn encrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &str, dest_path: &str, is_encrypt: bool) -> io::Result<()> {
        let source = Path::new(source_path);
        let dest = Path::new(dest_path);

        if !source.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source file not found: {}", source_path),
            ));
        }

        let mut source_file = fs::File::open(source)?;
        let mut dest_file = fs::File::create(dest)?;

        let mut buffer = [0u8; 4096];
        let operation = if is_encrypt { "encrypting" } else { "decrypting" };

        println!("{} file: {} -> {}", operation, source_path, dest_path);

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let processed_buffer: Vec<u8> = buffer[..bytes_read]
                .iter()
                .map(|&byte| byte ^ self.key)
                .collect();

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        println!("{} completed successfully", operation);

        Ok(())
    }

    pub fn encrypt_string(&self, input: &str) -> Vec<u8> {
        input.bytes().map(|byte| byte ^ self.key).collect()
    }

    pub fn decrypt_string(&self, data: &[u8]) -> String {
        data.iter()
            .map(|&byte| (byte ^ self.key) as char)
            .collect()
    }
}

pub fn calculate_file_hash(path: &str) -> io::Result<u32> {
    let content = fs::read(path)?;
    let mut hash: u32 = 0;

    for byte in content {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_string() {
        let encryptor = FileEncryptor::new(Some(0xAA));
        let original = "Hello, World!";
        
        let encrypted = encryptor.encrypt_string(original);
        let decrypted = encryptor.decrypt_string(&encrypted);
        
        assert_ne!(encrypted, original.as_bytes());
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let encryptor = FileEncryptor::new(None);
        
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Test data for encryption";
        temp_file.write_all(test_data)?;
        
        let source_path = temp_file.path().to_str().unwrap();
        let encrypted_path = "/tmp/encrypted_test.bin";
        let decrypted_path = "/tmp/decrypted_test.txt";
        
        encryptor.encrypt_file(source_path, encrypted_path)?;
        encryptor.decrypt_file(encrypted_path, decrypted_path)?;
        
        let decrypted_content = fs::read(decrypted_path)?;
        assert_eq!(decrypted_content, test_data);
        
        fs::remove_file(encrypted_path).ok();
        fs::remove_file(decrypted_path).ok();
        
        Ok(())
    }

    #[test]
    fn test_hash_calculation() -> io::Result<()> {
        let temp_file = NamedTempFile::new()?;
        let test_data = b"Hash test data";
        fs::write(temp_file.path(), test_data)?;
        
        let hash = calculate_file_hash(temp_file.path().to_str().unwrap())?;
        assert!(hash > 0);
        
        Ok(())
    }
}