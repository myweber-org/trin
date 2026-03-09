use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// XOR-based file encryption/decryption utility
pub struct FileCipher {
    key: Vec<u8>,
}

impl FileCipher {
    /// Create a new cipher with the given key
    pub fn new(key: &str) -> Self {
        FileCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    /// Encrypt or decrypt a file using XOR cipher
    pub fn process_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let input_data = fs::read(input_path)?;
        let processed_data = self.xor_transform(&input_data);
        fs::write(output_path, processed_data)?;
        Ok(())
    }

    /// Perform XOR transformation on data
    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }

    /// Process data from stdin to stdout (for piping)
    pub fn process_stream(&self) -> io::Result<()> {
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        let processed = self.xor_transform(&buffer);
        io::stdout().write_all(&processed)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_symmetry() {
        let cipher = FileCipher::new("secret_key");
        let original = b"Hello, World!";
        let encrypted = cipher.xor_transform(original);
        let decrypted = cipher.xor_transform(&encrypted);
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn test_file_processing() {
        let cipher = FileCipher::new("test_key");
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let test_data = b"Sample file content for encryption test";
        fs::write(input_file.path(), test_data).unwrap();

        cipher
            .process_file(
                input_file.path().to_str().unwrap(),
                output_file.path().to_str().unwrap(),
            )
            .unwrap();

        let processed = fs::read(output_file.path()).unwrap();
        let restored = cipher.xor_transform(&processed);

        assert_eq!(test_data.to_vec(), restored);
    }
}