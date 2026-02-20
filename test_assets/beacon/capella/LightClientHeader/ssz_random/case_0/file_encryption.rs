
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut content = fs::read(input_path)?;
    xor_cipher(&mut content, key);
    fs::write(output_path, content)
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        xor_cipher(&mut data, 0xAA);
        assert_eq!(data, vec![0xAA, 0x55, 0xFF, 0x00]);
        xor_cipher(&mut data, 0xAA);
        assert_eq!(data, vec![0x00, 0xFF, 0x55, 0xAA]);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original = b"Secret data";
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        fs::write(input_file.path(), original)?;
        encrypt_file(input_file.path(), output_file.path(), Some(0xCC))?;
        decrypt_file(output_file.path(), decrypted_file.path(), Some(0xCC))?;

        let restored = fs::read(decrypted_file.path())?;
        assert_eq!(original.as_slice(), &restored);
        Ok(())
    }
}