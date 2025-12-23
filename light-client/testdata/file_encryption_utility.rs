use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed_buffer = buffer[..bytes_read].to_vec();
            self.xor_transform(&mut processed_buffer, &mut key_index);

            dest_file.write_all(&processed_buffer)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    fn xor_transform(&self, data: &mut [u8], key_index: &mut usize) {
        for byte in data.iter_mut() {
            *byte ^= self.key[*key_index];
            *key_index = (*key_index + 1) % self.key.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XorCipher::new("secret_key_123");
        let test_data = b"Hello, this is a test message for XOR encryption!";

        let mut encrypted = test_data.to_vec();
        let mut key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);

        key_index = 0;
        cipher.xor_transform(&mut encrypted, &mut key_index);

        assert_eq!(encrypted, test_data);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = XorCipher::new("test_key");
        let original_content = b"File content to encrypt and decrypt";

        let source_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        fs::write(source_file.path(), original_content)?;

        cipher.encrypt_file(source_file.path(), encrypted_file.path())?;
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path())?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(decrypted_content, original_content);

        Ok(())
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        PasswordHasher, SaltString, PasswordHash, PasswordVerifier
    },
    Pbkdf2
};
use rand::RngCore;
use std::fs;
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Pbkdf2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Key derivation failed: {}", e))?;
        
        let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
        if hash_bytes.len() < 32 {
            return Err("Derived key too short".into());
        }
        
        let key = Key::<Aes256Gcm>::from_slice(&hash_bytes[..32]);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let data = fs::read(input_path)
            .map_err(|e| format!("Failed to read input file: {}", e))?;
        
        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let encrypted_data = self.cipher
            .encrypt(nonce, data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output = Vec::with_capacity(NONCE_LENGTH + encrypted_data.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&encrypted_data);
        
        fs::write(output_path, &output)
            .map_err(|e| format!("Failed to write output file: {}", e))?;
        
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let data = fs::read(input_path)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
        
        if data.len() < NONCE_LENGTH {
            return Err("File too short to contain nonce".into());
        }
        
        let (nonce_bytes, encrypted_data) = data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let decrypted_data = self.cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, &decrypted_data)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
        
        Ok(())
    }
}

pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| format!("Invalid hash format: {}", e))?;
    
    Ok(Pbkdf2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}