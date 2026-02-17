
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique nonce");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, encrypted_data)?;
    fs::write(format!("{}.key", output_path), key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    let key_bytes = fs::read(key_path)?;
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Test data for encryption";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let key_path = format!("{}.key", encrypted_file.path().to_str().unwrap());
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            &key_path,
            decrypted_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2, Params,
};
use std::fs;
use std::path::Path;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub salt: String,
}

pub fn derive_key(password: &str, salt: &str) -> Result<[u8; 32], String> {
    let salt_bytes = SaltString::from_b64(salt)
        .map_err(|e| format!("Invalid salt: {}", e))?
        .as_salt()
        .as_bytes();

    let params = Params::new(15 * 1024, 2, 1, Some(32))
        .map_err(|e| format!("Argon2 params error: {}", e))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt_bytes)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    let mut key = [0u8; 32];
    key.copy_from_slice(
        password_hash
            .hash
            .ok_or("No hash generated")?
            .as_bytes()
            .get(..32)
            .ok_or("Hash too short")?,
    );
    Ok(key)
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let plaintext = fs::read(input_path).map_err(|e| format!("Read failed: {}", e))?;

    let salt = SaltString::generate(&mut ArgonRng).to_string();
    let key = derive_key(password, &salt)?;

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher init failed: {}", e))?;
    let nonce_bytes: [u8; 12] = OsRng.fill(&mut [0u8; 12]).map_err(|e| format!("RNG error: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    fs::write(output_path, &ciphertext).map_err(|e| format!("Write failed: {}", e))?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; 12],
    salt: &str,
) -> Result<Vec<u8>, String> {
    let ciphertext = fs::read(input_path).map_err(|e| format!("Read failed: {}", e))?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher init failed: {}", e))?;
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, &plaintext).map_err(|e| format!("Write failed: {}", e))?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret data for encryption test";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        let enc_result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();

        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &enc_result.nonce,
            &enc_result.salt,
        ).unwrap();

        assert_eq!(test_data.to_vec(), decrypted);
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

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
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
    
    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    
    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }
    
    process_file(input_path, output_path, DEFAULT_KEY)?;
    
    println!("File processed successfully with XOR key 0x{:02X}", DEFAULT_KEY);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
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
    fn test_file_encryption() -> io::Result<()> {
        let test_data = b"Hello, World!";
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        process_file(input_file.path(), output_file.path(), DEFAULT_KEY)?;
        
        let encrypted = fs::read(output_file.path())?;
        assert_ne!(encrypted, test_data);
        
        let mut decrypted = encrypted.clone();
        xor_cipher(&mut decrypted, DEFAULT_KEY);
        assert_eq!(decrypted, test_data);
        
        Ok(())
    }
}