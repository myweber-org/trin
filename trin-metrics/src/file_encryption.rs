use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

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
    use std::env;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let original = b"Hello, World!";
        let mut data = original.to_vec();
        let key = 0xAA;

        xor_cipher(&mut data, key);
        assert_ne!(data.as_slice(), original);

        xor_cipher(&mut data, key);
        assert_eq!(data.as_slice(), original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;
        let test_data = b"Test encryption data";

        fs::write(temp_input.path(), test_data)?;
        encrypt_file(temp_input.path(), temp_output.path(), Some(0x77))?;

        let encrypted = fs::read(temp_output.path())?;
        assert_ne!(&encrypted, test_data);

        let temp_decrypted = NamedTempFile::new()?;
        decrypt_file(temp_output.path(), temp_decrypted.path(), Some(0x77))?;
        let decrypted = fs::read(temp_decrypted.path())?;
        assert_eq!(&decrypted, test_data);

        Ok(())
    }
}