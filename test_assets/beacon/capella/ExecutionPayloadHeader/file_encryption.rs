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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let processed_data: Vec<u8> = buffer
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect();

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&processed_data)?;

        Ok(())
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
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let key = "secret_key";
        let cipher = XORCipher::new(key);

        let original_text = b"Hello, this is a test message for XOR encryption!";
        
        let mut temp_input = NamedTempFile::new().unwrap();
        temp_input.write_all(original_text).unwrap();
        
        let temp_output = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        cipher.encrypt_file(temp_input.path(), temp_output.path()).unwrap();
        cipher.decrypt_file(temp_output.path(), temp_decrypted.path()).unwrap();

        let decrypted_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_content);
    }

    #[test]
    fn test_random_key_generation() {
        let key_length = 32;
        let key1 = generate_random_key(key_length);
        let key2 = generate_random_key(key_length);

        assert_eq!(key1.len(), key_length);
        assert_eq!(key2.len(), key_length);
        assert_ne!(key1, key2);
    }
}