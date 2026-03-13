
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn xor_encrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;
    
    let mut buffer = [0u8; 4096];
    
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        for byte in buffer.iter_mut().take(bytes_read) {
            *byte ^= key;
        }
        
        output_file.write_all(&buffer[..bytes_read])?;
    }
    
    Ok(())
}

pub fn xor_decrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_encryption_decryption() {
        let original_data = b"Hello, this is a secret message!";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        xor_encrypt_file(input_file.path(), encrypted_file.path(), Some(0x42)).unwrap();
        xor_decrypt_file(encrypted_file.path(), decrypted_file.path(), Some(0x42)).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.as_slice(), decrypted_data.as_slice());
    }
    
    #[test]
    fn test_default_key() {
        let test_data = b"Test with default key";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        xor_encrypt_file(input_file.path(), encrypted_file.path(), None).unwrap();
        xor_decrypt_file(encrypted_file.path(), decrypted_file.path(), None).unwrap();
        
        let result = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.as_slice(), result.as_slice());
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

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> io::Result<Self> {
        let salt_string = SaltString::encode_b64(salt).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Salt error: {}", e))
        })?;

        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Password hashing failed: {}", e),
                )
            })?;

        let hash_bytes = password_hash.hash.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Hash generation failed")
        })?;

        let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Key length mismatch"))?;

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes).into();
        Ok(Self { key })
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)?;

        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::generate(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Encryption failed: {}", e)))?;

        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce.as_slice())?;
        output.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < NONCE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(&self.key);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Decryption failed: {}", e)))?;

        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    ArgonRng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let password = "secure_password_123";
        let salt = generate_salt();
        let encryptor = FileEncryptor::from_password(password, &salt).unwrap();

        let test_data = b"Hello, this is a secret message!";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor
            .encrypt_file(input_file.path().to_str().unwrap(), encrypted_file.path().to_str().unwrap())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path().to_str().unwrap(), decrypted_file.path().to_str().unwrap())
            .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let salt = generate_salt();
        let encryptor1 = FileEncryptor::from_password("password1", &salt).unwrap();
        let encryptor2 = FileEncryptor::from_password("password2", &salt).unwrap();

        let test_data = b"Test data";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor1
            .encrypt_file(input_file.path().to_str().unwrap(), encrypted_file.path().to_str().unwrap())
            .unwrap();

        let result = encryptor2.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
        );
        assert!(result.is_err());
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data: Vec<u8> = buffer
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_text = b"Hello, World! This is a test message.";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), original_text).unwrap();
        
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
        assert_eq!(original_text.to_vec(), decrypted_data);
    }
    
    #[test]
    fn test_default_key() {
        let original_text = b"Testing with default key";
        let temp_input = NamedTempFile::new().unwrap();
        let temp_encrypted = NamedTempFile::new().unwrap();
        let temp_decrypted = NamedTempFile::new().unwrap();
        
        fs::write(temp_input.path(), original_text).unwrap();
        
        encrypt_file(
            temp_input.path().to_str().unwrap(),
            temp_encrypted.path().to_str().unwrap(),
            None
        ).unwrap();
        
        decrypt_file(
            temp_encrypted.path().to_str().unwrap(),
            temp_decrypted.path().to_str().unwrap(),
            None
        ).unwrap();
        
        let decrypted_data = fs::read(temp_decrypted.path()).unwrap();
        assert_eq!(original_text.to_vec(), decrypted_data);
    }
}