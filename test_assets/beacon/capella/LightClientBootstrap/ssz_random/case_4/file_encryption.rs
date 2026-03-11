use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// XOR cipher implementation for file encryption/decryption
pub struct XorCipher {
    key: Vec<u8>,
    key_index: usize,
}

impl XorCipher {
    /// Create a new XOR cipher with the given key
    pub fn new(key: &[u8]) -> Self {
        XorCipher {
            key: key.to_vec(),
            key_index: 0,
        }
    }

    /// Encrypt/decrypt a single byte using XOR
    pub fn process_byte(&mut self, byte: u8) -> u8 {
        let result = byte ^ self.key[self.key_index];
        self.key_index = (self.key_index + 1) % self.key.len();
        result
    }

    /// Process an entire buffer
    pub fn process_buffer(&mut self, buffer: &mut [u8]) {
        for byte in buffer.iter_mut() {
            *byte = self.process_byte(*byte);
        }
    }
}

/// Encrypt or decrypt a file using XOR cipher
pub fn process_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;

    let mut cipher = XorCipher::new(key);
    let mut buffer = [0u8; 4096];

    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let mut data_chunk = buffer[..bytes_read].to_vec();
        cipher.process_buffer(&mut data_chunk);
        output_file.write_all(&data_chunk)?;
    }

    Ok(())
}

/// Generate a random key of specified length
pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = b"secret_key";
        let mut cipher1 = XorCipher::new(key);
        let mut cipher2 = XorCipher::new(key);

        let original_data = b"Hello, World!";
        let mut encrypted_data = original_data.to_vec();
        cipher1.process_buffer(&mut encrypted_data);

        let mut decrypted_data = encrypted_data.clone();
        cipher2.process_buffer(&mut decrypted_data);

        assert_eq!(original_data, decrypted_data.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = b"test_key_123";
        let test_data = b"Sample file content for encryption test";

        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        let restored_file = NamedTempFile::new()?;

        fs::write(input_file.path(), test_data)?;

        process_file(input_file.path(), output_file.path(), key)?;
        process_file(output_file.path(), restored_file.path(), key)?;

        let restored_data = fs::read(restored_file.path())?;
        assert_eq!(test_data, restored_data.as_slice());

        Ok(())
    }
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

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileCipher;

impl FileCipher {
    pub fn encrypt_file(
        source_path: &Path,
        dest_path: &Path,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut source_file = fs::File::open(source_path)?;
        let mut plaintext = Vec::new();
        source_file.read_to_end(&mut plaintext)?;

        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);

        let key = Self::derive_key(password, &salt);

        let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut dest_file = fs::File::create(dest_path)?;
        dest_file.write_all(&salt)?;
        dest_file.write_all(&iv)?;
        dest_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(
        source_path: &Path,
        dest_path: &Path,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut source_file = fs::File::open(source_path)?;
        let mut encrypted_data = Vec::new();
        source_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < SALT_LEN + IV_LEN {
            return Err("Invalid encrypted file format".into());
        }

        let salt = &encrypted_data[..SALT_LEN];
        let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];

        let key = Self::derive_key(password, salt);

        let plaintext = Aes256CbcDec::new(&key.into(), iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut dest_file = fs::File::create(dest_path)?;
        dest_file.write_all(&plaintext)?;

        Ok(())
    }

    fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            salt,
            PBKDF2_ITERATIONS,
            &mut key,
        );
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";

        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), plaintext).unwrap();

        FileCipher::encrypt_file(source_file.path(), encrypted_file.path(), password)
            .expect("Encryption failed");

        FileCipher::decrypt_file(encrypted_file.path(), decrypted_file.path(), password)
            .expect("Decryption failed");

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Test data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let source_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(source_file.path(), plaintext).unwrap();

        FileCipher::encrypt_file(source_file.path(), encrypted_file.path(), password)
            .expect("Encryption failed");

        let result = FileCipher::decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            wrong_password,
        );

        assert!(result.is_err());
    }
}