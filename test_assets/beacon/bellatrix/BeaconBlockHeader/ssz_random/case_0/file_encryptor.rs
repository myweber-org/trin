
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    password: String,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Self {
        Self {
            password: password.to_string(),
        }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);

        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(
            self.password.as_bytes(),
            &salt,
            PBKDF2_ITERATIONS,
            &mut key,
        );

        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut output = Vec::with_capacity(SALT_LEN + IV_LEN + ciphertext.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&iv);
        output.extend_from_slice(&ciphertext);

        fs::File::create(output_path)
            .and_then(|mut f| f.write_all(&output))
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        if encrypted_data.len() < SALT_LEN + IV_LEN {
            return Err("Encrypted file too short".to_string());
        }

        let salt = &encrypted_data[..SALT_LEN];
        let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];

        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(
            self.password.as_bytes(),
            salt,
            PBKDF2_ITERATIONS,
            &mut key,
        );

        let cipher = Aes256CbcDec::new(&key.into(), iv.into());
        let plaintext = cipher
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        fs::File::create(output_path)
            .and_then(|mut f| f.write_all(&plaintext))
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password);
        
        let test_data = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), test_data).unwrap();
        
        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
        ).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::fs;
use std::io::{self, Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub struct FileEncryptor {
    key: [u8; 32],
    iv: [u8; 16],
}

impl FileEncryptor {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        rng.fill(&mut key);
        rng.fill(&mut iv);
        Self { key, iv }
    }

    pub fn from_key_iv(key: &str, iv: &str) -> Result<Self, &'static str> {
        let key_bytes = hex::decode(key).map_err(|_| "Invalid hex key")?;
        let iv_bytes = hex::decode(iv).map_err(|_| "Invalid hex iv")?;

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

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let cipher = Aes256CbcEnc::new(&self.key.into(), &self.iv.into());
        let encrypted_data = cipher.encrypt_padded_vec_mut::<Pkcs7>(&buffer);

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let mut file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let cipher = Aes256CbcDec::new(&self.key.into(), &self.iv.into());
        let decrypted_data = cipher
            .decrypt_padded_vec_mut::<Pkcs7>(&buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&decrypted_data)?;

        Ok(())
    }

    pub fn get_key_hex(&self) -> String {
        hex::encode(self.key)
    }

    pub fn get_iv_hex(&self) -> String {
        hex::encode(self.iv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        let test_data = b"Hello, this is a secret message!";
        let input_path = "test_input.txt";
        let encrypted_path = "test_encrypted.bin";
        let decrypted_path = "test_decrypted.txt";

        fs::write(input_path, test_data).unwrap();

        encryptor.encrypt_file(input_path, encrypted_path).unwrap();
        encryptor.decrypt_file(encrypted_path, decrypted_path).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(decrypted_data, test_data);

        fs::remove_file(input_path).unwrap();
        fs::remove_file(encrypted_path).unwrap();
        fs::remove_file(decrypted_path).unwrap();
    }

    #[test]
    fn test_key_iv_serialization() {
        let encryptor = FileEncryptor::new();
        let key_hex = encryptor.get_key_hex();
        let iv_hex = encryptor.get_iv_hex();

        let restored = FileEncryptor::from_key_iv(&key_hex, &iv_hex).unwrap();
        assert_eq!(restored.key, encryptor.key);
        assert_eq!(restored.iv, encryptor.iv);
    }
}