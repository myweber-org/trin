
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file> [key]", args[0]);
        std::process::exit(1);
    }
    
    let operation = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];
    let key = if args.len() > 4 {
        Some(args[4].parse::<u8>().unwrap_or(DEFAULT_KEY))
    } else {
        None
    };
    
    if !Path::new(input_file).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_file);
        std::process::exit(1);
    }
    
    match operation.as_str() {
        "encrypt" => encrypt_file(input_file, output_file, key)?,
        "decrypt" => decrypt_file(input_file, output_file, key)?,
        _ => {
            eprintln!("Error: Operation must be 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
    
    println!("Operation completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, World!";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), original_data).unwrap();
        
        encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap(),
            Some(0x55)
        ).unwrap();
        
        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex::encode;
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub struct FileEncryptor {
    key: [u8; 32],
    iv: [u8; 16],
}

impl FileEncryptor {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut key);
        rand::thread_rng().fill_bytes(&mut iv);
        Self { key, iv }
    }

    pub fn from_key_iv(key: &str, iv: &str) -> Result<Self, &'static str> {
        let key_bytes = hex::decode(key).map_err(|_| "Invalid key hex")?;
        let iv_bytes = hex::decode(iv).map_err(|_| "Invalid IV hex")?;

        if key_bytes.len() != 32 || iv_bytes.len() != 16 {
            return Err("Key must be 32 bytes, IV must be 16 bytes");
        }

        let mut key_arr = [0u8; 32];
        let mut iv_arr = [0u8; 16];
        key_arr.copy_from_slice(&key_bytes);
        iv_arr.copy_from_slice(&iv_bytes);

        Ok(Self {
            key: key_arr,
            iv: iv_arr,
        })
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

        let ciphertext = Aes256CbcEnc::new(&self.key.into(), &self.iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
        output_file
            .write_all(&ciphertext)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

        let plaintext = Aes256CbcDec::new(&self.key.into(), &self.iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
            .map_err(|e| e.to_string())?;

        let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
        output_file
            .write_all(&plaintext)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_key_hex(&self) -> String {
        encode(self.key)
    }

    pub fn get_iv_hex(&self) -> String {
        encode(self.iv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_encryption_roundtrip() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Secret data for encryption test";

        fs::write("test_input.txt", test_data).unwrap();
        encryptor
            .encrypt_file("test_input.txt", "test_encrypted.bin")
            .unwrap();
        encryptor
            .decrypt_file("test_encrypted.bin", "test_decrypted.txt")
            .unwrap();

        let decrypted = fs::read("test_decrypted.txt").unwrap();
        assert_eq!(decrypted, test_data);

        fs::remove_file("test_input.txt").unwrap();
        fs::remove_file("test_encrypted.bin").unwrap();
        fs::remove_file("test_decrypted.txt").unwrap();
    }
}