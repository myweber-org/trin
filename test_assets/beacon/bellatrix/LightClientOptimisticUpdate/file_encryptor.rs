
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key.as_bytes());
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn validate_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 256
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0xAA, 0xBB, 0xCC];
        let key = vec![0x12, 0x34];
        xor_cipher(&mut data, &key);
        assert_eq!(data, vec![0xB8, 0x8F, 0xDE]);
        
        xor_cipher(&mut data, &key);
        assert_eq!(data, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_process_file() -> io::Result<()> {
        let mut input_file = NamedTempFile::new()?;
        let original_data = b"Test data for encryption";
        input_file.write_all(original_data)?;
        
        let output_file = NamedTempFile::new()?;
        let key = "secret";
        
        process_file(input_file.path(), output_file.path(), key)?;
        
        let mut encrypted_data = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut encrypted_data)?;
        
        assert_ne!(encrypted_data, original_data);
        
        let mut decrypted_data = encrypted_data.clone();
        xor_cipher(&mut decrypted_data, key.as_bytes());
        assert_eq!(decrypted_data, original_data);
        
        Ok(())
    }

    #[test]
    fn test_validate_key() {
        assert!(validate_key("valid"));
        assert!(!validate_key(""));
        assert!(validate_key(&"a".repeat(256)));
        assert!(!validate_key(&"a".repeat(257)));
    }
}