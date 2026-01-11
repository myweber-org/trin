
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
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &str, output_path: &str, is_encrypt: bool) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let processed_data = self.xor_cipher(&buffer, is_encrypt);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&processed_data)?;

        Ok(())
    }

    fn xor_cipher(&self, data: &[u8], is_encrypt: bool) -> Vec<u8> {
        let key_length = self.key.len();
        if key_length == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| {
                let key_byte = self.key[i % key_length];
                byte ^ key_byte
            })
            .collect()
    }

    pub fn validate_key(&self, test_key: &str) -> bool {
        self.key == test_key.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new("secret_key");
        let test_content = b"Hello, this is a test message for encryption!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_content).unwrap();

        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();

        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_content.to_vec(), decrypted_content);
    }

    #[test]
    fn test_key_validation() {
        let encryptor = FileEncryptor::new("my_password");
        assert!(encryptor.validate_key("my_password"));
        assert!(!encryptor.validate_key("wrong_password"));
    }
}