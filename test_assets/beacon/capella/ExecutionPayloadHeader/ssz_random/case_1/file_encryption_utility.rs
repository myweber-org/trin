use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W, key: u8) -> io::Result<()> {
    let mut buffer = [0; 1024];
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for byte in buffer.iter_mut().take(bytes_read) {
            *byte ^= key;
        }
        
        writer.write_all(&buffer[..bytes_read])?;
    }
    
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_xor_encryption() {
        let data = b"Hello, World!";
        let key = 0x42;
        
        let encrypted: Vec<u8> = data.iter().map(|b| b ^ key).collect();
        let decrypted: Vec<u8> = encrypted.iter().map(|b| b ^ key).collect();
        
        assert_eq!(data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_stream_processing() {
        let input = b"Test data stream";
        let key = 0x77;
        
        let mut reader = Cursor::new(input);
        let mut output = Vec::new();
        
        process_stream(&mut reader, &mut output, key).unwrap();
        
        let mut decrypted = Vec::new();
        let mut reader2 = Cursor::new(&output);
        
        process_stream(&mut reader2, &mut decrypted, key).unwrap();
        
        assert_eq!(input.to_vec(), decrypted);
    }
}