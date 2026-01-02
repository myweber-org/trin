
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
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

pub fn decrypt_file(encrypted_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(encrypted_path)?;
    let key_bytes = fs::read(key_path)?;
    
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
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
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;

pub struct EncryptionResult {
    pub salt: Vec<u8>,
    pub iv: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub fn derive_key(password: &[u8], salt: &[u8]) -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    pbkdf2_hmac::<Sha256>(password, salt, PBKDF2_ITERATIONS, &mut key);
    key
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let key = derive_key(password.as_bytes(), &salt);
    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());

    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output_file
        .write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(EncryptionResult {
        salt: salt.to_vec(),
        iv: iv.to_vec(),
        ciphertext,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    salt: &[u8],
    iv: &[u8],
) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(input_path).map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    let key = derive_key(password.as_bytes(), salt);
    let cipher = Aes256CbcDec::new(&key.into(), iv.into());

    let plaintext = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output_file
        .write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(plaintext).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        let result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &result.salt,
            &result.iv,
        ).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }
}