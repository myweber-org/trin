
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub salt: Vec<u8>,
    pub iv: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> io::Result<EncryptionResult> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut iv);

    let key = derive_key(password, &salt);
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&salt)?;
    output_file.write_all(&iv)?;
    output_file.write_all(&ciphertext)?;

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
) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < SALT_LENGTH + IV_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain salt and IV",
        ));
    }

    let (salt, rest) = encrypted_data.split_at(SALT_LENGTH);
    let (iv, ciphertext) = rest.split_at(IV_LENGTH);

    let key = derive_key(password, salt);
    let plaintext = Aes256CbcDec::new(&key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        let enc_result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        assert_eq!(enc_result.salt.len(), SALT_LENGTH);
        assert_eq!(enc_result.iv.len(), IV_LENGTH);
        assert!(!enc_result.ciphertext.is_empty());

        let decrypted = decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);

        let wrong_password_result = decrypt_file(encrypted_file.path(), decrypted_file.path(), "wrong_password");
        assert!(wrong_password_result.is_err());
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_index: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_index: 0,
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_index];
            result.push(byte ^ key_byte);
            self.key_index = (self.key_index + 1) % self.key.len();
        }
        
        result
    }

    pub fn reset(&mut self) {
        self.key_index = 0;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data = cipher.encrypt(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let decrypted_data = cipher.decrypt(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let mut cipher = XorCipher::new(key);
        
        let original = b"Hello, World!";
        let encrypted = cipher.encrypt(original);
        
        cipher.reset();
        let decrypted = cipher.decrypt(&encrypted);
        
        assert_eq!(original.to_vec(), decrypted);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let test_data = b"This is a test file for encryption.";
        
        let input_file = NamedTempFile::new()?;
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        fs::write(input_file.path(), test_data)?;
        
        encrypt_file(input_file.path(), encrypted_file.path(), key)?;
        decrypt_file(encrypted_file.path(), decrypted_file.path(), key)?;
        
        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(test_data.to_vec(), decrypted_content);
        
        Ok(())
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_position: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_position: 0,
        }
    }

    pub fn encrypt_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.process_bytes(data)
    }

    pub fn decrypt_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.process_bytes(data)
    }

    fn process_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_position];
            result.push(byte ^ key_byte);
            self.key_position = (self.key_position + 1) % self.key.len();
        }
        
        result
    }

    pub fn reset(&mut self) {
        self.key_position = 0;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data = cipher.encrypt_bytes(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let decrypted_data = cipher.decrypt_bytes(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original_data = b"Hello, World! This is a test message.";
        
        let mut cipher1 = XorCipher::new(key);
        let mut cipher2 = XorCipher::new(key);
        
        let encrypted = cipher1.encrypt_bytes(original_data);
        let decrypted = cipher2.decrypt_bytes(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let test_content = b"Sample file content for encryption testing.";
        
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(test_content)?;
        let input_path = input_file.path();
        
        let encrypted_file = NamedTempFile::new()?;
        let encrypted_path = encrypted_file.path();
        
        let decrypted_file = NamedTempFile::new()?;
        let decrypted_path = decrypted_file.path();
        
        encrypt_file(input_path, encrypted_path, key)?;
        decrypt_file(encrypted_path, decrypted_path, key)?;
        
        let decrypted_content = fs::read(decrypted_path)?;
        assert_eq!(test_content.to_vec(), decrypted_content);
        
        Ok(())
    }
}