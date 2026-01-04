use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::fs;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let data = fs::read(input_path).map_err(|e| format!("Read error: {}", e))?;
    
    let mut rng = rand::thread_rng();
    let iv: [u8; 16] = rng.gen();
    
    let cipher = Aes256CbcEnc::new(key.into(), &iv.into());
    let encrypted_data = cipher.encrypt_padded_vec_mut::<Pkcs7>(&data);
    
    let mut output = iv.to_vec();
    output.extend_from_slice(&encrypted_data);
    
    fs::write(output_path, output).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let data = fs::read(input_path).map_err(|e| format!("Read error: {}", e))?;
    
    if data.len() < 16 {
        return Err("Invalid encrypted file".to_string());
    }
    
    let iv = &data[..16];
    let encrypted_data = &data[16..];
    
    let cipher = Aes256CbcDec::new(key.into(), iv.into());
    let decrypted_data = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(encrypted_data)
        .map_err(|e| format!("Decryption error: {}", e))?;
    
    fs::write(output_path, decrypted_data).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

pub fn generate_key() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut key = [0u8; 32];
    rng.fill(&mut key);
    key
}

pub fn key_to_hex(key: &[u8; 32]) -> String {
    hex::encode(key)
}

pub fn hex_to_key(hex_str: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(hex_str).map_err(|e| format!("Hex decode error: {}", e))?;
    if bytes.len() != 32 {
        return Err("Key must be 32 bytes".to_string());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}