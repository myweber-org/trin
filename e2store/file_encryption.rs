
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = fs::read(input_path)?;
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_data = key.to_vec();
    output_data.extend_from_slice(nonce);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let data = fs::read(input_path)?;
    if data.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File too short"));
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&data[..32]);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&data[32..32 + NONCE_SIZE]);
    let ciphertext = &data[32 + NONCE_SIZE..];
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)
}