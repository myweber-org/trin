
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, encrypted_data)?;
    fs::write(format!("{}.key", output_path), key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    let key_bytes = fs::read(key_path)?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;

pub struct EncryptionResult {
    pub salt: [u8; SALT_LENGTH],
    pub iv: [u8; IV_LENGTH],
    pub ciphertext: Vec<u8>,
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
    key
}

pub fn encrypt_file(password: &str, input_path: &str, output_path: &str) -> Result<(), String> {
    let mut input_file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let key = derive_key(password, &salt);
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let result = EncryptionResult { salt, iv, ciphertext };

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&result.salt).map_err(|e| e.to_string())?;
    output_file.write_all(&result.iv).map_err(|e| e.to_string())?;
    output_file.write_all(&result.ciphertext).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn decrypt_file(password: &str, input_path: &str, output_path: &str) -> Result<(), String> {
    let mut input_file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data).map_err(|e| e.to_string())?;

    if encrypted_data.len() < SALT_LENGTH + IV_LENGTH {
        return Err("Invalid encrypted file format".to_string());
    }

    let salt = &encrypted_data[0..SALT_LENGTH];
    let iv = &encrypted_data[SALT_LENGTH..SALT_LENGTH + IV_LENGTH];
    let ciphertext = &encrypted_data[SALT_LENGTH + IV_LENGTH..];

    let key = derive_key(password, salt);
    let decrypted = Aes256CbcDec::new(&key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&decrypted).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn encrypt_string(password: &str, plaintext: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let key = derive_key(password, &salt);
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());

    Ok(EncryptionResult { salt, iv, ciphertext })
}

pub fn decrypt_string(password: &str, result: &EncryptionResult) -> Result<String, String> {
    let key = derive_key(password, &result.salt);
    let decrypted = Aes256CbcDec::new(&key.into(), &result.iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&result.ciphertext)
        .map_err(|e| e.to_string())?;

    String::from_utf8(decrypted).map_err(|e| e.to_string())
}