
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if content.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short",
        ));
    }

    let (key_bytes, ciphertext) = content.split_at(32);
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// XOR cipher implementation for file encryption/decryption
pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    /// Create a new cipher with the given key
    pub fn new(key: &[u8]) -> Self {
        XorCipher { key: key.to_vec() }
    }

    /// Encrypt or decrypt data using XOR cipher
    pub fn process(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        for (i, &byte) in data.iter().enumerate() {
            result.push(byte ^ self.key[i % self.key.len()]);
        }
        result
    }
}

/// Encrypt a file using XOR cipher
pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data = cipher.process(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

/// Decrypt a file using XOR cipher
pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = b"secret_key";
        let cipher = XorCipher::new(key);
        
        let original = b"Hello, World!";
        let encrypted = cipher.process(original);
        let decrypted = cipher.process(&encrypted);
        
        assert_eq!(original, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = b"test_key_123";
        
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(b"Test file content")?;
        
        let output_file = NamedTempFile::new()?;
        
        encrypt_file(input_file.path(), output_file.path(), key)?;
        
        let mut encrypted_content = Vec::new();
        fs::File::open(output_file.path())?.read_to_end(&mut encrypted_content)?;
        
        assert_ne!(encrypted_content, b"Test file content");
        
        let decrypted_file = NamedTempFile::new()?;
        decrypt_file(output_file.path(), decrypted_file.path(), key)?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;
        
        assert_eq!(decrypted_content, b"Test file content");
        
        Ok(())
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let mut rng = rand::thread_rng();
    let mut iv = [0u8; 16];
    rng.fill_bytes(&mut iv);

    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let ciphertext = Aes256CbcEnc::new(key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output.write_all(&iv).map_err(|e| e.to_string())?;
    output.write_all(&ciphertext).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data).map_err(|e| e.to_string())?;

    if encrypted_data.len() < 16 {
        return Err("Invalid encrypted file format".to_string());
    }

    let (iv, ciphertext) = encrypted_data.split_at(16);
    let plaintext = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| e.to_string())?;

    let mut output = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output.write_all(&plaintext).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn generate_key() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    key
}

pub fn key_to_hex(key: &[u8; 32]) -> String {
    hex::encode(key)
}

pub fn hex_to_key(hex_str: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(hex_str).map_err(|e| e.to_string())?;
    if bytes.len() != 32 {
        return Err("Invalid key length".to_string());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}