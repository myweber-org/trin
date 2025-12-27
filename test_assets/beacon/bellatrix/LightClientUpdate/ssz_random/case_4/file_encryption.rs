
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileCipher {
    key: Vec<u8>,
}

impl FileCipher {
    pub fn new(key: &str) -> Self {
        FileCipher {
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
        let key_len = self.key.len();
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            for byte in processed_buffer.iter_mut() {
                *byte ^= self.key[key_index];
                key_index = (key_index + 1) % key_len;
            }

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_string(&self, input: &str) -> Vec<u8> {
        self.process_bytes(input.as_bytes())
    }

    pub fn decrypt_string(&self, encrypted: &[u8]) -> String {
        let decrypted = self.process_bytes(encrypted);
        String::from_utf8_lossy(&decrypted).to_string()
    }

    fn process_bytes(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_encryption() {
        let cipher = FileCipher::new("secret_key");
        let original = "Hello, World!";
        let encrypted = cipher.encrypt_string(original);
        let decrypted = cipher.decrypt_string(&encrypted);
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = FileCipher::new("test_key");
        let original_content = b"Test file content for encryption";
        
        let source_file = NamedTempFile::new()?;
        fs::write(source_file.path(), original_content)?;

        let dest_file = NamedTempFile::new()?;
        
        cipher.encrypt_file(source_file.path(), dest_file.path())?;
        
        let encrypted_content = fs::read(dest_file.path())?;
        assert_ne!(original_content, encrypted_content.as_slice());

        let decrypted_file = NamedTempFile::new()?;
        cipher.decrypt_file(dest_file.path(), decrypted_file.path())?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(original_content, decrypted_content.as_slice());

        Ok(())
    }
}