use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);
    key
}

pub fn generate_iv() -> [u8; 16] {
    let mut iv = [0u8; 16];
    rand::thread_rng().fill(&mut iv);
    iv
}

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 32], iv: &[u8; 16]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let ciphertext = Aes256CbcEnc::new(key.into(), iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8; 32], iv: &[u8; 16]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;

    let decrypted = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted)?;

    Ok(())
}

pub fn save_key_iv(key: &[u8; 32], iv: &[u8; 16], path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key_hex = hex::encode(key);
    let iv_hex = hex::encode(iv);
    let content = format!("KEY={}\nIV={}", key_hex, iv_hex);
    
    fs::write(path, content)?;
    Ok(())
}

pub fn load_key_iv(path: &str) -> Result<([u8; 32], [u8; 16]), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut lines = content.lines();
    
    let key_line = lines.next().ok_or("Missing KEY line")?;
    let iv_line = lines.next().ok_or("Missing IV line")?;
    
    let key_hex = key_line.strip_prefix("KEY=").ok_or("Invalid KEY format")?;
    let iv_hex = iv_line.strip_prefix("IV=").ok_or("Invalid IV format")?;
    
    let key_bytes = hex::decode(key_hex)?;
    let iv_bytes = hex::decode(iv_hex)?;
    
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];
    
    key.copy_from_slice(&key_bytes);
    iv.copy_from_slice(&iv_bytes);
    
    Ok((key, iv))
}