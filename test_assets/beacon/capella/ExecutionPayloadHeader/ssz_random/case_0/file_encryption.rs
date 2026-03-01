
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const KEY_LENGTH: usize = 32;
const IV_LENGTH: usize = 16;

pub struct EncryptionResult {
    pub iv: Vec<u8>,
    pub encrypted_data: Vec<u8>,
}

pub fn generate_key() -> Vec<u8> {
    let mut key = vec![0u8; KEY_LENGTH];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn encrypt_file(key: &[u8], input_path: &Path) -> Result<EncryptionResult, String> {
    if key.len() != KEY_LENGTH {
        return Err(format!("Key must be {} bytes", KEY_LENGTH));
    }

    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut iv = vec![0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut iv);

    let cipher = Aes256CbcEnc::new(key.into(), &iv.into());
    let encrypted_data = cipher.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    Ok(EncryptionResult {
        iv,
        encrypted_data,
    })
}

pub fn decrypt_file(key: &[u8], iv: &[u8], encrypted_data: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != KEY_LENGTH {
        return Err(format!("Key must be {} bytes", KEY_LENGTH));
    }
    if iv.len() != IV_LENGTH {
        return Err(format!("IV must be {} bytes", IV_LENGTH));
    }

    let cipher = Aes256CbcDec::new(key.into(), iv.into());
    let decrypted_data = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(decrypted_data)
}

pub fn save_encrypted_file(output_path: &Path, result: &EncryptionResult) -> Result<(), String> {
    let mut file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    file.write_all(&result.iv)
        .and_then(|_| file.write_all(&result.encrypted_data))
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
    
    Ok(())
}

pub fn load_encrypted_file(input_path: &Path) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut iv = vec![0u8; IV_LENGTH];
    file.read_exact(&mut iv)
        .map_err(|e| format!("Failed to read IV: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read encrypted data: {}", e))?;
    
    Ok((iv, encrypted_data))
}