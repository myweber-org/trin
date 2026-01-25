use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: &[u8] = b"secret-encryption-key-2024";

pub struct FileCipher {
    key: Vec<u8>,
}

impl FileCipher {
    pub fn new(key: Option<&[u8]>) -> Self {
        let key = match key {
            Some(k) => k.to_vec(),
            None => DEFAULT_KEY.to_vec(),
        };
        FileCipher { key }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path, true)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path, false)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path, is_encrypt: bool) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0u8; 4096];
        let key_len = self.key.len();
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % key_len;
            }

            dest_file.write_all(&buffer[..bytes_read])?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.xor_transform(data)
    }

    pub fn decrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.xor_transform(data)
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_symmetric_encryption() {
        let cipher = FileCipher::new(Some(b"test-key"));
        let original = b"Hello, World! This is a test message.";
        
        let encrypted = cipher.encrypt_data(original);
        let decrypted = cipher.decrypt_data(&encrypted);
        
        assert_eq!(original, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = FileCipher::new(Some(b"file-test-key"));
        
        let mut source_file = NamedTempFile::new()?;
        let test_data = b"Test file content for encryption verification";
        source_file.write_all(test_data)?;
        
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        cipher.encrypt_file(source_file.path(), encrypted_file.path())?;
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path())?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(test_data, decrypted_content.as_slice());
        
        Ok(())
    }

    #[test]
    fn test_key_generation() {
        let key1 = generate_random_key(32);
        let key2 = generate_random_key(32);
        
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
        assert_ne!(key1, key2);
    }
}