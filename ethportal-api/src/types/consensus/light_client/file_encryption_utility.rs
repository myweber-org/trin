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

            for i in 0..bytes_read {
                buffer[i] ^= self.key[key_index];
                key_index = (key_index + 1) % key_len;
            }

            dest_file.write_all(&buffer[..bytes_read])?;
        }

        Ok(())
    }
}

pub fn calculate_file_hash(path: &Path) -> io::Result<u32> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0; 1024];
    let mut hash: u32 = 0x811c9dc5;

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        for &byte in &buffer[..bytes_read] {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
    }

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let key = "secret_key";
        let cipher = XorCipher::new(key);

        let mut source_file = NamedTempFile::new().unwrap();
        let original_data = b"Hello, this is a test message for encryption!";
        source_file.write_all(original_data).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        cipher
            .encrypt_file(source_file.path(), encrypted_file.path())
            .unwrap();

        let decrypted_file = NamedTempFile::new().unwrap();
        cipher
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_file_hash() {
        let mut test_file = NamedTempFile::new().unwrap();
        test_file.write_all(b"Test data for hash calculation").unwrap();

        let hash = calculate_file_hash(test_file.path()).unwrap();
        assert_ne!(hash, 0);
    }
}