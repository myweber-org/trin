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

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.xor_transform(&buffer);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        self.encrypt_file(input_path, output_path)
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn process_file_encryption(
    source_path: &str,
    dest_path: &str,
    key: &str,
    encrypt: bool,
) -> Result<(), String> {
    let cipher = XorCipher::new(key);
    let source = Path::new(source_path);
    let dest = Path::new(dest_path);

    if !source.exists() {
        return Err("Source file does not exist".to_string());
    }

    if encrypt {
        cipher
            .encrypt_file(source, dest)
            .map_err(|e| format!("Encryption failed: {}", e))
    } else {
        cipher
            .decrypt_file(source, dest)
            .map_err(|e| format!("Decryption failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XorCipher::new("test_key_123");
        let original_data = b"Hello, this is a secret message!";

        let encrypted = cipher.xor_transform(original_data);
        let decrypted = cipher.xor_transform(&encrypted);

        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption_roundtrip() {
        let key = "strong_password";
        let cipher = XorCipher::new(key);

        let original_content = b"Confidential data: 42";
        let mut temp_input = NamedTempFile::new().unwrap();
        temp_input.write_all(original_content).unwrap();

        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();

        cipher
            .encrypt_file(temp_input.path(), temp_encrypted.path())
            .unwrap();
        cipher
            .decrypt_file(temp_encrypted.path(), temp_decrypted.path())
            .unwrap();

        let restored_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_content.to_vec(), restored_content);
    }
}