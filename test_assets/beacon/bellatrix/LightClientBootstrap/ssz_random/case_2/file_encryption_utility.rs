use std::fs::{self, File};
use std::io::{Read, Write};
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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> std::io::Result<()> {
        let mut input_file = File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.process_bytes(&buffer);

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> std::io::Result<()> {
        self.encrypt_file(input_path, output_path)
    }

    fn process_bytes(&self, data: &[u8]) -> Vec<u8> {
        let key_length = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_length])
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("secret_key");
        let original_data = b"Hello, XOR encryption!";
        
        let encrypted = cipher.process_bytes(original_data);
        let decrypted = cipher.process_bytes(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> std::io::Result<()> {
        let cipher = XORCipher::new("test_key");
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        let test_content = b"File encryption test data";
        fs::write(input_file.path(), test_content)?;

        cipher.encrypt_file(input_file.path(), output_file.path())?;
        cipher.decrypt_file(output_file.path(), decrypted_file.path())?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(test_content.to_vec(), decrypted_content);

        Ok(())
    }
}