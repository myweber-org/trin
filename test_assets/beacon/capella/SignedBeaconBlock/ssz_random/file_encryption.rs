
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
const KEY_LENGTH: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub salt: [u8; SALT_LENGTH],
    pub iv: [u8; IV_LENGTH],
    pub ciphertext: Vec<u8>,
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut file_data)?;

    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let key = derive_key(password, &salt);
    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());

    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&file_data);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;

    Ok(EncryptionResult {
        salt,
        iv,
        ciphertext,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    salt: &[u8],
    iv: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut ciphertext = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut ciphertext)?;

    let key = derive_key(password, salt);
    let cipher = Aes256CbcDec::new(&key.into(), iv.into());

    let plaintext = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::File::create(output_path)?.write_all(&plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Test data for encryption and decryption";
        let password = "secure_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        let result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &result.salt,
            &result.iv,
        )
        .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{pbkdf2_hmac, Params};
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;

pub struct EncryptionResult {
    pub salt: Vec<u8>,
    pub iv: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    let params = Params {
        rounds: KEY_ITERATIONS,
        output_length: KEY_LENGTH,
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key)
        .expect("PBKDF2 should not fail");
    key
}

pub fn encrypt_data(password: &str, plaintext: &[u8]) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut iv);
    
    let key = derive_key(password, &salt);
    
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    
    Ok(EncryptionResult {
        salt: salt.to_vec(),
        iv: iv.to_vec(),
        ciphertext,
    })
}

pub fn decrypt_data(password: &str, result: &EncryptionResult) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &result.salt);
    
    let plaintext = Aes256CbcDec::new(&key.into(), &result.iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&result.ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

pub fn encrypt_file(password: &str, input_path: &Path, output_path: &Path) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let result = encrypt_data(password, &plaintext)?;
    
    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output.write_all(&result.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output.write_all(&result.iv)
        .map_err(|e| format!("Failed to write IV: {}", e))?;
    output.write_all(&result.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(password: &str, input_path: &Path, output_path: &Path) -> Result<(), String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    if encrypted_data.len() < SALT_LENGTH + IV_LENGTH {
        return Err("Input file too short".to_string());
    }
    
    let salt = &encrypted_data[0..SALT_LENGTH];
    let iv = &encrypted_data[SALT_LENGTH..SALT_LENGTH + IV_LENGTH];
    let ciphertext = &encrypted_data[SALT_LENGTH + IV_LENGTH..];
    
    let result = EncryptionResult {
        salt: salt.to_vec(),
        iv: iv.to_vec(),
        ciphertext: ciphertext.to_vec(),
    };
    
    let plaintext = decrypt_data(password, &result)?;
    
    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "test_password_123";
        let plaintext = b"This is a secret message that needs encryption";
        
        let result = encrypt_data(password, plaintext).unwrap();
        let decrypted = decrypt_data(password, &result).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_operations() {
        let password = "file_encryption_test";
        let test_data = b"Sample file content for encryption testing";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(password, input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(password, encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
    
    #[test]
    fn test_wrong_password() {
        let password = "correct_password";
        let plaintext = b"Secret data";
        
        let result = encrypt_data(password, plaintext).unwrap();
        let decryption_result = decrypt_data("wrong_password", &result);
        
        assert!(decryption_result.is_err());
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        std::process::exit(1);
    }
    
    match process_file(input_path, output_path, DEFAULT_KEY) {
        Ok(_) => println!("File processed successfully"),
        Err(e) => eprintln!("Error processing file: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_cipher() {
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0x55;
        
        xor_cipher(&mut data, key);
        xor_cipher(&mut data, key);
        
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() {
        let test_content = b"Hello, World!";
        let input_path = "test_input.txt";
        let output_path = "test_output.txt";
        
        fs::write(input_path, test_content).unwrap();
        
        process_file(input_path, output_path, DEFAULT_KEY).unwrap();
        process_file(output_path, "test_decrypted.txt", DEFAULT_KEY).unwrap();
        
        let decrypted = fs::read("test_decrypted.txt").unwrap();
        assert_eq!(decrypted, test_content);
        
        fs::remove_file(input_path).unwrap();
        fs::remove_file(output_path).unwrap();
        fs::remove_file("test_decrypted.txt").unwrap();
    }
}