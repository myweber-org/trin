
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_cipher(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = DEFAULT_KEY;
    process_file(Path::new(input_path), Path::new(output_path), key)
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = DEFAULT_KEY;
    process_file(Path::new(input_path), Path::new(output_path), key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xAA;

        xor_cipher(&mut data, key);
        assert_ne!(data, original);

        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let mut temp_input = NamedTempFile::new()?;
        let test_data = b"Hello, World! This is a test file.";
        temp_input.write_all(test_data)?;

        let input_path = temp_input.path().to_str().unwrap();
        let temp_output = NamedTempFile::new()?;
        let output_path = temp_output.path().to_str().unwrap();

        encrypt_file(input_path, output_path)?;

        let encrypted_content = fs::read(output_path)?;
        assert_ne!(encrypted_content, test_data);

        let temp_decrypted = NamedTempFile::new()?;
        let decrypted_path = temp_decrypted.path().to_str().unwrap();

        decrypt_file(output_path, decrypted_path)?;
        let decrypted_content = fs::read(decrypted_path)?;
        assert_eq!(decrypted_content, test_data);

        Ok(())
    }
}