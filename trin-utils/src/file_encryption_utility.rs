use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, operation: fn(&str, &str, Option<u8>) -> io::Result<()>, key: Option<u8>) -> io::Result<()> {
    let path = Path::new(dir_path);
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.is_file() {
                let input_str = file_path.to_str().unwrap();
                let output_str = format!("{}.processed", input_str);
                operation(input_str, &output_str, key)?;
                println!("Processed: {}", input_str);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data to protect";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        fs::write(temp_input.path(), original_content).unwrap();

        encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap(),
            Some(0xAA),
        ).unwrap();

        decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap(),
            Some(0xAA),
        ).unwrap();

        let decrypted_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}use std::fs;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("secret_key");
        let test_data = b"Hello, XOR encryption!";
        
        let mut encrypted = test_data.to_vec();
        let mut key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);
        
        key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);
        
        assert_eq!(encrypted, test_data);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = XORCipher::new("test_key_123");
        
        let source_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(source_file.path(), b"Test file content")?;
        
        cipher.encrypt_file(source_file.path(), encrypted_file.path())?;
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path())?;
        
        let original_content = fs::read(source_file.path())?;
        let decrypted_content = fs::read(decrypted_file.path())?;
        
        assert_eq!(original_content, decrypted_content);
        Ok(())
    }
}