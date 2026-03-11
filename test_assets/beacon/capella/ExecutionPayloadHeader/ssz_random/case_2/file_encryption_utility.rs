use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use pbkdf2::{
    password_hash::{PasswordHasher, SaltString},
    Pbkdf2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::b64_encode(salt)
        .map_err(|e| format!("Failed to encode salt: {}", e))?;
    
    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("Empty password hash")?.as_bytes();
    if hash_bytes.len() < 32 {
        return Err("Derived key too short".to_string());
    }
    
    let key_slice = &hash_bytes[..32];
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_data(data: &[u8], password: &str) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        nonce,
    })
}

pub fn decrypt_data(encrypted: &EncryptionResult, password: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    cipher
        .decrypt(Nonce::from_slice(&encrypted.nonce), encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let encrypted = encrypt_data(&buffer, password)?;
    
    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output.write_all(&encrypted.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output.write_all(&encrypted.nonce)
        .map_err(|e| format!("Failed to write nonce: {}", e))?;
    output.write_all(&encrypted.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    let metadata = fs::metadata(input_path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    if metadata.len() < (SALT_LENGTH + NONCE_LENGTH) as u64 {
        return Err("File too short to contain encrypted data".to_string());
    }
    
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut salt = [0u8; SALT_LENGTH];
    file.read_exact(&mut salt)
        .map_err(|e| format!("Failed to read salt: {}", e))?;
    
    let mut nonce = [0u8; NONCE_LENGTH];
    file.read_exact(&mut nonce)
        .map_err(|e| format!("Failed to read nonce: {}", e))?;
    
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read ciphertext: {}", e))?;
    
    let encrypted = EncryptionResult {
        ciphertext,
        salt,
        nonce,
    };
    
    let decrypted = decrypt_data(&encrypted, password)?;
    
    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&decrypted)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Secret message for encryption testing";
        let password = "strong_password_123";
        
        let encrypted = encrypt_data(original_data, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_wrong_password_fails() {
        let original_data = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = encrypt_data(original_data, password).unwrap();
        let result = decrypt_data(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_file_encryption() {
        let temp_input = NamedTempFile::new().unwrap();
        let temp_output = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        let test_data = b"File encryption test content";
        fs::write(temp_input.path(), test_data).unwrap();
        
        let password = "file_encryption_password";
        
        encrypt_file(temp_input.path(), temp_output.path(), password).unwrap();
        decrypt_file(temp_output.path(), temp_decrypted.path(), password).unwrap();
        
        let decrypted_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_content);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| e.to_string())?;
    
    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?.as_bytes();
    if hash_bytes.len() < 32 {
        return Err("Insufficient hash length".to_string());
    }
    
    let key_bytes: [u8; 32] = hash_bytes[..32].try_into().map_err(|_| "Invalid key length")?;
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let encrypted_data = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&encrypted_data).map_err(|e| e.to_string())?;

    Ok(EncryptionResult {
        encrypted_data,
        nonce: nonce_bytes,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
) -> Result<Vec<u8>, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let decrypted_data = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&decrypted_data).map_err(|e| e.to_string())?;

    Ok(decrypted_data)
}

pub fn generate_secure_password(length: usize) -> Result<String, String> {
    if length < 12 {
        return Err("Password length must be at least 12 characters".to_string());
    }
    
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !@#$%^&*()-_=+[]{}|;:,.<>?";
    
    let mut rng = OsRng;
    let mut password = Vec::with_capacity(length);
    
    for _ in 0..length {
        let idx = rng.next_u32() as usize % CHARSET.len();
        password.push(CHARSET[idx]);
    }
    
    String::from_utf8(password).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Test encryption and decryption";
        let password = "StrongPassword123!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        let result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        
        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &result.nonce,
            &result.salt,
        ).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_password_generation() {
        let password = generate_secure_password(16).unwrap();
        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| !c.is_ascii_alphanumeric()));
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
}

pub fn encrypt_file(key: &[u8; 32], input_path: &Path, output_path: &Path) -> io::Result<()> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&nonce_bytes)?;
    output_file.write_all(&ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(key: &[u8; 32], input_path: &Path, output_path: &Path) -> io::Result<()> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    
    let mut file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)?;
    
    if encrypted_data.len() < NONCE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File too short"));
    }
    
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;
    
    Ok(())
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::str;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_random_key();
        let test_data = b"Test encryption data for AES-256-GCM";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(&key, input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(&key, encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_random_key();
        let mut key2 = generate_random_key();
        key2[0] = !key2[0];
        
        let test_data = b"Sensitive information";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encrypt_file(&key1, input_file.path(), encrypted_file.path()).unwrap();
        let result = decrypt_file(&key2, encrypted_file.path(), decrypted_file.path());
        
        assert!(result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_string = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| e.to_string())?;

    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Hash too short")?;

    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new(&key);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&ciphertext)
        .map_err(|e| e.to_string())?;

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
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let nonce = Nonce::from_slice(nonce);

    let cipher = Aes256Gcm::new(&key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output_file
        .write_all(&plaintext)
        .map_err(|e| e.to_string())?;

    Ok(plaintext)
}

pub fn generate_random_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()-_=+[]{}|;:,.<>?";

    let mut rng = OsRng;
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.next_u32() as usize % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect();

    password
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data for encryption test";
        let password = "StrongPassword123!";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(plaintext).unwrap();

        let output_encrypted = NamedTempFile::new().unwrap();
        let output_decrypted = NamedTempFile::new().unwrap();

        let enc_result = encrypt_file(
            input_file.path(),
            output_encrypted.path(),
            password,
        ).unwrap();

        let decrypted = decrypt_file(
            output_encrypted.path(),
            output_decrypted.path(),
            password,
            &enc_result.nonce,
            &enc_result.salt,
        ).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "CorrectPassword";
        let wrong_password = "WrongPassword";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(plaintext).unwrap();

        let output_encrypted = NamedTempFile::new().unwrap();
        let output_decrypted = NamedTempFile::new().unwrap();

        let enc_result = encrypt_file(
            input_file.path(),
            output_encrypted.path(),
            password,
        ).unwrap();

        let result = decrypt_file(
            output_encrypted.path(),
            output_decrypted.path(),
            wrong_password,
            &enc_result.nonce,
            &enc_result.salt,
        );

        assert!(result.is_err());
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{Read, Write};

const NONCE_SIZE: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(FileEncryptor { cipher })
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let nonce = Nonce::from_slice(&generate_nonce());
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();
        
        let original_content = b"Secret data that needs protection";
        let mut temp_input = NamedTempFile::new().unwrap();
        temp_input.write_all(original_content).unwrap();
        
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap()
        ).unwrap();
        
        encryptor.decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted_content = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}