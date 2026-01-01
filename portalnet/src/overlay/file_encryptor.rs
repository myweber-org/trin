use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::io::{self, Read, Write};

pub struct FileEncryptor {
    key: Vec<u8>,
}

impl FileEncryptor {
    pub fn new(key: &str) -> Self {
        FileEncryptor {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let encrypted = self.xor_cipher(&buffer);
        let encoded = general_purpose::STANDARD.encode(encrypted);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(encoded.as_bytes())?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encoded = String::new();
        input_file.read_to_string(&mut encoded)?;

        let decoded = general_purpose::STANDARD.decode(encoded)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let decrypted = self.xor_cipher(&decoded);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&decrypted)?;
        Ok(())
    }

    fn xor_cipher(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new("secret_key");
        let test_data = b"Hello, Rust encryption!";
        
        let mut temp_input = NamedTempFile::new().unwrap();
        temp_input.write_all(test_data).unwrap();
        
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap()
        ).unwrap();
        
        encryptor.decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap()
        ).unwrap();
        
        let mut result = Vec::new();
        fs::File::open(temp_decrypted.path())
            .unwrap()
            .read_to_end(&mut result)
            .unwrap();
        
        assert_eq!(test_data, result.as_slice());
    }
}