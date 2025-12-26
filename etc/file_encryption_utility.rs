use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, key: Option<u8>, encrypt: bool) -> io::Result<()> {
    let dir_entries = fs::read_dir(dir_path)?;
    let encryption_key = key.unwrap_or(DEFAULT_KEY);

    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let input_path = path.to_str().unwrap();
            let output_path = format!("{}.processed", input_path);
            
            if encrypt {
                encrypt_file(input_path, &output_path, Some(encryption_key))?;
            } else {
                decrypt_file(input_path, &output_path, Some(encryption_key))?;
            }
            
            println!("Processed: {}", input_path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data to protect";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(original_content).unwrap();
        
        let input_path = temp_file.path().to_str().unwrap();
        let encrypted_path = format!("{}.enc", input_path);
        let decrypted_path = format!("{}.dec", input_path);
        
        encrypt_file(input_path, &encrypted_path, Some(0xAA)).unwrap();
        decrypt_file(&encrypted_path, &decrypted_path, Some(0xAA)).unwrap();
        
        let decrypted_content = fs::read(decrypted_path).unwrap();
        assert_eq!(original_content, decrypted_content.as_slice());
    }
}