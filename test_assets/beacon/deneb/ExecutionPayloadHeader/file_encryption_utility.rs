use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_position: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_position: 0,
        }
    }

    fn next_key_byte(&mut self) -> u8 {
        let byte = self.key[self.key_position];
        self.key_position = (self.key_position + 1) % self.key.len();
        byte
    }

    pub fn process_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .map(|&byte| byte ^ self.next_key_byte())
            .collect()
    }

    pub fn process_stream<R: Read, W: Write>(
        &mut self,
        mut reader: R,
        mut writer: W,
    ) -> io::Result<()> {
        let mut buffer = [0u8; 4096];
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            let processed = self.process_bytes(&buffer[..bytes_read]);
            writer.write_all(&processed)?;
        }
        Ok(())
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let input_file = fs::File::open(input_path)?;
    let output_file = fs::File::create(output_path)?;
    let mut cipher = XorCipher::new(key);
    cipher.process_stream(input_file, output_file)
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original_data = b"Hello, World! This is a test message.";
        
        let mut encryptor = XorCipher::new(key);
        let encrypted = encryptor.process_bytes(original_data);
        
        let mut decryptor = XorCipher::new(key);
        let decrypted = decryptor.process_bytes(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_process_stream() {
        let key = "test_key";
        let input_data = b"Stream processing test data";
        let mut output = Vec::new();
        
        let mut cipher = XorCipher::new(key);
        let mut reader = Cursor::new(input_data);
        
        cipher.process_stream(&mut reader, &mut output).unwrap();
        
        assert_ne!(input_data, output.as_slice());
        
        let mut cipher2 = XorCipher::new(key);
        let mut reader2 = Cursor::new(output);
        let mut final_output = Vec::new();
        
        cipher2.process_stream(&mut reader2, &mut final_output).unwrap();
        
        assert_eq!(input_data, final_output.as_slice());
    }
}