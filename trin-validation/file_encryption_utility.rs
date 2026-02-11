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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path, is_encrypt: bool) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut output_file = fs::File::create(output_path)?;

        let mut buffer = [0u8; 4096];
        let operation_key = if is_encrypt { self.key } else { self.key };

        loop {
            let bytes_read = input_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for byte in buffer.iter_mut().take(bytes_read) {
                *byte ^= operation_key;
            }

            output_file.write_all(&buffer[..bytes_read])?;
        }

        output_file.flush()?;
        Ok(())
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|&byte| byte ^ self.key).collect()
    }

    pub fn decrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.encrypt_data(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new(Some(0xAA));
        let original_data = b"Hello, World!";
        
        let encrypted = encryptor.encrypt_data(original_data);
        let decrypted = encryptor.decrypt_data(&encrypted);
        
        assert_ne!(encrypted, original_data);
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_file_operations() -> io::Result<()> {
        let encryptor = FileEncryptor::new(None);
        
        let mut input_file = NamedTempFile::new()?;
        let test_data = b"Test file content";
        input_file.write_all(test_data)?;
        
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())?;
        encryptor.decrypt_file(encrypted_file.path(), decrypted_file.path())?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted_content, test_data);
        
        Ok(())
    }
}