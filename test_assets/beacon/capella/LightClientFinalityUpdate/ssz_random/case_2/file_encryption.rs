
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = data.iter().map(|byte| byte ^ key).collect();
    fs::write(output_path, encrypted_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original = b"Hello, World!";
        let mut temp_input = NamedTempFile::new().unwrap();
        temp_input.write_all(original).unwrap();
        let input_path = temp_input.path().to_str().unwrap();

        let temp_encrypted = NamedTempFile::new().unwrap();
        let encrypted_path = temp_encrypted.path().to_str().unwrap();

        let temp_decrypted = NamedTempFile::new().unwrap();
        let decrypted_path = temp_decrypted.path().to_str().unwrap();

        encrypt_file(input_path, encrypted_path, Some(0xCC)).unwrap();
        decrypt_file(encrypted_path, decrypted_path, Some(0xCC)).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(original, decrypted_data.as_slice());
    }
}