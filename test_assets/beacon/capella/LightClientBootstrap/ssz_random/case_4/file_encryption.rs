use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// XOR cipher implementation for file encryption/decryption
pub struct XorCipher {
    key: Vec<u8>,
    key_index: usize,
}

impl XorCipher {
    /// Create a new XOR cipher with the given key
    pub fn new(key: &[u8]) -> Self {
        XorCipher {
            key: key.to_vec(),
            key_index: 0,
        }
    }

    /// Encrypt/decrypt a single byte using XOR
    pub fn process_byte(&mut self, byte: u8) -> u8 {
        let result = byte ^ self.key[self.key_index];
        self.key_index = (self.key_index + 1) % self.key.len();
        result
    }

    /// Process an entire buffer
    pub fn process_buffer(&mut self, buffer: &mut [u8]) {
        for byte in buffer.iter_mut() {
            *byte = self.process_byte(*byte);
        }
    }
}

/// Encrypt or decrypt a file using XOR cipher
pub fn process_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;

    let mut cipher = XorCipher::new(key);
    let mut buffer = [0u8; 4096];

    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let mut data_chunk = buffer[..bytes_read].to_vec();
        cipher.process_buffer(&mut data_chunk);
        output_file.write_all(&data_chunk)?;
    }

    Ok(())
}

/// Generate a random key of specified length
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
        let key = b"secret_key";
        let mut cipher1 = XorCipher::new(key);
        let mut cipher2 = XorCipher::new(key);

        let original_data = b"Hello, World!";
        let mut encrypted_data = original_data.to_vec();
        cipher1.process_buffer(&mut encrypted_data);

        let mut decrypted_data = encrypted_data.clone();
        cipher2.process_buffer(&mut decrypted_data);

        assert_eq!(original_data, decrypted_data.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = b"test_key_123";
        let test_data = b"Sample file content for encryption test";

        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let restored_file = NamedTempFile::new()?;

        fs::write(input_file.path(), test_data)?;

        process_file(input_file.path(), output_file.path(), key)?;
        process_file(output_file.path(), restored_file.path(), key)?;

        let restored_data = fs::read(restored_file.path())?;
        assert_eq!(test_data, restored_data.as_slice());

        Ok(())
    }
}