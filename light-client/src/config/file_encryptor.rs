
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const BUFFER_SIZE: usize = 8192;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    let key_len = key.len();
    if key_len == 0 {
        return;
    }
    
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key_len];
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    
    let mut buffer = [0u8; BUFFER_SIZE];
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        let data_slice = &mut buffer[..bytes_read];
        xor_cipher(data_slice, key);
        output_file.write_all(data_slice)?;
    }
    
    output_file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_xor_cipher_symmetric() {
        let key = b"secret";
        let original = b"Hello, World!";
        let mut data = original.to_vec();
        
        xor_cipher(&mut data, key);
        assert_ne!(data.as_slice(), original);
        
        xor_cipher(&mut data, key);
        assert_eq!(data.as_slice(), original);
    }
    
    #[test]
    fn test_xor_cipher_empty_key() {
        let key = b"";
        let mut data = b"test".to_vec();
        let original = data.clone();
        
        xor_cipher(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_xor_cipher_single_byte() {
        let key = b"x";
        let mut data = vec![0xAA, 0xBB, 0xCC];
        
        xor_cipher(&mut data, key);
        assert_eq!(data, vec![0xAA ^ b'x', 0xBB ^ b'x', 0xCC ^ b'x']);
    }
}