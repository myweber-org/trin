use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn xor_encrypt(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|byte| byte ^ key).collect()
}

pub fn xor_decrypt(data: &[u8], key: u8) -> Vec<u8> {
    xor_encrypt(data, key)
}

pub fn process_file(input_path: &Path, output_path: &Path, key: u8, encrypt: bool) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = if encrypt {
        xor_encrypt(&buffer, key)
    } else {
        xor_decrypt(&buffer, key)
    };

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

pub fn generate_key_from_string(seed: &str) -> u8 {
    seed.bytes().fold(0u8, |acc, b| acc.wrapping_add(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_symmetry() {
        let original = b"Hello, World!";
        let key = 0x42;
        
        let encrypted = xor_encrypt(original, key);
        assert_ne!(original, encrypted.as_slice());
        
        let decrypted = xor_decrypt(&encrypted, key);
        assert_eq!(original, decrypted.as_slice());
    }

    #[test]
    fn test_file_processing() -> io::Result<()> {
        let test_data = b"Test file content for encryption";
        let key = DEFAULT_KEY;
        
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        process_file(input_file.path(), output_file.path(), key, true)?;
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(test_data, encrypted.as_slice());
        
        let decrypt_file = NamedTempFile::new()?;
        process_file(output_file.path(), decrypt_file.path(), key, false)?;
        let decrypted = fs::read(decrypt_file.path())?;
        
        assert_eq!(test_data, decrypted.as_slice());
        Ok(())
    }

    #[test]
    fn test_key_generation() {
        let seed = "my secret password";
        let key = generate_key_from_string(seed);
        assert_eq!(key, 0x7A);
    }
}