use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// XOR cipher implementation for file encryption/decryption
pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    /// Create a new cipher with the given key
    pub fn new(key: &[u8]) -> Self {
        XorCipher { key: key.to_vec() }
    }

    /// Process data using XOR cipher
    pub fn process(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }

    /// Encrypt/decrypt a file
    pub fn process_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let processed_data = self.process(&buffer);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&processed_data)?;

        Ok(())
    }
}

/// Generate a random key of specified length
pub fn generate_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_symmetry() {
        let key = b"secret_key";
        let cipher = XorCipher::new(key);
        let original_data = b"Hello, World! This is a test message.";

        let encrypted = cipher.process(original_data);
        let decrypted = cipher.process(&encrypted);

        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = generate_key(32);
        let cipher = XorCipher::new(&key);

        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let verification_file = NamedTempFile::new()?;

        let test_data = b"Test file content for encryption verification";
        fs::write(input_file.path(), test_data)?;

        cipher.process_file(input_file.path(), output_file.path())?;
        cipher.process_file(output_file.path(), verification_file.path())?;

        let final_content = fs::read(verification_file.path())?;
        assert_eq!(test_data.to_vec(), final_content);

        Ok(())
    }
}