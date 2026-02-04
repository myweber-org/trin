
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;
    output_file.write_all(&key)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key and ciphertext",
        ));
    }

    let key_slice = &encrypted_data[encrypted_data.len() - 32..];
    let ciphertext = &encrypted_data[..encrypted_data.len() - 32];

    let key = Key::<Aes256Gcm>::from_slice(key_slice);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data = xor_transform(&input_data, key);
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

fn xor_transform(data: &[u8], key: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    for (i, &byte) in data.iter().enumerate() {
        result.push(byte ^ key[i % key.len()]);
    }
    result
}

pub fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W, key: &[u8]) -> io::Result<()> {
    let mut buffer = [0u8; 4096];
    let mut position = 0;
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for i in 0..bytes_read {
            buffer[i] ^= key[(position + i) % key.len()];
        }
        
        writer.write_all(&buffer[..bytes_read])?;
        position += bytes_read;
    }
    
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_xor_transform() {
        let data = b"Hello, World!";
        let key = b"secret";
        let encrypted = xor_transform(data, key);
        let decrypted = xor_transform(&encrypted, key);
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_stream_processing() {
        let input = b"Test data for streaming";
        let key = b"key123";
        
        let mut encrypted = Vec::new();
        process_stream(Cursor::new(input), &mut encrypted, key).unwrap();
        
        let mut decrypted = Vec::new();
        process_stream(Cursor::new(encrypted), &mut decrypted, key).unwrap();
        
        assert_eq!(input.to_vec(), decrypted);
    }
}